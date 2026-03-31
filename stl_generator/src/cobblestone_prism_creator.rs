use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

struct Piece {
  bounds_pos: &'static [usize],
  bounds_neg: &'static [usize],
}

#[derive(Copy, Clone)]
pub enum SubGroup {
  Group(usize),
  Piece(usize),
}

struct Group {
  splitter: usize,
  lhs: SubGroup,
  rhs: SubGroup,
}

const BASIC: &[Point] = &[
  Point { x: 0.0, y: -0.70710677, z: 0.70710677 },
  Point { x: -0.70710677, y: 0.70710677, z: 0.0 },
  Point { x: 0.70710677, y: 0.0, z: -0.70710677 },
];

const SPLITTERS: &[Point] = &[
  Point { x: -0.70710677, y: 0.70710677, z: 0.0 },
  Point { x: -0.09731734, y: 0.7865661, z: 0.6097894 },
  Point { x: 0.6430476, y: -0.7269716, z: -0.24083567 },
  Point { x: 0.32029507, y: 0.8064309, z: 0.4970717 },
  Point { x: 0.25623608, y: 0.07945925, z: 0.96334267 },
  Point { x: -0.96334267, y: -0.07945925, z: -0.25623605 },
  Point { x: 0.70710677, y: 0.0, z: -0.70710677 },
  Point { x: -0.25623608, y: -0.9633427, z: -0.07945925 },
  Point { x: 0.9633427, y: 0.25623608, z: 0.07945925 },
  Point { x: 0.07945925, y: 0.96334267, z: 0.25623605 },
  Point { x: -0.07945925, y: -0.25623608, z: -0.96334267 },
  Point { x: -0.6097894, y: -0.7865661, z: 0.09731734 },
  Point { x: 0.0, y: -0.70710677, z: 0.70710677 },
];

const PIECES: &[Piece] = &[
  Piece { bounds_pos: &[], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[0, 1] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[0, 2] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 2] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 1] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[0, 1] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 1] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
];

use SubGroup::*;

