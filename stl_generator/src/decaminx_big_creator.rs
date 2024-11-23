use crate::common_for_twisty_puzzles::*;
use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

fn sqr(x: f32) -> f32 {
  x * x
}

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct DecaminxCreator {
  axis: Vec<Point>,
  normals: Vec<Point>,
  n_basis: Vec<(Point, Point)>,
  groove: Vec<f32>,
  max_angle: f32,

  axis_pos: RefCell<Vec<NearAxis>>,
  axis_neg: RefCell<Vec<NearAxis>>,
}

impl DecaminxCreator {
  pub fn new() -> Self {
    let e_cos = 2.0f32.sqrt() - 1.0;
    let e_sin = (1.0 - e_cos * e_cos).sqrt();
    let (c8, s8) = (PI * 0.125).sin_cos();

    let u = e_sin * c8;
    let v = e_sin * s8;
    let w = e_cos;

    let axis = vec![
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: u, y: v, z: w },
      Point { x: -v, y: u, z: w },
      Point { x: -u, y: -v, z: w },
      Point { x: v, y: -u, z: w },
      Point { x: u, y: -v, z: -w },
      Point { x: v, y: u, z: -w },
      Point { x: -u, y: v, z: -w },
      Point { x: -v, y: -u, z: -w },
    ];

    let min_angle = ((2.0 * 2.0f32.sqrt() + 1.0) / 7.0).sqrt().acos();
    let max_angle = (-e_cos).acos() * 0.5;
    let r: f32 = 7.0 / (max_angle - min_angle);

    let groove = vec![
      (max_angle - 2.0 / r).cos(),
      r + 0.2,
      (max_angle - 5.0 / r).cos(),
      r + 2.6,
      (max_angle - 2.0 / r).cos(),
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

    Self {
      axis,
      normals,
      n_basis,
      groove,
      max_angle,
      axis_pos: RefCell::new(Vec::new()),
      axis_neg: RefCell::new(Vec::new()),
    }
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    let n = self.normals[current_normal];
    let (n1, n2) = self.n_basis[current_normal];

    let maxd = 35.0;
    let pos = n.scale(maxd / n.sqr_len()) + n1.scale(pos.x) + n2.scale(pos.y);
    let result = self.get_part_index_impl(pos, current_normal);

    (result > 0) as PartIndex
  }

  pub fn faces(&self) -> usize {
    self.axis.len()
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    0.6
  }

  pub fn get_quality() -> usize {
    512
  }

  pub fn get_size() -> f32 {
    90.0
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    1
  }

  pub fn get_name(&self, current_normal: usize) -> Option<String> {
    None
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    let ball_r = self.groove[1] - 10.0;
    if r < ball_r {
      return 0;
      for i in 0..self.axis.len() {
        let a = self.axis[i];
        if dot(pos, self.axis[i]) > 0.0 {
          let dist_to_axle = cross(pos, self.axis[i]).len();
          if dist_to_axle < 1.25 {
            return 0;
          }
        }
      }

      for v in [(7.0, 7.0, 1), (-7.0, -7.0, 2)] {
        if (pos.x - v.0).abs() + (pos.y - v.1).abs() < 2.5 {
          return 2024 + v.2;
        } else if (pos.x - v.0).abs() + (pos.y - v.1).abs() < 2.8 {
          return 0;
        }
      }

      if pos.z.abs() < 0.1 {
        return 0;
      }
      if pos.z > 0.0 {
        return 2023;
      }

      return 2024;
    }

    let maxd = 35.0;

    assert!(maxd > self.groove[3] + 2.4);

    let sticker = current_normal < self.normals.len();
    let center_dist = if sticker { maxd - 1.0 } else { maxd };

    let mut nd = [f32::INFINITY; 10];

    let mut cup = false;
    for (i, &n) in self.normals.iter().enumerate() {
      let d = dot(pos, n);
      if d > center_dist && i != current_normal {
        return 0;
      }
      nd[i] = center_dist - d;
      if d > center_dist - 0.6 {
        cup = true;
      }
    }
    {
      nd.sort_by(|a, b| a.partial_cmp(b).unwrap());
      let nr = 1.0;
      if nd[0] < nr && nd[1] < nr && sqr(nr - nd[0]) + sqr(nr - nd[1]) > sqr(nr) {
        return 0;
      }

      if nd[0] < nr
        && nd[1] < nr
        && nd[2] < nr
        && sqr(nr - nd[0]) + sqr(nr - nd[1]) + sqr(nr - nd[2]) > sqr(nr)
      {
        return 0;
      }
    }

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);

