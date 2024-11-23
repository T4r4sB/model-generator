use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct U3minxCreator {
  axis: Vec<Point>,
  normals: Vec<Point>,
  n_basis: Vec<(Point, Point)>,
  split_cos: f32,
  split2_cos: f32,
  ball_radius: f32,

  axis_pos: RefCell<Vec<NearAxis>>,
  axis_neg: RefCell<Vec<NearAxis>>,
}

impl U3minxCreator {
  pub fn new() -> Self {
    let min_angle = 0.7619934;
    let max_angle = 1.0945351;
    let ball_radius: f32 = 18.0;

    // Wolfram alpha: x^2+y^2+z^2=3, x+y+z=-2*x*y-z^2, x*y+y*z+x*z=-x^2-2*y*z, x+y+z>0
    let x0 = -0.41417305819099574927768221;
    let y0 = 1.6779818342370664983535074;
    let z0 = 0.11330331786460596603162975;

    let axis: Vec<_> = [
      Point { x: 1.0, y: 1.0, z: 1.0 },
      Point { x: x0, y: y0, z: z0 },
      Point { x: y0, y: z0, z: x0 },
      Point { x: z0, y: x0, z: y0 },
      Point { x: -y0, y: -x0, z: -z0 },
      Point { x: -x0, y: -z0, z: -y0 },
      Point { x: -z0, y: -y0, z: -x0 },
      Point { x: -1.0, y: -1.0, z: -1.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let dmin = dot(axis[1], axis[2]);
    let dmax = dot(axis[1], axis[7]);
    let amin = dmin.acos() * 0.5;
    let amax = dmax.acos() * 0.5;

    let split_angle = amax;
    let split2_angle = amin;

    let ball_radius = 4.0 / (amax - amin);

    let normals = axis.clone();

    let n_basis = normals
      .iter()
      .map(|&n| {
        let n1 = n.any_perp().norm();
        let n2 = cross(n, n1).norm();
        (n1, n2)
      })
      .collect();

    Self {
      axis,
      normals,
      n_basis,
      split_cos: split_angle.cos(),
      split2_cos: split2_angle.cos(),
      ball_radius,
      axis_pos: RefCell::new(Vec::new()),
      axis_neg: RefCell::new(Vec::new()),
    }
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    let n = self.normals[current_normal];
    let (n1, n2) = self.n_basis[current_normal];

    let pos = n.scale(28.6 / n.sqr_len()) + n1.scale(pos.x) + n2.scale(pos.y);
    let result = self.get_part_index_impl(pos, current_normal);

    (result > 0) as PartIndex
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

  pub fn faces(&self) -> usize {
    self.axis.len()
  }

  pub fn get_quality() -> usize {
    128
  }

  pub fn get_size() -> f32 {
    90.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    let depth = self.ball_radius - r;
    if depth < 0.0 {
      return 0;
    }

    if depth > 4.0 {
      return 0;
    }

    let mut index = 0;
    for (i, &a) in self.axis.iter().enumerate() {
      if dot(pos, a) > r * (self.split_cos) {
        index += 1 << i;
      }
    }

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.axis.len())
  }
}
