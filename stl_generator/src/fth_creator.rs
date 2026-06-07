use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::FxHashSet;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct HouseCreator {
  axis: Vec<Point>,
  closed_axes: FxHashSet<PartIndex>,
  normals: Vec<Point>,
  axis1: Vec<Point>,
  axis2: Vec<Point>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
  sz: f32,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}
impl HouseCreator {
  pub fn new() -> Self {
    let axis: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 1.0, y: 1.0, z: 0.0 },
      Point { x: 1.0, y: -1.0, z: 0.0 },
      Point { x: -1.0, y: 1.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let closed_axes: FxHashSet<_> = [1 << 2, 1 << 3, 1 << 5, 1 << 6].into_iter().collect();

    let normals: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 1.0, y: 1.0, z: 0.0 },
      Point { x: 1.0, y: -1.0, z: 0.0 },
      Point { x: -1.0, y: 1.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let main_cos = 0.0;
    let main_angle = main_cos.acos();

    let corner_cos = ((main_cos * 2.0 + 1.0) / 3.0).sqrt();
    let corner_angle = corner_cos.acos();

    let lowest_angle = PI / 8.0;
    let inner_min_angle = PI / 4.0;
    let outer_min_angle = (1.0 / 3.0).sqrt().acos();

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let in_r = 16.0;
    let mid_r = 6.8;
    let sz = 28.5;

    let outer_max_angle = PI * 3.0 / 8.0;

    let groove = vec![
      (lowest_angle - 0.02).cos(),
      sz - 11.5,
      (inner_min_angle - 4.0 / (sz - 11.7)).cos(),
      sz - 9.5,
      (inner_min_angle + 5.0 / (sz - 7.7)).cos(),
      sz - 7.5,
      (inner_min_angle + 2.0 / (sz - 7.7)).cos(),
      sz - 5.5,
      (outer_max_angle).cos(),
      sz - 3.5,
      (outer_max_angle - 3.0 / (sz - 3.7)).cos(),
      sz - 1.5,
      (outer_max_angle - 0.95 / (sz - 3.7)).cos(),
    ];

    let mut axis1 = Vec::new();
    let mut axis2 = Vec::new();
    for &a in &axis {
      let a1 = a.any_perp();
      let a2 = cross(a, a1);
      axis1.push(a1);
      axis2.push(a2);
    }

    Self { axis, axis1, axis2, sz, closed_axes, normals, groove, axis_pos, axis_neg }
  }

  pub fn faces(&self) -> usize {
    self.normals.len() + 1
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.normals.len() + 1)
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
    if current_normal < self.normals.len() {
      let n = self.normals[current_normal];
      let n1 = n.any_perp();
      let n2 = cross(n, n1);
      let sz = self.sz;
      let pos = n.scale(sz - 0.5) + n1.scale(pos.x) + n2.scale(pos.y);
      let result = self.get_part_index_impl(pos, current_normal);
      (result > 0) as PartIndex
    } else {
      let pos = Point { x: pos.x, y: pos.y, z: 0.0 };
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
    if pos.x.abs() > 59.999 || pos.y.abs() > 59.999 || pos.z.abs() > 59.999 {
      return 0;
    }
    let r = pos.len();


    // let sphere_r = 8.0; // for springs
    let sphere_r = 14.0; // no springs
    let sz = self.sz;

    if pos.z < 0.0 {
      //return 0;
    }

    if r < sphere_r {
      if r > sphere_r - 0.3 {
        return 0;
      }
      for &a in &self.axis {
        let s = cross(pos, a).len();
        let d = dot(pos, a);
        if s < 1.45 && d > 0.0 { // for m3
       // if s < 1.2 && d > 0.0 { // for m2.5
          return 0;
        }
      }
      return 63;
    }

    // if r > self.groove[self.groove.len() - 2] + 0.4 { return 0; }

    let mut nd: Vec<_> = self.normals.iter().enumerate().map(|(i, &n)| sz - dot(n, pos)).collect();
    for &d in &nd {
      if d < 0.0 {
        return 0;
      }
    }
    nd.sort_by(|a, b| a.partial_cmp(&b).unwrap());

    let orr = if current_normal < self.normals.len() { 4.0 } else { 2.0 };
    let mut fd = orr
      - (sqr(f32::max(orr - nd[0], 0.0))
        + sqr(f32::max(orr - nd[1], 0.0))
        + sqr(f32::max(orr - nd[2], 0.0)))
      .sqrt();
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

    let extra = r / (last_groove_r + 0.2);

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
        axis_pos.push((check_in, a));
      } else {
        let check_out = shift_out * r - c;
        if check_out > 0.0 {
          axis_neg.push((check_out, a));
        } else {
          return 0;
        }
      }
    }

    /*
    let screw_border = self.groove[self.groove.len() - 6];
    if index.count_ones() == 1 {
      if r > screw_border + 0.2 {
        index += 128;
      } else if r > screw_border - 0.2 {
        return 0;
      }
    }
    */

    if index.count_ones() == 1
      && self.closed_axes.contains(&index)
      && r > self.groove[self.groove.len() - 6] - 0.2
    {
      if r > self.groove[self.groove.len() - 4] + 0.2 {
        index += 2000;
      } else {
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

    for (an, pn) in axis_neg.iter_mut() {
      for (_, pp) in axis_pos.iter() {
        if dot(*pn, *pp) < -0.5 {
          *an = f32::INFINITY;
        }
      }
    }

    axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    let mut cd = f32::INFINITY;
    if axis_pos.len() >= 1 {
      cd = f32::min(cd, axis_pos[0].0);
    }
    if axis_neg.len() >= 1 {
      cd = f32::min(cd, axis_neg[0].0);
    }

    if axis_pos.len() >= 2 {
      cd = f32::min(cd, in_sr(axis_pos[0].0, axis_pos[1].0, orr));
      if cd < 0.0 {
        return 0;
      }
    }
    if axis_neg.len() >= 2 {
      cd = f32::min(cd, in_sr(axis_neg[0].0, axis_neg[1].0, orr));
      if cd < 0.0 {
        return 0;
      }
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
      let d = dot(axis_pos[0].1, axis_neg[0].1);
      if r > self.groove[1] - 0.2 {
        cd = f32::min(cd, in_sr(axis_pos[0].0, axis_neg[0].0, orr));
        if cd < 0.0 {
          return 0;
        }
      }
    }

    if fd < orr && cd < orr && sqr(orr - fd) + sqr(orr - cd) > sqr(orr) {
      return 0;
    }

    if index.count_ones() == 1 {
      if nd[0] < 3.0 || nd[0] < 5.0 && nd[1] > 21.0 {
        if nd[0] < 2.7 || nd[0] < 4.7 && nd[1] > 21.12 {
          index += 1000;
        } else {
          return 0;
        }
      }

      if r < sz - 1.0 {
      //  let hole_r = if r < sphere_r + 2.0 { 1.5 } else { 2.8 }; // for m2.5
        let hole_r = if r < sphere_r + 2.0 { 1.75 } else { 3.2 }; // for m3
        for &a in &self.axis {
          let s = cross(pos, a).len();
          let d = dot(pos, a);
          if d > 0.0 && s < hole_r {
            return 0;
          }
        }
      }
    }

    index
  }
}
