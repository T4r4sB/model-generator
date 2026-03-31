use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;

pub struct JigsawCreator {
  a: Vec<Point>,
  n: Vec<Point>,
  c: f32,
  ra: f32,
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

impl JigsawCreator {
  pub fn new() -> Self {
    let c = 0.284;
    let s = (1.0 - c * c).sqrt();
    let mut a: Vec<_> = [
      Point { x: 0.0, y: s, z: c },
      Point { x: s, y: 0.0, z: -c },
      Point { x: 0.0, y: -s, z: c },
      Point { x: -s, y: 0.0, z: -c },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let se = -c * c;
    let be = 2.0 * c * c - 1.0;
    let ra = 2.0 * PI - ((be - sqr(se)) / (1.0 - sqr(se))).acos() * 2.0;

    let sq2 = 2.0.sqrt();

    let n = vec![
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();
    Self { a, n, c, ra }
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
    302
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    //  let pos=Point { x: 14.530849, y: 14.530849, z: 14.530849 };

    let near_b = pos.x.abs() > 25.3 || pos.y.abs() > 25.3 || pos.z.abs() > 25.3;

    let tol = if near_b { 0.11 } else { 0.17 };

    fn out_line(x: f32, mx: f32, ms: &[i8], z: f32, tol: f32) -> Option<bool> {
      let sq2 = 0.5.sqrt();
      let x = x * sq2;
      let mx = mx * sq2;
      let m = x.rem_euclid(mx);
      let d = x.div_euclid(mx) as i8;

      let (z, rev) = if ms.iter().any(|&ms| ms == d) { (-z, true) } else { (z, false) };

      let z = z - (z - 4.0).clamp(0.0, 1.0);

      let dist = f32::min(z, (sqr(m - mx * 0.5) + sqr(z - 4.0)).sqrt() - 3.0);
      let dist = if z < 4.0 { f32::min(dist, (m - mx * 0.5).abs() - 1.5) } else { dist };
      if dist > tol {
        return Some(!rev);
      }
      if dist < -tol {
        return Some(rev);
      }
      return None;
    };

    let get_index = || {
      let mut index: PartIndex = 0;
      if out_line(pos.x + pos.y, 26.0, &[0], -pos.z, tol)? {
        index += 1;

        if out_line(pos.x + pos.z, 26.0, &[-1], -pos.y, tol)? {
          index += 2;

          if out_line(pos.y - pos.z, 26.0, &[0], -pos.x, tol)? {
            index += 4;
          }
        } else {
          if out_line(pos.y + pos.z, 26.0, &[-1], pos.x, tol)? {
            index += 4;
          }
        }
      } else {
        if out_line(pos.y - pos.z, 26.0, &[0], -pos.x, tol)? {
          index += 2;

          if out_line(pos.x - pos.z, 26.0, &[], pos.y, tol)? {
            index += 4;
          }
        } else {
          if out_line(pos.x + pos.z, 26.0, &[], -pos.y, tol)? {
            index += 4;
          }
        }
      }
      Some(index)
    };

    let mut index = if let Some(index) = get_index() {
      index
    } else {
      return 0;
    };

    if index == 0 {
      index = 255;
    }

    if index == 5 || index == 255 { return 0; }

    let mut v = [
      26.0 - dot(pos, self.n[0]),
      26.0 - dot(pos, self.n[1]),
      26.0 - dot(pos, self.n[2]),
      26.0 - dot(pos, self.n[3]),
      26.0 - dot(pos, self.n[4]),
      26.0 - dot(pos, self.n[5]),
    ];

    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rr = 3.0;
    if v[0] < 0.0 {
      return 0;
    }
    if sqr(rr - f32::min(v[0], rr)) + sqr(rr - f32::min(v[1], rr)) + sqr(rr - f32::min(v[2], rr))
      > sqr(rr)
    {
      return 0;
    }

    return index;
  }
}
