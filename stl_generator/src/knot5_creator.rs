use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;



pub struct Knot5Creator {
  lines: Vec<[(f32, f32); 5]>
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

fn proj(center: Point, tar: Point, p: Point) -> Point {
  let ct = cross(center, tar).norm();
  let cct = cross(ct, center);
  let d = dot(center, p);
  let c = cross(center, p).len();
  let result = cct.scale(c) + center.scale(d);

  result
}

impl Knot5Creator {
  pub fn new() -> Self {
   let lines = vec![
      [(0.0, 1.0), (1.0, 1.0), (2.0, 1.0), (3.0, 1.0), (4.0, 1.0)],
      [(0.0, 1.0), (1.0, 1.0), (2.0, 1.0), (3.0, 1.0), (4.0, 2.0)],
      [(1.0, 2.0), (2.0, 2.0), (1.0, 0.0), (4.0, 0.0), (3.0, 1.0)],
      [(2.0, 2.0), (3.0, 0.0), (0.0, 0.0), (3.0, 2.0), (2.0, 0.0)],
      [(3.0, 0.0), (4.0, 2.0), (1.0, 2.0), (2.0, 2.0), (1.0, 0.0)],
      [(4.0, 0.0), (3.0, 2.0), (2.0, 0.0), (1.0, 2.0), (0.0, 0.0)],
      [(3.0, 0.0), (2.0, 2.0), (3.0, 2.0), (0.0, 2.0), (1.0, 0.0)],
      [(2.0, 0.0), (1.0, 2.0), (4.0, 0.0), (1.0, 0.0), (2.0, 2.0)],
      [(1.0, 1.0), (0.0, 0.0), (3.0, 0.0), (2.0, 2.0), (3.0, 2.0)],
      [(0.0, 2.0), (1.0, 1.0), (2.0, 1.0), (3.0, 1.0), (4.0, 1.0)],
   ];

    Self { lines }
  }

  pub fn faces(&self) -> usize {
    1
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
    30.0
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    self.get_part_index(Point { x: pos.x, y: 0.0, z: pos.y })
  }

  pub fn get_part_index(&self, mut pos: Point) -> PartIndex {
    let r = (sqr(pos.x) + sqr(pos.y)).sqrt();
    let p = f32::atan2(pos.y, pos.x);
    let loc_r = 1.0 + ((f32::min(p.abs() * 4.0, PI)).cos() + 1.0) * 0.5;
    let loc_z = 3.0;
    if sqr((r - 12.0) / 1.0) + sqr(pos.z / loc_z) > sqr(loc_r) { return 0; }
    let ll = self.lines.len() as f32;

    let seg = (p + 0.5) * ll;
    let (p, c, f) = if seg < 0.0 {
      (0, 0, 0.0)
    } else if seg < ll - 1.0 {
      (seg as usize, seg as usize + 1, seg.fract())
    } else if seg < ll {
      (self.lines.len() - 1, 0, seg.fract())
    } else {
      (0, 0, 0.0)
    };

    let segp = &self.lines[p];
    let segc = &self.lines[c];
    let z = (pos.z / loc_z / loc_r + 1.0) * segp.len() as f32 * 0.5 - 0.5;
    let rf = r - 11.0;
    let mut index = 0;
    let mut best = f32::INFINITY;
    for i in 0 .. segp.len() {
      let cz = segp[i].0 + (segc[i].0 - segp[i].0) * f;
      let cr = segp[i].1 + (segc[i].1 - segp[i].1) * f;
      let dst = sqr(z - cz) + sqr(rf - cr);
      if dst < best {
        best = dst;
        index = i as PartIndex + 1;
      }
    }

    return index;
  }
}
