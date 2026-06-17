#![allow(unused)]

use std::num::NonZeroU32;
use std::ops::Deref;
use std::time::Duration;

use common::common_for_twisty_puzzles::*;
use common::contour::*;
use common::matrix::*;
use common::model::*;
use common::points2d;
use common::points2d::AABB;
use common::points3d::*;
use common::solid::*;
use fxhash::FxHashMap;

use common::solid::PartIndex;

mod gl_utils;
mod gl_window;
mod resources;

#[macro_use]
mod errors;

mod confusing_creator;
type PartCreator = confusing_creator::ConfusingCreator;

//mod railroad_creator;
//type PartCreator = railroad_creator::RailroadCreator;

//mod sphere_creator;
//type PartCreator = sphere_creator::SphereCreator;

fn generate_models() -> FxHashMap<PartIndex, Model> {
  let part_creator = PartCreator::new();
  let mut pf_timer = std::cell::RefCell::new(Duration::ZERO);
  let part_func = &|p| {
    let start = std::time::Instant::now();
    let result = part_creator.get_part_index(p);
    *pf_timer.borrow_mut() += std::time::Instant::now() - start;
    result
  };

  let start = std::time::Instant::now();
  let mut cc = ContourCreator::new(points2d::AABB::around_zero(100.0), 0.15, 20);

  let mut total_length = 0.0;
  let mut total_square = 0.0;

  for i in 0..part_creator.faces() {
    let mut contours = cc.make_contour(&|p| part_creator.get_sticker_index(p, i));
    let h = part_creator.get_height(i);
    let name = part_creator.get_name(i).map(|s| s.to_string()).unwrap_or(format!("part_{i}"));

    let thickness = h;
    let count = part_creator.get_count(i);
    let name = format!("(THICK={thickness}, AMOUNT={count}) {name}");

    let mut single_i = contours.len() == 1;

    if !single_i {
      // if need single file
      let mut common = ContourSet::new();
      for (_, cc) in std::mem::take(&mut contours) {
        common.append(cc)
      }
      contours.insert(1, common);
      single_i = true;
    }

    for (index, mut cc) in contours {
      cc.optimize(0.01);
      cc.remove_trash();

      let name = if single_i { name.clone() } else { format!("{name}_{index}") };

      let square = cc.get_square();
      let length = cc.get_length();

      total_length += length * count as f32;
      total_square += square * count as f32;

      println!(
        "save {name} ({} points, {square} square, {length} length) to dxf...",
        cc.points_count()
      );
      if let Err(msg) =
        cc.save_to_dxf(&std::path::Path::new("contours").join(format!("{name}.dxf")))
      {
        println!("{}", msg);
      }

      let ex = cc.extrude(h);

      let single_j = ex.len() == 1;
      for j in 0..ex.len() {
        let name = if single_j { name.clone() } else { format!("{name}_{j}") };

        if let Err(msg) =
          ex[j].save_to_stl(&std::path::Path::new("extruded").join(format!("{name}.stl")))
        {
          println!("{}", msg);
        }
      }

      let mut appended = Model::new();
      for ex in ex {
        appended.append(ex);
      }

      let name = format!("{name}_appended");
      if let Err(msg) =
        appended.save_to_stl(&std::path::Path::new("extruded").join(format!("{name}.stl")))
      {
        println!("{}", msg);
      }
    }
  }

  println!("total {total_length} length, {total_square} square");

  let quality = PartCreator::get_quality();

  let mut mc = ModelCreator::new(quality, PartCreator::get_size(), 20, 0, part_func);
  let width = 0.05;
  println!();
  while !mc.finished() {
    mc.fill_next_layer(part_func);
  }
  println!();
  println!("got {} points {} edges", mc.got_points(), mc.got_edges());

  let end_layers = std::time::Instant::now();

  let mut max_v = 0;
  let mut sum_v = 0;
  let mut max_v_after = 0;
  let mut sum_v_after = 0;
  let mut models = mc.get_models();
  let mut sum_volumes = 0.0;
  let mut weights = Vec::new();
  let mut groups_of_models = FxHashMap::<u32, Model>::default();
  let mut sum_t_before = 0;
  let mut sum_t_after = 0;

  for (&m_index, m) in &mut models {
    sum_v += m.vertices.len();
    max_v = std::cmp::max(max_v, m.vertices.len());
    m.validate_and_delete_small_groups();
    let smooth_cnt = quality / 50;
    if smooth_cnt > 0 {
      println!();
      for i in 0..smooth_cnt {
        m.smooth(0.4);
        print!("\rmake model {m_index} smooth, progress [{i}/{smooth_cnt}]");
      }
    }
    sum_t_before += m.triangles.len();
    if quality > 0 {
      println!("tcount before = {}", m.triangles.len());
      m.optimize(width, 0.5, 1000, 0.99);
      println!("tcount after {}", m.triangles.len());
    }
    sum_t_after += m.triangles.len();
    m.delete_unused_v();

    let volume = m.get_volume();
    sum_volumes += volume;

    sum_v_after += m.vertices.len();
    max_v_after = std::cmp::max(max_v_after, m.vertices.len());

    weights.push((m_index, volume * 7.850 * 0.001));

    if quality > 30 {
      println!(
        "save {m_index} to stl... {} vertices {} triangles {} volume {} mass",
        m.vertices.len(),
        m.triangles.len(),
        volume,
        volume * 7.850 * 0.001
      );
      if let Err(msg) =
        m.save_to_stl(&std::path::Path::new("output").join(format!("output_{}.stl", m_index)))
      {
        println!("{}", msg);
      }
    }
  }

  // models = groups_of_models;

  let end_opt = std::time::Instant::now();

  println!(
    "models created, sum_v={}, max_v={}, after: sum_v={}, max_v={}, total volume = {}, total mass = {}",
    sum_v,
    max_v,
    sum_v_after,
    max_v_after,
    sum_volumes,
    sum_volumes * 7.850 * 0.001
  );

  println!(
    "model compression {sum_t_before} to {sum_t_after}: {} times",
    sum_t_before as f32 / sum_t_after as f32
  );

  println!(
    "layers time: {:?}, opt time: {:?}, pf time: {:?}",
    end_layers - start,
    end_opt - end_layers,
    *pf_timer.borrow()
  );

  weights.sort_by(|(_, w1), (_, w2)| w1.partial_cmp(w2).unwrap());
  for (i, w) in weights {
    println!("{i}\t{w}");
  }
  models
}

fn main() {
  let mut models = generate_models();
  /*
  let mut model = Model::cuboid(500, 500, 500, 0.1);
  model.optimize(0.0, 0.9999, 1, 0.0);
  println!("{} triangles", model.triangles.len());
  let mut models = FxHashMap::<PartIndex, Model>::default();
  models.insert(1, model);*/

  if let Err(_) = crate::gl_window::run(
    "ОКНО С ПРИКОЛАМИ",
    &mut models.iter().map(|(m_index, m)| {
      println!("model {m_index} has {} vertices", m.vertices.len());
      let color = match m_index / 10000 {
        1 => 0x00FF00,
        2 => 0xFF2000,
        3 => 0xEEEEEE,
        4 => 0x0080FF,
        5 => 0xFF8000,
        6 => 0xFFFF00,
        7 => 0xFF00FF,
        8 => 0xFF80FF,
        _ => (m_index + 1).wrapping_mul(0x274381) as u32 | 0x808080,
      };
      (color, m)
    }),
  ) {
    // Do nothing, read message and exit
  }
}
