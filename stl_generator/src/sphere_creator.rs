use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;

pub struct SphereCreator {
  starts: Vec<Point>,
  rounds: Vec<(f32, f32)>,
  rounds_deep: Vec<(f32, f32)>,
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

impl SphereCreator {
  pub fn new() -> Self {
    let mut edges = Vec::new();

    let a0 = -Point::X;
    let a1 = -Point::Y;
    let a2 = a0.rotate(a1, PI / 6.0);
    let a3 = a0.rotate(a1, PI / 3.0);

    edges.push((a0, a1));
    edges.push((a2, a1));
    edges.push((a3, a1));
    edges.push((a1.rotate(a2, PI / 6.0), a2));
    edges.push((a1.rotate(a2, PI / 3.0), a2));
    edges.push((a1.rotate(a3, PI / 6.0), a3));
    edges.push((a1.rotate(a3, PI / 3.0), a3));

    let mut rounds = Vec::new();
    let mut rounds_deep = Vec::new();
    let mut starts = Vec::new();

    let mut first = true;
    for e in edges {
      let factor = 0.35;
      for (da, rounds) in [(1.0 / 23.1, &mut rounds), (0.2 / 23.1, &mut rounds_deep)] {
        let p = (e.0 + (e.1 - e.0).scale(factor)).norm();
        if first {
          starts.push(p);
        }
        println!("dp={}", dot(p, e.0).acos().to_degrees());
        let a = dot(p, a0).acos();
        rounds.push(((a + da).cos(), (a - da).cos()));
        let p = (e.1 + (e.0 - e.1).scale(factor)).norm();
        if first {
          starts.push(p);
        }
        let a = dot(p, a0).acos();
        rounds.push(((a + da).cos(), (a - da).cos()));
        first = false;
      }
    }

    Self { starts, rounds, rounds_deep }
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
    200
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    self.get_part_index(Point { x: pos.x, y: 0.0, z: pos.y })
  }

  pub fn get_part_index(&self, mut pos: Point) -> PartIndex {
    let r = pos.len();
    if r < 18.5 {
      return 0;
    }

    if r > 23.1 {
      return 0;
    }

    if r > 22.0 {
      for a in [Point::X, Point::Y, Point::Z, -Point::X, -Point::Y, -Point::Z] {
        let d = dot(pos, a) / r;
        for r in &self.rounds {
          if d > r.0 && d < r.1 {
            return 0;
          }
        }
      }
    }

    if r > 21.0 {
      for a in [Point::X, Point::Y, Point::Z, -Point::X, -Point::Y, -Point::Z] {
        let d = dot(pos, a) / r;
        for r in &self.rounds_deep {
          if d > r.0 && d < r.1 {
            return 0;
          }
        }
      }
    }

    for s in &self.starts {
      if (pos.norm().scale(23.1) - s.scale(23.1)).len() < 1.0 {
        return 0;
      }
    }

    return 1;
  }
}
