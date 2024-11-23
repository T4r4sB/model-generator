use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;
use std::ops::DerefMut;

pub struct HouseCreator {}

const PI: f32 = std::f32::consts::PI;

pub fn sqr(x: f32) -> f32 {
  x * x
}

pub fn max(a: &[f32]) -> f32 {
  let mut result = f32::MIN;
  for &a in a {
    result = f32::max(result, a)
  }
  result
}

pub fn min(a: &[f32]) -> f32 {
  let mut result = f32::MAX;
  for &a in a {
    result = f32::min(result, a)
  }
  result
}

/*
pub fn max(a: &[f32]) -> f32 {
  let mut result = 0.0;
  for &a in a {
    result += a.powi(5)
  }
  result.powf(0.2)
}

pub fn min(a: &[f32]) -> f32 {
  let mut result = 0.0;
  for &a in a {
    result += a.powi(-5)
  }
  result.powf(-0.2)
}*/

pub fn in_plate(pos: Point, s: f32, h: f32) -> f32 {
  max(&[pos.x.abs() / s, pos.y.abs() / s, pos.z.abs() / h])
}

impl HouseCreator {
  pub fn new() -> Self {
    Self {}
  }

  pub fn faces(&self) -> usize {
    0
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    let wall = 3.5;
    let win = 8.0;

    let (s, c) = (pos.z * 0.5).sin_cos();
    let pos = pos + Point { x: c * 3.0, y: s * 3.0, z: 0.0 };

    let pos1 = pos - Point { x: 0.0, y: 0.0, z: -40.0 };
    let p1 = in_plate(pos1, 20.0, wall);

    let axle = Point { x: 0.0, y: 1.0, z: 0.0 };

    let pos2 = pos - Point { x: 20.0, y: 0.0, z: -20.0 };
    let pos2 = pos2.rotate(axle, PI * 0.5);
    let p2 = in_plate(pos2, 20.0, wall);
    let w2 = in_plate(pos2, 10.0, win);
    let w2 = 1.1 / (w2 + 0.1);

    let pos3 = pos - Point { x: -20.0, y: 0.0, z: -20.0 };
    let pos3 = pos3.rotate(axle, PI * 0.5);
    let p3 = in_plate(pos3, 20.0, wall);
    let w3 = in_plate(pos3, 10.0, win);
    let w3 = 1.1 / (w3 + 0.1);

    let pos4 = pos - Point { x: 14.0, y: 0.0, z: 6.3 };
    let pos4 = pos4.rotate(axle, -PI * 0.25);
    let h = pos.x.rem_euclid(2.0) * 0.5 + (pos.y).sin();
    let pos4 = pos4 - Point { x: 0.0, y: 0.0, z: h * 0.3 };
    let p4 = in_plate(pos4, 25.0, wall);

    let pos5 = pos - Point { x: -14.0, y: 0.0, z: 6.3 };
    let pos5 = pos5.rotate(axle, PI * 0.25);
    let h = pos.x.rem_euclid(2.0) * 0.5 + (pos.y).sin();
    let pos5 = pos5 + Point { x: 0.0, y: 0.0, z: h * 0.3 };
    let p5 = in_plate(pos5, 25.0, wall);

    let axle = Point { x: 1.0, y: 0.0, z: 0.0 };
    let pos6 = pos - Point { x: 0.0, y: 20.0, z: -20.0 };
    let pos6 = pos6.rotate(axle, PI * 0.5);
    let p6 = in_plate(pos6, 20.0, wall);
    let w6 = in_plate(pos6, 10.0, win);
    let w6 = 1.1 / (w6 + 0.1);

    let pos7 = pos - Point { x: 0.0, y: -20.0, z: -20.0 };
    let pos7 = pos7.rotate(axle, PI * 0.5);
    let p7 = in_plate(pos7, 20.0, wall);

    let pos7 = pos - Point { x: 0.0, y: -20.0, z: -28.0 };
    let pos7 = pos7.rotate(axle, PI * 0.5);
    let w7 = in_plate(pos7, 12.0, win);
    let w7 = 1.1 / (w7 + 0.1);

    let f = min(&[
      p1,
      max(&[p2, w2]),
      max(&[p3, w3]),
      p4,
      p5,
      max(&[p6, w6]),
      max(&[p7, w7]),
    ]);

    let wave = 0.0;
    /*
    let wave = wave + (pos - Point { x: -11.0, y: -10.0, z: 0.0 }).len().sin();
    let wave = wave + (pos - Point { x: 9.0, y: 10.0, z: 0.0 }).len().sin();
    let wave = wave + (pos - Point { x: -10.0, y: 9.0, z: 20.0 }).len().sin();
    let wave = wave + (pos - Point { x: 10.0, y: -9.0, z: 20.0 }).len().sin();
    */

    (f < 1.0 + wave * 0.05) as PartIndex
  }
}
