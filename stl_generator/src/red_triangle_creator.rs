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

pub struct RedTriangleCreator {
  axis: Vec<Point>,
  axis1: Vec<Point>,
  axis2: Vec<Point>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<f32>>,
  axis_neg: RefCell<Vec<f32>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}
impl RedTriangleCreator {
  pub fn new() -> Self {
    let axis: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 1.0, y: 1.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let main_cos = 0.0;
    let main_angle = main_cos.acos();

    let corner_cos = ((main_cos * 2.0 + 1.0) / 3.0).sqrt();
    let corner_angle = corner_cos.acos();

    let inner_min_angle = PI / 4.0;

    let outer_max_angle = 0.0.acos();
    let outer_min_angle = ((5.0 - 8.0.sqrt()) / 17.0).sqrt().acos();
    let out_r = 7.0 / (outer_max_angle - outer_min_angle);

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let in_r = 16.0;

    let mid_r = 6.8;

    let groove = vec![
      (inner_min_angle + 2.0 / (out_r - 7.2)).cos(),
      out_r - 7.0,
      (inner_min_angle + 5.0 / (out_r - 7.2)).cos(),
      out_r - 4.6,
      (inner_min_angle + 2.0 / (out_r - 7.2)).cos(),
      out_r - 2.2,
      (outer_min_angle + 2.0 / out_r).cos(),
      out_r + 0.2,
      (outer_min_angle + 5.0 / out_r).cos(),
      out_r + 2.6,
      (outer_min_angle + 2.0 / out_r).cos(),
    ];

    let mut axis1 = Vec::new();
    let mut axis2 = Vec::new();
    for &a in &axis {
      let a1 = a.any_perp();
      let a2 = cross(a, a1);
      axis1.push(a1);
      axis2.push(a2);
    }

    Self { axis, axis1, axis2, groove, axis_pos, axis_neg }
  }

  pub fn faces(&self) -> usize {
    self.axis.len() + 2
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
    if current_normal < self.axis.len() {
      let n = self.axis[current_normal];
      let n1 = n.any_perp();
      let n2 = cross(n, n1);
      let sz = self.groove[self.groove.len() - 2] + 2.2;
      let pos = n.scale(sz - 0.5) + n1.scale(pos.x) + n2.scale(pos.y);
      let result = self.get_part_index_impl(pos, current_normal);
      (result > 0) as PartIndex
    } else if current_normal == self.axis.len() {
      let pos = Point { x: pos.x, y: pos.y, z: 0.0 };
      self.get_part_index_impl(pos, current_normal)
    } else  {
      let pos = Point { x: pos.x, y: 0.0, z: pos.y };
      self.get_part_index_impl(pos, current_normal)
    }
  }

  pub fn get_quality() -> usize {
    384
  }

  pub fn get_size() -> f32 {
    120.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if current_normal == self.faces() {
      return 0;
    }

    if pos.x.abs() > 59.999 || pos.y.abs() > 59.999 || pos.z.abs() > 59.999 {
      return 0;
    }
    let r = pos.len();

    let sphere_r = 8.0;
    let sz = self.groove[self.groove.len() - 2] + 2.2;

    if pos.z < 0.0 {
      // return 0;
    }

    if r < sphere_r {
      //return 0; // tmp
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

    // if r > self.groove[self.groove.len() - 2] + 0.4 { return 0; }

    let mut nd: Vec<_> = self.axis.iter().enumerate().map(|(i, &n)| sz - dot(n, pos)).collect();
    for &d in &nd {
      if d < 0.0 {
        return 0;
      }
    }

    let orr;
    let orr234;

    if current_normal < 6 {
      orr = 4.0;
      orr234 = 13.0;
    } else {
      orr = 2.0;
      orr234 = 10.0;
    }

    let h23 = orr - (sqr(f32::max(orr - nd[2], 0.0)) + sqr(f32::max(orr - nd[3], 0.0))).sqrt();
    let min23 = f32::min(nd[2], nd[3]);
    let h234 =
      orr234 - (sqr(f32::max(orr234 - min23, 0.0)) + sqr(f32::max(orr234 - nd[4], 0.0))).sqrt();
    let min01 = f32::min(nd[0], nd[1]);
    let h = f32::min(h23, h234);
    let mut fd = orr - (sqr(f32::max(orr - min01, 0.0)) + sqr(f32::max(orr - h, 0.0))).sqrt();
    if fd < 0.0 {
      return 0;
    }

    let mut index: PartIndex = 0;
    let shaft_r = f32::min(1.5, self.groove[1] - 0.9 - r);

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);
    let last_groove_r = self.groove[self.groove.len() - 2];
    /* if shaft_r > 0.0 {
      shift_out += shaft_r * 0.05;
      shift_in += shaft_r * 0.05;
    }*/

    let extra = r / (last_groove_r + 1.0);

    if extra >= 1.0 {
      shift_out /= extra;
      shift_in /= extra;
    }

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    axis_pos.clear();
    axis_neg.clear();

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

    let screw_border = self.groove[self.groove.len() - 6];
    if index.count_ones() == 1 {
      if r > screw_border + 0.2 {
        index += 128;
      } else if r > screw_border - 0.2 {
        return 0;
      }
    }

    let rr = f32::min(1.0, 1.0 * r / sz);

    let mut in_sr = |a, b, r| -> f32 {
      let rr = r * rr;
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
      cd = f32::min(cd, in_sr(axis_pos[0], axis_pos[1], orr));
      if cd < 0.0 {
        return 0;
      }
    }
    if axis_neg.len() >= 2 {
      cd = f32::min(cd, in_sr(axis_neg[0], axis_neg[1], orr));
      if cd < 0.0 {
        return 0;
      }
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
      cd = f32::min(cd, in_sr(axis_pos[0], axis_neg[0], orr));
      if cd < 0.0 {
        return 0;
      }
    }

    if fd < orr && cd < orr && sqr(orr - fd) + sqr(orr - cd) > sqr(orr) {
      return 0;
    }

    if r < screw_border + 1.0 {
      let hole_r = if r < sphere_r + 2.0 { 1.5 } else { 3.2 };
      for &a in &self.axis {
        let s = cross(pos, a).len();
        let d = dot(pos, a);
        if d > 0.0 && s < hole_r {
          return 0;
        }
      }
    }

    index
  }
}
