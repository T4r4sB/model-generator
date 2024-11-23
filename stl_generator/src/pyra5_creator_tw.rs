use crate::common_for_twisty_puzzles::*;
use crate::model::*;
use crate::points3d::*;
use crate::solid::*;
use num::Float;
use num::PrimInt;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

fn sqr(x: f32) -> f32 {
  x * x
}

pub struct Pyra5Creator {
  axis: Vec<Point>,
  normals: Vec<Point>,

  a02_l: Point,
  a02_r: Point,
  a03_l: Point,
  a03_r: Point,
  a04_l: Point,
  a04_r: Point,
  a12_l: Point,
  a12_r: Point,
  a24_l: Point,
  a24_r: Point,

  r23_l: Point,
  r23_r: Point,
  r34_l: Point,
  r34_r: Point,
  r01_l: Point,
  r01_r: Point,
  r14_l: Point,
  r14_r: Point,

  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
}

impl Pyra5Creator {
  pub fn new() -> Self {
    let ball_radius = 25.0;

    let max_angle = (1.0 / 7.0).sqrt().acos();
    let ahole = max_angle - 0.5.acos();
    let r = 4.0 / ahole;

    let groove = vec![
      (max_angle).cos(),
      r + 0.2,
      (max_angle - 3.0 / r).cos(),
      r + 2.6,
      (max_angle).cos(),
    ];

    let tr = 0.75.sqrt();

    let axis = vec![
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.5 * tr, y: -tr, z: 0.25 },
      Point { x: -tr, y: 0.0, z: -0.5 },
      Point { x: 0.0, y: tr, z: -0.5 },
      Point { x: tr, y: 0.0, z: -0.5 },
    ];

