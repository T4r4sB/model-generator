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

pub struct SixdecaminxCreator {
  axis: Vec<Point>,
  normals: Vec<Point>,
  holes: Vec<Point>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl SixdecaminxCreator {
  pub fn new() -> Self {
    let ha = PI / 7.0;
    let hac = ha.cos();
    let cx = hac / (hac + 1.0);
    let sx = (1.0 - sqr(cx)).sqrt();

    let pa = (1.0 + 2.0 * cx).sqrt();
    let pb = (1.0 - 2.0 * cx).sqrt();

    let ca = (pb + pa) / 2.0;
    let sa = (pb - pa) / 2.0;

    let a = ha * 2.0;

    let mut axis: Vec<_> = [
      Point { x: ca, y: sa, z: 0.0 },
      Point { x: 0.0, y: ca, z: sa },
      Point { x: sa, y: 0.0, z: ca },
      Point { x: -ca, y: -sa, z: 0.0 },
      Point { x: 0.0, y: -ca, z: -sa },
      Point { x: -sa, y: 0.0, z: -ca },
    ]
    .into_iter()
    .collect();

    axis.push(axis[5].rotate(axis[0], a));
    axis.push(axis[5].rotate(axis[0], a * 2.0));
    axis.push(axis[5].rotate(axis[0], a * 3.0));

    axis.push(axis[5].rotate(axis[1], a));
    axis.push(axis[5].rotate(axis[1], a * 2.0));

    axis.push(axis[2].rotate(axis[3], a));
    axis.push(axis[2].rotate(axis[3], a * 2.0));

    axis.push(axis[2].rotate(axis[4], a));
    axis.push(axis[2].rotate(axis[4], a * 2.0));
    axis.push(axis[2].rotate(axis[4], a * 3.0));

    let mut holes = Vec::new();
    for i in 0..axis.len() {
      for j in i + 1..axis.len() {
        for k in j + 1..axis.len() {
          if dot(axis[i], axis[j]) + dot(axis[i], axis[k]) + dot(axis[j], axis[k]) > 1.6 {
            holes.push((axis[i] + axis[j] + axis[k]).norm());
          }
        }
      }
    }

    let normals = axis.clone();

    let r_a = -sqr(hac) + (1.0 - sqr(hac)) * cx;
    let r = ((cx - r_a) / (1.0 - r_a)).sqrt();
    let nd1 = sqr(cx) + sqr(sx) * (a).cos();
    let nd2 = sqr(cx) + sqr(sx) * (a * 2.0).cos();

    let maximal_angle_1 = nd1.acos();
    let maximal_angle_2 = nd2.acos() * 0.5;
    let minimal_angle = r.acos();
    let sphere_r =
      f32::max(6.25 / (maximal_angle_1 - minimal_angle), 3.0 / (maximal_angle_2 - minimal_angle));

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let groove = vec![
      (minimal_angle + 4.5 / sphere_r).cos(),
      sphere_r + 0.2,
      (minimal_angle + 1.5 / sphere_r).cos(),
      sphere_r + 2.6,
      (maximal_angle_2).cos(),
    ];

    Self { axis, holes, normals, groove, axis_pos, axis_neg }
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

    let last_groove = self.groove[self.groove.len() - 2];
    let sz = last_groove + 2.2;
    let p = n0.scale(sz) + n1.scale(pos.x) + n2.scale(pos.y);
    (self.get_part_index_impl(p, current_normal) > 0) as PartIndex
  }

  pub fn get_quality() -> usize {
    128
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();

    if pos.y < 0.0 {
      // return 0;
    }

    let sphere_r = self.groove[1] - 2.2;

    if r < sphere_r {
      return 0; // tmp
      if r > sphere_r - 0.2 || r < sphere_r - 5.2 {
        return 0;
      }
      for &a in &self.axis {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < 1.5 {
          return 0;
        }
      }
      for &h in &self.holes {
        let c = dot(pos, h) / r;
        let s = cross(pos, h).len();
        if c > 0.0 && s < 6.0 {
          return 0;
        }
      }
      return 31;
    }

    let mut out_core = false;
    let last_groove = self.groove[self.groove.len() - 2];
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

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);
    let dr = (self.groove[1] - 0.2) - r;
    if dr > 0.0 {
      shift_out += dr * 0.003;
      shift_in += dr * 0.003;
    }

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

    for (i, &a) in self.axis.iter().enumerate() {
      if !match_axis(&mut index, a, i, shift_in, shift_out) {
        return 0;
      }
    }

    let mut thick = false;

    if index.count_ones() == 1 && r < sz - 3.0 {
      let hole_r = 1.25;
      for (i, &a) in self.axis.iter().enumerate() {
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
        if in_sr(axis_pos[0].0, axis_pos[1].0, 1.0) {
          return 0;
        }
      }
      if axis_neg.len() >= 2 {
        let d = dot(axis_neg[0].1, axis_neg[1].1);
        if in_sr(axis_neg[0].0, axis_neg[1].0, 1.0) {
          return 0;
        }
      }
      if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
        let d = dot(axis_pos[0].1, axis_neg[0].1);
        if d <= 0.06 {
          if axis_pos.len() >= 2 {
            let d = dot(axis_pos[1].1, axis_neg[0].1);
            if d >= 0.06 {
              if in_sr(axis_pos[1].0, axis_neg[0].0, 1.0) {
                return 0;
              }
            }
          }
          if axis_neg.len() >= 2 {
            let d = dot(axis_pos[0].1, axis_neg[1].1);
            if d >= 0.06 {
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

    return index;
  }
}
