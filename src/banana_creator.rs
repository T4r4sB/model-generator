use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;

pub struct BananaCreator {
  axis: Vec<Point>,
  basic_angle: f32,
  split_cos: f32,
  split2_cos: f32,
  ball_radius: f32,
}

impl BananaCreator {
  pub fn new() -> Self {
    let basic_angle = std::f32::consts::PI * 2.0 / 4.0;
    let rotation_angle = basic_angle * 2.0;
    let cosa = rotation_angle.cos();
    let cos_edge = cosa / (1.0 - cosa);
    let edge_a = cos_edge.acos();

    let cross_y = edge_a * 0.5;
    let cross_x = (cos_edge / cross_y.cos()).acos();

    fn opposite_angle(a1: f32, a2: f32, x: f32) -> f32 {
      (a1.cos() * a2.cos() + a1.sin() * a2.sin() * x.cos()).acos()
    }

    let diamond_angle = opposite_angle(edge_a, edge_a * 0.5, basic_angle);
    let ball_radius = 33.23;

    let split_angle = diamond_angle + 1.00 / ball_radius;

    Self {
      axis: vec![
        Point { x: 1.0, y: 0.0, z: 0.0 }.norm(),
        Point { x: -0.5, y: 0.75f32.sqrt(), z: 0.0 }.norm(),
        Point { x: -0.5, y: -0.75f32.sqrt(), z: 0.0 }.norm(),
      ],
      basic_angle,
      split_cos: split_angle.cos(),
      split2_cos: (split_angle + 3.0 / ball_radius).cos(),
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
      self.split_cos
    } else {
      self.split2_cos
    };

    let match_axis = |index: &mut PartIndex, bit: usize, axis: Point| -> bool {
      let d = dot(pos, axis);

      if d > split_cos * r {
        *index += (1 << bit);
        return true;
      }
      return false;
    };

    for i in 0..self.axis.len() {
      if match_axis(&mut index, i, self.axis[i]) {
        let dist_to_axle = cross(pos, self.axis[i]).len();
        if dist_to_axle < 1.25 {
          return 0;
        }
      }
    }

    if (index & 1) != 0 {
      match_axis(&mut index, 4, self.axis[1].rotate(self.axis[0], self.basic_angle));
      match_axis(&mut index, 5, self.axis[1].rotate(self.axis[0], self.basic_angle * 3.0));
    }
    if (index & 2) != 0 {
      match_axis(&mut index, 7, self.axis[0].rotate(self.axis[1], self.basic_angle));
      match_axis(&mut index, 8, self.axis[0].rotate(self.axis[1], self.basic_angle * 3.0));
    }
    if (index & 4) != 0 {
      match_axis(&mut index, 10, self.axis[0].rotate(self.axis[2], self.basic_angle));
      match_axis(&mut index, 11, self.axis[0].rotate(self.axis[2], self.basic_angle * 3.0));
    }
    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.axis.len())
  }
}
