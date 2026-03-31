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

pub struct BeanCreator {
  axis: Vec<Point>,
  axis1: Vec<Point>,
  axis2: Vec<Point>,
  normals: Vec<Point>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl BeanCreator {
  pub fn new() -> Self {
    let phi = (5.0.sqrt() - 1.0) / 2.0;

    let a = 0.48;
    let b = ((4.0 / 9.0) / (a * a + 1.0 / 3.0) - 1.0 / 3.0).sqrt();
    let sa = (1.0 - a * a).sqrt();
    let sb = (1.0 - b * b).sqrt();

    //(a*a+1/3)(b*b+1/3)=4/9

    let a = 1.0/3.0;
    let b = 2.0/3.0;

    let axis: Vec<_> = [
      Point { x: -a, y: b, z: b },
      Point { x: b, y: -a, z: b },
      Point { x: b, y: b, z: -a },
      Point { x: a, y: -b, z: -b },
      Point { x: -b, y: a, z: -b },
      Point { x: -b, y: -b, z: a },
    ]
    .into_iter()
    .collect();

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

    let maximal_angle = a.sqrt().acos();
    let sphere_r = 20.0;

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let groove = vec![
      (maximal_angle + 0.03 / sphere_r).cos(),
      sphere_r + 0.2,
      (maximal_angle - 0.17 / sphere_r).cos(),
      sphere_r + 5.0,
      (maximal_angle + 0.03 / sphere_r).cos(),
    ];

    let mut axis1 = Vec::new();
    let mut axis2 = Vec::new();
    for &a in &axis {
      let a1 = a.any_perp();
      let a2 = cross(a, a1);
      axis1.push(a1);
      axis2.push(a2);
    }

    Self { axis, axis1, axis2, normals, groove, axis_pos, axis_neg }
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
      return 0;
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
    let sz = last_groove + 2.2;

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

    let color_r = self.groove[3] - 2.0;
    let hole_r = if r < 14.4 {
      1.5
    } else if r < color_r {
      3.2
    } else {
      4.2
    };

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

    let get_pos_rel_axis = |pos: Point, i: usize| {
      let a = self.axis[i];
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

    let mut match_axis = |index: &mut PartIndex, pos: Point, i: usize| {
      match get_pos_rel_axis(pos, i) {
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

    for (i, &a) in self.axis.iter().enumerate() {
      if !match_axis(&mut index, pos, i) {
        return 0;
      }
    }

    let mut thick = false;

    if index.count_ones() == 1 {
      if r < color_r - 0.4 {
        index += 256;
      } else if r < color_r {
        return 0;
      }

      if r < 25.0 {
        for (i, &a) in self.axis.iter().enumerate() {
          let c = dot(pos, a) / r;
          let s = cross(pos, a).len();
          if c > 0.0 && s < hole_r {
            return 0;
          }
        }
      }
    }

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
        if d > -9.99 && in_sr(axis_pos[0].0, axis_neg[0].0, 1.0) {
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
