use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::FxHashMap;
use num::Float;

use std::cell::RefCell;
use std::ops::Deref;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct ZmeyGorynychCurvyCopterCreator {
  axis: Vec<Point>,
  axis1: Vec<Point>,
  axis2: Vec<Point>,
  normals: Vec<Point>,
  groove: Vec<f32>,
  n_dists: RefCell<Vec<f32>>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
  centers: FxHashMap<PartIndex, Point>,
  extra_split: FxHashMap<PartIndex, Point>,
  extra_cuts: FxHashMap<PartIndex, Vec<Point>>,
  extra_cutsp: FxHashMap<PartIndex, Vec<Point>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}
impl ZmeyGorynychCurvyCopterCreator {
  pub fn new() -> Self {
    let u = 1.0;
    let v = 0.85;

    let axis: Vec<_> = [
      Point { x: 0.0, y: -u, z: -v },
      Point { x: 0.0, y: -u, z: v },
      Point { x: 0.0, y: u, z: -v },
      Point { x: 0.0, y: u, z: v },
      Point { x: -v, y: 0.0, z: -u },
      Point { x: v, y: 0.0, z: -u },
      Point { x: -v, y: 0.0, z: u },
      Point { x: v, y: 0.0, z: u },
      Point { x: -u, y: -v, z: 0.0 },
      Point { x: -u, y: v, z: 0.0 },
      Point { x: u, y: -v, z: 0.0 },
      Point { x: u, y: v, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let normals: Vec<_> = [
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let c_min = dot(axis[0], axis[1]);
    let c_max = dot(axis[0], axis[2]);
    let edge = dot(axis[0], axis[4]);
    let tmme = (c_min - sqr(edge)) / (1.0 - sqr(edge));
    let tac = (-tmme - 1.0 + (1.0 - tmme) * edge) * 0.5;
    let ta = tac.acos();

    let tc_min = ((edge - tac) / (1.0 - tac)).sqrt();
    let a_min = tc_min.acos();
    let a_max = c_max.acos() * 0.5;

    let r = 4.5 / f32::max(0.1, a_max - a_min);
    println!("r={r}, ta={}", tac.acos().to_degrees());

    let groove = vec![r - 3.0, a_max.cos(), r + 0.1, (a_max - 3.0 / r).cos(), r + 3.2, a_max.cos()];

    let mut axis1 = Vec::new();
    let mut axis2 = Vec::new();
    for &a in &axis {
      let a1 = a.any_perp();
      let a2 = cross(a, a1);
      axis1.push(a1);
      axis2.push(a2);
    }

    let cf = find_factors_for_triangle(c_min, tc_min, tc_min);
    println!("cf={cf:?}");

    let mut extra_split = FxHashMap::default();
    let mut extra_cuts = FxHashMap::default();
    let mut extra_cutsp = FxHashMap::default();
    let mut centers = FxHashMap::default();
    for i1 in 0..axis.len() {
      let a1 = axis[i1];
      for i2 in 0..axis.len() {
        let a2 = axis[i2];
        if (dot(a1, a2) - c_min).abs() > 0.001 {
          continue;
        }

        for i3 in 0..axis.len() {
          let a3 = axis[i3];
          if (dot(a1, a3) - edge).abs() > 0.001
            || (dot(a2, a3) - edge).abs() > 0.001
            || dot(a3, cross(a1, a2)) < 0.0
          {
            continue;
          }

          let center = a1.scale(cf.0) + a2.scale(cf.1) + cross(a1, a2).scale(cf.2);
          extra_split.insert(1 << i1 | 1 << i2 | 1 << i3, a3.rotate(center, PI));
          extra_split.insert(1 << i1 | 1 << i3, a3.rotate(center, ta * 2.0));
          extra_split.insert(1 << i2 | 1 << i3, a3.rotate(center, -ta * 2.0));
          centers.insert(1 << i1 | 1 << i2 | 1 << i3 | 1 << axis.len(), center);

          extra_cuts.insert(
            1 << i1 | 1 << i2 | 1 << i3 | 1 << axis.len(),
            vec![
              a3.rotate(center, ta - PI * 0.5),
              a3.rotate(center, PI * 0.5 - ta),
              a3.rotate(center, -ta - PI * 0.5),
              a3.rotate(center, PI * 0.5 + ta),
              a3.rotate(center, PI * 0.5),
              a3.rotate(center, -PI * 0.5),
              a3.rotate(center, PI - ta),
              a3.rotate(center, PI + ta),
            ],
          );

          extra_cutsp.insert(
            1 << i1 | 1 << i2 | 1 << i3,
            vec![a3.rotate(center, ta * 2.0), a3.rotate(center, -ta * 2.0)],
          );
          extra_cutsp.insert(
            1 << i1 | 1 << i3 | 1 << axis.len(),
            vec![a1.rotate(center, PI), a1.rotate(center, ta * 2.0)],
          );
          extra_cutsp.insert(
            1 << i2 | 1 << i3 | 1 << axis.len(),
            vec![a2.rotate(center, PI), a2.rotate(center, -ta * 2.0)],
          );
        }
      }
    }

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());
    let n_dists = RefCell::new(Vec::new());
    Self {
      axis,
      axis1,
      axis2,
      normals,
      groove,
      axis_pos,
      axis_neg,
      n_dists,
      extra_split,
      extra_cuts,
      extra_cutsp,
      centers,
    }
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
    0
  }

  pub fn get_quality() -> usize {
    320
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 49.999 || pos.y.abs() > 49.999 || pos.z.abs() > 49.999 {
      return 0;
    }

    let sphere_r = self.groove[0] - 2.4;

    if r < sphere_r {
      for &a in &self.axis {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < 1.25 {
          return 0;
        }
      }
      return 6;
    }

    let mut out_core = false;
    let last_groove = self.groove[self.groove.len() - 2];
    let sz = last_groove + 0.3;

    let mut n_dists = self.n_dists.borrow_mut();
    let n_dists: &mut _ = n_dists.deref_mut();

    n_dists.clear();
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
      n_dists.push(d);
    }

    n_dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let out_r = 1.0;
    if sqr(out_r - f32::min(n_dists[0], out_r))
      + sqr(out_r - f32::min(n_dists[1], out_r))
      + sqr(out_r - f32::min(n_dists[2], out_r))
      > sqr(out_r)
    {
      return 0;
    }

    let mut index: PartIndex = 0;

    let (mut shift_out, mut shift_in, inter) = get_diag_groove(r, &self.groove);

    let depth = self.groove[1] + 0.2 - r;

    let mut axis_pos = self.axis_pos.borrow_mut();
    let axis_pos = axis_pos.deref_mut();
    axis_pos.clear();

    let mut axis_neg = self.axis_neg.borrow_mut();
    let axis_neg = axis_neg.deref_mut();
    axis_neg.clear();

    let mut spiral = false;

    let mut match_axis = |index: &mut PartIndex, a: Point, i: usize| {
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

    for (i, &a) in self.axis.iter().enumerate() {
      if !match_axis(&mut index, a, i) {
        return 0;
      }
    }

    if index.count_ones() == 1 {
      let hole_r = if r < sphere_r + 2.0 { 1.5 } else { 3.2 };
      for (i, &a) in self.axis.iter().enumerate() {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < hole_r {
          return 0;
        }
      }
    }

    if let Some(split) = self.extra_split.get(&index) {
      if !match_axis(&mut index, *split, self.axis.len()) {
        return 0;
      }
    }

    if let Some(cut) = self.extra_cuts.get(&index) {
      for cut in cut {
        let mut index2 = index;
        if !match_axis(&mut index2, *cut, self.axis.len() + 1) || index2 == index {
          return 0;
        }
      }
    }

    if let Some(cut) = self.extra_cutsp.get(&index) {
      for cut in cut {
        let mut index2 = index;
        if !match_axis(&mut index2, *cut, self.axis.len() + 1) || index2 != index {
          return 0;
        }
      }
    }

    if let Some(center) = self.centers.get(&index) {
      if r > sz {
        return 0;
      }
    }

    let rr = if index.count_ones() < 3 { 2.0 } else {
      f32::min(0.2 + (r - self.groove[2]).abs() * 0.8 / 3.0, 1.0)
    };

    let mut in_sr = |a: f32, b: f32| {
      let r = 0.03 * rr;
      if a < r && b < r && sqr(r - a) + sqr(r - b) > sqr(r) {
        return true;
      }
      false
    };

    axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());


    for a1 in &*axis_pos {
      for a2 in &*axis_pos {
        if dot(a1.1, a2.1) < 0.8 {
          if in_sr(a1.0, a2.0) {
            return 0;
          }
        }
      }
    }
    for a1 in &*axis_neg {
      for a2 in &*axis_neg {
        if dot(a1.1, a2.1) < 0.8 {
          if in_sr(a1.0, a2.0) {
            return 0;
          }
        }
      }
    }
    if !inter {
      for a1 in &*axis_pos {
        for a2 in &*axis_neg {
          if dot(a1.1, a2.1) > 0.0 {
            if in_sr(a1.0, a2.0) {
              return 0;
            }
          }
        }
      }
    }

    return index;
  }
}
