use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;

pub struct SphereCreator {
  a: Vec<Point>,
  n: Vec<Point>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

pub fn reflect(p: Point, a: Point, b: Point) -> Point {
  let c = cross(a, b).norm();
  p - c.scale(2.0 * dot(p, c))
}

pub fn reflectp(p: Point, a: Point, b: Point) -> Point {
  let c = (a + b).norm();
  c.scale(2.0 * dot(p, c)) - p
}

impl SphereCreator {
  pub fn new() -> Self {
    let edge = 0.26120387496374137;
    let c = ((edge * 2.0 + 1.0) / 3.0).sqrt();
    let s = (1.0-sqr(c)).sqrt();
    let sq3 = 0.75.sqrt();


    let mut a: Vec<_> = [
      Point { x: 0.0, y: s, z: c },
      Point { x: s*sq3, y: -s*0.5, z: c },
      Point { x: -s*sq3, y: -s*0.5, z: c },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    a.push(reflect(a[0], a[1], a[2]));
    a.push(reflect(a[1], a[2], a[0]));
    a.push(reflect(a[2], a[0], a[1]));

    let n = vec![
      Point { x: 1.0, y: 1.0, z: 1.0 },
      Point { x: 1.0, y: -1.0, z: -1.0 },
      Point { x: -1.0, y: 1.0, z: -1.0 },
      Point { x: -1.0, y: -1.0, z: 1.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();
    Self { a, n }
  }

  pub fn faces(&self) -> usize {
    0
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

  pub fn get_quality() -> usize {
    100
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    let r = pos.len();
    if r > 45.0 {
      return 0;
    }
    let mut index: PartIndex = 0;
    for i in 0..self.a.len() {
      let a = &self.a[i];
      if dot(pos, *a) > 0.0 * r {
        index += 1 << i;
      }
    }

    return index;
  }
}
