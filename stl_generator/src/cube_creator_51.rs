use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct CubeCreator {
  axis: Vec<Point>,
  axis1: Vec<Point>,
  axis2: Vec<Point>,
  groove: Vec<f32>,
  groove_s: Vec<f32>,
  l: Vec<f32>,
  axis_pos: RefCell<Vec<f32>>,
  axis_neg: RefCell<Vec<f32>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}
impl CubeCreator {
  pub fn new() -> Self {
    let axis: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let main_cos = 0.0;
    let main_angle = main_cos.acos();

    let corner_cos = ((main_cos * 2.0 + 1.0) / 3.0).sqrt();
    let corner_angle = corner_cos.acos();

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let in_r = 16.0;

    let groove = vec![
      (main_angle * 0.5 + 4.0 / (in_r - 4.8)).cos(),
      in_r - 4.6,
      (main_angle * 0.5 + 1.5 / (in_r - 4.8)).cos(),
      in_r - 2.2,
      (corner_angle + 5.0 / in_r).cos(),
      in_r + 0.2,
      (corner_angle + 2.0 / in_r).cos(),
    ];

    let mut groove_s = groove.clone();
    groove_s.extend(&[in_r + 2.6, 2.6 / (in_r + 2.8)]);

    let mut axis1 = Vec::new();
    let mut axis2 = Vec::new();
    for &a in &axis {
      let a1 = a.any_perp();
      let a2 = cross(a, a1);
      axis1.push(a1);
      axis2.push(a2);
    }

    let l = vec![21.5, 28.5, 28.5, 28.5, 28.5, 28.5];

    Self { axis, axis1, axis2, groove, groove_s, l, axis_pos, axis_neg }
  }

  pub fn faces(&self) -> usize {
    6
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, 6)
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    0.6
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    1
  }

  pub fn get_name(&self, current_normal: usize) -> Option<String> {
    None
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    let n = self.axis[current_normal];
    let n1 = n.any_perp();
    let n2 = cross(n, n1);
    let pos = n.scale(self.l[current_normal] - 0.01) + n1.scale(pos.x) + n2.scale(pos.y);
    let result = self.get_part_index_impl(pos, current_normal);
    (result > 0) as PartIndex
  }

  pub fn get_quality() -> usize {
    128
  }

  pub fn get_size() -> f32 {
    60.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 29.999 || pos.y.abs() > 29.999 || pos.z.abs() > 29.999 {
      return 0;
    }

    if pos.y < 0.0 {
      //  return 0;
    }

    let sphere_r = self.groove[1] - 4.2;

    if r < sphere_r {
      for &a in &self.axis {
        let s = cross(pos, a).len();
        if s < 1.25 {
          return 0;
        }
      }
      return 63;
    }

    let mut depth_to_face = f32::INFINITY;
    let sticker_gap = 0.5;
    for (i, &a) in self.axis.iter().enumerate() {
      let mut d = self.l[i] - dot(pos, a);
      if current_normal < 6 && current_normal != i {
        d -= sticker_gap;
      }
      depth_to_face = f32::min(depth_to_face, d);
    }

    if depth_to_face < 0.0 {
      return 0;
    }

    let mut index: PartIndex = 0;
    let shaft_r = f32::min(1.5, self.groove[1] - 0.9 - r);

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);
    let last_groove_r = self.groove[self.groove.len() - 2];
    if shaft_r > 0.0 {
      shift_out += shaft_r * 0.05;
      shift_in += shaft_r * 0.05;
    }
    if r > last_groove_r {
      shift_out = f32::min(shift_out, 9.5 / r);
      shift_in = f32::min(shift_in, 9.5 / r);
    }

    let (mut shift_out_s, mut shift_in_s, inter_s) = get_groove(r, &self.groove_s, 0.03);
    let last_groove_r_s = self.groove_s[self.groove_s.len() - 2];
    if shaft_r > 0.0 {
      shift_out_s += shaft_r * 0.05;
      shift_in_s += shaft_r * 0.05;
    }
    if r > last_groove_r_s + 0.2 {
      shift_out_s -= (r - (last_groove_r_s + 0.2)) * 0.1;
      shift_out_s = f32::max(shift_out_s, 2.5 / r);
      shift_in_s -= (r - (last_groove_r_s + 0.2)) * 0.1;
      shift_in_s = f32::max(shift_in_s, 2.5 / r);
    }

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    axis_pos.clear();
    axis_neg.clear();

    for (i, &a) in self.axis.iter().enumerate() {
      let shift_in = if i == 0 { shift_in_s } else { shift_in };
      let shift_out = if i == 0 { shift_out_s } else { shift_out };

      let c = dot(pos, a);

      let check_in = c - shift_in * r;
      if check_in > 0.0 {
        index += (1 << i);
        axis_pos.push(check_in);
      } else {
        let check_out = shift_out * r - c;
        if check_out > 0.0 {
          axis_neg.push(check_out);
        } else {
          return 0;
        }
      }
    }

    if index.count_ones() == 1 {
      if depth_to_face > 0.5 {
        let hole_r = if r < sphere_r + 2.0 { 1.5 } else { 3.2 };
        for (i, &a) in self.axis.iter().enumerate() {
          let s = cross(pos, a).len();

          if s < hole_r {
            return 0;
          }
        }
      }
    } else if r < self.groove[1] - 2.2 {
      return 0;
    }

    let rr;

    if index.count_ones() == 3 && r > last_groove_r - 0.2 {
      rr = f32::clamp(1.0 + (r - (last_groove_r + 2.6)) * 1.0, 1.0, 6.0);
    } else {
      rr = f32::clamp((r - 17.0) / (20.0 - 17.0), 0.0, 1.0) * 4.0 + 2.0;
    }

    if current_normal < 6 {
      for p in axis_pos.iter_mut() {
        if *p < sticker_gap {
          return 0;
        }
        *p -= 0.5;
      }
      for n in axis_neg.iter_mut() {
        if *n < sticker_gap {
          return 0;
        }
        *n -= 0.5;
      }
    }

    let rr = if current_normal < 6 { rr - sticker_gap } else { rr };

    let mut in_sr = |a, b| {
      if a < rr && b < rr && sqr(rr - a) + sqr(rr - b) > sqr(rr) {
        return true;
      }
      false
    };

    axis_pos.sort_by(|a, b| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if axis_pos.len() >= 2 && in_sr(axis_pos[0], axis_pos[1]) {
      return 0;
    }
    if axis_neg.len() >= 2 && in_sr(axis_neg[0], axis_neg[1]) {
      return 0;
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 && in_sr(axis_pos[0], axis_neg[0]) {
      return 0;
    }

    if index.count_ones() == 1 {
      let ai0 = index.ilog2() as usize;
      let ai1 = (ai0 + 2) % 6;
      let ai2 = (ai0 + 4) % 6;
      let p_dot = |i| {
        let d = dot(pos, self.axis[i]);
        if d > 0.0 {
          if i == 0 {
            d - 2.5
          } else {
            d - 9.5
          }
        } else {
          if i == 1 {
            -d - 2.5
          } else {
            -d - 9.5
          }
        }
      };

      let a_dist = f32::max(p_dot(ai1), p_dot(ai2));

      if depth_to_face <= 1.5 || a_dist > -2.86 && depth_to_face <= 3.5 {
        index += 64;
      } else if depth_to_face <= 2.0 || a_dist > -3.0 && depth_to_face <= 4.0 {
        return 0;
      }
    }

    return index;
  }
}
