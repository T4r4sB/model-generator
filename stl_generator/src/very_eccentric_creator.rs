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

pub struct VeryEccentricCreator {
  axis: Vec<Point>,
  axis1: Vec<Point>,
  axis2: Vec<Point>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<f32>>,
  axis_shape: RefCell<Vec<f32>>,
  axis_neg: RefCell<Vec<f32>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}
impl VeryEccentricCreator {
  pub fn new() -> Self {
    let phi = (5.0.sqrt() - 1.0) / 2.0;

    let axis: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let main_cos = 0.0;
    let main_angle = main_cos.acos();

    let corner_cos = ((main_cos * 2.0 + 1.0) / 3.0).sqrt();
    let corner_angle = corner_cos.acos();

    let edge_cos = ((main_cos + 1.0) / 2.0).sqrt();
    let edge_angle = edge_cos.acos();

    let main_r = f32::max(16.5 - 4.4, 10.7 / (main_angle - corner_angle));
    println!("outR={}", main_r + 4.4);

    let split_cos_out = 0.05;
    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());
    let axis_shape = RefCell::new(Vec::new());

    let groove = vec![
      (corner_angle + 0.0 / main_r).cos(),
      main_r - 2.6,
      (corner_angle + 5.0 / main_r).cos(),
      main_r + 0.2,
      (corner_angle + 0.0 / main_r).cos(),
    ];

    let mut axis1 = Vec::new();
    let mut axis2 = Vec::new();
    for &a in &axis {
      let a1 = a.any_perp();
      let a2 = cross(a, a1);
      axis1.push(a1);
      axis2.push(a2);
    }

