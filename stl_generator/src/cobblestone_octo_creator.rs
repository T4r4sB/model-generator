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
  Point { x: 0.0, y: 0.0, z: 1.0 },
  Point { x: 0.0, y: 0.0, z: -1.0 },
  Point { x: 0.0, y: 1.0, z: 0.0 },
  Point { x: 0.0, y: -1.0, z: 0.0 },
  Point { x: 1.0, y: 0.0, z: 0.0 },
  Point { x: -1.0, y: 0.0, z: 0.0 },
];

const SPLITTERS: &[Point] = &[
  Point { x: -1.0, y: 0.0, z: 0.0 },
  Point { x: 0.0, y: 0.8660254, z: -0.5 },
  Point { x: 0.0, y: 0.5, z: 0.8660254 },
  Point { x: 0.0, y: 0.8660254, z: 0.5 },
  Point { x: 0.0, y: 0.5, z: -0.8660254 },
  Point { x: 0.0, y: 1.0, z: 0.0 },
  Point { x: 0.5, y: 0.8660254, z: 0.0 },
  Point { x: -0.8660254, y: 0.5, z: 0.0 },
  Point { x: 0.4330127, y: 0.25, z: -0.8660254 },
  Point { x: -0.75, y: -0.43301266, z: -0.5 },
  Point { x: 0.8660254, y: 0.5, z: 0.0 },
  Point { x: -0.5, y: 0.8660254, z: 0.0 },
  Point { x: -0.5, y: -0.8660254, z: 0.0 },
  Point { x: -0.8660254, y: -0.5, z: 0.0 },
  Point { x: 0.0, y: 0.0, z: -1.0 },
  Point { x: 0.5, y: -0.8660254, z: 0.0 },
  Point { x: -0.5, y: 0.0, z: -0.8660254 },
  Point { x: 0.8660254, y: 0.0, z: -0.5 },
  Point { x: -0.8660254, y: 0.0, z: 0.5 },
  Point { x: -0.8660254, y: 0.0, z: -0.5 },
  Point { x: -0.5, y: 0.0, z: 0.8660254 },
  Point { x: 0.0, y: -1.0, z: 0.0 },
  Point { x: 0.0, y: 0.8660254, z: 0.5 },
  Point { x: 0.8660254, y: 0.0, z: 0.5 },
  Point { x: 0.5, y: 0.0, z: -0.8660254 },
  Point { x: 0.0, y: -0.8660254, z: -0.5 },
  Point { x: 0.0, y: -0.5, z: 0.8660254 },
  Point { x: 0.0, y: 0.5, z: -0.8660254 },
  Point { x: 0.8660254, y: 0.25, z: 0.4330127 },
  Point { x: 0.5, y: -0.43301266, z: -0.75 },
  Point { x: 0.0, y: 0.5, z: 0.8660254 },
  Point { x: 0.0, y: -0.5, z: -0.8660254 },
  Point { x: 0.0, y: 0.8660254, z: -0.5 },
  Point { x: 1.0, y: 0.0, z: 0.0 },
  Point { x: 0.8660254, y: -0.5, z: 0.0 },
  Point { x: 0.25, y: -0.4330127, z: 0.8660254 },
  Point { x: -0.4330127, y: 0.75, z: 0.5 },
  Point { x: 0.5, y: 0.8660254, z: 0.0 },
  Point { x: 0.5, y: -0.8660254, z: 0.0 },
  Point { x: -0.5, y: 0.8660254, z: 0.0 },
  Point { x: 0.8660254, y: 0.5, z: 0.0 },
  Point { x: -0.8660254, y: -0.5, z: 0.0 },
  Point { x: -0.8660254, y: 0.5, z: 0.0 },
  Point { x: -0.5, y: -0.8660254, z: 0.0 },
  Point { x: 0.0, y: 0.0, z: 1.0 },
];

const PIECES: &[Piece] = &[
  Piece { bounds_pos: &[], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 2, 3, 4] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 3, 4] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[4, 5] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 5] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[3, 4] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[1, 2, 3, 4, 5] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[1, 4] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[5, 6] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[4, 6] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[3, 5] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 4] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[3, 4] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2, 3] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[3, 4] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 4] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 3] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[3, 4] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 2] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2, 3, 4] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[0, 1] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[3, 4] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 4] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 3] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[1, 3] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 3] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2, 3, 4] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 3] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[4, 5] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 5] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[3, 4] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 2] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 3] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 2] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
];

