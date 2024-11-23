use crate::points2d::*;
use crate::solid::*;
use num::Float;

use crate::slots_and_holes::*;

pub struct GearContour {
  tc: usize,
  name: &'static str,
  pin_r: f32,
  edge: f32,
  pin_c_r: f32,
  pairing_r: f32,
  circles: Vec<Point>,
}

impl GearContour {
  pub fn new(tc: usize, name: &'static str) -> Self {
    let pin_r = 8.51 * 0.5;
    let edge = 12.7;
    let pin_c_r = edge * 0.5 / (PI / tc as f32).sin();
    let circles = (0..tc)
      .map(|i| {
        let a = i as f32 * 2.0 * PI / tc as f32;
        let (s, c) = a.sin_cos();
        Point { x: c * pin_c_r, y: s * pin_c_r }
      })
      .collect();

    let pairing_r = (sqr(pin_c_r) - sqr(edge * 0.5) + sqr(edge * 0.5 - pin_r)).sqrt();

    Self { tc, name, pin_r, edge, pin_c_r, pairing_r, circles }
  }

  pub fn contains(&self, pos: Point) -> bool {
    let r = pos.len();
    if r > self.pin_c_r + self.pin_r {
      return false;
    } else if r > self.pairing_r {
      let mut match_c = 0;
      for &c in &self.circles {
        if (pos - c).len() < self.edge - self.pin_r {
          match_c += 1;
        }
      }
      if match_c < 2 {
        return false;
      }
    } else {
      for &c in &self.circles {
        if (pos - c).len() < self.pin_r {
          return false;
        }
      }
    }
    return true;
  }

  fn inner_r(&self) -> f32 {
    self.pin_c_r - self.pin_r - 5.0
  }

  fn in_driver_hole(pos: Point) -> bool {
    let r0 = 34.9 * 0.5;
    let cr = 3.0;
    let cd = 1.5;
    let r1 = r0 + cr - cd;
    let sq3 = 0.75.sqrt();

    let p0 = Point { x: r1, y: 0.0 };
    let p1 = Point { x: -r1 * 0.5, y: r1 * sq3 };
    let p2 = Point { x: -r1 * 0.5, y: -r1 * sq3 };

    pos.len() < r0 && (pos - p0).len() > cr && (pos - p1).len() > cr && (pos - p2).len() > cr
  }

  fn in_driver_hole_6(pos: Point) -> bool {
    let r0 = 39.8 * 0.5;
    let cr = 3.1366;
    let cd = 1.4;
    let r1 = r0 + cr - cd;
    let sq3 = 0.75.sqrt();

    let p0 = Point { x: r1, y: 0.0 };
    let p1 = Point { x: -r1 * 0.5, y: r1 * sq3 };
    let p2 = Point { x: -r1 * 0.5, y: -r1 * sq3 };

    pos.len() < r0
      && (pos - p0).len() > cr
      && (pos - p1).len() > cr
      && (pos - p2).len() > cr
      && (pos + p0).len() > cr
      && (pos + p1).len() > cr
      && (pos + p2).len() > cr
  }
}

pub struct ChaingearCreator {
  gears: Vec<GearContour>,
}

impl ChaingearCreator {
  pub fn new() -> Self {
    let gears = vec![
      GearContour::new(18, "18-rear"),
      GearContour::new(19, "19-rear"),
      GearContour::new(22, "22-rear"),
      GearContour::new(39, "39-front"),
      GearContour::new(41, "41-front"),
      GearContour::new(35, "35-rear"),
      GearContour::new(37, "37-rear"),
      GearContour::new(30, "30-rear"),
    ];

    Self { gears }
  }

  /*
  // 7-bolt adapter


    let bolt_r = 65.0 * 0.5;
    let bolt_in_r = 18.95;

    let bolt_holes: Vec<_> = (0..7)
      .map(|i| {
        let a = i as f32 * 2.0 * PI / 7.0;
        let (s, c) = a.sin_cos();
        Hole { position: Point { x: c * bolt_r, y: s * bolt_r }, r_in: 2.5, r_out: 5.5 }
      })
      .collect();

    let bolt_holes_in: Vec<_> = (0..7)
      .map(|i| {
        let a = i as f32 * 2.0 * PI / 7.0;
        let (s, c) = a.sin_cos();
        Hole { position: Point { x: c * bolt_in_r, y: s * bolt_in_r }, r_in: 0.0, r_out: 3.0 }
      })
      .collect();

          */

  pub fn get_count(&self, current_normal: usize) -> usize {
    1
  }

  pub fn faces(&self) -> usize {
    self.gears.len()
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    3.0
  }

  pub fn get_quality() -> usize {
    1
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_name(&self, current_normal: usize) -> Option<&str> {
    Some(self.gears[current_normal].name)
  }

  pub fn get_sticker_index(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if current_normal < self.gears.len() {
      let g = &self.gears[current_normal];

      if current_normal < 3 || current_normal > 4 {
        for i in 0..15 {
          let a = i as f32 / 15.0 * 2.0 * PI;
          let a = Point::from_angle(a).scale(g.inner_r() * 0.5 + 11.0);
          let r = (i % 5) as f32 * 0.5 + 1.5 + (g.tc as f32 - 18.0) * 0.15;
          if (pos - a).len() < r {
            return 0;
          }
        }

        if  current_normal < 3 {
          if GearContour::in_driver_hole(pos) { return 0; }
        } else {
          if GearContour::in_driver_hole_6(pos) { return 0; }
        }

        return g.contains(pos) as PartIndex;
      } else {
        let bcd;
        let cnt;

        if current_normal == 3 {
          bcd = 104.0;
          cnt = 4;
        } else {
          bcd = 104.0;
          cnt = 5;
        }

        if r < g.inner_r() - 2.0 {
          let mut v = [f32::INFINITY; 11];
          v[0] = (g.inner_r() - 2.0) - r;

          if r < 104.0 * 0.5 - 5.0 - 2.0 {
            return 0;
          }

          for i in 0..cnt {
            let a = i as f32 / 4.0 * 2.0 * PI;
            let a1 = Point::from_angle(a).scale(104.0 * 0.5);
            let l1 = (pos - a1).len();
            if l1 < 5.0 {
              return 0;
            }

            let m2 = 10.0 - l1;

            if l1 < 10.0 {
              return 1;
            }

            v[i * 2 + 1] = l1 - 10.0;

            let a2 = Point::from_angle(a).scale(104.0 * 0.5 + 40.0);
            let l2 = (pos - a2).len();
            if (l2 - 40.0).abs() < 5.0 {
              return 1;
            }

            v[i * 2 + 2] = (l2 - 40.0).abs() - 5.0;
          }

          v.sort_by(|a, b| a.partial_cmp(b).unwrap());
          let rr = 2.0;
          if v[0] < rr && v[1] < rr {
            if sqr(rr - v[0]) + sqr(rr - v[1]) > sqr(rr) {
              return 1;
            }
          }

          return 0;
        } else {
          return g.contains(pos) as PartIndex;
        }
      }
    }

    0
  }

  pub fn get_part_index(&self, pos: crate::points3d::Point) -> PartIndex {
    0
  }
}
