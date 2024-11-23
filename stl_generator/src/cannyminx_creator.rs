use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;

pub struct CannyminxCreator {
  axis: Vec<Point>,
  basic_axis_len: usize,
  basic_angle: f32,
  split_cos: f32,
  split2_cos: f32,
  ball_radius: f32,
}

impl CannyminxCreator {
  pub fn new() -> Self {
    let basic_angle = std::f32::consts::PI / 5.0;
    let basic_cos = (basic_angle * 2.0).cos();
    let edge_length = (basic_cos / (1.0 - basic_cos)).acos();

    fn opposite_angle(a1: f32, a2: f32, x: f32) -> f32 {
      (a1.cos() * a2.cos() + a1.sin() * a2.sin() * x.cos()).acos()
    }

    let edge2_length = opposite_angle(edge_length, edge_length, basic_angle * 4.0);
    let corner_angle = ((edge_length.cos() + 0.5) * 2.0 / 3.0).sqrt().acos();
    let diamond_angle = opposite_angle(edge_length, edge_length * 0.5, basic_angle);
   // let max_split_angle = opposite_angle(edge_length, edge_length, basic_angle * 3.0) * 0.5;
   let max_split_angle =  std::f32::consts::PI / 4.0;
    // panic!("reccomended radous={}", 4.0 / (max_split_angle - diamond_angle));
    let ball_radius = 50.0;
    let split_angle = max_split_angle - 3.0 / ball_radius;

    let mut axis = Vec::new();
    axis.push(Point { x: 0.0, y: 0.0, z: 1.0 });
    for step in [9, 1, 4, 6] {
      axis.push(Point {
        x: edge_length.sin() * (basic_angle * step as f32).cos(),
        y: edge_length.sin() * (basic_angle * step as f32).sin(),
        z: edge_length.cos(),
      });
    }

    for step in [0, 5] {
      axis.push(Point {
        x: -edge_length.sin() * (basic_angle * step as f32).cos(),
        y: -edge_length.sin() * (basic_angle * step as f32).sin(),
        z: -edge_length.cos(),
      });
    }

    axis.push(Point {x: 0.0, y: 1.0, z: 0.0 });
    axis.push(Point {x: 0.0, y: -1.0, z: 0.0 });
    axis.push(Point {x: 0.0, y: 0.0, z: -1.0 });

    Self {
      basic_axis_len: axis.len(),
      axis,
      basic_angle,
      split_cos: split_angle.cos(),
      split2_cos: max_split_angle.cos(),
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
    if depth < -self.ball_radius * 0.3 {
      return 0;
    }
    if depth > 6.0 {
      return 0x80000000;
    }

    for i in 0..self.basic_axis_len {}

    let mut index: PartIndex = 0;
    let mut out_of_shape = false;

    let split_cos = if depth < 0.0 {
      self.split2_cos
    } else if depth < 2.0 || depth > 4.0 {
      self.split2_cos
    } else {
      self.split_cos
    };

    let match_axis = |index: &mut PartIndex, bit: usize, axis: Point, out_of_shape: &mut bool| -> bool {
      if *out_of_shape {
        return false;
      }
      let d = dot(pos, axis);

      if d > split_cos * r {
        if dot(pos, axis) > self.ball_radius {
          *out_of_shape = true;
        } else {
          *index += (1 << bit);
        }

        return true;
      }
      return false;
    };

    for i in 0..self.basic_axis_len {
      if match_axis(&mut index, i, self.axis[i], &mut out_of_shape) {
        let dist_to_axle = cross(pos, self.axis[i]).len();
        if dist_to_axle < 1.25 {
          return 0;
        }
      }
    }

    if index.count_ones() == 2 {
      let a1 = index.ilog2() as usize;
      let a2 = (index - (1 << a1)).ilog2() as usize;
      let n1 = self.axis[a1].rotate(self.axis[a2], self.basic_angle * 2.0);
      let n2 = self.axis[a2].rotate(self.axis[a1], self.basic_angle * 2.0);
      if dot(pos, n1) > self.ball_radius || dot(pos, n2) > self.ball_radius {
        return 0;
      }
    }

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.axis.len())
  }
}