use SubGroup::*;

const GROUPS: &[Group] = &[
  Group { splitter: 0, lhs: Piece(0), rhs: Piece(1) },
  Group { splitter: 1, lhs: Piece(4), rhs: Piece(5) },
  Group { splitter: 2, lhs: Piece(3), rhs: Group(1) },
  Group { splitter: 3, lhs: Piece(6), rhs: Piece(7) },
  Group { splitter: 4, lhs: Group(2), rhs: Group(3) },
  Group { splitter: 0, lhs: Piece(2), rhs: Group(4) },
  Group { splitter: 5, lhs: Group(0), rhs: Group(5) },
  Group { splitter: 6, lhs: Piece(8), rhs: Piece(9) },
  Group { splitter: 7, lhs: Piece(11), rhs: Piece(12) },
  Group { splitter: 6, lhs: Piece(10), rhs: Group(8) },
  Group { splitter: 8, lhs: Piece(13), rhs: Piece(14) },
  Group { splitter: 9, lhs: Group(9), rhs: Group(10) },
  Group { splitter: 10, lhs: Group(11), rhs: Piece(15) },
  Group { splitter: 11, lhs: Group(7), rhs: Group(12) },
  Group { splitter: 12, lhs: Piece(17), rhs: Piece(18) },
  Group { splitter: 7, lhs: Piece(16), rhs: Group(14) },
  Group { splitter: 11, lhs: Group(15), rhs: Piece(19) },
  Group { splitter: 13, lhs: Group(13), rhs: Group(16) },
  Group { splitter: 14, lhs: Group(6), rhs: Group(17) },
  Group { splitter: 15, lhs: Piece(22), rhs: Piece(23) },
  Group { splitter: 13, lhs: Piece(21), rhs: Group(19) },
  Group { splitter: 16, lhs: Piece(24), rhs: Piece(25) },
  Group { splitter: 17, lhs: Group(20), rhs: Group(21) },
  Group { splitter: 14, lhs: Piece(20), rhs: Group(22) },
  Group { splitter: 14, lhs: Piece(27), rhs: Piece(28) },
  Group { splitter: 18, lhs: Group(24), rhs: Piece(29) },
  Group { splitter: 16, lhs: Piece(26), rhs: Group(25) },
  Group { splitter: 19, lhs: Piece(30), rhs: Piece(31) },
  Group { splitter: 20, lhs: Group(26), rhs: Group(27) },
  Group { splitter: 0, lhs: Group(23), rhs: Group(28) },
  Group { splitter: 21, lhs: Group(18), rhs: Group(29) },
  Group { splitter: 22, lhs: Piece(32), rhs: Piece(33) },
  Group { splitter: 23, lhs: Piece(35), rhs: Piece(36) },
  Group { splitter: 24, lhs: Piece(34), rhs: Group(32) },
  Group { splitter: 25, lhs: Piece(37), rhs: Piece(38) },
  Group { splitter: 26, lhs: Group(33), rhs: Group(34) },
  Group { splitter: 21, lhs: Group(31), rhs: Group(35) },
  Group { splitter: 27, lhs: Piece(40), rhs: Piece(41) },
  Group { splitter: 25, lhs: Piece(39), rhs: Group(37) },
  Group { splitter: 21, lhs: Group(38), rhs: Piece(42) },
  Group { splitter: 14, lhs: Group(36), rhs: Group(39) },
  Group { splitter: 28, lhs: Piece(44), rhs: Piece(45) },
  Group { splitter: 29, lhs: Piece(43), rhs: Group(41) },
  Group { splitter: 30, lhs: Piece(46), rhs: Piece(47) },
  Group { splitter: 27, lhs: Group(43), rhs: Piece(48) },
  Group { splitter: 22, lhs: Group(42), rhs: Group(44) },
  Group { splitter: 31, lhs: Group(45), rhs: Piece(49) },
  Group { splitter: 32, lhs: Group(40), rhs: Group(46) },
  Group { splitter: 33, lhs: Group(30), rhs: Group(47) },
  Group { splitter: 5, lhs: Piece(50), rhs: Piece(51) },
  Group { splitter: 33, lhs: Piece(53), rhs: Piece(54) },
  Group { splitter: 21, lhs: Piece(52), rhs: Group(50) },
  Group { splitter: 34, lhs: Group(49), rhs: Group(51) },
  Group { splitter: 35, lhs: Piece(56), rhs: Piece(57) },
  Group { splitter: 36, lhs: Piece(55), rhs: Group(53) },
  Group { splitter: 37, lhs: Piece(58), rhs: Piece(59) },
  Group { splitter: 34, lhs: Group(54), rhs: Group(55) },
  Group { splitter: 38, lhs: Group(56), rhs: Piece(60) },
  Group { splitter: 39, lhs: Group(57), rhs: Piece(61) },
  Group { splitter: 40, lhs: Group(52), rhs: Group(58) },
  Group { splitter: 39, lhs: Piece(63), rhs: Piece(64) },
  Group { splitter: 41, lhs: Piece(62), rhs: Group(60) },
  Group { splitter: 37, lhs: Piece(65), rhs: Piece(66) },
  Group { splitter: 0, lhs: Group(62), rhs: Piece(67) },
  Group { splitter: 5, lhs: Group(61), rhs: Group(63) },
  Group { splitter: 42, lhs: Group(59), rhs: Group(64) },
  Group { splitter: 21, lhs: Piece(69), rhs: Piece(70) },
  Group { splitter: 0, lhs: Piece(68), rhs: Group(66) },
  Group { splitter: 41, lhs: Piece(71), rhs: Piece(72) },
  Group { splitter: 38, lhs: Group(67), rhs: Group(68) },
  Group { splitter: 42, lhs: Group(69), rhs: Piece(73) },
  Group { splitter: 34, lhs: Group(70), rhs: Piece(74) },
  Group { splitter: 43, lhs: Group(65), rhs: Group(71) },
  Group { splitter: 44, lhs: Group(48), rhs: Group(72) },
];

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct OctoCreator {
  groove: Vec<f32>,
  normals: Vec<Point>,
  axis_pos: RefCell<Vec<f32>>,
  axis_neg: RefCell<Vec<f32>>,
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

impl OctoCreator {
  pub fn new() -> Self {
    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let basic_angle = 2.0 * PI / 4.0;
    let basic_cos = basic_angle.cos();
    let edge = basic_cos / (1.0 - basic_cos);
    let minimal_angle = ((edge * 2.0 + 1.0) / 3.0).sqrt().acos();

    let ca2 = (basic_angle * 2.0 / 3.0).cos();
    let cba = -sqr(ca2) + (1.0 - sqr(ca2)) * edge;
    let maximal_angle = ((edge - cba) / (1.0 - cba)).sqrt().acos();
    let sphere_r = 4.0 / (maximal_angle - minimal_angle);

    let edge_angle = edge.acos() / 2.0;

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

    let groove = vec![
      (edge_angle + 4.5 / (sphere_r - 4.8)).cos(),
      sphere_r - 4.6,
      (edge_angle + 1.5 / (sphere_r - 4.8)).cos(),
      sphere_r - 2.2,
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
    let p = n0.scale(sz) - n1.scale(pos.x) - n2.scale(pos.y);
    (self.get_part_index_impl(p, current_normal) > 0) as PartIndex
  }

  pub fn get_quality() -> usize {
    384
  }

  pub fn get_size() -> f32 {
    120.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();

    if pos.y < 0.0 {
      //return 0;
    }

    let sphere_r = self.groove[1] - 2.2;

    if r < sphere_r {
      if r > sphere_r - 0.2 {
        return 0;
      }
      for &a in BASIC {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < 1.25 {
          return 0;
        }
      }
      return 10000;
    }

    let mut out_core = false;
    let last_groove = self.groove[self.groove.len() - 2];
    let sz = last_groove + 2.2;

    let mut n_dists = Vec::new();
    for i in 0..self.normals.len() {
      if i == current_normal {
        continue;
      }
      let d = sz - dot(pos, self.normals[i]);
      if d < 0.0 {
        return 0;
      }
      n_dists.push((d, i));
    }

    n_dists.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    let out_r = 2.0;
    if sqr(out_r - f32::min(n_dists[0].0, out_r))
      + sqr(out_r - f32::min(n_dists[1].0, out_r))
      + sqr(out_r - f32::min(n_dists[2].0, out_r))
      > sqr(out_r)
    {
      return 0;
    }

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();

    axis_pos.clear();
    axis_neg.clear();

    let mut spiral = false;

    let mut axis = None;

    let mut match_axis = |a: Point, shift_in: f32, shift_out: f32| {
      let c = dot(pos, a) / r;
      let check_in = c - shift_in;
      if check_in > 0.0 {
        axis = Some(a);
        axis_pos.push(check_in);
        return 1;
      } else {
        let check_out = shift_out - c;
        if check_out > 0.0 {
          axis_neg.push(check_out);
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

    if axis_pos.len() == 3 && r < self.groove[3] + 0.2 {
      return 0;
    }

    if current_normal == self.normals.len() && axis_pos.len() == 1 && n_dists[0].0 > 1.5 {
      let hole_r = if r < self.groove[1] { 1.5 } else { 3.2 };
      for (i, &a) in BASIC.iter().enumerate() {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < hole_r {
          return 0;
        }
      }
    }

    if spiral {
      //  return 0;
    }

    let sr = if axis_pos.len() == 3 && axis_neg.len() == 0 {
      if r > self.groove[self.groove.len() - 2] + 0.2 {
        0.22f32
      } else if r > self.groove[self.groove.len() - 4] - 0.2 {
        0.02f32
      } else {
        0.05f32
      }
    } else {
      0.05f32
    };

    if current_normal < self.normals.len() {
      let hole = 0.006;
      for a in axis_pos.iter_mut() {
        if *a < hole {
          return 0;
        }
        *a -= hole;
      }
      for a in axis_neg.iter_mut() {
        if *a < hole {
          return 0;
        }
        *a -= hole;
      }
      if axis_pos.len() < 3 {
        for a in [n_dists[0].0, n_dists[1].0] {
          let a = a * 0.05;
          if a < hole {
            return 0;
          }
          axis_pos.push(a - hole);
        }
      }
    }

    axis_pos.sort_by(|a, b| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|a, b| a.partial_cmp(b).unwrap());

    fn r_dist(a: f32, b: f32, r: f32) -> f32 {
      if a < r && b < r {
        r - (sqr(r - a) + sqr(r - b)).sqrt()
      } else {
        f32::min(a, b)
      }
    }

    let mut d = f32::INFINITY;

    if axis_pos.len() >= 2 {
      d = f32::min(d, r_dist(axis_pos[0], axis_pos[1], sr));
    }
    if axis_neg.len() >= 2 {
      d = f32::min(d, r_dist(axis_neg[0], axis_neg[1], sr));
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
      d = f32::min(d, r_dist(axis_pos[0], axis_neg[0], sr));
    }
    if d < 0.0 {
      return 0;
    }
    if r_dist(d * 20.0, n_dists[0].0, 2.0) < 0.0 {
      return 0;
    }

    if current_normal == self.normals.len() && axis_pos.len() == 1 {
      index += 1000;
      let a0 = axis.unwrap();
      let a1 = a0.any_perp();
      let a2 = cross(a0, a1);

      let x1 = r - (self.groove[self.groove.len() - 2] - 0.2);
      let m = f32::max(dot(pos, a1).abs(), dot(pos, a2).abs());
      let x2 = (m - 6.2).abs() - 1.0;
      let x3 = 1.5 - n_dists[0].0;
      let x = f32::min(x1, f32::max(x2, x3));

      if x1 > 0.16 && (x2 > 0.14 || x3 > 0.16 || x1 > 6.0) {
        index += 500;
      } else if x1 > 0.0 && (x2 > 0.0 || x3 > -1.0) {
        return 0;
      } else if x1 > 5.0 {
        return 0;
      }
    } else if axis_pos.len() == 2 {
      index += 2000;
    }

    if axis_pos.len() > 1 || index / 500 % 2 == 1 {
      let w = 0.12;
      let h = 2.0 * w / 3.0.sqrt();
      if (n_dists[0].0 - n_dists[1].0).abs() < h {
        return 0;
      }
      index += 10000 * (n_dists[0].1 as PartIndex + 1);
    }

    return index;
  }
}