    let normals = vec![
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: -0.5 * tr, y: -tr, z: -0.25 },
      Point { x: 0.0, y: tr, z: 0.5 },
      Point { x: -tr, y: 0.0, z: 0.5 },
      Point { x: tr, y: 0.0, z: 0.5 },
    ];

    fn reflect(p: Point, p1: Point, p2: Point) -> Point {
      let a = (p1 + p2).norm();
      a.scale(2.0 * dot(a, p)) - p
    }

    fn find_spec(p1: Point, p2: Point, dir: bool) -> Point {
      let mut cr = cross(p1, p2);
      p1.scale(0.4) + p2.scale(-0.6) + cr.scale(if dir { 0.8 } else { -0.8 })
    }

    let a02_l = find_square(axis[0], axis[2]);
    let a02_r = find_square(axis[2], axis[0]);
    let a03_l = find_square(axis[0], axis[3]);
    let a03_r = find_square(axis[3], axis[0]);
    let a04_l = find_square(axis[0], axis[4]);
    let a04_r = find_square(axis[4], axis[0]);
    let a12_l = find_square(axis[1], axis[2]);
    let a12_r = find_square(axis[2], axis[1]);
    let a24_l = find_square(axis[2], axis[4]);
    let a24_r = find_square(axis[4], axis[2]);

    let r23_l = find_spec(axis[2], axis[3], false);
    let r23_r = find_spec(axis[3], axis[2], true);

    let r34_l = find_spec(axis[3], axis[4], false);
    let r34_r = find_spec(axis[4], axis[3], true);

    let r01_l = find_spec(axis[0], axis[1], false);
    let r01_r = find_spec(axis[1], axis[0], true);

    let r14_l = find_spec(axis[1], axis[4], false);
    let r14_r = find_spec(axis[4], axis[1], true);

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    Self {
      axis,
      normals,

      a02_l,
      a02_r,
      a03_l,
      a03_r,
      a04_l,
      a04_r,
      a12_l,
      a12_r,
      a24_l,
      a24_r,

      r23_l,
      r23_r,
      r34_l,
      r34_r,
      r01_l,
      r01_r,
      r14_l,
      r14_r,

      groove,
      axis_pos,
      axis_neg,
    }
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    0.9
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    1
  }

  pub fn get_name(&self, current_normal: usize) -> Option<String> {
    None
  }

  pub fn get_size() -> f32 {
    120.0
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    let n = self.normals[current_normal];

    fn sinc(x: f32) -> f32 {
      if x == 0.0 {
        1.0
      } else {
        x.sin() / x
      }
    }

    fn versinc(x: f32) -> f32 {
      if x == 0.0 {
        0.0
      } else {
        (1.0 - x.cos()) / x
      }
    }

    let k = 0.006;
    let k2 = k * 2.0;
    let last_groove = self.groove[self.groove.len() - 2];
    let max = last_groove + 2.2;

    let r = pos.len();
    let a = r * k2;

    let n1 = n.any_perp().norm();
    let n2 = cross(n, n1).norm();

    let pos = pos.scale(sinc(a));
    let pos = n.scale(max - r * versinc(a)) + n1.scale(pos.x) + n2.scale(pos.y);

    let r = pos.len();
    let d = dot(pos, n);

    let control_c = n.scale(max - k2.recip());
    let delta = d + (max * max + r * r) * k - (2.0 * d * k + 1.0) * max;

    let result = self.get_part_index_impl(pos, current_normal);

    (result > 0) as PartIndex
  }

  pub fn faces(&self) -> usize {
    self.normals.len()
  }

  pub fn get_quality() -> usize {
    32
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if pos.x.abs() > 59.0 || pos.y.abs() > 59.0 || pos.z.abs() > 59.0 {
      return 0;
    }

    let r = pos.len();

    let inner_r = self.groove[1] - 2.2;
    if r < inner_r {
      if r > inner_r - 0.3 {
        return 0;
      }
      let cz = pos.z * 0.98 - pos.y * 0.2;
      let mut index = if cz > 0.0 { 33 } else { 34 };
      if cz.abs() < 0.15 {
        return 0;
      }

      //let small_cos = (5.0/8.0).sqrt();
      let flat_r = inner_r; // * small_cos;

      for (i, &a) in self.axis.iter().enumerate() {
        let d = dot(pos, a);
        let s = cross(pos, a).len();
        let max = if d < flat_r - 3.0 { 3.2 } else { 1.5 };
        if d > 0.0 && s < max {
          return 0;
        }
      }

      let shaft_begin = Point { x: 0.0, y: 0.0, z: -inner_r };
      for shaft_dir in [
        Point { x: 0.35, y: 0.35, z: 1.0 },
        Point { x: -0.35, y: -0.35, z: 1.0 },
      ] {
        let shaft_dir = shaft_dir.norm();
        let shaft_head_l = inner_r / shaft_dir.z - 5.0;

        let d = dot(pos - shaft_begin, shaft_dir);
        let s = cross(pos - shaft_begin, shaft_dir).len();
        let max_s = if d < shaft_head_l {
          3.2
        } else if cz < 0.0 {
          1.5
        } else {
          1.25
        };
        if s < max_s {
          return 0;
        }
      }

      return index;
    }

    let m = self.groove[3] + 2.2;
    let mut cup = false;
    let mut near_another_wall = false;
    for ni in 0..self.normals.len() {
      if ni == current_normal {
        cup = true;
        continue;
      }

      let mut m = m;

      if current_normal < self.normals.len() {
        m -= 2.0;
      }

      let d = dot(pos, self.normals[ni]);
      let k = 0.006;
      if d + (m * m + r * r) * k > (2.0 * d * k + 1.0) * m {
        return 0;
      }
      {
        let m = m - 2.0;
        if d + (m * m + r * r) * k > (2.0 * d * k + 1.0) * m {
          cup = true;
        }
      }
      if current_normal < self.normals.len() {
        let m = m - 1.0;
        if d + (m * m + r * r) * k > (2.0 * d * k + 1.0) * m {
          near_another_wall = true;
        }
      }
    }

    if !cup {
      for &a in &self.axis {
        let d = dot(pos, a);
        let s = cross(pos, a).len();
        if d > 0.0 && s < 1.25 {
          return 0;
        }
      }
    }

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();

    axis_pos.clear();
    axis_neg.clear();

    let mut index: PartIndex = 0;

    let mut match_all = || -> Option<()> {
      let mut match_axis =
        |pos: Point, index: &mut PartIndex, bit: usize, axis: Point| -> Option<()> {
          let d = dot(pos, axis) / r;

          let check_in = d - shift_in;
          if check_in > 0.0 {
            *index |= (1 << bit);
            axis_pos.push((check_in, axis));
          } else {
            let check_out = shift_out - d;
            if check_out > 0.0 {
              axis_neg.push((check_out, axis));
            } else {
              return None;
            }
          }

          return Some(());
        };
      for i in 0..self.axis.len() {
        match_axis(pos, &mut index, i, self.axis[i])?;
      }

      if index == 5 {
        match_axis(pos, &mut index, 5, self.a02_l)?;
        match_axis(pos, &mut index, 6, self.a02_r)?;
      }
      if index == 9 {
        match_axis(pos, &mut index, 5, self.a03_l)?;
        match_axis(pos, &mut index, 6, self.a03_r)?;
      }
      if index & 2 == 2 {
        match_axis(pos, &mut index, 7, self.a04_r)?;
      }
      if index & 8 == 8 {
        match_axis(pos, &mut index, 7, self.a24_r)?;
      }
      if index == 6 {
        match_axis(pos, &mut index, 5, self.a12_l)?;
        match_axis(pos, &mut index, 6, self.a12_r)?;
      }

      if index == 12 {
        match_axis(pos, &mut index, 5, self.r23_l)?;
        match_axis(pos, &mut index, 6, self.r23_r)?;
      }
      if index == 24 {
        match_axis(pos, &mut index, 5, self.r34_l)?;
        match_axis(pos, &mut index, 6, self.r34_r)?;
      }
      if index == 3 {
        match_axis(pos, &mut index, 5, self.r01_l)?;
        match_axis(pos, &mut index, 6, self.r01_r)?;
      }
      if index == 18 {
        match_axis(pos, &mut index, 5, self.r14_l)?;
        match_axis(pos, &mut index, 6, self.r14_r)?;
      }

      if index != 1 && index != 16 {
      //  return None;
      }

      Some(())
    };

    if match_all().is_none() {
      return 0;
    }

    let mut in_sr = |a, b, d| {
      let r = 0.096 * d;
      if a < r && b < r {
        return r - (sqr(r - a) + sqr(r - b)).sqrt();
      }
      return f32::INFINITY;
    };

    let thick = index.count_ones() == 3 && r > self.groove[1] - 0.2 && r < self.groove[3] + 0.2;

    let mut rr = if index.count_ones() == 4 {
      if thick {
        0.1f32
      } else {
        0.3f32
      }
    } else {
      if thick {
        0.3f32
      } else {
        1.0f32
      }
    };

    axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    let mut minimal = axis_pos
      .iter()
      .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
      .map(|(a, _)| *a)
      .unwrap_or(f32::INFINITY);
    minimal = f32::min(
      minimal,
      axis_neg
        .iter()
        .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
        .map(|(a, _)| *a)
        .unwrap_or(f32::INFINITY),
    );

    if axis_pos.len() >= 2 {
      let d = dot(axis_pos[0].1, axis_pos[1].1);
      if !thick || d > 0.0 {
        minimal = f32::min(minimal, in_sr(axis_pos[0].0, axis_pos[1].0, rr));
      }
    }
    if axis_neg.len() >= 2 {
      minimal = f32::min(minimal, in_sr(axis_neg[0].0, axis_neg[1].0, rr));
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
      minimal = f32::min(minimal, in_sr(axis_pos[0].0, axis_neg[0].0, rr));
    }

    if minimal < 0.0 {
      return 0;
    }
    if current_normal < self.normals.len() {
      if minimal < 0.03 {
        return 0;
      }
      if !near_another_wall && minimal > 0.07 {
        return 0;
      }
    }

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.axis.len())
  }
}
