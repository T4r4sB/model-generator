use crate::model::*;
use crate::points3d::*;
use crate::solid::*;
use num::Float;
use num::PrimInt;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

fn sqr(x: f32) -> f32 {
  x * x
}

fn reflect(pos: Point, p1: Point, p2: Point) -> Point {
  let n = cross(p1, p2).norm();
  pos - n.scale(dot(n, pos) * 2.0)
}

pub struct LunaMinxPlusCreator {
  axis: Vec<Point>,
  normals: Vec<Point>,

  split_cos: f32,
  split2_cos: f32,
  ball_radius: f32,
}

impl LunaMinxPlusCreator {
  pub fn new() -> Self {
    let ball_radius = 35.0;

    let split2_angle = 0.60.acos() + 1.0 / ball_radius;
    let split_angle = split2_angle + 3.0 / ball_radius;

    let tr = 0.75.sqrt();

    let rc = 0.2.sqrt().sqrt();
    let rs = (1.0 - sqr(rc)).sqrt();
    let mut axis = Vec::new();
    for i in 0..5 {
      let a = i as f32 * PI * 2.0 / 5.0;
      let (s, c) = a.sin_cos();

      axis.push(Point { x: c * rs, y: s * rs, z: rc });
      axis.push(Point { x: -c * rs, y: -s * rs, z: -rc });
    }

    let mut normals = Vec::new();
    normals.push(Point { x: 0.0, y: 0.0, z: 1.0 });
    normals.push(Point { x: 0.0, y: 0.0, z: -1.0 });
    let rc2 = 2.0 * sqr(rc) - 1.0;
    let rs2 = (1.0 - sqr(rc2)).sqrt();

    for i in 0..5 {
      let a = i as f32 * PI * 2.0 / 5.0;
      let (s, c) = a.sin_cos();

      normals.push(Point { x: c * rs2, y: s * rs2, z: rc2 });
      normals.push(Point { x: -c * rs2, y: -s * rs2, z: -rc2 });
    }

    let normals2 = vec![
      reflect(normals[0], axis[0], axis[4]),
      reflect(normals[0], axis[2], axis[6]),
      reflect(normals[0], axis[4], axis[8]),
      reflect(normals[0], axis[6], axis[0]),
      reflect(normals[0], axis[8], axis[2]),
    ];
    
    let normals3 = vec![
      reflect(normals[1], axis[1], axis[5]),
      reflect(normals[1], axis[3], axis[7]),
      reflect(normals[1], axis[5], axis[9]),
      reflect(normals[1], axis[7], axis[1]),
      reflect(normals[1], axis[9], axis[3]),
    ];

    normals.extend(normals2);
    normals.extend(normals3);

    Self {
      axis,
      normals,
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
    // if pos.x > 0.0 { return 0; }
    let r = pos.len();

    if r > 64.5 {
      return 0;
    }

    let depth = self.ball_radius - r;
    if depth < -self.ball_radius * 2.5 {
      return 0;
    }

    let shaft_r = if depth > 3.0 {
      1.25
    } else if depth > -6.0 {
      1.5
    } else if depth > -8.0 {
      3.1
    } else {
      0.0
    };

    for i in 0..self.axis.len() {
      if dot(pos, self.axis[i]) > 0.0 {
        let dist_to_axle = cross(pos, self.axis[i]).len();
        if dist_to_axle < shaft_r {
          return 0;
        }
      }
    }

    if depth > 3.0 {
      return 0x80000000;
    }

    for ni in 0..self.normals.len() {
      let s = cross(pos, self.normals[ni]).len();
      let c = dot(pos, self.normals[ni]);
      let m = self.ball_radius + 4.8;
      if c > m {
        return 0;
      }
    }

    let rc = self.normals[2];
    if dot(pos, rc) > 0.0 && cross(pos, rc).len() < 1.5 {
      return 0;
    }
    let rc = self.axis[0];
    if dot(pos, rc) > 0.0 && cross(pos, rc).len() < 1.5 {
      return 0;
    }

    let split_cos = if depth < 0.0 && depth > -2.4 {
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

    let mut index: PartIndex = 0;

    for i in 0..self.axis.len() {
      match_axis(pos, &mut index, i, self.axis[i]);
    }

    if index & 32 == 32 {
      match_axis(pos, &mut index, 11, self.axis[0].rotate(self.normals[2], PI * 2.0 / 5.0));
    }

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.axis.len())
  }
}