const GROUPS: &[Group] = &[
  Group { splitter: 0, lhs: Piece(0), rhs: Piece(1) },
  Group { splitter: 1, lhs: Piece(2), rhs: Piece(3) },
  Group { splitter: 2, lhs: Piece(5), rhs: Piece(6) },
  Group { splitter: 3, lhs: Piece(4), rhs: Group(2) },
  Group { splitter: 4, lhs: Group(3), rhs: Piece(7) },
  Group { splitter: 5, lhs: Group(1), rhs: Group(4) },
  Group { splitter: 6, lhs: Group(0), rhs: Group(5) },
  Group { splitter: 7, lhs: Piece(10), rhs: Piece(11) },
  Group { splitter: 8, lhs: Piece(9), rhs: Group(7) },
  Group { splitter: 9, lhs: Piece(12), rhs: Piece(13) },
  Group { splitter: 10, lhs: Group(8), rhs: Group(9) },
  Group { splitter: 0, lhs: Piece(8), rhs: Group(10) },
  Group { splitter: 0, lhs: Piece(15), rhs: Piece(16) },
  Group { splitter: 11, lhs: Group(12), rhs: Piece(17) },
  Group { splitter: 1, lhs: Piece(14), rhs: Group(13) },
  Group { splitter: 0, lhs: Piece(18), rhs: Piece(19) },
  Group { splitter: 4, lhs: Group(15), rhs: Piece(20) },
  Group { splitter: 5, lhs: Group(14), rhs: Group(16) },
  Group { splitter: 6, lhs: Group(11), rhs: Group(17) },
  Group { splitter: 12, lhs: Group(6), rhs: Group(18) },
];

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct PrismCreator {
  groove: Vec<f32>,
  normals: Vec<Point>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

fn apply_indices<T: Copy>(v: &mut Vec<T>, indices: &[usize]) {
  for (new_index, &old_index) in indices.iter().enumerate() {
    v[new_index] = v[old_index];
  }
  v.truncate(indices.len());
}

impl PrismCreator {
  pub fn new() -> Self {
    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let basic_angle = 2.0 * PI / 2.0;
    let basic_cos = basic_angle.cos();
    let edge = basic_cos / (1.0 - basic_cos);
    let minimal_angle = ((edge * 2.0 + 1.0) / 3.0).sqrt().acos();

    let ca2 = (basic_angle * 2.0 / 3.0).cos();
    let cba = -sqr(ca2) + (1.0 - sqr(ca2)) * edge;
    let maximal_angle = PI - ((edge - cba) / (1.0 - cba)).sqrt().acos();
    let sphere_r = 4.0 / (maximal_angle - minimal_angle);

    /*
        let normals = [
          Point { x: -1.0, y: -1.0, z: -1.0 },
          Point { x: -1.0, y: -1.0, z: 1.0 },
         Point { x: -1.0, y: 1.0, z: -1.0 },
          Point { x: -1.0, y: 1.0, z: 1.0 },
          Point { x: 1.0, y: -1.0, z: -1.0 },
          Point { x: 1.0, y: -1.0, z: 1.0 },
          Point { x: 1.0, y: 1.0, z: -1.0 },
          Point { x: 1.0, y: 1.0, z: 1.0 },
        ]
        .into_iter()
        .map(Point::norm)
        .collect();
    */
    /*
    let normals = [
      Point { x: 0.0, y: -0.52573115, z: -0.85065085 },
      Point { x: 0.0, y: -0.52573115, z: 0.85065085 },
      Point { x: 0.0, y: 0.52573115, z: -0.85065085 },
      Point { x: 0.0, y: 0.52573115, z: 0.85065085 },
      Point { x: -0.85065085, y: 0.0, z: -0.52573115 },
      Point { x: 0.85065085, y: 0.0, z: -0.52573115 },
      Point { x: -0.85065085, y: 0.0, z: 0.52573115 },
      Point { x: 0.85065085, y: 0.0, z: 0.52573115 },
      Point { x: -0.52573115, y: -0.85065085, z: 0.0 },
      Point { x: -0.52573115, y: 0.85065085, z: 0.0 },
      Point { x: 0.52573115, y: -0.85065085, z: 0.0 },
      Point { x: 0.52573115, y: 0.85065085, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();*/

    /*
    let normals = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();*/

    let normals = BASIC.to_vec();

    let groove = vec![
      (maximal_angle - 0.0 / sphere_r).cos(),
      sphere_r + 0.2,
      (maximal_angle - 3.0 / sphere_r).cos(),
      sphere_r + 2.6,
      (maximal_angle + 1.0 / sphere_r).cos(),
    ];

    Self { groove, normals, axis_pos, axis_neg }
  }

  pub fn faces(&self) -> usize {
    self.normals.len()
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
    let n0 = self.normals[current_normal];
    let n1 = n0.any_perp().norm();
    let n2 = cross(n0, n1);

    let last_groove = self.groove[self.groove.len() - 2];
    let sz = last_groove + 2.2;
    let p = n0.scale(sz) + n1.scale(pos.x) + n2.scale(pos.y);
    (self.get_part_index_impl(p, current_normal) > 0) as PartIndex
  }

  pub fn get_quality() -> usize {
    128
  }

  pub fn get_size() -> f32 {
    80.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();

    if pos.y < 0.0 {
      // return 0;
    }

    let sphere_r = self.groove[1] - 2.2;

    if r < sphere_r {
      if r > sphere_r - 0.2 || r < sphere_r - 5.2 {
        return 0;
      }
      for &a in BASIC {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < 1.5 {
          return 0;
        }
      }
      return PIECES.len() as PartIndex;
    }

    let mut out_core = false;
    let last_groove = self.groove[self.groove.len() - 2];
    let sz = last_groove + 2.2;

    // panic!("sphere_r={sphere_r}, sz={sz}");

    let mut n_dists = Vec::new();
    for i in 0..self.normals.len() {
      if i == current_normal {
        continue;
      }
      let d = sz - dot(pos, self.normals[i]) - 0.017 * cross(pos, self.normals[i]).sqr_len();
      if current_normal < self.normals.len() && d < 1.0 {
        return 0;
      }
      if d < 0.0 {
        return 0;
      }
      n_dists.push(d);
    }

    n_dists.push(f32::INFINITY);
    n_dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let out_r = 1.0;
    if sqr(out_r - f32::min(n_dists[0], out_r))
      + sqr(out_r - f32::min(n_dists[1], out_r))
      + sqr(out_r - f32::min(n_dists[2], out_r))
      > sqr(out_r)
    {
      return 0;
    }

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);
    let dr = (self.groove[1] - 0.2) - r;
    if dr > 0.0 {
      shift_out += dr * 0.003;
      shift_in += dr * 0.003;
    }

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();

    axis_pos.clear();
    axis_neg.clear();

    let mut spiral = false;

    let mut match_axis = |a: Point, shift_in: f32, shift_out: f32| {
      let c = dot(pos, a) / r;
      let check_in = c - shift_in;
      if check_in > 0.0 {
        axis_pos.push((check_in, a));
        return 1;
      } else {
        let check_out = shift_out - c;
        if check_out > 0.0 {
          axis_neg.push((check_out, a));
          return -1;
        } else {
          return 0;
        }
      }
    };

    let mut index: PartIndex = 0;
    let mut current_group = GROUPS.len() - 1;
    loop {
      let g = &GROUPS[current_group];
      let s = SPLITTERS[g.splitter];

      let next = match match_axis(s, shift_in, shift_out) {
        -1 => g.lhs,
        1 => g.rhs,
        _ => return 0,
      };
      match next {
        SubGroup::Group(g) => {
          current_group = g;
          continue;
        }
        SubGroup::Piece(p) => {
          apply_indices(&mut axis_pos, PIECES[p].bounds_pos);
          apply_indices(&mut axis_neg, PIECES[p].bounds_neg);
          index = p as PartIndex;
          break;
        }
      }
    }

    let mut thick = false;

    if axis_pos.len() == 1 && r < sz - 3.0 {
      let hole_r = 1.25;
      for (i, &a) in BASIC.iter().enumerate() {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < hole_r {
          return 0;
        }
      }
    }

    if !thick {
      if spiral {
        //  return 0;
      }

      let sr = if r > self.groove[self.groove.len() - 2] + 0.2
        && axis_pos.len() == 3
        && axis_neg.len() == 0
      {
        0.5f32
      } else {
        0.1f32
      };

      let mut in_sr = |a, b, d| {
        let r = sr * d;
        if a < r && b < r && sqr(r - a) + sqr(r - b) > sqr(r) {
          return true;
        }
        false
      };

      if current_normal < self.normals.len() {
        let hole = 0.006;
        for a in axis_pos.iter_mut() {
          if a.0 < hole {
            return 0;
          }
          a.0 -= hole;
        }
        for a in axis_neg.iter_mut() {
          if a.0 < hole {
            return 0;
          }
          a.0 -= hole;
        }
      }

      axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
      axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

      if axis_pos.len() >= 2 {
        let d = dot(axis_pos[0].1, axis_pos[1].1);
        if in_sr(axis_pos[0].0, axis_pos[1].0, 1.0) {
          return 0;
        }
      }
      if axis_neg.len() >= 2 {
        let d = dot(axis_neg[0].1, axis_neg[1].1);
        if in_sr(axis_neg[0].0, axis_neg[1].0, 1.0) {
          return 0;
        }
      }
      if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
        let d = dot(axis_pos[0].1, axis_neg[0].1);
        if d <= -0.7 {
          if axis_pos.len() >= 2 {
            let d = dot(axis_pos[1].1, axis_neg[0].1);
            if d >= 0.06 {
              if in_sr(axis_pos[1].0, axis_neg[0].0, 1.0) {
                return 0;
              }
            }
          }
          if axis_neg.len() >= 2 {
            let d = dot(axis_pos[0].1, axis_neg[1].1);
            if d >= -0.7 {
              if in_sr(axis_pos[0].0, axis_neg[1].0, 1.0) {
                return 0;
              }
            }
          }
        } else {
          if in_sr(axis_pos[0].0, axis_neg[0].0, 1.0) {
            return 0;
          }
        }
      }
    }

    return index;
  }
}
