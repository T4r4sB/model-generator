use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;

pub struct SunKittyCreator {
  axis: Vec<Point>,
  basic_angle: f32,
  split_cos: f32,
  split2_cos: f32,
  ball_radius: f32,
}

impl SunKittyCreator {
  pub fn new() -> Self {
    let basic_angle = std::f32::consts::PI * 2.0 / 9.0;
    let rotation_angle = basic_angle * 2.0;
    let cosa = rotation_angle.cos();
    let cos_edge = cosa / (1.0 - cosa);
    let edge_a = cos_edge.acos();

    fn opposite_angle(a1: f32, a2: f32, x: f32) -> f32 {
      (a1.cos() * a2.cos() + a1.sin() * a2.sin() * x.cos()).acos()
    }

    let biggest_angle = opposite_angle(edge_a, edge_a, basic_angle * 3.0);
    let lower_angle = opposite_angle(edge_a, edge_a, basic_angle * 3.5);
    let diamond_angle = opposite_angle(edge_a, edge_a * 0.5, basic_angle);
    let split_angle = biggest_angle * 0.5;
    let ball_radius = 35.0;

    Self {
      axis: vec![
        Point { x: 0.0, y: 0.0, z: 1.0 },
        Point { x: edge_a.sin(), y: 0.0, z: edge_a.cos() },
        Point { x: -edge_a.sin() * 0.5, y: edge_a.sin() * 0.75f32.sqrt(), z: edge_a.cos() },
        Point { x: -edge_a.sin() * 0.5, y: -edge_a.sin() * 0.75f32.sqrt(), z: edge_a.cos() },
        Point { x: -lower_angle.sin(), y: 0.0, z: lower_angle.cos() },
        Point {
          x: lower_angle.sin() * 0.5,
          y: -lower_angle.sin() * 0.75f32.sqrt(),
          z: lower_angle.cos(),
        },
        Point {
          x: lower_angle.sin() * 0.5,
          y: lower_angle.sin() * 0.75f32.sqrt(),
          z: lower_angle.cos(),
        },
      ],
      basic_angle,
      split_cos: split_angle.cos(),
      split2_cos: (split_angle - 3.0 / ball_radius).cos(),
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
    if depth > 10.0 {
      return 0x80000000;
    }

    let mut index: PartIndex = 0x80000000;

    let split_cos = if depth < 4.0 && depth > 2.0 {
      self.split2_cos
    } else {
      self.split_cos
    };

    let match_axis = |index: &mut PartIndex, bit: usize, axis: Point| -> bool {
      let d = dot(pos, axis);
      if d > split_cos * r {
        *index += (1 << bit);
        return true;
      }
      return false;
    };

    for i in 1..self.axis.len() {
      if match_axis(&mut index, i, self.axis[i]) {
        let dist_to_axle = cross(pos, self.axis[i]).len();
        if dist_to_axle < 1.25 {
          return 0;
        }
      }
    }

    if (index & 2) != 0 {
      match_axis(&mut index, 7, self.axis[5].rotate(self.axis[1], self.basic_angle));
      match_axis(&mut index, 8, self.axis[5].rotate(self.axis[1], self.basic_angle * 3.0));
      match_axis(&mut index, 9, self.axis[5].rotate(self.axis[1], self.basic_angle * 4.0));
      match_axis(&mut index, 10, self.axis[5].rotate(self.axis[1], self.basic_angle * 5.0));
      match_axis(&mut index, 11, self.axis[5].rotate(self.axis[1], self.basic_angle * 6.0));
      match_axis(&mut index, 12, self.axis[5].rotate(self.axis[1], self.basic_angle * 7.0));
      match_axis(&mut index, 13, self.axis[5].rotate(self.axis[1], self.basic_angle * 8.0));
    }

    if (index & 4) != 0 {
      match_axis(&mut index, 14, self.axis[6].rotate(self.axis[2], self.basic_angle));
      match_axis(&mut index, 15, self.axis[6].rotate(self.axis[2], self.basic_angle * 3.0));
      match_axis(&mut index, 16, self.axis[6].rotate(self.axis[2], self.basic_angle * 4.0));
      match_axis(&mut index, 17, self.axis[6].rotate(self.axis[2], self.basic_angle * 5.0));
      match_axis(&mut index, 18, self.axis[6].rotate(self.axis[2], self.basic_angle * 6.0));
      match_axis(&mut index, 19, self.axis[6].rotate(self.axis[2], self.basic_angle * 7.0));
      match_axis(&mut index, 20, self.axis[6].rotate(self.axis[2], self.basic_angle * 8.0));
    }

    if (index & 8) != 0 {
      match_axis(&mut index, 21, self.axis[4].rotate(self.axis[3], self.basic_angle));
      match_axis(&mut index, 22, self.axis[4].rotate(self.axis[3], self.basic_angle * 3.0));
      match_axis(&mut index, 23, self.axis[4].rotate(self.axis[3], self.basic_angle * 4.0));
      match_axis(&mut index, 24, self.axis[4].rotate(self.axis[3], self.basic_angle * 5.0));
      match_axis(&mut index, 25, self.axis[4].rotate(self.axis[3], self.basic_angle * 6.0));
      match_axis(&mut index, 26, self.axis[4].rotate(self.axis[3], self.basic_angle * 7.0));
      match_axis(&mut index, 27, self.axis[4].rotate(self.axis[3], self.basic_angle * 8.0));
    }

    if (index & 16) != 0 {
      match_axis(&mut index, 7, self.axis[3].rotate(self.axis[4], self.basic_angle));
      match_axis(&mut index, 8, self.axis[3].rotate(self.axis[4], self.basic_angle * 2.0));
      match_axis(&mut index, 9, self.axis[3].rotate(self.axis[4], self.basic_angle * 4.0));
      match_axis(&mut index, 10, self.axis[3].rotate(self.axis[4], self.basic_angle * 6.0));
      match_axis(&mut index, 11, self.axis[3].rotate(self.axis[4], self.basic_angle * 8.0));
    }

    if (index & 32) != 0 {
      match_axis(&mut index, 14, self.axis[1].rotate(self.axis[5], self.basic_angle));
      match_axis(&mut index, 15, self.axis[1].rotate(self.axis[5], self.basic_angle * 2.0));
      match_axis(&mut index, 16, self.axis[1].rotate(self.axis[5], self.basic_angle * 4.0));
      match_axis(&mut index, 17, self.axis[1].rotate(self.axis[5], self.basic_angle * 6.0));
      match_axis(&mut index, 18, self.axis[1].rotate(self.axis[5], self.basic_angle * 8.0));
    }

    if (index & 64) != 0 {
      match_axis(&mut index, 21, self.axis[2].rotate(self.axis[6], self.basic_angle));
      match_axis(&mut index, 22, self.axis[2].rotate(self.axis[6], self.basic_angle * 2.0));
      match_axis(&mut index, 23, self.axis[2].rotate(self.axis[6], self.basic_angle * 4.0));
      match_axis(&mut index, 24, self.axis[2].rotate(self.axis[6], self.basic_angle * 6.0));
      match_axis(&mut index, 25, self.axis[2].rotate(self.axis[6], self.basic_angle * 8.0));
    }

    if index == 0x80000000 {
      if match_axis(&mut index, 0, self.axis[0]) {
        let dist_to_axle = cross(pos, self.axis[0]).len();
        if dist_to_axle < 1.25 {
          return 0;
        }

        match_axis(&mut index, 7, self.axis[1].rotate(self.axis[0], self.basic_angle));
        match_axis(&mut index, 8, self.axis[1].rotate(self.axis[0], self.basic_angle * 2.0));
        match_axis(&mut index, 9, self.axis[1].rotate(self.axis[0], self.basic_angle * 3.0));
        match_axis(&mut index, 10, self.axis[1].rotate(self.axis[0], self.basic_angle * 4.0));
        match_axis(&mut index, 11, self.axis[1].rotate(self.axis[0], self.basic_angle * 5.0));
        match_axis(&mut index, 12, self.axis[1].rotate(self.axis[0], self.basic_angle * 6.0));
        match_axis(&mut index, 13, self.axis[1].rotate(self.axis[0], self.basic_angle * 7.0));
        match_axis(&mut index, 14, self.axis[1].rotate(self.axis[0], self.basic_angle * 8.0));
      }
    }

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.axis.len())
  }
}
