use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;
use std::ops::DerefMut;

pub struct CubeCreator {
  axis: Vec<Point>,
  normals: Vec<Point>,
  n_basis: Vec<(Point, Point)>,

  screw_diam: f32,
  head_diam: f32,
  thread_diam: f32,

  axis_dst: RefCell<Vec<f32>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl CubeCreator {
  pub fn new() -> Self {
    let axis = vec![
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
    ];

    let normals = axis.clone();

    let n_basis = normals
      .iter()
      .map(|&n| {
        let n1 = n.any_perp().norm();
        let n2 = cross(n, n1).norm();
        (n1, n2)
      })
      .collect();

    let cone_angle = 0.883;
    let screw_diam = 3.0;
    let head_diam = 5.6;
    let thread_diam = 2.5;

    let axis_dst = RefCell::new(Vec::new());

    Self { axis, normals, n_basis, screw_diam, head_diam, thread_diam, axis_dst }
  }

  pub fn faces(&self) -> usize {
    self.normals.len()
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.normals.len())
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    let n = self.normals[current_normal];
    let (n1, n2) = self.n_basis[current_normal];
    let pos = if pos.x > 30.0 {
      let pos = crate::points2d::Point { x: pos.x - 45.0, y: pos.y };
      if pos.len() > 10.8 {
        return 0;
      }
      (n.scale(25.1 / n.sqr_len()) + n1.scale(pos.x) + n2.scale(pos.y))
        .norm()
        .scale(25.1)
    } else if pos.x < -30.0 {
      let pos = crate::points2d::Point { x: pos.x + 35.0, y: pos.y };
      let y_size = if current_normal < 3 { 16.0 } else { 4.0 };
      return ((pos.x.abs() - 1.0).abs() < 0.35 && pos.y.abs() < y_size * 0.5) as PartIndex;
    } else {
      n.scale(35.0 / n.sqr_len()) + n1.scale(pos.x) + n2.scale(pos.y)
    };

    let result = self.get_part_index_impl(pos, current_normal);
    (result > 0) as PartIndex
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 64.999 || pos.y.abs() > 64.999 || pos.z.abs() > 64.999 {
      return 0;
    }

    let sticker = current_normal < self.normals.len();

    let mut cup = false;
    for i in 0..self.normals.len() {
      let n = self.normals[i];
      let d = dot(n, pos);
      if d > 24.0 {
        cup = true;
      }
    }

    if !cup {
      for i in 0..self.axis.len() {
        let a = self.axis[i];
        let diam = if r > 16.0 {
          self.head_diam
        } else if r > 10.0 {
          self.screw_diam
        } else {
          self.thread_diam
        };
        if dot(pos, a) > 0.0 && cross(pos, a).sqr_len() < sqr(diam * 0.5) {
          return 0;
        }
      }
    }

    if r < 10.0 {
      if r > 10.0 {
        return 0;
      }
      return 1024;
    }

    let mut dists_in = Vec::new();
    let mut dists_out = Vec::new();
    let mut det_r = 2.5;

    let mut index: PartIndex = 0;
    let mut combine = true;

    let mut match_axis = |i, index: &mut PartIndex| {
      let border_in;
      let border_out;
      if r < 13.6 {
        border_in = 2.5;
        border_out = 2.5;
      } else if r < 14.0 {
        border_in = 2.5;
        border_out = r * 6.0 / 14.0;
        combine = false;
      } else if r < 22.6 {
        border_in = r * 6.0 / 14.0;
        border_out = r * 6.0 / 14.0;
      } else if r < 23.0 {
        border_in = 4.0;
        border_out = r * 6.0 / 14.0;
        combine = false;
      } else if r < 26.0 {
        border_in = 4.0;
        border_out = 4.0;
      } else if r < 27.0 {
        border_in = -4.0;
        border_out = 14.0 * r / 29.0;
        combine = false;
      } else if r < 29.0 {
        border_in = 14.0 * r / 29.0;
        border_out = 14.0 * r / 29.0;
      } else if r < 29.4 {
        border_in = 10.0;
        border_out = 14.0 * r / 29.0;
        combine = false;
      } else {
        border_in = 10.0;
        border_out = 10.0;
      }

      let d = dot(pos, self.axis[i]);

      if d > border_out {
        if d - border_out < det_r {
          dists_out.push(det_r - (d - border_out));
        }
        *index += (1 << i) as u32;
      } else if d < border_in {
        if border_in - d < det_r {
          dists_in.push(det_r - (border_in - d));
        }
      } else {
        return false;
      }

      return true;
    };

    for i in 0..6 {
      if !match_axis(i, &mut index) {
        return 0;
      }
    }

    if r < 10.5 && index.count_ones() == 1 {
      return 0;
    }

    if combine {
      dists_in.append(&mut dists_out);
    }

    if dists_out.len() > dists_in.len() {
      dists_in = dists_out;
    }

    if sticker {
      det_r -= 0.5;
    }

    if dists_in.len() >= 2 {
      if sqr(dists_in[0]) + sqr(dists_in[1]) > sqr(det_r) {
        return 0;
      }
    }

    for &d in &dists_in {
      if d > det_r {
        return 0;
      }
    }

    if r <= 27.0 {
      let m = |x, y| (sqr(x) + sqr(y)).sqrt();
      if r > 25.0
        && !sticker
        && f32::min(m(pos.x, pos.y), f32::min(m(pos.x, pos.z), m(pos.y, pos.z))) < 11.0
      {
        return 0;
      }
    } else if index.count_ones() < 3 {
      index += 128;
    }

    if r > 27.0 {
      let rhomb = |x: f32, y: f32| x.abs() < 10.0 && y.abs() < 10.0 && x.abs() + y.abs() < 13.0;

      if rhomb(pos.x, pos.y) {
        return 0;
      }
      if rhomb(pos.x, pos.z) {
        return 0;
      }
      if rhomb(pos.y, pos.z) {
        return 0;
      }
    }

    for i in 0..self.normals.len() {
      let n = self.normals[i];
      let d = dot(n, pos);
      let center_dist = if !sticker {
        28.499
      } else if i == current_normal {
        29.499
      } else {
        27.999
      };
      if i != current_normal && d > center_dist {
        return 0;
      }
    }

    if r > 14.0 && index.count_ones() == 1 {
      for i in 0..self.axis.len() {
        let a = self.axis[i];
        if dot(pos, a) <= 0.0 {
          continue;
        }
        let n1 = a.any_perp().norm();
        let n2 = cross(a, n1).norm();
        let x = dot(pos, n1).abs();
        let y = dot(pos, n2).abs();
        let m = f32::max(x, y);
        if r > 16.4 && m < 4.0 {
          index += 1 << (i + 8);
        } else if r > 16.0 && m < 4.2 || r > 19.0 && m < 4.7 {
          return 0;
        }
      }
    }

    return index;
  }
}
