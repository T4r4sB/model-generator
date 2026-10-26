use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct DeltoidCreator {
  axis: Vec<Point>,
  normals: Vec<(Point, Point, Point)>,
  n_dists: RefCell<Vec<(f32, usize)>>,
  sz: f32,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<f32>>,
  axis_neg: RefCell<Vec<f32>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}
impl DeltoidCreator {
  pub fn new() -> Self {
    let (s, c) = (5.0 * PI / 180.0).sin_cos();

    let axis: Vec<_> = [
      Point { x: c, y: s, z: 0.0 },
      Point { x: -s, y: c, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: -c, y: s, z: 0.0 },
      Point { x: -s, y: -c, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let main_cos = 0.0;
    let main_angle = main_cos.acos();

    let ha = (50.0 * PI / 180.0).cos();
    let va = -ha * ha;
    let corner_cos = (-va / (1.0 - va)).sqrt();
    let corner_angle = corner_cos.acos();

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());
    let n_dists = RefCell::new(Vec::new());

    let in_r = 23.0;

    let sz = 28.5;

    let groove = vec![
      (main_angle * 0.5 + 6.0 / (in_r - 4.8)).cos(),
      in_r - 4.6,
      (main_angle * 0.5 + 3.0 / (in_r - 4.8)).cos(),
      in_r - 2.2,
      (corner_angle + 5.0 / in_r).cos(),
      in_r + 0.2,
      (corner_angle + 2.0 / in_r).cos(),
      in_r + 2.6,
      (corner_angle + 4.5 / in_r).cos(),
    ];

    let normals: Vec<_> = [
      Point { x: 1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: -1.0 },
    ]
    .into_iter()
    .map(|n| {
      let n = n.norm();
      let n2 = n.any_perp().norm();
      let n3 = cross(n, n2);
      (n, n2, n3)
    })
    .collect();

    Self { axis, normals, n_dists, sz, groove, axis_pos, axis_neg }
  }

  pub fn faces(&self) -> usize {
    3
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, 6)
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
    /*
    let n = self.axis[current_normal];
    let n1 = n.any_perp();
    let n2 = cross(n, n1);
    let pos = n.scale(29.9) + n1.scale(pos.x) + n2.scale(pos.y);
    (result > 0) as PartIndex*/
    let pos = match current_normal {
      0 => Point { x: 0.0, y: pos.x, z: pos.y },
      1 => Point { x: pos.x, y: pos.y, z: 0.0 },
      _ => Point { x: pos.y, y: 0.0, z: pos.x },
    };
    self.get_part_index_impl(pos, current_normal)
  }

  pub fn get_quality() -> usize {
    200
  }

  pub fn get_size() -> f32 {
    90.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if pos.x.abs() > 44.999 || pos.y.abs() > 44.999 || pos.z.abs() > 44.999 {
      return 0;
    }
    let r = pos.len();

    let sphere_r = self.groove[1] - 6.2;

    //if pos.y < 0.0 { return 0; }

    if r < sphere_r {
      // return 0; // tmp
      if r > sphere_r - 0.3 {
        return 0;
      }
      for &a in &self.axis {
        let s = cross(pos, a).len();
        let d = dot(pos, a);
        if s < 1.2 && d > 0.0 {
          return 0;
        }
      }
      return 63;
    }

    let mut depth_to_face = f32::INFINITY;
    let sticker_gap = 0.5;

    let sz = self.sz;

    let mut axis_pos = self.axis_pos.borrow_mut();
    let axis_pos = axis_pos.deref_mut();
    axis_pos.clear();
    let mut axis_neg = self.axis_neg.borrow_mut();
    let axis_neg = axis_neg.deref_mut();
    axis_neg.clear();
    let mut n_dists = self.n_dists.borrow_mut();
    let n_dists = n_dists.deref_mut();
    n_dists.clear();

    for (i, n) in self.normals.iter().enumerate() {
      n_dists.push((sz - dot(pos, n.0), i));
    }
    n_dists.sort_by(|(a, _), (b, _)| a.partial_cmp(&b).unwrap());

    let orr = 2.0;
    let fd = orr
      - (sqr(f32::max(orr - n_dists[0].0, 0.0))
        + sqr(f32::max(orr - n_dists[1].0, 0.0))
        + sqr(f32::max(orr - n_dists[2].0, 0.0)))
      .sqrt();
    if fd < 0.0 {
      return 0;
    }

    let mut index: PartIndex = 0;
    let shaft_r = f32::min(1.5, self.groove[1] - 0.9 - r);

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);
    let last_groove_r = self.groove[self.groove.len() - 2];
    let last_groove_a = self.groove[self.groove.len() - 1];
    /* if shaft_r > 0.0 {
      shift_out += shaft_r * 0.05;
      shift_in += shaft_r * 0.05;
    }*/

