#![allow(unused)]

use common::contour::*;
mod clickbox2_creator;
type PartCreator = clickbox2_creator::ClickboxCreator;

fn main() {
  let part_creator = PartCreator::new();

  let mut cc = ContourCreator::new(1024, 400.0, 20);

  let mut total_length = 0.0;
  let mut total_square = 0.0;

  for i in 0..part_creator.faces() {
    let contours = cc.make_contour(&|p| part_creator.get_sticker_index(p, i));
    let h = part_creator.get_height(i);
    let name = part_creator.get_name(i).map(|s| s.to_string()).unwrap_or(format!("part_{i}"));

    let thickness = h;
    let count = part_creator.get_count(i);
    let name = format!("(THICK={thickness}, AMOUNT={count}) {name}");

    let single_i = contours.len() == 1;
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
    }
  }

  println!("total {total_length} length, {total_square} square");
}
