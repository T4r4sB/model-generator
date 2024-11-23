use crate::common_for_twisty_puzzles::*;
use crate::model::*;
use crate::points3d::*;
use crate::solid::*;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct PacificCreator {
  axis: Vec<Point>,
  axis1: Vec<Point>,
  axis2: Vec<Point>,
  normals: Vec<Point>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
  extras: Vec<PartIndex>,
  corner_cos: f32,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}
impl PacificCreator {
  pub fn new() -> Self {
    let phi = (5.0.sqrt() - 1.0) / 2.0;

    let s17 = (1.0 / 7.0).sqrt();
    let s37 = (3.0 / 7.0).sqrt();
    let s47 = (4.0 / 7.0).sqrt();
    let s67 = (6.0 / 7.0).sqrt();

    let axis: Vec<_> = [
      Point { x: -s37, y: -s37, z: -s17 },
      Point { x: -s37, y: s37, z: -s17 },
      Point { x: s37, y: -s37, z: -s17 },
      Point { x: s37, y: s37, z: -s17 },
      Point { x: -s37, y: -s37, z: s17 },
      Point { x: -s37, y: s37, z: s17 },
      Point { x: s37, y: -s37, z: s17 },
      Point { x: s37, y: s37, z: s17 },
      Point { x: -s37, y: 0.0, z: -s47 },
      Point { x: s37, y: 0.0, z: -s47 },
      Point { x: 0.0, y: -s37, z: s47 },
      Point { x: 0.0, y: s37, z: s47 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let mut extras = Vec::<PartIndex>::new();

    for i in 0..axis.len() {
      for j in i + 1..axis.len() {
        let a1 = axis[i];
        let a2 = axis[j];
        let d = dot(a1, a2);
        if d > 0.14 && d < 0.15 {
          /*  let asum = a1 + a2;
          let across = cross(a1, a2);
          let shift1 = across.scale(-2.0 * 7.0.sqrt() / 8.0) + asum.scale(-2.0 / 8.0);
          let shift2 = across.scale(2.0 * 7.0.sqrt() / 8.0) + asum.scale(-2.0 / 8.0);
          extras.push((1 << i | 1 << j, vec![a1 + shift1, a1 + shift2, a2 + shift1, a2 + shift2]));*/
          extras.push(1 << i | 1 << j);
        }
      }
    }

    let normals: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let main_cos = 0.2.sqrt();
    let main_angle = main_cos.acos();

    let corner_cos = s37;
    let corner_angle = corner_cos.acos();

    let minimal_angle_cos = s67;
    let minimal_angle = minimal_angle_cos.acos();

    let split_cos_out = 0.5;

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let groove = vec![
      (minimal_angle + 5.0 / 21.6).cos(),
      21.8,
      (minimal_angle + 2.0 / 21.6).cos(),
      24.2,
      (corner_angle + 0.0 / 26.4).cos(),
      26.6,
      (corner_angle - 3.0 / 26.4).cos(),
      29.0,
      (corner_angle + 0.0 / 26.4).cos(),
    ];

    let mut axis1 = Vec::new();
    let mut axis2 = Vec::new();
    for &a in &axis {
      let a1 = a.any_perp();
      let a2 = cross(a, a1);
      axis1.push(a1);
      axis2.push(a2);
    }

    Self { axis, axis1, axis2, normals, groove, axis_pos, axis_neg, extras, corner_cos }
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
    256
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 49.999 || pos.y.abs() > 49.999 || pos.z.abs() > 49.999 {
      return 0;
    }

    if r < 12.0 {
      return 0;
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
    let sz = last_groove + 2.0;

    let dx = sz - pos.x.abs();
    let dy = sz - pos.y.abs();
    let dz = sz - pos.z.abs();

    if dx < 0.0 || dy < 0.0 || dz < 0.0 {
      return 0;
    }

    let out_r = 2.0;

    let in_r = |a, b| {
      if a < out_r && b < out_r {
        if sqr(out_r - a) + sqr(out_r - b) > sqr(out_r) {
          return true;
        }
      }
      false
    };

    if in_r(dx, dy) || in_r(dx, dz) || in_r(dy, dz) {
      return 0;
    }

    if dx < out_r
      && dy < out_r
      && dz < out_r
      && sqr(out_r - dx) + sqr(out_r - dy) + sqr(out_r - dz) > sqr(out_r)
    {
      return 0;
    }

    let mut index: PartIndex = 0;

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);

    let depth = self.groove[1] + 0.2 - r;
    if depth >= 0.0 {
      shift_out += depth * 0.009;
      shift_in += depth * 0.009;
    }

    let hole_r = if r < 13.5 { 1.5 } else { 3.2 };

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();

    axis_pos.clear();
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

    if index != 17 {
      return 0;
    }

    if index >= 1 << self.axis.len() {
      return 0;
    }

    let mut thick = index.count_ones() == 2 && r > self.groove[1] - 0.2 && r < self.groove[3] + 0.2;
    if r > self.groove[5] - 0.2 && r < self.groove[7] + 0.2 {
      for &e in &self.extras {
        if index & e == e {
          thick = true;
          break;
        }
      }
    }

    if index.count_ones() == 1 {
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
        let r = 0.096 * d;
        if a < r && b < r && sqr(r - a) + sqr(r - b) > sqr(r) {
          return true;
        }
        false
      };

      axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
      axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

      let mut i = 0;
      while i + 1 < axis_pos.len() && dot(axis_pos[i].1, axis_pos[i + 1].1) > 0.99 {
        i += 1;
      }
      axis_pos.drain(..i);

      let mut i = 0;
      while i + 1 < axis_neg.len() && dot(axis_neg[i].1, axis_neg[i + 1].1) > 0.99 {
        i += 1;
      }
      axis_neg.drain(..i);

      if axis_pos.len() >= 2 {
        let d = dot(axis_pos[0].1, axis_pos[1].1);
        if d > 0.14 && in_sr(axis_pos[0].0, axis_pos[1].0, 1.0) {
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
        if d > 0.0 && in_sr(axis_pos[0].0, axis_neg[0].0, d) {
          return 0;
        }
      }
    }

    let mut sum_a = Point::zero();
    for i in 0..self.axis.len() {
      if index & (1 << i) != 0 {
        sum_a += self.axis[i];
      }
    }

    if index.count_ones() > 1 {
      let x_case = sum_a.x.abs() > sum_a.y.abs() + 0.1 && sum_a.x.abs() > sum_a.z.abs() + 0.1;
      let y_case = sum_a.y.abs() > sum_a.x.abs() + 0.1 && sum_a.y.abs() > sum_a.z.abs() + 0.1;
      let z_case = sum_a.z.abs() > sum_a.y.abs() + 0.1 && sum_a.z.abs() > sum_a.x.abs() + 0.1;
      let any_case = x_case || y_case || z_case;

      if dx < dy - 0.17 && dx < dz - 0.17 {
        if !any_case || r > last_groove + 2.54 && !x_case {
          return index | (1 << self.axis.len());
        } else if r > last_groove + 2.54 || x_case {
          return index;
        }
      }
      if dy < dx - 0.17 && dy < dz - 0.17 {
        if !any_case || r > last_groove + 2.54 && !y_case {
          return index | (1 << self.axis.len() + 1);
        } else if r > last_groove + 2.54 || y_case {
          return index;
        }
      }
      if dz < dy - 0.17 && dz < dx - 0.17 {
        if !any_case || r > last_groove + 2.54 && !z_case {
          return index | (1 << self.axis.len() + 2);
        } else if r > last_groove + 2.54 || z_case {
          return index;
        }
      }

      if any_case && r < last_groove + 2.30 {
        return index;
      }

      return 0;
    }

    if index == 0 {
      return 7;
    }

    return index;
  }
}
