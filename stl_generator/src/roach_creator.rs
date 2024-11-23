use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;
use std::ops::DerefMut;

pub struct RoachCreator {}

pub fn sqr(x: f32) -> f32 {
  x * x
}

pub fn in_cylinder(pos: Point, l1: Point, l2: Point, r: f32) -> bool {
  let delta = (l2 - l1).len();
  let d = dot(pos - l1, l2 - l1) / delta;
  if d < 0.0 {
    return (pos - l1).sqr_len() < sqr(r);
  }
  if d > delta {
    return (pos - l2).sqr_len() < sqr(r);
  }

  cross(pos - l1, pos - l2).sqr_len() < sqr(r * delta)
}

pub fn in_cylinder_hairy(pos: Point, l1: Point, l2: Point, r: f32) -> bool {
  let delta = (l2 - l1).len();
  let d = dot(pos - l1, l2 - l1) / delta;
  if d < 0.0 {
    return (pos - l1).sqr_len() < sqr(r);
  }
  if d > delta {
    return (pos - l2).sqr_len() < sqr(r);
  }

  cross(pos - l1, pos - l2).sqr_len() < sqr((r + (1.0 - d.fract()) * 0.3) * delta)
}

impl RoachCreator {
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
    let mut pos = pos;
    pos.z *= 0.5;
    if pos.z < 0.0 || pos.z > 6.0 {
      return 0;
    }

    {
      let mut pos = pos;
      let d = f32::max(0.0, 1.0 - (15.0 - (sqr(pos.y - 30.0) + sqr(pos.x)).sqrt()).abs());
      pos.z += d;

      let d = f32::max(0.0, 0.5 - (15.0 - (sqr(pos.y - 38.0) + sqr(pos.x)).sqrt()).abs());
      pos.z += d;

      if pos.y < 15.0 && sqr((pos.x + 5.0) * 3.0) + sqr(pos.y + 1.0) < sqr(23.0) {
        pos.z -= 1.0;
      } else if pos.y < 15.0 && sqr((pos.x - 5.0) * 3.0) + sqr(pos.y + 1.0) < sqr(23.0) {
        pos.z -= 0.6;
      }

      if sqr(pos.x * 3.0) + sqr(pos.y) + sqr((pos.z - 0.5) * 6.0) < sqr(25.0) {
        return 1;
      }
    }

    let mut pos = pos;
    pos.x = pos.x.abs();

    if in_cylinder(pos, Point { x: 0.0, y: -20.0, z: 0.1 }, Point { x: 7.0, y: -22.0, z: 0.1 }, 1.0)
    {
      return 1;
    }
    if in_cylinder(pos, Point { x: 9.0, y: -23.0, z: 0.1 }, Point { x: 7.0, y: -22.0, z: 0.1 }, 0.8)
    {
      return 1;
    }

    if in_cylinder(pos, Point { x: 0.0, y: 0.0, z: 0.1 }, Point { x: 12.0, y: 2.0, z: 0.1 }, 1.3) {
      return 1;
    }
    if in_cylinder_hairy(pos, Point { x: 15.0, y: -18.0, z: 0.1 }, Point { x: 12.0, y: 2.0, z: 0.1 }, 1.0)
    {
      return 1;
    }
    if in_cylinder(
      pos,
      Point { x: 15.0, y: -18.0, z: 0.1 },
      Point { x: 25.0, y: -35.0, z: 0.1 },
      0.8,
    ) {
      return 1;
    }

    if in_cylinder(pos, Point { x: 0.0, y: 10.0, z: 0.1 }, Point { x: 10.0, y: 10.0, z: 0.1 }, 1.3)
    {
      return 1;
    }
    if in_cylinder_hairy(pos, Point { x: 17.0, y: 5.0, z: 0.1 }, Point { x: 10.0, y: 10.0, z: 0.1 }, 1.0)
    {
      return 1;
    }
    if in_cylinder(pos, Point { x: 17.0, y: 5.0, z: 0.1 }, Point { x: 37.0, y: 0.0, z: 0.1 }, 0.8) {
      return 1;
    }

    if in_cylinder(pos, Point { x: 0.0, y: 15.0, z: 0.1 }, Point { x: 8.0, y: 20.0, z: 0.1 }, 1.0) {
      return 1;
    }
    if in_cylinder(pos, Point { x: 15.0, y: 25.0, z: 0.1 }, Point { x: 8.0, y: 20.0, z: 0.1 }, 0.8)
    {
      return 1;
    }

    if in_cylinder(pos, Point { x: 0.0, y: 20.0, z: 0.1 }, Point { x: 4.0, y: 30.0, z: 0.1 }, 1.0) {
      return 1;
    }
    if in_cylinder(pos, Point { x: 4.0, y: 30.0, z: 0.1 }, Point { x: 20.0, y: 60.0, z: 0.1 }, 0.5)
    {
      return 1;
    }

    return 0;
  }
}
