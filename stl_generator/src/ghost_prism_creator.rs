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

pub struct GhostPrismCreator {
  axis: Vec<Point>,
  axis1: Vec<Point>,
  axis2: Vec<Point>,
  z_axis: Point,
  a2_for_1: Point,
  maximal_angle_cos: f32,
  normals: Vec<Point>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
  axis_seq: Vec<(usize, f32)>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl GhostPrismCreator {
  pub fn new() -> Self {
    let a = 65.0 * PI / 180.0;
    let b = (2.0 * PI - 2.0 * a) / 3.0;
    let x = PI * 0.5 - ((a.cos() - b.cos()) / (2.0 + a.cos() - b.cos())).sqrt().acos();

    fn from_polar_coords(longitude: f32, lattitude: f32) -> Point {
      let (s1, c1) = longitude.sin_cos();
      let (s2, c2) = lattitude.sin_cos();
      Point { x: c2 * c1, y: c2 * s1, z: s2 }
    }

    fn make_ghost(p: Point) -> Point {
      p.rotate(Point { x: 1.0, y: 2.0, z: 3.0 }.norm(), PI * 0.64)
    }

    let axis: Vec<_> = [
      from_polar_coords(0.0, x),
      from_polar_coords(b, x),
      from_polar_coords(b + b, x),
      from_polar_coords(b + b + a, -x),
      from_polar_coords(b + b + a + b, -x),
    ]
    .map(make_ghost)
    .into_iter()
    .collect();

    let z_axis = make_ghost(Point { x: 0.0, y: 0.0, z: 1.0 });

    let edge = dot(axis[0], axis[1]);
    let long_edge = dot(axis[0], axis[3]);
    let rot_angle = ((long_edge - sqr(edge)) / (1.0 - sqr(edge))).acos();
    let rot_angle_s = ((dot(axis[0], axis[2]) - sqr(edge)) / (1.0 - sqr(edge))).acos();

    let a2_for_1 = axis[0].rotate(axis[1], rot_angle);

    let axis_seq = vec![(2, PI * 0.5), (1, rot_angle_s), (0, -rot_angle), (4, rot_angle)];

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

    let maximal_angle = long_edge.acos() * 0.5;
    let maximal_angle_cos = maximal_angle.cos();
    let sphere_r = 15.0;

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let groove = vec![
      (maximal_angle - 2.5 / 15.0).cos(),
      15.2,
      (maximal_angle - 4.5 / 15.0).cos(),
      23.8,
      (maximal_angle - 0.0).cos(),
    ];

    let mut axis1 = Vec::new();
    let mut axis2 = Vec::new();
    for &a in &axis {
      let a1 = a.any_perp();
      let a2 = cross(a, a1);
      axis1.push(a1);
      axis2.push(a2);
    }

    Self {
      axis,
      axis1,
      axis2,
      z_axis,
      a2_for_1,
      maximal_angle_cos,
      normals,
      groove,
      axis_pos,
      axis_neg,
      axis_seq,
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
    128
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
      for &a in &self.axis {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < 1.25 {
          return 0;
        }
      }
      return 31;
    }

    let mut out_core = false;
    let last_groove = self.groove[self.groove.len() - 2];
    let sz = 26.0;

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
    let over_r = f32::min(sz - 1.0, r) - (self.groove[1] + 2.2);

    if over_r > 0.0 {
      //  shift_in = f32::max(self.maximal_angle_cos, shift_in - over_r * 0.021);
      //   shift_out = f32::max(self.maximal_angle_cos, shift_out - over_r * 0.021);
    }

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();

    axis_pos.clear();
    axis_neg.clear();

    let mut spiral = false;

    enum PositionRelAxis {
      Inside(f32),
      Between,
      Outside(f32),
    };

    let get_pos_rel_axis = |pos: Point, a: Point, i: usize| {
      let c = dot(pos, a) / r;
      let s = cross(pos, a).len();
      let check_in = c - shift_in;
      if check_in > 0.0 {
        return PositionRelAxis::Inside(check_in);
      } else {
        let check_out = shift_out - c;
        if check_out > 0.0 {
          return PositionRelAxis::Outside(check_out);
        } else {
          return PositionRelAxis::Between;
        }
      }
    };

    let mut match_axis = |index: &mut PartIndex, pos: Point, a: Point, i: usize| {
      match get_pos_rel_axis(pos, a, i) {
        PositionRelAxis::Inside(check_in) => {
          *index |= (1 << i);
          axis_pos.push((check_in, self.axis[i]));
        }
        PositionRelAxis::Outside(check_out) => {
          axis_neg.push((check_out, self.axis[i]));
        }
        PositionRelAxis::Between => return false,
      }

      true
    };

    let mut pos = pos;

    for &(i, ra) in &self.axis_seq {
      match get_pos_rel_axis(pos, self.axis[i], i) {
        PositionRelAxis::Inside(_) => {
          pos = pos.rotate(self.axis[i], ra);
        }
        _ => {}
      }
    }

    for (i, &a) in self.axis.iter().enumerate() {
      let a = if index & 2 != 0 && i == 2 { self.a2_for_1 } else { self.axis[i] };
      if !match_axis(&mut index, pos, a, i) {
        return 0;
      }
    }

    let color_r = self.groove[1] + 0.4;
    let screw_r = self.groove[1] + 2.2;
    let hole_r = if r < screw_r {
      1.5
    } else if r < color_r {
      3.2
    } else {
      3.2
    };

    if r < 24.0 {
      for (i, &a) in self.axis.iter().enumerate() {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 {
          if s < hole_r {
            return 0;
          }
          if index.count_ones() == 1 && r > screw_r && s < hole_r + 0.0 {
            return 0;
          }
        }
      }
    }

    let mut thick = false;

    if !thick {
      if spiral {
        //  return 0;
      }

      let mut in_sr = |a, b, d: f32| {
        let r = 0.1 * d;
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
        if d > -0.99 && in_sr(axis_pos[0].0, axis_pos[1].0, 1.0) {
          return 0;
        }
      }
      if axis_neg.len() >= 2 {
        let d = dot(axis_neg[0].1, axis_neg[1].1);
        if in_sr(axis_neg[0].0, axis_neg[1].0, 1.0) {
          return 0;
        }
      }
      if axis_pos.len() >= 1 && axis_neg.len() >= 1 {
        let d = dot(axis_pos[0].1, axis_neg[0].1);
        if d > -0.5 && in_sr(axis_pos[0].0, axis_neg[0].0, 1.0) {
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

    if index == 0 {
      if dot(pos, self.z_axis) > 0.0 {
        index = 29;
      } else {
        index = 30;
      }
    }

    return index; // if dont need colors

    if index < 256 {
      if dx < dy - 0.17 && dx < dz - 0.17 {
        return index | (1 << self.axis.len() + 1);
      }
      if dy < dx - 0.17 && dy < dz - 0.17 {
        return index | (1 << self.axis.len() + 2);
      }
      if dz < dy - 0.17 && dz < dx - 0.17 {
        return index | (1 << self.axis.len() + 3);
      }

      return 0;
    }

    return index;
  }
}