    Self { axis, axis1, axis2, groove, axis_pos, axis_neg, axis_shape }
  }

  pub fn faces(&self) -> usize {
    0
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, 1)
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    0.6
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    1
  }

  pub fn get_name(&self, current_normal: usize) -> Option<&str> {
    None
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn get_quality() -> usize {
    512
  }

  pub fn get_size() -> f32 {
    120.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 44.999 || pos.y.abs() > 44.999 || pos.z.abs() > 44.999 {
      return 0;
    }

    if pos.x > 0.0 {
    //  return 0;
    }

    let ball_r = 6.25;
    if r < ball_r {
      if r > ball_r - 0.3 {
        return 0;
      }
      return 0; // tmp
      for i in 0..6 {
        let a = self.axis[i];
        let d = dot(pos, a);
        let s = cross(pos, a).len();
        if d > 0.0 && s < 1.25 {
          return 0;
        }
      }
      return 65;
    }

    //return 0;

    let mut index: PartIndex = 0;

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.04);

    shift_out -= 1.5 / r;
    shift_in -= 1.5 / r;

    let min_r = self.groove[1] - 0.2;
    let max_r = 13.0;

    let mut out_shape = false;

    shift_in = f32::max(shift_in, 4.2 / r);
    shift_out = f32::max(shift_out, 4.2 / r);

    let (mut shift_in_unfiltered, mut shift_out_unfiltered) = (shift_in, shift_out);
    if shift_in > max_r / r {
      shift_in = max_r / r;
      shift_out = max_r / r;
      out_shape = true;
    }
    shift_in_unfiltered = f32::min(shift_in_unfiltered, (max_r + 4.0) / r);
    shift_out_unfiltered = f32::min(shift_out_unfiltered, (max_r - 4.0) / r);

    let hole_r = if r > ball_r + 2.0 { 3.2 } else { 1.5 };

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    let mut axis_shape = self.axis_shape.borrow_mut();
    axis_pos.clear();
    axis_neg.clear();
    axis_shape.clear();

    for i in 0..6 {
      let a = self.axis[i];
      let c = dot(pos, a) / r;
      let s = cross(pos, a).len();
      let check_in = c - shift_in;
      if check_in > 0.0 {
        index += (1 << i) as PartIndex;
        axis_pos.push(check_in);
      } else {
        let check_out = shift_out - c;
        if check_out > 0.0 {
          axis_neg.push(check_out);
        } else {
          return 0;
        }
      }
    }

    if index != 1 && index != 3 {
    //  return 0;
    }

    let mut dist_to_surface = f32::INFINITY;
    let sphere_r = self.groove[self.groove.len() - 2] + 4.2;

    if index & 56 != 0 {
      dist_to_surface = sphere_r - r;
    } else {
      dist_to_surface = f32::min(dist_to_surface, max_r * 3.0 + pos.x);
      dist_to_surface = f32::min(dist_to_surface, max_r * 3.0 + pos.y);
      dist_to_surface = f32::min(dist_to_surface, max_r * 3.0 + pos.z);
      dist_to_surface = f32::min(dist_to_surface, max_r * 1.0 - pos.x);
      dist_to_surface = f32::min(dist_to_surface, max_r * 1.0 - pos.y);
      dist_to_surface = f32::min(dist_to_surface, max_r * 1.0 - pos.z);
    }

    if dist_to_surface < 0.0 {
      return 0;
    }

    for i in 0..6 {
      let a = self.axis[i];
      let c = dot(pos, a) / r;
      let s = cross(pos, a).len();
      if c > 0.0 && dist_to_surface > 1.0 && s < hole_r {
        return 0;
      }
    }

    axis_pos.sort_by(|a, b| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if index & 56 == 0 && r > sphere_r {
      let mut empty2 = false;

      for i in 0..6 {
        let a = self.axis[i];
        let c = dot(pos, a);
        if c > max_r * 3.0 - 4.0 {
          empty2 = true;
        } else if c > shift_in_unfiltered * r {
        } else if c > shift_out_unfiltered * r {
          empty2 = true;
        }
      }
      if index & 1 != 0 && pos.z > -max_r * 1.0
        || index & 2 != 0 && pos.y > -max_r * 1.0
        || index & 4 != 0 && pos.x > -max_r * 1.0
        || index & 8 != 0 && pos.z < max_r * 1.0
        || index & 16 != 0 && pos.y < max_r * 1.0
        || index & 32 != 0 && pos.x < max_r * 1.0
      {
        empty2 = false;
      }

      if empty2 {
        let mut surf_index = 0;
        let mut closest = 25.0;
        axis_shape.clear();

        macro_rules! try_snap {
          ($field: ident, $wi: expr, $i: expr, $t: expr) => {
            if index != $wi {
              let t = $t;
              let cur = (pos.$field - $t).abs();
              axis_shape.push(cur);
              if cur < closest {
                closest = cur;
                surf_index = $i;
              }
            }
          };
        };

        try_snap!(z, 1, 1, -max_r);
        try_snap!(y, 2, 2, -max_r);
        try_snap!(x, 4, 3, -max_r);
        try_snap!(z, 8, 4, max_r);
        try_snap!(y, 16, 5, max_r);
        try_snap!(x, 32, 6, max_r);
        try_snap!(z, 0, 7, -max_r * 3.0);
        try_snap!(y, 0, 8, -max_r * 3.0);
        try_snap!(x, 0, 9, -max_r * 3.0);
        axis_shape.sort_by(|a, b| a.partial_cmp(b).unwrap());

        if axis_shape.len() >= 2 {
          let a0 = axis_shape[0];
          let a1 = axis_shape[1];
          if (a0 - a1).abs() < 0.3 {
            return 0;
          } else {
            let mut save = false;
            if index.count_ones() == 1 && surf_index > 6 {
              if a0 > 2.22 && a1 > 7.2 {
                save = true;
              } else if a0 > 2.0 && a1 > 7.0 {
                return 0;
              }
            }
            if save {
              // save index as is
            } else if a0 < 3.8 {
              index += 256 * surf_index;
            } else {
              return 0;
            }
          }
        }
      }
    } else {
      if dist_to_surface < 3.0 && index.count_ones() == 1 {
        let px = f32::max(pos.y.abs(), pos.z.abs());
        let py = f32::max(pos.x.abs(), pos.z.abs());
        let pz = f32::max(pos.x.abs(), pos.y.abs());
        let p = f32::min(f32::min(px, py), pz);
        if dist_to_surface > 1.2 && p > 4.52 {
          // save index as is
        } else if dist_to_surface > 1.0 && p > 4.3 {
          return 0;
        } else {
          index += 256;
        }
      }
    }

    let mr = 1.0 / f32::min(r, max_r);
    let mut in_sr = |a, b| {
      if a < mr && b < mr && sqr(mr - a) + sqr(mr - b) > sqr(mr) {
        return true;
      }
      false
    };
    let mut in_sr_3 = |a, b, c| {
      if a < mr && b < mr && c < mr && sqr(mr - a) + sqr(mr - b) + sqr(mr - c) > sqr(mr) {
        return true;
      }
      false
    };

    let sf = 1.0 / (2.0 * max_r);
    if axis_shape.len() >= 2 && in_sr(axis_shape[0] * sf, axis_shape[1] * sf) {
      return 0;
    }
    if axis_shape.len() >= 3 && in_sr_3(axis_shape[0] * sf, axis_shape[1] * sf, axis_shape[2] * sf)
    {
      return 0;
    }

    if axis_pos.len() >= 2 && in_sr(axis_pos[0], axis_pos[1]) {
      return 0;
    }
    if axis_neg.len() >= 2 && in_sr(axis_neg[0], axis_neg[1]) {
      return 0;
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 && in_sr(axis_pos[0], axis_neg[0]) {
      return 0;
    }

    return index;
  }
}
