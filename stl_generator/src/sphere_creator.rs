use crate::model::*;
use crate::points3d::*;
use crate::solid::*;
use num::Float;

use std::cell::RefCell;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;

pub struct SphereCreator {
  a: Vec<Point>,
  ar: Vec<Point>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

fn reflect(p0: Point, p1: Point, p2: Point) -> Point {
  let n = cross(p1,p2).norm();
  p0 - n.scale(2.0* dot(n, p0))
}

impl SphereCreator {
  pub fn new() -> Self {
    let h = 1.0;
    let sq3 = 0.75.sqrt();
    let a: Vec::<_> = [
      Point { x:  1.0, y:  0.0, z:  h * 0.5 },
      Point { x:  0.5, y:  sq3, z: -h * 0.5 },
      Point { x: -0.5, y:  sq3, z:  h * 0.5 },
      Point { x: -1.0, y:  0.0, z: -h * 0.5 },
      Point { x: -0.5, y: -sq3, z:  h * 0.5 },
      Point { x:  0.5, y: -sq3, z: -h * 0.5 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let ar = vec![
      reflect(a[0], a[2], a[4]),
      reflect(a[1], a[3], a[5]),
      reflect(a[2], a[4], a[0]),
      reflect(a[3], a[5], a[1]),
      reflect(a[4], a[0], a[2]),
      reflect(a[5], a[1], a[3]),
    ];

    Self { a, ar }
  }

  pub fn faces(&self) -> usize {
    0
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    let r = pos.len();
    if r > 28.0 {
      return 0;
    }
    if r < 20.0 { return 1 << 8; }

    let c = 0.1;

    let mut index = 0;
    for (i, &a) in self.a.iter().enumerate() {
      if dot(pos, a) > r * c {
        index += (1 << i) as PartIndex;
      }
    }


    for i in 0 .. self.a.len() {
      let i1 = (i + 2) % 6;
      let i2 = (i + 4) % 6;
      let mask = (1 << i1) + (1 << i2);
      if index & mask == mask {
        if dot(pos, self.ar[i]) > r * c {
          index += (1 << (i + 8)) as PartIndex;
        }

      }
    }

    return index;
  }
}
