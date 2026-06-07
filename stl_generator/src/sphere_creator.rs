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
  hs: Vec<Vec<Point>>,
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
    let edge = -0.15;
    let c = ((edge * 2.0 + 1.0) / 3.0).sqrt();
    let s = (1.0 - sqr(c)).sqrt();
    let sq3 = 0.75.sqrt();

    let mut a = vec![
      Point { x: s, y: 0.0, z: c },
      Point { x: -s * 0.5, y: s * sq3, z: c },
      Point { x: -s * 0.5, y: -s * sq3, z: c },
    ];

    a.push(reflect(a[0], a[1], a[2]));
    a.push(reflect(a[1], a[2], a[0]));
    a.push(reflect(a[2], a[0], a[1]));

    let cn = (1.0 / 3.0).sqrt();
    let sn = (2.0 / 3.0).sqrt();
    let mut n: Vec<Point> = vec![
      Point { x: sn, y: 0.0, z: cn },
      Point { x: -sn * 0.5, y: sn * sq3, z: cn },
      Point { x: -sn * 0.5, y: -sn * sq3, z: cn },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();
    n.push(-n[0]);
    n.push(-n[1]);
    n.push(-n[2]);

    let mut hs = Vec::new();
    hs.resize_with(5, || Vec::new());
    let ch = -0.97;
    let sh = (1.0 - sqr(ch)).sqrt();

    hs[0] = vec![
      Point { x: -sh, y: 0.0, z: ch },
      Point { x: sh * 0.5, y: sh * sq3, z: ch },
      Point { x: sh * 0.5, y: -sh * sq3, z: ch },
    ];

    Self { a, n, hs }
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

    let mut dists = [(f32::INFINITY, 0); 6];
    for (i, n) in self.n.iter().enumerate() {
      let d = 28.5 - dot(pos, *n);
      if d < 0.0 {
        return 0;
      }
      dists[i] = (d, i);
    }
    dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let out_r = 3.0;
    if sqr(out_r - f32::min(dists[0].0, out_r))
      + sqr(out_r - f32::min(dists[1].0, out_r))
      + sqr(out_r - f32::min(dists[2].0, out_r))
      > sqr(out_r)
    {
      return 0;
    }

    fn get_dist(d: f32, c: f32) -> f32 {
      let x1 = d - 2.0;
      let x2 = f32::min(c - 26.5, d);
      let x3 = f32::min(1.0 - (c - 22.5).abs(), d + 1.0);
      let base = f32::max(f32::max(x1, x2), x3);
      let mut d2c = f32::INFINITY;

      let c2 = 0.75.sqrt();

      d2c = f32::min(d2c, -d);
      d2c = f32::min(d2c, d + 2.5);
      d2c = f32::min(d2c, 24.5 - c);
      d2c = f32::min(d2c, c - 20.5);
      d2c = f32::min(d2c, 1.8 - (-(d + 1.0) * c2 + (c - 22.5) * c2));
      d2c = f32::min(d2c, 1.8 - (-(d + 1.0) * c2 - (c - 22.5) * c2));
      f32::max(base, d2c)
    }

    let mut index: PartIndex = 0;
    for i in 0..self.a.len() {
      if i < 3 || index & 7 == (1 << (i - 3)) || index & 7 == 7 - (1 << (i - 3)) {
        let a = self.a[i];
        let dd = get_dist(dot(pos, a), cross(pos, a).len());
        if dd > 0.2 {
          index += 1 << i;
        } else if dd > -0.2 {
          return 0;
        }
      }
    }

    if index == 0 || index == 1 || index == 2 || index == 4 {
      for &hs in &self.hs[index as usize] {
        let hr = if r > 22.5 + 2.0 {
          2.8
        } else if r > 22.5 {
          1.5
        } else {
          1.2
        };
        if dot(pos, hs) > 0.0 && cross(pos, hs).len() < hr {
          return 0;
        }
      }
      if r > 22.5 + 0.2 {
        index += 64
      } else if r > 22.5 - 0.2 {
        return 0;
      }
    }

    if index == 0 {
      index = 31;
    }

    return index;
  }
}
