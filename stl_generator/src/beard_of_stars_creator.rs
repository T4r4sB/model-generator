use crate::model::*;
use crate::points3d::*;
use crate::solid::*;
use num::Float;

use std::cell::RefCell;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct BeardOfStarsCreator {
  axis: Vec<Point>,
  axis_add: Vec<Point>,
  normals: Vec<Point>,
  split_angle: f32,
  split_angle2: f32,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}
impl BeardOfStarsCreator {
  pub fn new() -> Self {
    let sq5 = 5.0f32.sqrt();
    let a = (2.0 * sq5 - 4.0).sqrt();

    let axis = vec![
      Point { x: 0.0, y: a, z: -1.0 }.norm(),
      Point { x: 0.0, y: -a, z: -1.0 }.norm(),
      Point { x: a, y: 0.0, z: 1.0 }.norm(),
      Point { x: -a, y: 0.0, z: 1.0 }.norm(),
    ];

    let p = (2.0 / sq5).sqrt();
    let q = (1.0 - sqr(p)).sqrt();

    let b = dot(axis[0], axis[2]);
    let s = dot(axis[0], axis[1]);
    let ca = (s - sqr(b)) / (1.0 - sqr(b));
    let ra = ca.acos();
    println!("ra={ra}");

    let mut normals = vec![
      Point { x: 0.0, y: p, z: q },
      Point { x: 0.0, y: -p, z: q },
      Point { x: p, y: 0.0, z: -q },
      Point { x: -p, y: 0.0, z: -q },
      /*
      Point { x: 0.0, y: 0.0, z: 1.4},
      Point { x: 0.0, y: 0.0, z: -1.4},*/
    ];

    normals.push(normals[0].rotate(axis[3], -ra));
    normals.push(normals[1].rotate(axis[2], -ra));
    normals.push(normals[2].rotate(axis[1], -ra));
    normals.push(normals[3].rotate(axis[0], -ra));

    let main_angle = std::f32::consts::PI * 2.0 / 5.0;

    let axis_add = vec![
      axis[0].rotate(normals[0], main_angle),
      axis[0].rotate(normals[0], main_angle * 4.0),
      axis[1].rotate(normals[1], main_angle),
      axis[1].rotate(normals[1], main_angle * 4.0),
      axis[2].rotate(normals[2], main_angle),
      axis[2].rotate(normals[2], main_angle * 4.0),
      axis[3].rotate(normals[3], main_angle),
      axis[3].rotate(normals[3], main_angle * 4.0),
    ];

    let split_angle = 1.51.cos();
    let split_angle2 = (1.51 - 3.0 / 30.0).cos();

    Self { axis, axis_add, normals, split_angle, split_angle2 }
  }

  pub fn faces(&self) -> usize {
    self.normals.len()
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.normals.len())
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 64.999 || pos.y.abs() > 64.999 || pos.z.abs() > 64.999 {
      return 0;
    }

    let sticker = current_normal < self.normals.len();
    let mut index = 0;

    let mut m = 30.0;
    for ni in 0..self.normals.len() {
      let s = cross(pos, self.normals[ni]).len();
      let c = dot(pos, self.normals[ni]);

      if c > m{
      
     // if sqr(c + m * 2.5) + sqr(s) > sqr(m + m * 2.5) {
        return 0;
      }
    }

    for ai in 0..self.axis.len() {
      let s = cross(pos, self.axis[ai]).len();
      let c = dot(pos, self.axis[ai]);
      if c > 0.0 && s < 2.0 {
        return 0;
      }
    }

    if r > 64.0 {
      return 0;
    }
    if r < 24.0 {
      return 100500;
    }

    let sa = if r < 28.0 && r > 26.0 {
      self.split_angle2
    } else {
      self.split_angle
    };

    for (i, &a) in self.axis.iter().enumerate() {
      if dot(pos, a) > r * sa {
        index += (1 << i);
      }
    }

    if (index & 5) == 5 {
      if dot(pos, self.axis_add[0]) > r * sa {
        index += (1 << 4);
      }
    }

    if (index & 8) == 8 {
      // 9
      if dot(pos, self.axis_add[1]) > r * sa {
        index += (1 << 5);
      }
    }

    if (index & 10) == 10 {
      if dot(pos, self.axis_add[2]) > r * sa {
        index += (1 << 6);
      }
    }

    if (index & 4) == 4 {
      // 6
      if dot(pos, self.axis_add[3]) > r * sa {
        index += (1 << 7);
      }
    }

    if (index & 5) == 5 {
      if dot(pos, self.axis_add[4]) > r * sa {
        index += (1 << 8);
      }
    }

    if (index & 2) == 2 {
      // 6
      if dot(pos, self.axis_add[5]) > r * sa {
        index += (1 << 9);
      }
    }

    if (index & 10) == 10 {
      if dot(pos, self.axis_add[6]) > r * sa {
        index += (1 << 10);
      }
    }

    if (index & 1) == 1 {
      // 8
      if dot(pos, self.axis_add[7]) > r * sa {
        index += (1 << 11);
      }
    }

    return index;
  }
}
