use crate::common_for_twisty_puzzles::*;
use crate::model::*;
use crate::points3d::*;
use crate::solid::*;
use fxhash::*;
use num::Float;

use std::cell::RefCell;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct GallaTripCreator {
  axis: Vec<Basis>,
  long_edges: Vec<(PartIndex, usize)>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, usize)>>,
  axis_neg: RefCell<Vec<(f32, usize)>>,
  k: f32,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}
impl GallaTripCreator {
  pub fn new() -> Self {
    let t = (PI / 8.0).tan().recip();

    let mut axis: Vec<_> = [
      Point { x: 1.0, y: t, z: 1.0 },
      Point { x: t, y: 1.0, z: -1.0 },
      Point { x: t, y: -1.0, z: 1.0 },
      Point { x: 1.0, y: -t, z: -1.0 },
      Point { x: -1.0, y: -t, z: 1.0 },
      Point { x: -t, y: -1.0, z: -1.0 },
      Point { x: -t, y: 1.0, z: 1.0 },
      Point { x: -1.0, y: t, z: -1.0 },
    ]
    .into_iter()
    .map(|p| {
      p.norm()
       // .rotate(Point { x: 0.0, y: 0.0, z: 1.0 }, PI / 8.0)
      //  .rotate(Point { x: 1.0, y: 0.0, z: 0.0 }, PI / 4.0)
    })
    .collect();

    let mut long_edges = Vec::<(PartIndex, usize)>::default();

    for i in 0..8 {
      for j in i + 1..8 {
        if dot(axis[i], axis[j]) < -0.5 {
          if j - i == 4 {
            long_edges.push((1 << i | 1 << j, 0));
            continue;
          }
          long_edges.push((1 << i | 1 << j, axis.len()));
          axis.push(find_square(axis[i], axis[j]));
          axis.push(find_square(axis[j], axis[i]));
        }
      }
    }

    let corner_cos_1 = half_cos(dot(axis[0], axis[1]));
    let corner_angle_1 = corner_cos_1.acos();
    let corner_cos_2 = half_cos(dot(axis[0], axis[3]));
    let corner_angle_2 = corner_cos_2.acos();
    let corner_cos_3 = ((dot(axis[0], axis[4]) + 1.0) / 2.0).sqrt();
    let corner_angle_3 = corner_cos_3.acos();

    let split_cos_out = 0.5;
    let axis_pos = RefCell::new(
      (0..axis.len())
        .into_iter()
        .map(|i| (0.0, i))
        .collect::<Vec<_>>(),
    );
    let axis_neg = axis_pos.clone();

    let groove = vec![
      (corner_angle_1 + 2.0 / 19.2).cos(),
      19.4,
      (corner_angle_1 + 5.0 / 19.2).cos(),
      21.8,
      (corner_angle_1 + 2.0 / 19.2).cos(),
      24.2,
      (corner_angle_2 - 2.0 / 24.0).cos(),
      26.6,
      (corner_angle_2 - 5.0 / 24.0).cos(),
      29.0,
      (corner_angle_3 + 4.0 / 31.2).cos(),
      31.4,
      (corner_angle_3 + 1.0 / 31.2).cos(),
      33.6,
      (corner_angle_3 + 7.0 / 31.2).cos(),
    ];

    fn a_to_triple(a: Point) -> (Point, Point, Point) {
      let a2 = a.any_perp().norm();
      let a3 = cross(a, a2);
      (a, a2, a3)
    }

    let axis = axis.into_iter().map(to_basis).collect();

    Self { axis, long_edges, groove, axis_pos, axis_neg, k: 0.006 }
  }

