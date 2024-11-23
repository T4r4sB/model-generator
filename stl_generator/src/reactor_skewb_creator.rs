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

pub struct ReactorScewbCreator {
  axis: Vec<Point>,
  normals: Vec<Point>,
  n_dists: Vec<f32>,
  n_basis: Vec<(Point, Point)>,
  groove1: Vec<f32>,
  groove2: Vec<f32>,

  axis_pos: RefCell<Vec<NearAxis>>,
  axis_neg: RefCell<Vec<NearAxis>>,
}

impl ReactorScewbCreator {
  pub fn new() -> Self {
    let axis = [
      Point { x: 1.0, y: 1.0, z: 1.0 },
      Point { x: 1.0, y: -1.0, z: -1.0 },
      Point { x: -1.0, y: 1.0, z: -1.0 },
      Point { x: -1.0, y: -1.0, z: 1.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let normals: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let size = 57.0;
    let small_dist = size * 0.4;
    let big_dist = size - small_dist;

    let n_dists = vec![
      small_dist, small_dist, small_dist, small_dist, small_dist, small_dist,
    ];

    let n_basis = normals
      .iter()
      .map(|&n| {
        let n1 = n.any_perp().norm();
        let n2 = cross(n, n1).norm();
        (n1, n2)
      })
      .collect();

    let shift = small_dist / 12.0.sqrt();

    let groove1 = vec![-shift, small_dist - 6.8, shift];

    let groove2 = vec![
      -shift,
      small_dist - 4.4,
      -shift + 3.0,
      small_dist - 2.0,
      -shift,
    ];

    Self {
      axis,
      normals,
      n_dists,
      n_basis,
      groove1,
      groove2,
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
    let r = pos.len();
    if r > self.n_dists[0] {
      return 0;
    }
    if r < 15.0 {
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

    for i in 0..self.normals.len() {
      let n = self.normals[i];
      let nd = self.n_dists[i];
      let d = dot(pos, n);
      let center_dist = if sticker { nd - 1.0 } else { nd };
      if d > center_dist && i != current_normal {
        return 0;
      }
    }

    let (mut shift_in1, mut shift_out1, inter1) = get_groove(r, &self.groove1, 2.0);
    let (mut shift_in2, mut shift_out2, inter2) = get_groove(r, &self.groove2, 2.0);

    let mut index: PartIndex = 0;
    let mut fail = false;
    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    axis_pos.clear();
    axis_neg.clear();

    let mut rounding_r = 1.0;
    // if r > 24.0 {
    //   rounding_r = f32::max(0.0, rounding_r + (r - 24.0) * 1.5);
    //  }

    let match_axis = |index: &mut PartIndex,
                      fail: &mut bool,
                      axis_pos: &mut Vec<NearAxis>,
                      axis_neg: &mut Vec<NearAxis>,
                      bit: usize,
                      axis: Point|
     -> bool {
      let d = dot(pos, axis);

      /*
      if d > shift_out1 {
        *index += (1 << (bit * 2 + 1));
        let dist = 1.0 - (d - shift_out1) * rounding_r;
        if dist > 0.0 {
          axis_pos.push(NearAxis { dist, pos: axis });
        }
        return true;
      }

      if d > shift_in1 {
        *fail = true;
        return true;
      }*/

      if d > shift_out2 {
        *index += (1 << (bit * 2 + 0));
        let dist = 1.0 - (d - shift_out1) * rounding_r;
        if dist > 0.0 {
          axis_pos.push(NearAxis { dist, pos: axis });
        }
        return true;
      }

      if d > shift_in2 {
        *fail = true;
        return true;
      }

      let dist = 1.0 - (shift_in1 - d) * rounding_r;
      if dist > 0.0 {
        axis_neg.push(NearAxis { dist, pos: axis });
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
      let dist_to_axle = cross(pos, self.axis[index.ilog2() as usize / 2]).len();
      let max_dist_to_axle = if r > 55.0 { 3.2 } else { 1.35 };
      if dist_to_axle < max_dist_to_axle {
        return 0;
      }
    }

    /*
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
      && !inter1
      && !inter2
      && !validate(&[axis_pos[0].dist, axis_neg[0].dist], 1.0)
    {
      return 0;
    }*/

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.faces())
  }
}
