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

pub struct OctopusCreator {
  axis: Vec<Point>,
  normals: Vec<Point>,

  cone_angle: f32,
  screw_diam: f32,
  head_diam: f32,
  thread_diam: f32,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl OctopusCreator {
  pub fn new() -> Self {
    let phi = (5.0.sqrt() - 1.0) * 0.5;

    let normals = [
      Point { x: 0.0, y: phi, z: 1.0 },
      Point { x: 0.0, y: -phi, z: 1.0 },
      Point { x: 0.0, y: phi, z: -1.0 },
      Point { x: 0.0, y: -phi, z: -1.0 },
      Point { x: 1.0, y: 0.0, z: phi },
      Point { x: 1.0, y: 0.0, z: -phi },
      Point { x: -1.0, y: 0.0, z: phi },
      Point { x: -1.0, y: 0.0, z: -phi },
      Point { x: phi, y: 1.0, z: 0.0 },
      Point { x: -phi, y: 1.0, z: 0.0 },
      Point { x: phi, y: -1.0, z: 0.0 },
      Point { x: -phi, y: -1.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let phi2 = sqr(phi);

    let dcorners: Vec<_> = [
      Point { x: 1.0, y: 1.0, z: 1.0 },
      Point { x: phi2, y: 0.0, z: 1.0 },
      Point { x: -phi2, y: 0.0, z: 1.0 },
      Point { x: -1.0, y: 1.0, z: 1.0 },
      Point { x: 0.0, y: 1.0, z: phi2 },
      Point { x: 0.0, y: 1.0, z: -phi2 },
      Point { x: 1.0, y: 1.0, z: -1.0 },
      Point { x: phi2, y: 0.0, z: -1.0 },
      Point { x: 1.0, y: -1.0, z: -1.0 },
      Point { x: 0.0, y: -1.0, z: -phi2 },
      Point { x: -1.0, y: -1.0, z: -1.0 },
      Point { x: -phi2, y: 0.0, z: -1.0 },
      Point { x: -1.0, y: 1.0, z: -1.0 },
      Point { x: -1.0, y: phi2, z: 0.0 },
      Point { x: -1.0, y: -phi2, z: 0.0 },
      Point { x: -1.0, y: -1.0, z: 1.0 },
      Point { x: 0.0, y: -1.0, z: phi2 },
      Point { x: 1.0, y: -1.0, z: 1.0 },
      Point { x: 1.0, y: -phi2, z: 0.0 },
      Point { x: 1.0, y: phi2, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let t = (-phi + (3.0 * phi + 1.0).sqrt()) / 2.0;

    let mut axis = Vec::new();
    for i in 0..dcorners.len() {
      let ci = dcorners[i];
      let ni = dcorners[(i + 1) % dcorners.len()];
      axis.push((ci + ni.scale(t)).norm());
    }

    let cone_angle = ((dot(axis[2], axis[4]) + 1.0) / 2.0).sqrt();
    let screw_diam = 3.0;
    let head_diam = 6.4;
    let thread_diam = 2.5;

    Self { axis, normals, cone_angle, screw_diam, head_diam, thread_diam }
  }

  pub fn faces(&self) -> usize {
    self.normals.len() + 4 - 4
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.normals.len())
  }

  pub fn get_height(current_normal: usize) -> f32 {
    0.6
  }

  pub fn get_quality() -> usize {
    128
  }

  pub fn get_size() -> f32 {
    130.0
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();

    if r < 46.0 { return 0; }

    for i in 0..self.normals.len() {
      if dot(pos, self.normals[i]) > 52.0 {
        return 0;
      }
    }

    let mut index = 0;

    let c = if r > 50.0  || r < 48.0 { self.cone_angle} else { (self.cone_angle.acos() - 3.0 / 50.0).cos() };

    for i in 0..self.axis.len() {
      if dot(pos, self.axis[i]) > r * c {
        index += (1 << i);
      }
    }

    return index;
  }
}
