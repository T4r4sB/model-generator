use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;

pub struct CubeCreator {
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

impl CubeCreator {
  pub fn new() -> Self {
    let edge = -0.12;
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
    hs.resize_with(36, || Vec::new());

    let hs0 = vec![
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
      Point { x: -0.5, y: sq3, z: 0.0 },
      Point { x: -0.5, y: -sq3, z: 0.0 },
    ];
    for &hs0 in &hs0 {
      hs[14].push(reflect(hs0, a[1], a[2]));
    }
    for &hs0 in &hs0 {
      hs[21].push(reflect(hs0, a[0], a[2]));
    }
    for &hs0 in &hs0 {
      hs[35].push(reflect(hs0, a[0], a[1]));
    }

    hs[7] = hs0;

    Self { a, n, hs }
  }

  pub fn faces(&self) -> usize {
    2
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
    320
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    if current_normal == 0 {
      let n = self.n[0];
      let n1 = n.any_perp().norm();
      let n2 = cross(n, n1);
      self.get_part_index_impl(n1.scale(pos.x) + n2.scale(pos.y), true)
    } else if current_normal == 1 {
      self.get_part_index_impl(Point { x: pos.x, y: 0.0, z: pos.y }, true)
    } else {
      0
    }
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, false)
  }

  fn get_part_index_impl(&self, mut pos: Point, for_section: bool) -> PartIndex {
    let r = pos.len();
    if r < 22.0 {
      return 0;
    }

    if r > 27.5 {
      //return 0; // tmp
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

    fn get_dist(d: f32, c: f32) -> (f32, f32) {
      let c = 28.5 - c;
      let ofs = -0.1;
      let x1 = d - 2.0 + ofs;
      let x2 = f32::min(d, 2.5 - c);
      let x3 = f32::min(d + 2.0 + ofs, c - 5.0);
      let x4 = f32::min(f32::min(d + 2.0 + ofs, -ofs - d), c - 4.0);

      (f32::max(f32::max(f32::max(x1, x2), x3), x4), f32::INFINITY)
    }

    let mut index: PartIndex = 0;
    let mut dcut = f32::INFINITY;
    let mut dcuts = [f32::INFINITY; 6];
    for i in 0..self.a.len() {
      if i < 3 || index & 7 == (1 << (i - 3)) || index & 7 == 7 - (1 << (i - 3)) {
        let a = self.a[i];
        let (dd, d2r) = get_dist(dot(pos, a), cross(pos, a).len());
        if dd > 0.2 {
          index += 1 << i;
        } else if dd > -0.2 {
          return 0;
        }
        dcuts[i] = f32::abs(d2r) - 0.2;
        dcut = f32::min(dcut, dot(pos, a).abs());
      }
    }
    dcuts.sort_by(|a, b| a.partial_cmp(&b).unwrap());
   
   /* let rr = 0.7;
    if dcuts[0] < rr && dcuts[1] < rr {
      if sqr(rr - dcuts[0]) + sqr(rr - dcuts[1]) > sqr(rr) {
        return 0;
      }
    }*/

    let compound_part = index == 7 || index == 14 || index == 21 || index == 35;

    if dists[0].0 < 3.0 {
      let mut d = (dists[1].0 + dists[0].0 - 5.0) * 0.5;
      if compound_part {
        for r in [(15.0, 15.0), (7.0, 22.0), (22.0, 7.0)] {
          d = f32::min(d, (sqr(dists[1].0 - r.0) + sqr(dists[2].0 - r.1)).sqrt() - 2.0);
        }
      }
      if dists[0].0 < 1.0 || d < 0.0 && dists[0].0 < 2.5 {
        if dists[0].0 + 0.2 * 0.5.sqrt() > dists[1].0 {
          return 0;
        }
        index += (dists[0].1 as PartIndex + 1) * 10000;
      } else if dists[0].0 < 1.2 || d < 0.11 {
        return 0;
      }
    }

    let part_split = 25.5;

    let mut pin = false;
    if compound_part && index < 10000 {
      let hs = &self.hs[index as usize];
      let hs0 = hs[0];
      let hr = if r > part_split + 5.0 {
        2.8
      } else if r > part_split {
        1.5
      } else {
        1.2
      };
      if dot(pos, hs0) > 0.0 && cross(pos, hs0).len() < hr {
        return 0;
      }

      let mut npin = false;
      for &pinc in &hs[1..] {
        let c = cross(pos - pinc.scale(7.0), hs0).len();
        if c < 2.5 {
          pin = true;
        }
        if c < 2.7 {
          npin = true;
        }
      }

      let gap = if dcut > 3.0 { 0.5 } else { 0.1 };

      if !npin && r > part_split + gap || r > part_split + 7.0 + gap {
        index += 64;
      } else if !pin && r > part_split - gap || r > part_split + 7.0 - gap {
        return 0;
      }
    }

    if dcut > 4.5 && (index == 0 || index == 1 || index == 2 || index == 4) {
      return 0;
    }

    if index == 0 {
      index = 31;
    }

    return index;
  }
}
