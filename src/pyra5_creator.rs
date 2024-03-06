use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;

pub struct Pyra5Creator {
  axis: Vec<Point>,
  normals: Vec<Point>,

  a32_l: Point,
  a32_r: Point,
  a31_l: Point,
  a31_r: Point,
  a42_l: Point,
  a42_r: Point,
  a41_l: Point,
  a41_r: Point,
  split_cos: f32,
  split2_cos: f32,
  ball_radius: f32,
}

impl Pyra5Creator {
  pub fn new() -> Self {
    let ball_radius = 24.0;

    let split2_angle = (1.0f32 / 7.0f32).sqrt().acos() + 1.0 / ball_radius;
    let split_angle = split2_angle + 2.0 / ball_radius;

    let axis = vec![
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: -0.75f32.sqrt(), z: -0.5 },
      Point { x: 0.0, y: 0.75f32.sqrt(), z: -0.5 },
      Point { x: -0.75f32.sqrt(), y: 0.0, z: -0.5 },
      Point { x: 0.75f32.sqrt(), y: 0.0, z: -0.5 },
    ];

    let normals =  vec![
      Point { x: 1.0, y: 1.0, z: 1.0 }.norm(),
      Point { x: 1.0, y: 1.0, z: -1.0 }.norm(),
      Point { x: 1.0, y: -1.0, z: 1.0 }.norm(),
      Point { x: 1.0, y: -1.0, z: -1.0 }.norm(),
      Point { x: -1.0, y: 1.0, z: 1.0 }.norm(),
      Point { x: -1.0, y: 1.0, z: -1.0 }.norm(),
      Point { x: -1.0, y: -1.0, z: 1.0 }.norm(),
      Point { x: -1.0, y: -1.0, z: -1.0 }.norm(),
    ];

    let a32_l = axis[2].rotate(axis[3], 2.0 * (-0.6f32).acos());
    let a32_r = axis[3].rotate(axis[2], -2.0 * (-0.6f32).acos());
    let a31_l = axis[1].rotate(axis[3], -2.0 * (-0.6f32).acos());
    let a31_r = axis[3].rotate(axis[1], 2.0 * (-0.6f32).acos());
    let a42_l = axis[2].rotate(axis[4], -2.0 * (-0.6f32).acos());
    let a42_r = axis[4].rotate(axis[2], 2.0 * (-0.6f32).acos());
    let a41_l = axis[1].rotate(axis[4], 2.0 * (-0.6f32).acos());
    let a41_r = axis[4].rotate(axis[1], -2.0 * (-0.6f32).acos());

    Self {
      axis,
      normals,
      a32_l,
      a32_r,
      a31_l,
      a31_r,
      a42_l,
      a42_r,
      a41_l,
      a41_r,
      split_cos: split_angle.cos(),
      split2_cos: split2_angle.cos(),
      ball_radius,
    }
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn faces(&self) -> usize {
    0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    let depth = self.ball_radius - r;
    if depth < 0.0 {
      return 0;
    }
    if depth > 6.0 {
      return 0x80000000;
    }

    let mut index: PartIndex = 0;

    let split_cos = if depth < 4.0 && depth > 2.0 {
      self.split2_cos
    } else {
      self.split_cos
    };

    let match_axis = |pos: Point, index: &mut PartIndex, bit: usize, axis: Point| -> bool {
      let d = dot(pos, axis);

      if d > split_cos * r {
        *index += (1 << bit);
        return true;
      }
      return false;
    };

    let mut pos = pos;
    let mut ii = 0;

    for i in 0..self.axis.len() {
      if match_axis(pos, &mut ii, i, self.axis[i]) {
        let dist_to_axle = cross(pos, self.axis[i]).len();
        if dist_to_axle < 1.25 {
          return 0;
        }
      }
    }

/*
    if match_axis(pos, &mut ii, 1, self.axis[1]) {
      pos = pos.rotate(self.axis[1], -(-3.0 / 5.0f32).acos());
    }
    if match_axis(pos, &mut ii, 3, self.axis[3]) {
      pos = pos.rotate(self.axis[3], (-3.0 / 5.0f32).acos());
    }
    if match_axis(pos, &mut ii, 1, self.axis[1]) {
      pos = pos.rotate(self.axis[1], (-3.0 / 5.0f32).acos());
    }*/

    for &n in &self.normals {
      if dot(pos, n) > self.ball_radius {
        return 0;
      }
    }

    for i in 0..self.axis.len() {
      match_axis(pos, &mut index, i, self.axis[i]);
    }
    if index == 1 << 3 | 1 << 2 {
      match_axis(pos, &mut index, 5, self.a31_l);
      match_axis(pos, &mut index, 6, self.a42_r);
    }

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.axis.len())
  }
}