  pub fn faces(&self) -> usize {
    1
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, usize::MAX)
  }

  pub fn get_height(current_normal: usize) -> f32 {
    0.6
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    let n = &self.axis[current_normal];

    fn sinc(x: f32) -> f32 {
      if x == 0.0 {
        1.0
      } else {
        x.sin() / x
      }
    }

    fn versinc(x: f32) -> f32 {
      if x == 0.0 {
        0.0
      } else {
        (1.0 - x.cos()) / x
      }
    }

    let k = self.k;
    let k2 = k * 2.0;
    let last_groove = self.groove[self.groove.len() - 2];
    let max = last_groove + 2.2;

    let r = pos.len();
    let a = r * k2;

    let pos = pos.scale(sinc(a));
    let pos = n.0.scale(max - r * versinc(a)) + n.1.scale(pos.x) + n.2.scale(pos.y);

    let r = pos.len();
    let d = dot(pos, n.0);

    let control_c = n.0.scale(max - k2.recip());
    let delta = d + (max * max + r * r) * k - (2.0 * d * k + 1.0) * max;

    let result = self.get_part_index_impl(pos, current_normal);

    (result > 0) as PartIndex
  }

  pub fn get_quality() -> usize {
    256
  }

  pub fn get_size() -> f32 {
    150.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 49.999 || pos.y.abs() > 49.999 || pos.z.abs() > 49.999 {
      return 0;
    }

    if r < 12.0 {
      return 0;
      for i in 0..8 {
        let a = self.axis[i].0;
        if dot(pos, a) < 0.0 {
          continue;
        }
        let s = cross(pos, a).len();
        if s < 1.25 {
          return 0;
        }
      }

      return 1024;
    }

    let last_groove = self.groove[self.groove.len() - 2];

    let mut out_core = false;
    for i in 0..8 {
      if i == current_normal {
        out_core = true;
        continue;
      }
      let d = dot(pos, self.axis[i].0);
      let k = self.k;
      let max = last_groove + 2.2;
      if d + (max * max + r * r) * k > (2.0 * d * k + 1.0) * max {
        return 0;
      }
      let max = last_groove + 1.2;
      if d + (max * max + r * r) * k > (2.0 * d * k + 1.0) * max {
        out_core = true;
      }
    }

    let mut index: PartIndex = 0;
    let groove_r = if current_normal < self.axis.len() {
      last_groove + 2.0
    } else {
      r
    };
    let (mut shift_out, mut shift_in, inter) = get_groove(groove_r, &self.groove, 0.03);

    let hole_r = if r < 13.5 { 1.5 } else { 3.2 };

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    for (i, a) in axis_pos.iter_mut().enumerate() {
      *a = (f32::INFINITY, i);
    }
    for (i, a) in axis_neg.iter_mut().enumerate() {
      *a = (f32::INFINITY, i);
    }

    let mut spiral = false;
    let mut match_axis = |ai: usize, i: usize, index: &mut PartIndex| {
      let a = &self.axis[ai];
      let c = dot(pos, a.0) / r;
      let check_in = c - shift_in;
      if check_in > 0.0 {
        if !out_core && in_spiral(pos, a.0, a.1, a.2, check_in, 0.01, 0.2) {
          spiral = true;
        }
        *index += (1 << i);
        axis_pos[ai].0 = check_in;
        true
      } else {
        let check_out = shift_out - c;
        if check_out > 0.0 {
          if !out_core && in_spiral(pos, a.0, a.1, -a.2, check_out, 0.01, 0.2) {
            spiral = true;
          }
          axis_neg[ai].0 = check_out;
          true
        } else {
          false
        }
      }
    };

    for i in 0..8 {
      if !match_axis(i, i, &mut index) {
        return 0;
      }
    }

    let mut long_edge_axis = usize::MAX;
    let mut long_edge_mask = PartIndex::MAX;
    for &(l, ei) in &self.long_edges {
      if index & l == l {
        long_edge_mask = l;
        long_edge_axis = ei;
        if ei >= 8 {
          if !match_axis(long_edge_axis, 8, &mut index) {
            return 0;
          }
          if !match_axis(long_edge_axis + 1, 9, &mut index) {
            return 0;
          }
        }
      }
    }

    let in_thick_layer =
      self.groove.len() > 13 && r > self.groove[11] - 0.2 && r < self.groove[13] + 0.2;

    if (long_edge_axis >= self.axis.len() || !in_thick_layer) && spiral {
      return 0;
    }

    if index.count_ones() == 1 {
      for i in 0..8 {
        let a = self.axis[i].0;
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < hole_r {
          return 0;
        }
      }
    }

    if index.count_ones() > 2 && r < self.groove[5] + 0.2 {
      return 0;
    }

    if current_normal < self.axis.len() {
      let mut m = f32::INFINITY;
      for i in 0..axis_pos.len() {
        axis_pos[i].0 -= 0.02;
        axis_neg[i].0 -= 0.02;
        if axis_pos[i].0 < 0.0 || axis_neg[i].0 < 0.0 {
          return 0;
        }
        m = f32::min(m, axis_pos[i].0);
        m = f32::min(m, axis_neg[i].0);
      }

      for i in 0..8 {
        if i == current_normal {
          continue;
        }
        let d = dot(pos, self.axis[i].0);
        let dn = cross(self.axis[i].0, self.axis[current_normal].0).len();
        if d < 0.0 {
          continue;
        }
        let k = self.k;
        let max = last_groove + 2.2;
        let delta = (2.0 * d * k + 1.0) * max - (d + (max * max + r * r) * k);
        let delta = delta * 0.025 / dn;
        if delta < 0.02 {
          return 0;
        }
        m = f32::min(m, delta);
      }

      if m > 0.05 {
        return 0;
      }
    }

    let check_square = 1 << 8 | 1 << 9;
    if index & check_square == check_square || index == 0b1010101 || index == 0x10101010 {
      return index;
    }

    if long_edge_axis < self.axis.len() {
      let mut long_edge_mask = long_edge_mask;
      if long_edge_axis < 8 {
        long_edge_mask |=
          ((long_edge_mask & 0b00111111) << 2) | ((long_edge_mask & 0b11000000) >> 6);
      } else {
        long_edge_mask |= 1 << long_edge_axis;
        long_edge_mask |= 1 << (long_edge_axis + 1);
      }

      for i in 0..axis_pos.len() {
        if (1 << i) & long_edge_mask == 0 {
          axis_pos[i].0 = f32::INFINITY;
          axis_neg[i].0 = f32::INFINITY;
        }
      }
    }

    /*
    if r < 19.85 && index.count_ones() == 1 {
      let a = index.ilog2() as usize;
      for i in 0..5 {
        let j = (i + a + 2) % 8;
          axis_pos[i] = f32::INFINITY;
          axis_neg[i] = f32::INFINITY;

      }
    };*/

    axis_pos.sort_by(|a, b| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut in_sr = |a: (f32, usize), b: (f32, usize)| {
      let sr = if in_thick_layer && long_edge_axis < self.axis.len() {
        0.03
      } else if r > self.groove[9] + 0.2 && (1 << a.1 | 1 << b.1) & long_edge_mask == long_edge_mask
      {
        if r > self.groove[11] - 0.2 {
          0.46
        } else {
          0.06
        }
      } else {
        0.06
      };
      if a.0 < sr && b.0 < sr && sqr(sr - a.0) + sqr(sr - b.0) > sqr(sr) {
        return true;
      }
      false
    };

    if in_sr(axis_pos[0], axis_pos[1]) {
      return 0;
    }
    if in_sr(axis_neg[0], axis_neg[1]) {
      return 0;
    }
    if !inter && in_sr(axis_pos[0], axis_neg[0]) {
      return 0;
    }

    if index == 0 {
      index = if dot(pos, self.axis[0].0 + self.axis[2].0 + self.axis[4].0 + self.axis[6].0) > 0.0 {
        34
      } else {
        68
      };
    }

    return index;
  }
}