    let extra = sz / 3.0 / last_groove_a / r;

    if r > last_groove_r - 0.2 && extra <= 1.0 {
      shift_out *= extra;
      shift_in *= extra;
    }

    for (i, &a) in self.axis.iter().enumerate() {
      let c = dot(pos, a);

      let check_in = c - shift_in * r;
      if check_in > 0.0 {
        index += (1 << i);
        axis_pos.push(check_in);
      } else {
        let check_out = shift_out * r - c;
        if check_out > 0.0 {
          axis_neg.push(check_out);
        } else {
          return 0;
        }
      }
    }

    if index.count_ones() > 2 && r < self.groove[self.groove.len() - 6] + 0.2 {
      return 0;
    }

    let rr;

    rr = 2.0 * f32::min(1.0, 1.0 * r / (sz - 2.0));

    if current_normal < 0 {
      for p in axis_pos.iter_mut() {
        if *p < sticker_gap {
          return 0;
        }
        *p -= 0.5;
      }
      for n in axis_neg.iter_mut() {
        if *n < sticker_gap {
          return 0;
        }
        *n -= 0.5;
      }
    }

    let rr = if current_normal < 0 { rr - sticker_gap } else { rr };

    let mut in_sr = |a, b| -> f32 {
      if a < 0.0 {
        return 0.0;
      }
      if b < 0.0 {
        return 0.0;
      }
      if a < rr && b < rr {
        return rr - (sqr(rr - a) + sqr(rr - b)).sqrt();
      }
      return f32::min(a, b);
    };

    axis_pos.sort_by(|a, b| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut cd = f32::INFINITY;
    if axis_pos.len() >= 1 {
      cd = f32::min(cd, axis_pos[0]);
    }
    if axis_neg.len() >= 1 {
      cd = f32::min(cd, axis_neg[0]);
    }

    if axis_pos.len() >= 2 {
      cd = f32::min(cd, in_sr(axis_pos[0], axis_pos[1]));
      if cd < 0.0 {
        return 0;
      }
    }
    if axis_neg.len() >= 2 {
      cd = f32::min(cd, in_sr(axis_neg[0], axis_neg[1]));
      if cd < 0.0 {
        return 0;
      }
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
      cd = f32::min(cd, in_sr(axis_pos[0], axis_neg[0]));
      if cd < 0.0 {
        return 0;
      }
    }

    if fd < orr && cd < orr && sqr(orr - fd) + sqr(orr - cd) > sqr(orr) {
      return 0;
    }

    let depth = 2.0;
    let cn = self.normals[n_dists[0].1];
    let d1 = dot(pos, cn.1);
    let d2 = dot(pos, cn.2);
    let d = f32::max(d1.abs(), d2.abs()) - 5.0;
    let fd0 = n_dists[0].0;

    if fd0 < depth || fd0 < depth + 2.0 && d < 0.0 || fd0 + n_dists[1].0 < 6.0 {
      if fd0 + 0.15 > n_dists[1].0 {
        return 0;
      }
      if fd0 < depth - 0.2 || fd0 < depth + 1.5 && d < -0.12 || fd0 + n_dists[1].0 < 5.7 {
        index += (n_dists[0].1 as PartIndex + 1) * 100000;
      } else {
        return 0;
      }
    }

    if index.count_ones() == 1 {
      let hole_r = if r < sphere_r + 2.0 { 1.5 } else { 3.2 };
      for (i, &a) in self.axis.iter().enumerate() {
        let s = cross(pos, a).len();
        let d = dot(pos, a);
        if d > 0.0 && s < hole_r {
          return 0;
        }
      }
    }

    return index;
  }
}
