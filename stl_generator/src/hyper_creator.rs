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

pub struct HyperCreator {
  axis: Vec<Point>,
  axis1: Vec<Point>,
  axis2: Vec<Point>,
  mid_r: f32,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<f32>>,
  axis_neg: RefCell<Vec<f32>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}
impl HyperCreator {
  pub fn new() -> Self {
    let axis: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let main_cos = 0.0;
    let main_angle = main_cos.acos();

    let corner_cos = ((main_cos * 2.0 + 1.0) / 3.0).sqrt();
    let corner_angle = corner_cos.acos();

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let in_r = 12.0;

    let mid_r = 9.5;

    let groove = vec![
      (main_angle * 0.5 + 3.0 / in_r).cos(),
      in_r + 0.2,
      (main_angle * 0.5 + 0.5 / in_r).cos(),
      // in_r + 2.6,
      // mid_r / (in_r + 2.4),
    ];

    let mut axis1 = Vec::new();
    let mut axis2 = Vec::new();
    for &a in &axis {
      let a1 = a.any_perp();
      let a2 = cross(a, a1);
      axis1.push(a1);
      axis2.push(a2);
    }

    Self { axis, axis1, axis2, mid_r, groove, axis_pos, axis_neg }
  }

  pub fn faces(&self) -> usize {
    0
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
    let n = self.axis[current_normal];
    let n1 = n.any_perp();
    let n2 = cross(n, n1);
    let pos = n.scale(29.9) + n1.scale(pos.x) + n2.scale(pos.y);
    let result = self.get_part_index_impl(pos, current_normal);
    (result > 0) as PartIndex
  }

  pub fn get_quality() -> usize {
    240
  }

  pub fn get_size() -> f32 {
    90.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if pos.x.abs() > 44.999 || pos.y.abs() > 44.999 || pos.z.abs() > 44.999 {
      return 0;
    }
    let r = pos.len();

    let sphere_r = self.groove[1] - 2.2;

    if pos.y < 0.0 {
      return 0;
    }

    let cyl = (sqr(pos.x) + sqr(pos.y)).sqrt();

    if r < sphere_r {
      if cyl < 4.0 {
        if cyl > 3.7 {
          return 0;
        }

        if pos.z > -4.0 {
          if cyl < 1.5 || pos.z > -2.0 && cyl < 2.8 {
            return 0;
          }
          return 2;
        } else if pos.z < -5.0 {
          if cyl < 1.2 {
            return 0;
          }
          return 1;
        }

        return 0;
      } else {
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
    }

    let mr = self.mid_r;

    let nx = f32::min(mr * 3.0 - pos.x, mr * 3.0 + pos.x);
    let ny = f32::min(mr * 3.0 - pos.y, mr * 3.0 + pos.y);
    let nz = f32::min(mr * 3.0 - pos.z, mr * 3.0 + pos.z);

    let orr = 2.0;
    let fd = orr
      - (sqr(f32::max(orr - nx, 0.0))
        + sqr(f32::max(orr - ny, 0.0))
        + sqr(f32::max(orr - nz, 0.0)))
      .sqrt();
    if fd < 0.0 {
      return 0;
    }

    let mut index: PartIndex = 0;
    let shaft_r = f32::min(1.5, self.groove[1] - 0.9 - r);

    let (mut shift_out_c, mut shift_in_c, inter) = get_groove(r, &self.groove, 0.03);
    let last_groove_r = self.groove[self.groove.len() - 2];

    shift_in_c -= 1.0 / r;
    shift_out_c -= 1.0 / r;

    let shift_in = f32::min(shift_in_c, mr / r);
    let shift_out = f32::min(shift_out_c, mr / r);

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

    if index.count_ones() > 2 {
      return 0;
    }

    let bound = self.groove[self.groove.len() - 2] + 3.0;
    if (index & 3) != 0 && r > bound {
      return 0;
    }

    let rr;

    rr = 2.0 * f32::min(1.0, 1.0 * r / (mr * 3.0 - 2.0));

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

    let hx = f32::min(pos.x.abs(), (pos.x.abs() - mr * 2.0).abs());
    let hy = f32::min(pos.y.abs(), (pos.y.abs() - mr * 2.0).abs());
    let hz = pos.z.abs();
    let hxy = f32::max(hx, hy);
    let hxz = f32::max(hx, hz);
    let hyz = f32::max(hy, hz);

    let px = if index & 0x0C != 0 { (mr - pos.x).rem_euclid(mr * 2.0) } else { mr * 3.0 - pos.x };
    let mx = if index & 0x0C != 0 { (mr + pos.x).rem_euclid(mr * 2.0) } else { mr * 3.0 + pos.x };
    let py = if index & 0x30 != 0 { (mr - pos.y).rem_euclid(mr * 2.0) } else { mr * 3.0 - pos.y };
    let my = if index & 0x30 != 0 { (mr + pos.y).rem_euclid(mr * 2.0) } else { mr * 3.0 + pos.y };
    let pz = if index & 0x3C != 0 { mr - pos.z } else { bound - r };
    let mz = if index & 0x3C != 0 { mr + pos.z } else { f32::INFINITY };

    let mut ns =
      [(0, px, hyz), (1, mx, hyz), (2, py, hxz), (3, my, hxz), (4, pz, hxy), (5, mz, hxy)];
    ns.sort_by(|(_, a, _), (_, b, _)| a.partial_cmp(b).unwrap());

    let mut bb = f32::INFINITY;
    if index.count_ones() == 2 {
      if index & 3 != 0 {
        bb = 0.0;
      } else {
        bb = 0.0;
        for (i, &a) in self.axis.iter().enumerate() {
          if (1 << i) & index != 0 {
            let c = dot(pos, a);
            let check_in = c - shift_in_c * r;
            bb = f32::max(bb, -check_in);
          }
        }
        bb = f32::min(bb, r - bound);
        bb = f32::max(bb, pos.x.abs() + 4.5 - mr * 2.0);
        bb = f32::max(bb, pos.y.abs() + 4.5 - mr * 2.0);
        bb = f32::max(bb, pos.z.abs() + 4.0 - mr);
      }
    }

    if bb > 0.0 && (ns[0].1 < 2.0 || (ns[0].1 < 4.0 && ns[0].2 < 4.0)) {
      if bb > 0.2 && (ns[0].1 < 1.8 || (ns[0].1 < 3.5 && ns[0].2 < 3.88)) {
        if ns[0].1 > ns[1].1 - 0.14 {
          return 0;
        }
        index += (ns[0].0 + 1) * 1000;
      } else {
        return 0;
      }
    }

    if ns[0].1 > 1.0 {
      let hole_r = if r < sphere_r + 2.0 { 1.5 } else { 2.8 };
      for (i, &a) in self.axis.iter().enumerate().skip(1) {
        let s = cross(pos, a).len();
        let d = dot(pos, a);
        let hole_r = if i == 1 { 2.8 } else { hole_r };
        if d > 0.0 && s < hole_r {
          return 0;
        }
      }
    }

    return index;
  }
}
