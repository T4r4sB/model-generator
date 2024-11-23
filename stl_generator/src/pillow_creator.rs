use crate::common_for_twisty_puzzles::*;
use crate::model::*;
use crate::points3d::*;
use crate::solid::*;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct PillowCreator {
  axis: Vec<Point>,
  axis1: Vec<Point>,
  axis2: Vec<Point>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<f32>>,
  axis_neg: RefCell<Vec<f32>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}
impl PillowCreator {
  pub fn new() -> Self {
    let phi = (5.0.sqrt() - 1.0) / 2.0;

    let axis: Vec<_> = [
      Point { x: 0.0, y: phi, z: -1.0 },
      Point { x: 0.0, y: -phi, z: -1.0 },
      Point { x: 0.0, y: phi, z: 1.0 },
      Point { x: 0.0, y: -phi, z: 1.0 },
      Point { x: -1.0, y: 0.0, z: phi },
      Point { x: -1.0, y: 0.0, z: -phi },
      Point { x: 1.0, y: 0.0, z: phi },
      Point { x: 1.0, y: 0.0, z: -phi },
      Point { x: phi, y: -1.0, z: 0.0 },
      Point { x: -phi, y: -1.0, z: 0.0 },
      Point { x: phi, y: 1.0, z: 0.0 },
      Point { x: -phi, y: 1.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let main_cos = 0.2.sqrt();
    let main_angle = main_cos.acos();

    let corner_cos = ((main_cos * 2.0 + 1.0) / 3.0).sqrt();
    let corner_angle = corner_cos.acos();

    let split_cos_out = 0.5;
    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let groove = vec![
      (corner_angle + 5.0 / 19.2).cos(),
      19.4,
      (corner_angle + 2.0 / 19.2).cos(),
      21.8,
      (corner_angle + 5.0 / 19.2).cos(),
      24.2,
      0.5,
    ];

    let mut axis1 = Vec::new();
    let mut axis2 = Vec::new();
    for &a in &axis {
      let a1 = a.any_perp();
      let a2 = cross(a, a1);
      axis1.push(a1);
      axis2.push(a2);
    }

    Self { axis, axis1, axis2, groove, axis_pos, axis_neg }
  }

  pub fn faces(&self) -> usize {
    0
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, 1)
  }

  pub fn get_height(current_normal: usize) -> f32 {
    0.6
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn get_quality() -> usize {
    512
  }

  pub fn get_size() -> f32 {
    60.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 29.999 || pos.y.abs() > 29.999 || pos.z.abs() > 29.999 {
      return 0;
    }

    if r < 12.0 {
      return 0;
      for &a in &self.axis {
        let s = cross(pos, a).len();
        if s < 1.25 {
          return 0;
        }
      }
      return 17;
    }

    let dx = 26.0 - 22.0 * sqr(pos.y / 52.0) + 22.0 * sqr(pos.z / 52.0) - pos.x.abs();
    let dy = 26.0 - 22.0 * sqr(pos.z / 52.0) + 22.0 * sqr(pos.x / 52.0) - pos.y.abs();
    let dz = 26.0 - 22.0 * sqr(pos.x / 52.0) + 22.0 * sqr(pos.y / 52.0) - pos.z.abs();

    if dx < 0.0 || dy < 0.0 || dz < 0.0 {
      return 0;
    }

    let out_r = 4.0;

    let in_r = |a, b| {
      if a < out_r && b < out_r {
        if sqr(out_r - a) + sqr(out_r - b) > sqr(out_r) {
          return true;
        }
      }
      false
    };

    if in_r(dx, dy) || in_r(dx, dz) || in_r(dy, dz) {
      return 0;
    }

    if dx < out_r
      && dy < out_r
      && dz < out_r
      && sqr(out_r - dx) + sqr(out_r - dy) + sqr(out_r - dz) > sqr(out_r)
    {
      return 0;
    }

    let mut index: PartIndex = 0;
    let in_core = r < 24.0;
    let out_core = r > 24.4;

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);

    if r < 19.6 {
      if r < 14.0 {
        shift_out *= (39.6 / (14.0 + 20.0));
        shift_in *= (39.6 / (14.0 + 20.0));
      } else {
        shift_out *= (39.6 / (r + 20.0));
        shift_in *= (39.6 / (r + 20.0));
      }
    }

    let hole_r = if r < 13.5 { 1.5 } else { 3.2 };

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    axis_pos.clear();
    axis_neg.clear();

    for (i, &a) in self.axis.iter().enumerate() {
      let c = dot(pos, a) / r;
      let s = cross(pos, a).len();
      if in_core && c > 0.0 && s < hole_r {
        return 0;
      }
      let check_in = c - shift_in;
      if check_in > 0.0 {
        if !out_core && in_spiral(pos, a, self.axis1[i], self.axis2[i], check_in, 0.02, 0.2) {
          return 0;
        }
        index += (1 << i);
        axis_pos.push(check_in);
      } else {
        let check_out = shift_out - c;
        if check_out > 0.0 {
          if !out_core && in_spiral(pos, a, self.axis1[i], -self.axis2[i], check_out, 0.02, 0.2) {
            return 0;
          }
          axis_neg.push(check_out);
        } else {
          return 0;
        }
      }
    }

    if index != 1 && index != 3 {
      return 0;
    }

    let mut in_sr = |a, b| {
      let r = 0.05;
      if a < r && b < r && sqr(r - a) + sqr(r - b) > sqr(r) {
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
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 2 && in_sr(axis_pos[0], axis_neg[0]) {
      return 0;
    }

    if index.count_ones() == 3 {
      if (index & 0xf != 0) as usize + (index & 0xf0 != 0) as usize + (index & 0xf00 != 0) as usize
        == 3
      {
        let corner_vec = Point { x: pos.x.signum(), y: pos.y.signum(), z: pos.z.signum() }.norm();
        let corner_s = cross(pos, corner_vec).len();
        let corner_d = dot(pos, corner_vec);
        if corner_d > 28.0 && corner_d < 31.0 && corner_s < 8.0 {
          if corner_d > 28.15 && corner_d < 30.85 && corner_s < 7.8 {
            return index + 3;
          }
          return 0;
        }
        if corner_d > 16.5 && corner_d < 18.0 && corner_s < 2.5 {
          if corner_d > 16.65 && corner_d < 17.85 && corner_s < 2.3 {
            return index + 4;
          }
          return 0;
        }

        let mind = f32::min(dx, f32::min(dy, dz));
        let hole = f32::min(0.2, 0.0 + mind * 0.05);
        let rot = f32::min(PI / 3.0, f32::max(0.0, mind - 8.0) * 0.5);
        let pos = pos.rotate(corner_vec, rot);

        let dx = 26.0 - 22.0 * sqr(pos.y / 52.0) + 22.0 * sqr(pos.z / 52.0) - pos.x.abs();
        let dy = 26.0 - 22.0 * sqr(pos.z / 52.0) + 22.0 * sqr(pos.x / 52.0) - pos.y.abs();
        let dz = 26.0 - 22.0 * sqr(pos.x / 52.0) + 22.0 * sqr(pos.y / 52.0) - pos.z.abs();

        if dx < dy - hole && dx < dz - hole {
          index += 1;
        } else if dy < dz - hole && dy < dx - hole {
          index += 2;
        } else if dz < dx - hole && dz < dy - hole {
        } else {
          return 0;
        }
      }
    } else if !in_core {
      return 0;
    }

    return index;
  }
}
