use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct FlowerHybridCreator {
  axis_h: Vec<Point>,
  axis_m: Vec<Point>,
  normals: Vec<Point>,
  groove_h: Vec<f32>,
  groove_m: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl FlowerHybridCreator {
  pub fn new() -> Self {
    let mut axis_h: Vec<_> = [
      Point { x: -1.0, y: -1.0, z: -1.0 },
      Point { x: -1.0, y: 1.0, z: 1.0 },
      Point { x: 1.0, y: -1.0, z: 1.0 },
      Point { x: 1.0, y: 1.0, z: -1.0 },
      Point { x: -1.0, y: -1.0, z: 1.0 },
      Point { x: -1.0, y: 1.0, z: -1.0 },
      Point { x: 1.0, y: -1.0, z: -1.0 },
      Point { x: 1.0, y: 1.0, z: 1.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let mut axis_m: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
    ]
    .into_iter()
    .collect();

    let normals = axis_h[4..].to_owned();

    let maximal_angle = 3.0.recip().sqrt().acos();
    let neiborhood_angle = 3.0.recip().acos();
    let edge_angle = 2.0.recip().sqrt().acos();
    let sphere_r = 13.5;
    let sphere_or = sphere_r + 4.0;

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let groove_h = vec![
      (neiborhood_angle * 0.5).cos(),
      sphere_r - 3.8,
      (maximal_angle - 3.5 / sphere_r).cos(),
      sphere_r - 1.8,
      (maximal_angle - 0.5 / sphere_r).cos(),
      sphere_r + 0.2,
      (maximal_angle - 3.5 / sphere_r).cos(),
      sphere_r + 2.2,
      (maximal_angle + 4.0 / sphere_or).cos(),
      sphere_r + 4.2,
      (maximal_angle + 1.0 / sphere_or).cos(),
      sphere_r + 6.2,
      (neiborhood_angle - 0.0 / sphere_or).cos(),
    ];

    let groove_m = vec![
      (0.0).cos(),
      sphere_r - 3.8,
      (6.1 / sphere_r).cos(),
      sphere_r - 1.8,
      (9.1 / sphere_r).cos(),
      sphere_r + 0.2,
      (6.1 / sphere_r).cos(),
      sphere_r + 2.2,
      (edge_angle + 4.0 / sphere_or).cos(),
      sphere_r + 4.2,
      (edge_angle + 1.0 / sphere_or).cos(),
      sphere_r + 6.2,
      (maximal_angle - 0.0 / sphere_or).cos(),
    ];

    Self { axis_m, axis_h, normals, groove_h, groove_m, axis_pos, axis_neg }
  }

  pub fn faces(&self) -> usize {
    self.normals.len()
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.faces())
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

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    let n0 = self.normals[current_normal];
    let n1 = n0.any_perp().norm();
    let n2 = cross(n0, n1);

    let last_groove = self.groove_h[self.groove_h.len() - 2];
    let sz = last_groove + 2.2;
    let p = n0.scale(sz) + n1.scale(pos.x) + n2.scale(pos.y);
    (self.get_part_index_impl(p, current_normal) > 0) as PartIndex
  }

  pub fn get_quality() -> usize {
    384
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();

    let sphere_r = 7.0;

    if r < sphere_r {
      return 0; // tmp
      for &a in &self.axis_h {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < 1.25 {
          return 0;
        }
      }
      return 31;
    }

    let mut out_core = false;
    let last_groove = self.groove_h[self.groove_h.len() - 2];
    let sz = last_groove + 2.2;

    // panic!("sphere_r={sphere_r}, sz={sz}");

    let mut n_dists = Vec::new();
    for i in 0..self.normals.len() {
      if i == current_normal {
        continue;
      }
      let d = sz - dot(pos, self.normals[i]);
      if current_normal < self.normals.len() && d < 1.0 {
        return 0;
      }
      if d < 0.0 {
        return 0;
      }
      n_dists.push((d, i));
    }

    n_dists.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    let out_r = 5.0;
    if sqr(out_r - f32::min(n_dists[0].0, out_r))
      + sqr(out_r - f32::min(n_dists[1].0, out_r))
      + sqr(out_r - f32::min(n_dists[2].0, out_r))
      > sqr(out_r)
    {
      return 0;
    }

    let mut index: PartIndex = 0;

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();

    axis_pos.clear();
    axis_neg.clear();

    let mut spiral = false;

    let mut match_axis =
      |index: &mut PartIndex, a: Point, i: usize, shift_in: f32, shift_out: f32| {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        let check_in = c - shift_in;
        if check_in > 0.0 {
          *index |= (1 << i);
          axis_pos.push((check_in, a));
        } else {
          let check_out = shift_out - c;
          if check_out > 0.0 {
            axis_neg.push((check_out, a));
          } else {
            return false;
          }
        }

        true
      };

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove_m, 0.03);
    let dr = (self.groove_m[5] - 0.2) - r;
    if dr > 0.0 {
      shift_out += dr * 0.015;
      shift_in += dr * 0.015;
    }

    for (i, &a) in self.axis_m.iter().enumerate() {
      if !match_axis(&mut index, a, i, shift_in, shift_out) {
        return 0;
      }
    }

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove_h, 0.03);
    let dr = (self.groove_h[1] - 0.2) - r;
    if dr > 0.0 {}

    for (i, &a) in self.axis_h.iter().enumerate() {
      if i < 4 || index & 63 == 0 {
        if !match_axis(&mut index, a, i + 10, shift_in, shift_out) {
          return 0;
        }
      }
    }

    let mut thick = false;

    if index.count_ones() == 1 {
      let hole_r = if r < sphere_r + 2.0 { 1.5 } else { 3.2 };
      for (i, &a) in self.axis_h.iter().enumerate() {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < hole_r {
          return 0;
        }
      }
    }

    if !thick {
      if spiral {
        //  return 0;
      }

      let mut in_sr = |a, b, d| {
        let r = 0.048f32 * d;
        if a < r && b < r && sqr(r - a) + sqr(r - b) > sqr(r) {
          return true;
        }
        false
      };

      if current_normal < self.normals.len() {
        let hole = 0.006;
        for a in axis_pos.iter_mut() {
          if a.0 < hole {
            return 0;
          }
          a.0 -= hole;
        }
        for a in axis_neg.iter_mut() {
          if a.0 < hole {
            return 0;
          }
          a.0 -= hole;
        }
      }

      axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
      axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

      if axis_pos.len() >= 2 {
        let d = dot(axis_pos[0].1, axis_pos[1].1);
        let r = if r < self.groove_h[self.groove_h.len() - 2] + 0.2 { 1.0 } else { 3.0 };
        if in_sr(axis_pos[0].0, axis_pos[1].0, r) {
          return 0;
        }
      }
      if axis_neg.len() >= 2 {
        let d = dot(axis_neg[0].1, axis_neg[1].1);
        let r = if d > -0.5 { 1.0 } else { 3.0 };
        if in_sr(axis_neg[0].0, axis_neg[1].0, r) {
          return 0;
        }
      }
      if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
        let d = dot(axis_pos[0].1, axis_neg[0].1);
        if d <= 0.35 {
          if axis_pos.len() >= 2 {
            let d = dot(axis_pos[1].1, axis_neg[0].1);
            if d >= 0.35 {
              if in_sr(axis_pos[1].0, axis_neg[0].0, 1.0) {
                return 0;
              }
            }
          }
          if axis_neg.len() >= 2 {
            let d = dot(axis_pos[0].1, axis_neg[1].1);
            if d >= 0.35 {
              if in_sr(axis_pos[0].0, axis_neg[1].0, 1.0) {
                return 0;
              }
            }
          }
        } else {
          if in_sr(axis_pos[0].0, axis_neg[0].0, 1.0) {
            return 0;
          }
        }
      }
    }

    if index.count_ones() >= 3 {
      if n_dists[1].0 - n_dists[0].0 < 0.13 {
        return 0;
      }
      index += 1000000 * (n_dists[0].1 + 1) as PartIndex;
    } else if r > self.groove_h[11] && n_dists[0].0 < 7.0 {
      if r > self.groove_h[11] + 0.2 && n_dists[0].0 < 6.8 {
        if n_dists[1].0 - n_dists[0].0 < 0.13 {
          return 0;
        }
        index += 1000000 * (n_dists[0].1 + 1) as PartIndex;
      } else {
        return 0;
      }
    }

    if index / 1000000 != 1 {
      return 0; // tmp
    }

    return index;
  }
}
