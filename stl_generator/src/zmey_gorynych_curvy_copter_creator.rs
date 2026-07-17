use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::{FxHashMap, FxHashSet};
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
  sz: f32,
  groove_inner: Vec<f32>,
  groove: Vec<f32>,
  n_dists: RefCell<Vec<(f32, usize)>>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
  centers: FxHashMap<PartIndex, Vec<Point>>,
  extra_split: FxHashMap<PartIndex, Point>,
  extra_cuts: FxHashMap<PartIndex, Vec<Point>>,
  extra_cutsp: FxHashMap<PartIndex, Vec<Point>>,
  corners: FxHashSet<PartIndex>,
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

    let ia_max = c_min.acos() * 0.5;
    let ia_min = ((edge * 2.0 + 1.0) / 3.0).sqrt().acos();

    let ir = 4.0 / (ia_max - ia_min);
    println!("ir={ir}");

    let r = 4.8 / f32::max(0.1, a_max - a_min);
    let groove_inner = vec![
      (ia_max + 3.0 / (r - 8.0)).cos(),
      r - 7.8,
      (ia_max + 0.0 / (r - 8.0)).cos(),
      r - 5.4,
      a_max.cos(),
    ];
    let groove = vec![r - 3.0, a_max.cos(), r + 0.1, (a_max - 3.0 / r).cos(), r + 3.2, a_max.cos()];
    let sz = groove[groove.len() - 2] + 0.3;
    println!("r={r}, ta={}, sz={sz}", tac.acos().to_degrees());

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
    let mut corners = FxHashSet::default();
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

          let mut cn = (Point::ZERO, -f32::INFINITY);
          for &n in &normals {
            let ca = dot(center, n);
            if ca > cn.1 {
              cn = (n, ca);
            }
          }
          let cn = cn.0;

          extra_split.insert(1 << i1 | 1 << i2 | 1 << i3, a3.rotate(center, PI));
          extra_split.insert(1 << i1 | 1 << i3, a3.rotate(center, ta * 2.0));
          extra_split.insert(1 << i2 | 1 << i3, a3.rotate(center, -ta * 2.0));
          centers.insert(
            1 << i1 | 1 << i2 | 1 << i3 | 1 << axis.len(),
            vec![cn.rotate(center, PI * 0.5), cn.rotate(center, PI), cn.rotate(center, -PI * 0.5)],
          );

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

    for i1 in 0..axis.len() {
      let a1 = axis[i1];
      for i2 in 0..axis.len() {
        let a2 = axis[i2];
        if (dot(a1, a2) - edge).abs() > 0.001 {
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

          corners.insert(1 << i1 | 1 << i2 | 1 << i3);
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
      sz,
      normals,
      groove_inner,
      groove,
      axis_pos,
      axis_neg,
      n_dists,
      extra_split,
      extra_cuts,
      extra_cutsp,
      centers,
      corners,
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
    if current_normal == 0 {
      self.get_part_index_impl(Point { x: pos.x, y: pos.y, z: 0.0 }, self.normals.len())
    } else if current_normal == 1 {
      self.get_part_index_impl(Point { x: pos.x, y: pos.y, z: self.sz - 9.5 }, self.normals.len())
    } else {
      0
    }
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

    if r > self.groove_inner[1] + 2.0 {
      //  return 0;
    }

    let sphere_r = self.groove_inner[1] - 2.4;
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
    let sz = self.sz;

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
      n_dists.push((d, i));
    }

    n_dists.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    let out_r = 3.0;
    let mut fd = out_r
      - (sqr(out_r - f32::min(n_dists[0].0, out_r))
        + sqr(out_r - f32::min(n_dists[1].0, out_r))
        + sqr(out_r - f32::min(n_dists[2].0, out_r)))
      .sqrt();

    if fd < 0.0 {
      return 0;
    }

    let mut index: PartIndex = 0;

    let (mut shift_out, mut shift_in, inter);

    if r > self.groove[0] {
      (shift_out, shift_in, inter) = get_diag_groove(r, &self.groove);
    } else {
      (shift_out, shift_in, inter) = get_groove(r, &self.groove_inner, 0.03);
    }

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

    if let Some(split) = self.extra_split.get(&index) {
      if !match_axis(&mut index, *split, self.axis.len()) {
        return 0;
      }
    }

    if r < self.groove_inner[3] + 0.2 && index.count_ones() >= 3 && !self.corners.contains(&index) {
      return 0;
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

    if let Some(normals) = self.centers.get(&index) {
      for &n in normals {
        fd = f32::min(fd, sz - dot(pos, n));
      }
      if fd < 0.0 {
        return 0;
      }
    }

    let rr = if index.count_ones() < 3 {
      2.0
    } else {
      f32::min(0.2 + (r - self.groove[2]).abs() * 0.8 / 3.0, 1.0)
    };

    let mut in_sr = |a: f32, b: f32| -> f32 {
      let r = 0.03 * rr;
      if a < r && b < r {
        return r - (sqr(r - a) + sqr(r - b)).sqrt();
      }
      return f32::min(a, b);
    };

    axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    let mut cd = f32::INFINITY;
    if axis_pos.len() >= 1 {
      cd = f32::min(cd, axis_pos[0].0);
    }
    if axis_neg.len() >= 1 {
      cd = f32::min(cd, axis_neg[0].0);
    }

    for a1 in &*axis_pos {
      for a2 in &*axis_pos {
        if dot(a1.1, a2.1) < 0.8 {
          cd = f32::min(cd, in_sr(a1.0, a2.0));
          if cd < 0.0 {
            return 0;
          }
        }
      }
    }
    for a1 in &*axis_neg {
      for a2 in &*axis_neg {
        if dot(a1.1, a2.1) < 0.8 {
          cd = f32::min(cd, in_sr(a1.0, a2.0));
          if cd < 0.0 {
            return 0;
          }
        }
      }
    }
    if !inter {
      for a1 in &*axis_pos {
        for a2 in &*axis_neg {
          let tan_case = r < self.groove_inner[3] - 0.2 && r > self.groove_inner[1] + 0.2;
          if dot(a1.1, a2.1) > if tan_case { 0.2 } else { 0.0 } {
            cd = f32::min(cd, in_sr(a1.0, a2.0));
            if cd < 0.0 {
              return 0;
            }
          }
        }
      }
    }

    cd *= 200.0;
    if fd < out_r && cd < out_r && sqr(out_r - fd) + sqr(out_r - cd) > sqr(out_r) {
      // return 0;
    }

    if index.count_ones() == 1 || self.corners.contains(&index) {
      if n_dists[0].0 < 3.7 {
        let mut cp = f32::INFINITY;
        cp = f32::min(
          cp,
          f32::max((n_dists[1].0 - 9.0).abs() - 4.5, (n_dists[2].0 - 9.0).abs() - 4.5),
        );
        if index.count_ones() == 1 {
          let ma = axis_pos[0].1;
          if dot(ma, self.normals[n_dists[0].1]) > dot(ma, self.normals[n_dists[1].1]) {
            cp = f32::min(
              cp,
              f32::max((n_dists[1].0 - 10.0).abs() - 5.5, (n_dists[2].0 - sz).abs() - 3.5),
            );
            cp = f32::min(
              cp,
              f32::max((n_dists[1].0 - 6.5).abs() - 2.0, (n_dists[2].0 - sz).abs() - 6.5),
            );
          } else {
            cp = f32::min(
              cp,
              f32::max((n_dists[1].0 - 6.5).abs() - 2.0, (n_dists[2].0 - sz).abs() - 2.5),
            );
          }
        }

        if n_dists[0].0 < 1.7 || cp < 0.12 && n_dists[0].0 < 3.7 {
          if n_dists[0].0 + 0.2 * 0.5.sqrt() > n_dists[1].0 {
            return 0;
          }

          if n_dists[0].0 < 1.4 || cp < 0.0 && n_dists[0].0 < 3.4 {
            index += 100000 * (n_dists[0].1 + 1) as PartIndex;
          } else {
            return 0;
          }
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
    } else {
      let mut sum = Point::ZERO;
      for (_, p) in axis_pos {
        sum += *p;
      }
      let mut cn = (0, -f32::INFINITY);
      for i in 0..self.normals.len() {
        let ca = dot(sum, self.normals[i]);
        if ca > cn.1 {
          cn = (i + 1, ca);
        }
      }
      index += 100000 * cn.0 as PartIndex;
    }

    return index;
  }
}
