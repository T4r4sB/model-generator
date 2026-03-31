use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::*;
use num::Float;
use num::PrimInt;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

fn sqr(x: f32) -> f32 {
  x * x
}

pub struct BubbloidCreator {
  axis: Vec<Point>,
  normals: Vec<Point>,

  adj_small_petals: FxHashMap<u32, (Point, Point)>,
  adj_big_petals: FxHashMap<u32, (Point, Point)>,

  groove: Vec<f32>,
  groove_trivial_tip: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, i32, Point)>>,
  axis_neg: RefCell<Vec<(f32, i32, Point)>>,
}

impl BubbloidCreator {
  pub fn new() -> Self {
    // let a = 0.5949; // > 0.5
    let a = 0.596;
    let e = 2.0 * sqr(a) - 1.0;
    let se = (1.0 - sqr(e)).sqrt();
    let b = e / a;
    let sa = (1.0 - sqr(a)).sqrt();
    let sb = (1.0 - sqr(b)).sqrt();
    let e2 = 2.0 * sqr(b) - 1.0;
    let se2 = (1.0 - sqr(e2)).sqrt();
    let rota = (e / (e + 1.0)).acos();
    let rotb = ((e - e * e2) / (se * se2)).acos();
    let rotd = rota - rotb;
    let depth = a * e2 + sa * se2 * rotd.cos();
    let e3 = e * e2 + se * se2 * rotd.cos();
    // e=106.83209, e2=121.86335
    // rota=114.053795,131.89243
    let sq2 = 0.5.sqrt();

    let axis = vec![
      Point { x: -sa * sq2, y: -sa * sq2, z: a },
      Point { x: sa * sq2, y: sa * sq2, z: a },
      Point { x: sb * sq2, y: -sb * sq2, z: b },
      Point { x: -sb * sq2, y: sb * sq2, z: b },
    ];

    let mut adj_small_petals = FxHashMap::<u32, (Point, Point)>::default();
    let mut adj_big_petals = FxHashMap::<u32, (Point, Point)>::default();

    let f1 = find_factors_for_triangle(e, e2, e3);
    let f2 = find_factors_for_triangle(e2, e, e3);

    println!("e={e}, e2={e2}, e3={e3}");
    for i1 in 0..axis.len() {
      for i2 in i1 + 1..axis.len() {
        let a1 = axis[i1];
        let a2 = axis[i2];
        if (dot(a1, a2) - e).abs() < 0.01 {
          let p1 = a1.scale(f1.0) + a2.scale(f1.1) + cross(a1, a2).scale(f1.2);
          let p2 = a2.scale(f1.0) + a1.scale(f1.1) + cross(a2, a1).scale(f1.2);

          adj_small_petals.insert(1 << i1 | 1 << i2, (p1, p2));
        } else if (dot(a1, a2) - e2).abs() < 0.01 {
          let p1 = a1.scale(f2.0) + a2.scale(f2.1) - cross(a1, a2).scale(f2.2);
          let p1s = p1.scale(f1.0) + a1.scale(f1.1) + cross(p1, a1).scale(f1.2);
          let p2 = a2.scale(f2.0) + a1.scale(f2.1) - cross(a2, a1).scale(f2.2);
          let p2s = p2.scale(f1.0) + a2.scale(f1.1) + cross(p2, a2).scale(f1.2);
          adj_big_petals.insert(1 << i1, (p1, p1s));
          adj_big_petals.insert(1 << i2, (p2, p2s));
        } else {
          panic!("Wrong dot {}", dot(a1, a2));
        }
      }
    }

    let normals: Vec<_> = [
      Point { x: -1.0, y: -1.0, z: -1.0 },
      Point { x: -1.0, y: 1.0, z: 1.0 },
      Point { x: 1.0, y: -1.0, z: 1.0 },
      Point { x: 1.0, y: 1.0, z: -1.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let max_angle = depth.acos();
    let min_angle = e2.acos() * 0.5;
    let in_angle = e.acos() * 0.5;
    let ahole = max_angle - min_angle;
    let r = 3.0 / ahole;

    let groove = vec![
      /*
      (in_angle + 3.0 / (r - 7.2)).cos(),
      r - 7.0,
      (in_angle + 1.0 / (r - 7.2)).cos(),
      r - 4.6,
      (max_angle - 2.0 / r).cos(),
      r - 2.2,
      */
      (max_angle).cos(),
      r + 0.2,
      (max_angle - 2.0 / r).cos(),
      r + 2.6,
      (max_angle).cos(),
    ];

    assert!(groove[0].acos() <= max_angle + 1.0e-5);

    let tt_angle = e.acos() - max_angle;

    let groove_trivial_tip = vec![(tt_angle - 2.0 / r).cos(), r + 2.6, tt_angle.cos()];

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    Self {
      axis,
      normals,
      adj_small_petals,
      adj_big_petals,
      groove,
      groove_trivial_tip,
      axis_pos,
      axis_neg,
    }
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    0.9
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    1
  }

  pub fn get_name(&self, current_normal: usize) -> Option<String> {
    None
  }

  pub fn get_size() -> f32 {
    120.0
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    let n = self.normals[current_normal];

    fn sinc(x: f32) -> f32 {
      if x == 0.0 {
        1.0
      } else {
        x.sin() / x
      }
    }

    fn versinc(x: f32) -> f32 {
      if x == 0.0 {
        0.0
      } else {
        (1.0 - x.cos()) / x
      }
    }

    let k = 0.006;
    let k2 = k * 2.0;
    let last_groove = self.groove[self.groove.len() - 2];
    let max = last_groove + 2.2;

    let r = pos.len();
    let a = r * k2;

    let n1 = n.any_perp().norm();
    let n2 = cross(n, n1).norm();

    let pos = pos.scale(sinc(a));
    let pos = n.scale(max - r * versinc(a)) + n1.scale(pos.x) + n2.scale(pos.y);

    let r = pos.len();
    let d = dot(pos, n);

    let control_c = n.scale(max - k2.recip());
    let delta = d + (max * max + r * r) * k - (2.0 * d * k + 1.0) * max;

    let result = self.get_part_index_impl(pos, current_normal);

    (result > 0) as PartIndex
  }

  pub fn faces(&self) -> usize {
    0
  }

  pub fn get_quality() -> usize {
    128
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if pos.x.abs() > 59.0 || pos.y.abs() > 59.0 || pos.z.abs() > 59.0 {
      return 0;
    }

    let r = pos.len();

    let inner_r = self.groove[1] - 2.2;
    if r < inner_r - 0.3 {
      let in_sphere = || -> bool {
        if r < inner_r - 5.3 {
          return false;
        }

        for (i, &a) in self.axis.iter().enumerate() {
          let d = dot(pos, a);
          let s = cross(pos, a).len();
          if d > 0.0 && s < 1.5 {
            return false;
          }
          if d < 0.0 && s < 10.0 {
            return false;
          }
        }

        return true;
      };

      if in_sphere() {
        return 31;
      }

      let p1 = -self.axis[0].scale(inner_r);
      let p2 = -self.axis[1].scale(inner_r);
      if cross(p1 - pos, p2 - pos).sqr_len() < sqr(2.0) * (p1 - p2).sqr_len() {
        return 63;
      }

      return 0;
    }

    let last_groove = self.groove[self.groove.len() - 2];
    let m = last_groove + 2.2;
    let mut depths = Vec::<f32>::new();

    for ni in 0..self.normals.len() {
      let k = 0.003;
      let r = 0.5 / k;
      let center = self.normals[ni].scale(m - r);
      let depth = r - (pos - center).len();
      if depth < 0.0 {
        return 0;
      }
      depths.push(depth);
    }

    depths.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if depths[0] > 2.0 {
      for &a in &self.axis {
        let d = dot(pos, a);
        let s = cross(pos, a).len();
        if d > 0.0 && s < 1.25 {
          return 0;
        }
      }
    }

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();

    axis_pos.clear();
    axis_neg.clear();

    let mut index: PartIndex = 0;

    let (mut shift_out_tt, mut shift_in_tt, inter_tt) =
      get_groove(r, &self.groove_trivial_tip, 0.03);

    let mut match_all = || -> Option<()> {
      let mut match_axis =
        |pos: Point, index: &mut PartIndex, bit: usize, axis: Point| -> Option<()> {
          let d = dot(pos, axis) / r;

          let check_in = d - shift_in;
          if check_in > 0.0 {
            *index |= (1 << bit);
            axis_pos.push((check_in, 0, axis));

            let check_in_tt = d - shift_in_tt;
            if check_in_tt > 0.0 {
              axis_pos.push((check_in_tt, 1, axis));
              *index |= (1 << (bit + 6));
            } else {
              let check_out_tt = shift_out_tt - d;
              if check_out_tt > 0.0 {
                axis_neg.push((check_out_tt, 1, axis));
              } else {
                return None;
              }
            }
          } else {
            let check_out = shift_out - d;
            if check_out > 0.0 {
              axis_neg.push((check_out, 0, axis));
            } else {
              return None;
            }
          }

          return Some(());
        };
      for i in 0..self.axis.len() {
        match_axis(pos, &mut index, i, self.axis[i])?;
      }

      if let Some(&(p1, p2)) = self.adj_small_petals.get(&index) {
        match_axis(pos, &mut index, self.axis.len() + 0, p1)?;
        match_axis(pos, &mut index, self.axis.len() + 1, p2)?;
      }

      if let Some(&(p1, p2)) = self.adj_big_petals.get(&index) {
        match_axis(pos, &mut index, self.axis.len() + 0, p1)?;
        if index & (1 << (self.axis.len() + 0)) > 0 {
          match_axis(pos, &mut index, self.axis.len() + 1, p2)?;
        }
      }

      Some(())
    };

    if match_all().is_none() {
      return 0;
    }

    let mut in_sr = |a, b, d| {
      let r = 0.096 * d;
      if a < r && b < r {
        return r - (sqr(r - a) + sqr(r - b)).sqrt();
      }
      return f32::INFINITY;
    };

    let gl = self.groove.len();
    let thick =
      index.count_ones() == 2 && r > self.groove[gl - 4] - 0.2 && r < self.groove[gl - 2] + 0.2;

    let mut rr = 1.0f32;

    axis_pos.sort_by(|(a, _, _), (b, _, _)| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|(a, _, _), (b, _, _)| a.partial_cmp(b).unwrap());

    let mut minimal = f32::INFINITY;
    for &(a, _, _) in axis_pos.iter() {
      minimal = f32::min(minimal, a);
    }
    for &(a, _, _) in axis_neg.iter() {
      minimal = f32::min(minimal, a);
    }

    if axis_pos.len() >= 2 {
      let k = if thick { 0.3 } else { 1.0 };
      minimal = f32::min(minimal, in_sr(axis_pos[0].0, axis_pos[1].0, rr * k));
    }
    if axis_neg.len() >= 2 {
      minimal = f32::min(minimal, in_sr(axis_neg[0].0, axis_neg[1].0, rr));
    }
    if !inter
      && axis_pos.len() >= 1
      && axis_neg.len() >= 1
      && axis_pos[0].1 == 0
      && axis_neg[0].1 == 0
    {
      let d = dot(axis_pos[0].2, axis_neg[0].2);

      if d > -0.74 {
        let k = 0.5;
        minimal = f32::min(minimal, in_sr(axis_pos[0].0, axis_neg[0].0, rr * k));
      }
    }

    if minimal < 0.0 {
      return 0;
    }

    let minimal = f32::min(minimal, depths[1] * 0.015);

    if depths[0] < 0.5 && minimal > 0.02 + depths[0] * 0.01 {
      return 0;
    }

    if index == 0 && (dot(pos.norm(), -self.axis[0]) > 0.5 || dot(pos.norm(), -self.axis[1]) > 0.5)
    {
      return 63;
    }

    return 0; // tmp

    if r < inner_r {
      return 0;
    }

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.axis.len())
  }
}