    let out_r = r - (self.groove[3] + 0.2);
    if out_r > 0.0 {
      shift_out -= out_r * 0.002;
      shift_in -= out_r * 0.002;
    }

    let mut index: PartIndex = 0;
    let mut fail = false;
    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    axis_pos.clear();
    axis_neg.clear();

    let mut match_all = || -> Option<()> {
      let mut match_axis =
        |pos: Point, index: &mut PartIndex, bit: usize, axis: Point| -> Option<()> {
          let d = dot(pos, axis) / r;

          let check_in = d - shift_in;
          if check_in > 0.0 {
            *index |= (1 << bit);
            axis_pos.push(NearAxis { dist: check_in, pos: axis });
          } else {
            let check_out = shift_out - d;
            if check_out > 0.0 {
              axis_pos.push(NearAxis { dist: check_out, pos: axis });
            } else {
              return None;
            }
          }

          return Some(());
        };
      for i in 0..self.axis.len() {
        match_axis(pos, &mut index, i, self.axis[i])?;
      }

      Some(())
    };

    if match_all().is_none() {
      return 0;
    }

     if index != 1 && index != 4 {
       return 0;
      }

    if index.count_ones() == 1 {
      let aindex = index.ilog2() as usize;
      let a = self.axis[aindex];
      let pa = a.any_perp();
      let dist_to_axle = cross(pos, a).len();
      if r > maxd - 2.0
        || dist_to_axle < 4.3 && {
          let pr = if dot(pos, pa) > 0.4 { maxd - 4.0 } else { maxd - 5.5 };
          r > pr
        }
      {
        index += 1 << 10;
      } else if r > maxd - 2.4
        || dist_to_axle < 4.4 && {
          let pr = if dot(pos, pa) > -0.4 { maxd - 4.2 } else { maxd - 5.7 };
          r > pr
        }
      {
        return 0;
      }

      if !cup {
        if r > ball_r + 2.0 && dist_to_axle < 3.2 {
          return 0;
        } else if dist_to_axle < 1.5 {
          return 0;
        }
      }
    }

    axis_pos.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap());
    axis_neg.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap());

    let mut in_sr = |a, b, d| {
      let r = 0.096 * d;
      if a < r && b < r {
        return r - (sqr(r - a) + sqr(r - b)).sqrt();
      }
      return f32::INFINITY;
    };

    let mut minimal = axis_pos
      .iter()
      .min_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap())
      .map(|a| a.dist)
      .unwrap_or(f32::INFINITY);
    minimal = f32::min(
      minimal,
      axis_neg
        .iter()
        .min_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap())
        .map(|a| a.dist)
        .unwrap_or(f32::INFINITY),
    );

    let rr = 0.5f32;

    if axis_pos.len() >= 2 {
      let d = dot(axis_pos[0].pos, axis_pos[1].pos);
      minimal = f32::min(minimal, in_sr(axis_pos[0].dist, axis_pos[1].dist, rr));
    }
    if axis_neg.len() >= 2 {
      minimal = f32::min(minimal, in_sr(axis_neg[0].dist, axis_neg[1].dist, rr));
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
      minimal = f32::min(minimal, in_sr(axis_pos[0].dist, axis_neg[0].dist, rr));
    }

    if minimal < 0.0 {
      return 0;
    }
    if current_normal < self.normals.len() {
      if minimal < 0.02 {
        return 0;
      }
    }

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.axis.len())
  }
}
