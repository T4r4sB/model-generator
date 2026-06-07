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

pub struct Tower4Creator {
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
impl Tower4Creator {
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

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let in_r = 16.0;

    let mid_r = 5.0;

    let groove = vec![
      (main_angle * 0.5 + 4.0 / (in_r - 4.8)).cos(),
      in_r - 4.6,
      (main_angle * 0.5 + 1.5 / (in_r - 4.8)).cos(),
      in_r - 2.2,
      (corner_angle + 5.0 / in_r).cos(),
      in_r + 0.2,
      (corner_angle + 2.0 / in_r).cos(),
      in_r + 2.6,
      mid_r / (in_r + 2.6),
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
    128
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

    if r < sphere_r {
      //return 0; //tmp

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

    let mr = self.mid_r;

    let (n0, n1);
    let p = Point { x: (sqr(pos.x + mr) + sqr(pos.y + mr)).sqrt(), y: 0.0, z: pos.z };

    let top_h = 29.0 * 0.8;

    if p.z > mr + 5.0 {
      n0 = top_h - (p.x * 0.6 + p.z * 0.8);
      n1 = top_h - (-p.x * 0.6 + p.z * 0.8);
    } else if p.z > mr {
      n0 = top_h - (p.x * 0.6 + p.z * 0.8);
      n1 = p.z - mr;
    } else if p.z > -mr {
      n0 = 27.0 - p.x;
      n1 = 35.0 + p.z;
    } else {
      n0 = 35.0 + p.z;
      n1 = 27.0 - p.x;
    }
    let rr = 2.0;
    if n0 < 0.0 || n1 < 0.0 || n0 < rr && n1 < rr && sqr(rr - n0) + sqr(rr - n1) > sqr(rr) {
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

    let extra = r - (last_groove_r - 0.0);

    if extra >= 0.0 {
      shift_out = f32::max(shift_out - extra * 0.03, mr / r);
      shift_in = f32::max(shift_in - extra * 0.03, mr / r);
    }

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    axis_pos.clear();
    axis_neg.clear();

    for (i, &a) in self.axis.iter().enumerate().take(4) {
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


    if index == 0 {
      index = 256;
    }

    if index.count_ones() == 1 && n0 > 1.0 {
      if depth_to_face > 0.5 {
        let hole_r = if r < sphere_r + 2.0 { 1.5 } else { 3.2 };
        for (i, &a) in self.axis.iter().enumerate() {
          let s = cross(pos, a).len();
          let d = dot(pos, a);
          if d > 0.0 && s < hole_r {
            return 0;
          }
        }
      }
    } else if r < self.groove[1] - 2.2 {
      return 0;
    }

    let rr;

    if index.count_ones() == 3 && r > last_groove_r + 0.2 {
      rr = 3.0;
    } else {
      rr = f32::clamp((r - 17.0) / (20.0 - 17.0), 0.0, 1.0) * 1.0 + 2.0;
    }

    if current_normal < 6 {
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

    let rr = if current_normal < 6 { rr - sticker_gap } else { rr };

    let mut in_sr = |a, b| {
      if a < rr && b < rr && sqr(rr - a) + sqr(rr - b) > sqr(rr) {
        return true;
      }
      false
    };

    axis_pos.sort_by(|a, b| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if axis_pos.len() >= 2 && in_sr(axis_pos[0], axis_pos[1]) {
      return 0;
    }
    if axis_neg.len() >= 2 && in_sr(axis_neg[0], axis_neg[1]) {
      return 0;
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 && in_sr(axis_pos[0], axis_neg[0]) {
      return 0;
    }

    if index.count_ones() == 1 {
      let mut min_c = f32::INFINITY;
      let w;
      let ray;
      let clutches: &[_];
      
      if index == 1 {
        w = 1.0;
        ray = Point { x: 0.0, y: 0.0, z: -1.0 }.norm();
        clutches = &[
          Point { x: 18.0, y: -1.0, z: 0.0 },
          Point { x: -1.0, y: 18.0, z: 0.0 },
          Point { x: 4.0, y: 4.0, z: 0.0 },
        ];
      } else if index == 2 {
        w = 1.0;
        ray = Point { x: 1.0, y: 1.0, z: 1.0 }.norm();
        clutches = &[
          Point { x: 21.0, y: 0.0, z: 8.5 },
          Point { x: 0.0, y: 21.0, z: 8.5 },
          Point { x: 0.0, y: 0.0, z: 12.0 },
        ];
      } else if index == 4 {
        w = 2.0;
        ray = Point { x: 1.0, y: -1.0, z: 0.0 }.norm();
        clutches = &[
          Point { x: 6.0, y: 0.0, z: 0.0 },
          Point { x: 0.0, y: -19.0, z: 0.0 },
        ];
      } else if index == 8 {
        w = 2.0;
        ray = Point { x: -1.0, y: 1.0, z: 0.0 }.norm();
        clutches = &[
          Point { x: 0.0, y: 6.0, z: 0.0 },
          Point { x: -19.0, y: 0.0, z: 0.0 },
        ];
      } else if index == 256 {
        w = 2.0;
        ray = Point { x: 1.0, y: 1.0, z: 0.0 }.norm();
        clutches = &[Point { x: 19.0, y: 0.0, z: 0.0 }, Point { x: 0.0, y: 19.0, z: 0.0 }];
      } else {
        unreachable!();
      }

      for &v in clutches {
        min_c = f32::min(min_c, cross((pos - v), ray).len());
      }

      if n0 < w || min_c < 1.88 && n0 < w + 2.0 {
        index += 1000;
      } else if n0 < w + 0.2 || min_c < 2.0 && n0 < w + 2.2 {
        return 0;
      }
    }

    return index;
  }
}
