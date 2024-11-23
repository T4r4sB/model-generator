use crate::common_for_twisty_puzzles::get_groove;
use crate::model::*;
use crate::points3d::*;
use crate::solid::*;
use num::Float;
const PI: f32 = std::f32::consts::PI;

use std::cell::RefCell;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct HoodCubeCreator {
  axis: Vec<Point>,
  normals: Vec<Point>,
  n_basis: Vec<(Point, Point)>,
  groove: Vec<f32>,

  axis_pos: RefCell<Vec<NearAxis>>,
  axis_neg: RefCell<Vec<NearAxis>>,
}

impl HoodCubeCreator {
  pub fn new() -> Self {
    let axis = [
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 1.0, y: 1.0, z: 0.0 },
      Point { x: 1.0, y: -1.0, z: 0.0 },
      Point { x: -1.0, y: 1.0, z: 0.0 },
      Point { x: -1.0, y: -1.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let normals: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let n_basis = normals
      .iter()
      .map(|&n| {
        let n1 = n.any_perp().norm();
        let n2 = cross(n, n1).norm();
        (n1, n2)
      })
      .collect();

    let min_angle = PI * 0.5;
    let max_angle = 2.0.sqrt().atan();
    let groove = vec![
      (min_angle - 9.0 / 19.2).cos(),
      19.4,
      (min_angle - 6.0 / 19.2).cos(),
      21.8,
      (min_angle - 9.0 / 19.2).cos(),
    ];

    Self {
      axis,
      normals,
      n_basis,
      groove,
      axis_pos: RefCell::new(Vec::new()),
      axis_neg: RefCell::new(Vec::new()),
    }
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    return 0;
    let n = self.normals[current_normal];
    let (n1, n2) = self.n_basis[current_normal];

    let pos = n.scale(28.6 / n.sqr_len()) + n1.scale(pos.x) + n2.scale(pos.y);
    let result = self.get_part_index_impl(pos, current_normal);

    (result > 0) as PartIndex
  }

  pub fn faces(&self) -> usize {
    self.normals.len()
  }

  pub fn get_height(current_normal: usize) -> f32 {
    0.6
  }

  pub fn get_quality() -> usize {
    128
  }

  pub fn get_size() -> f32 {
    90.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if pos.x > 0.0 {
      return 0;
    };
    let r = pos.len();
    if r < 14.0 {
      for i in 0..self.axis.len() {
        let a = self.axis[i];
        if dot(pos, self.axis[i]) > 0.0 {
          let dist_to_axle = cross(pos, self.axis[i]).len();
          if dist_to_axle < 1.25 {
            return 0;
          }
        }
      }

      return 15;
    }

    let sticker = current_normal < self.normals.len();

    assert!(!sticker);
    let sz = 28.5;
    let center_dist = if sticker {sz-1.0 } else { sz };

    for (i, &n) in self.normals.iter().enumerate() {
      let d = dot(pos, n);
      if d > center_dist && i != current_normal {
        return 0;
      }
    }

    let (mut shift_in, mut shift_out, inter) = get_groove(r, &self.groove, 0.03);

    if r > 24.0 {
      let h = r - 24.0;
      shift_in = f32::max(-0.00001, shift_in - h * (r + 10.0) * 0.0007);
      shift_out = f32::max(-0.00001, shift_out - h * (r + 10.0) * 0.0007);
    }

    if sticker {
      shift_in -= 0.03;
      shift_out += 0.03;
    }

    let mut index: PartIndex = 0;
    let mut fail = false;
    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    axis_pos.clear();
    axis_neg.clear();

    let mut rounding_r = 10.0;
    if r > 24.0 {
      rounding_r = f32::max(0.0, rounding_r + (r - 24.0) * 1.5);
    }

    let match_axis = |index: &mut PartIndex,
                      fail: &mut bool,
                      axis_pos: &mut Vec<NearAxis>,
                      axis_neg: &mut Vec<NearAxis>,
                      bit: usize,
                      axis: Point|
     -> bool {
      let d = dot(pos, axis) / r;

      if d > shift_out {
        *index += (1 << bit);
        let dist = 1.0 - (d - shift_out) * rounding_r;
        if dist > 0.0 {
          axis_pos.push(NearAxis { dist, pos: axis });
        }
        return true;
      } else if d > shift_in {
        *fail = true;
        return true;
      } else if d > 0.0 {
        let dist = 1.0 - (shift_in - d) * rounding_r;
        if dist > 0.0 {
          axis_neg.push(NearAxis { dist, pos: axis });
        }
      }
      return false;
    };

    for i in 0..self.axis.len() {
      if match_axis(&mut index, &mut fail, &mut axis_pos, &mut axis_neg, i, self.axis[i]) {
        if fail {
          return 0;
        }
      }
    }

    if index.count_ones() == 1 {
      if r > 24.0 {
        return 0;
      }
      let dist_to_axle = cross(pos, self.axis[index.ilog2() as usize]).len();
      let max_dist_to_axle = if r > 15.0 { 3.2 } else { 1.35 };
      if dist_to_axle < max_dist_to_axle {
        return 0;
      }
    }

    let validate = |ap: &[f32], k: f32| -> bool {
      let d1 = f32::max(0.0, 1.0 - k * (1.0 - ap[0]));
      let d2 = f32::max(0.0, 1.0 - k * (1.0 - ap[1]));
      return d1 * d1 + d2 * d2 < 1.0;
    };

    if axis_pos.len() == 2 && !validate(&[axis_pos[0].dist, axis_pos[1].dist], 1.0) {
      return 0;
    }

    if axis_neg.len() == 2 && !validate(&[axis_neg[0].dist, axis_neg[1].dist], 1.0) {
      return 0;
    }

    if axis_pos.len() == 1
      && axis_neg.len() == 1
      && !inter
      && !validate(&[axis_pos[0].dist, axis_neg[0].dist], 1.0)
    {
      return 0;
    }

    let dx = sz - pos.x.abs();
    let dy = sz - pos.y.abs();
    let dz = sz - pos.z.abs();

    if index.count_ones() == 3 {
      if dz * 0.7 < dx && dz * 0.7 < dy {
        index += 64;
      }
    }

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.faces())
  }
}
