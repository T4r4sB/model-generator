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
  Point { x: -0.57735026, y: -0.57735026, z: -0.57735026 },
  Point { x: 0.57735026, y: 0.57735026, z: -0.57735026 },
  Point { x: 0.57735026, y: -0.57735026, z: 0.57735026 },
  Point { x: -0.57735026, y: 0.57735026, z: 0.57735026 },
];

/*
const SPLITTERS: &[Point] = &[
  Point { x: 0.57735026, y: 0.57735026, z: -0.57735026 },
  Point { x: 0.05877565, y: 0.3972512, z: -0.91582584 },
  Point { x: -0.39725122, y: -0.058775663, z: -0.91582584 },
  Point { x: 0.9158258, y: -0.058775663, z: 0.3972512 },
  Point { x: 0.9158258, y: 0.3972512, z: -0.058775634 },
  Point { x: -0.57735026, y: 0.57735026, z: 0.57735026 },
  Point { x: -0.9158258, y: 0.058775663, z: 0.3972512 },
  Point { x: 0.09323321, y: -0.93416035, z: 0.34445903 },
  Point { x: 0.47813344, y: 0.63014245, z: 0.6118078 },
  Point { x: 0.39725122, y: 0.058775663, z: -0.91582584 },
  Point { x: -0.9158258, y: -0.3972512, z: -0.058775634 },
  Point { x: 0.3972512, y: 0.9158258, z: -0.058775634 },
  Point { x: -0.040441185, y: -0.6829345, z: 0.7293592 },
  Point { x: 0.86303353, y: -0.09323317, z: -0.49646795 },
  Point { x: -0.058775634, y: 0.9158257, z: 0.39725122 },
  Point { x: 0.57735026, y: -0.57735026, z: 0.57735026 },
  Point { x: -0.9158258, y: -0.058775663, z: -0.3972512 },
  Point { x: 0.3972512, y: -0.9158258, z: 0.058775634 },
  Point { x: -0.058775634, y: -0.9158257, z: -0.39725122 },
  Point { x: -0.9158258, y: 0.3972512, z: 0.058775634 },
  Point { x: 0.058775634, y: -0.9158257, z: 0.39725122 },
  Point { x: -0.3972512, y: -0.9158258, z: -0.058775634 },
  Point { x: -0.05877565, y: -0.3972512, z: -0.91582584 },
  Point { x: -0.57735026, y: -0.57735026, z: -0.57735026 },
];

const PIECES: &[Piece] = &[
  Piece { bounds_pos: &[], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2, 3] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[0, 2] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2, 3] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[0, 1] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[0, 2] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 3] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 2] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
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
  Group { splitter: 3, lhs: Group(2), rhs: Piece(7) },
  Group { splitter: 1, lhs: Piece(4), rhs: Group(3) },
  Group { splitter: 4, lhs: Group(1), rhs: Group(4) },
  Group { splitter: 5, lhs: Group(0), rhs: Group(5) },
  Group { splitter: 6, lhs: Piece(8), rhs: Piece(9) },
  Group { splitter: 7, lhs: Piece(11), rhs: Piece(12) },
  Group { splitter: 8, lhs: Piece(10), rhs: Group(8) },
  Group { splitter: 9, lhs: Group(7), rhs: Group(9) },
  Group { splitter: 10, lhs: Piece(14), rhs: Piece(15) },
  Group { splitter: 11, lhs: Group(11), rhs: Piece(16) },
  Group { splitter: 6, lhs: Piece(13), rhs: Group(12) },
  Group { splitter: 12, lhs: Piece(17), rhs: Piece(18) },
  Group { splitter: 9, lhs: Group(14), rhs: Piece(19) },
  Group { splitter: 13, lhs: Group(13), rhs: Group(15) },
  Group { splitter: 14, lhs: Group(10), rhs: Group(16) },
  Group { splitter: 15, lhs: Group(6), rhs: Group(17) },
  Group { splitter: 16, lhs: Piece(22), rhs: Piece(23) },
  Group { splitter: 17, lhs: Piece(21), rhs: Group(19) },
  Group { splitter: 18, lhs: Piece(24), rhs: Piece(25) },
  Group { splitter: 19, lhs: Group(20), rhs: Group(21) },
  Group { splitter: 0, lhs: Piece(20), rhs: Group(22) },
  Group { splitter: 2, lhs: Piece(27), rhs: Piece(28) },
  Group { splitter: 20, lhs: Piece(26), rhs: Group(24) },
  Group { splitter: 21, lhs: Piece(29), rhs: Piece(30) },
  Group { splitter: 0, lhs: Group(26), rhs: Piece(31) },
  Group { splitter: 1, lhs: Group(25), rhs: Group(27) },
  Group { splitter: 5, lhs: Group(23), rhs: Group(28) },
  Group { splitter: 5, lhs: Piece(33), rhs: Piece(34) },
  Group { splitter: 22, lhs: Group(30), rhs: Piece(35) },
  Group { splitter: 6, lhs: Piece(32), rhs: Group(31) },
  Group { splitter: 0, lhs: Piece(36), rhs: Piece(37) },
  Group { splitter: 10, lhs: Group(33), rhs: Piece(38) },
  Group { splitter: 9, lhs: Group(32), rhs: Group(34) },
  Group { splitter: 15, lhs: Group(29), rhs: Group(35) },
  Group { splitter: 23, lhs: Group(18), rhs: Group(36) },
];
*/

const SPLITTERS: &[Point] = &[
  Point { x: 0.57735026, y: 0.57735026, z: -0.57735026 },
  Point { x: 0.9158258, y: 0.3972512, z: -0.058775634 },
  Point { x: -0.9341603, y: 0.09323326, z: 0.34445897 },
  Point { x: 0.9158258, y: -0.058775663, z: 0.3972512 },
  Point { x: 0.63014245, y: 0.47813347, z: 0.61180794 },
  Point { x: 0.05877565, y: 0.3972512, z: -0.91582584 },
  Point { x: -0.57735026, y: 0.57735026, z: 0.57735026 },
  Point { x: -0.058775634, y: -0.9158257, z: -0.39725122 },
  Point { x: -0.9158258, y: 0.3972512, z: 0.058775634 },
  Point { x: -0.9158258, y: -0.058775663, z: -0.3972512 },
  Point { x: 0.3972512, y: -0.9158258, z: 0.058775634 },
  Point { x: -0.3972512, y: -0.9158258, z: -0.058775634 },
  Point { x: -0.3972512, y: 0.9158258, z: 0.058775634 },
  Point { x: 0.05877565, y: -0.3972512, z: 0.91582584 },
  Point { x: -0.57735026, y: -0.57735026, z: -0.57735026 },
  Point { x: -0.05877565, y: -0.3972512, z: -0.91582584 },
  Point { x: 0.9341603, y: -0.34445897, z: 0.093233235 },
  Point { x: -0.63014245, y: -0.61180794, z: 0.4781335 },
  Point { x: -0.058775634, y: 0.9158257, z: 0.39725122 },
  Point { x: -0.9158258, y: 0.058775663, z: 0.3972512 },
  Point { x: 0.39725122, y: 0.058775663, z: -0.91582584 },
  Point { x: -0.9158258, y: -0.3972512, z: -0.058775634 },
  Point { x: 0.39725122, y: -0.058775663, z: 0.91582584 },
  Point { x: 0.57735026, y: -0.57735026, z: 0.57735026 },
];

const PIECES: &[Piece] = &[
  Piece { bounds_pos: &[], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2, 3] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[0, 2] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 3] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 2] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 1] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2, 3] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[0, 1] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[0, 2] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 3] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[0, 1] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 1] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
];

use SubGroup::*;

const GROUPS: &[Group] = &[
  Group { splitter: 0, lhs: Piece(0), rhs: Piece(1) },
  Group { splitter: 1, lhs: Piece(2), rhs: Piece(3) },
  Group { splitter: 2, lhs: Piece(5), rhs: Piece(6) },
  Group { splitter: 3, lhs: Group(2), rhs: Piece(7) },
  Group { splitter: 4, lhs: Piece(4), rhs: Group(3) },
  Group { splitter: 5, lhs: Group(1), rhs: Group(4) },
  Group { splitter: 6, lhs: Group(0), rhs: Group(5) },
  Group { splitter: 7, lhs: Piece(10), rhs: Piece(11) },
  Group { splitter: 8, lhs: Piece(9), rhs: Group(7) },
  Group { splitter: 9, lhs: Piece(12), rhs: Piece(13) },
  Group { splitter: 10, lhs: Group(8), rhs: Group(9) },
  Group { splitter: 0, lhs: Piece(8), rhs: Group(10) },
  Group { splitter: 11, lhs: Piece(15), rhs: Piece(16) },
  Group { splitter: 0, lhs: Group(12), rhs: Piece(17) },
  Group { splitter: 5, lhs: Piece(14), rhs: Group(13) },
  Group { splitter: 12, lhs: Piece(18), rhs: Piece(19) },
  Group { splitter: 13, lhs: Group(14), rhs: Group(15) },
  Group { splitter: 6, lhs: Group(11), rhs: Group(16) },
  Group { splitter: 14, lhs: Group(6), rhs: Group(17) },
  Group { splitter: 15, lhs: Piece(20), rhs: Piece(21) },
  Group { splitter: 16, lhs: Piece(23), rhs: Piece(24) },
  Group { splitter: 17, lhs: Piece(22), rhs: Group(20) },
  Group { splitter: 18, lhs: Group(19), rhs: Group(21) },
  Group { splitter: 19, lhs: Piece(26), rhs: Piece(27) },
  Group { splitter: 15, lhs: Piece(25), rhs: Group(23) },
  Group { splitter: 18, lhs: Piece(28), rhs: Piece(29) },
  Group { splitter: 14, lhs: Group(25), rhs: Piece(30) },
  Group { splitter: 6, lhs: Group(24), rhs: Group(26) },
  Group { splitter: 20, lhs: Group(27), rhs: Piece(31) },
  Group { splitter: 21, lhs: Group(22), rhs: Group(28) },
  Group { splitter: 14, lhs: Piece(33), rhs: Piece(34) },
  Group { splitter: 22, lhs: Group(30), rhs: Piece(35) },
  Group { splitter: 7, lhs: Piece(32), rhs: Group(31) },
  Group { splitter: 6, lhs: Piece(36), rhs: Piece(37) },
  Group { splitter: 20, lhs: Group(33), rhs: Piece(38) },
  Group { splitter: 18, lhs: Group(32), rhs: Group(34) },
  Group { splitter: 0, lhs: Group(29), rhs: Group(35) },
  Group { splitter: 23, lhs: Group(18), rhs: Group(36) },
];

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct SkewbCreator {
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

impl SkewbCreator {
  pub fn new() -> Self {
    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let basic_angle = 2.0 * PI / 3.0;
    let basic_cos = basic_angle.cos();
    let edge = basic_cos / (1.0 - basic_cos);
    let minimal_angle = ((edge * 2.0 + 1.0) / 3.0).sqrt().acos();

    let ca2 = (basic_angle * 2.0 / 3.0).cos();
    let cba = -sqr(ca2) + (1.0 - sqr(ca2)) * edge;
    let maximal_angle = ((edge - cba) / (1.0 - cba)).sqrt().acos();
    let sphere_r = 5.0 / (maximal_angle - minimal_angle);

    let edge_angle = edge.acos() / 2.0;

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
    let p = n0.scale(sz) + n1.scale(pos.x) + n2.scale(pos.y);
    (self.get_part_index_impl(p, current_normal) > 0) as PartIndex
  }

  pub fn get_quality() -> usize {
    1
  }

  pub fn get_size() -> f32 {
    80.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();

    if pos.y + pos.x < 0.0 {
      //    return 0;
    }

    let sphere_r = self.groove[1] - 2.2;

    if r < sphere_r {
      return 0; // tmp
      if r > sphere_r - 0.2 || r < sphere_r - 5.2 {
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
      let d = sz - dot(pos, self.normals[i]); // - 0.007 * cross(pos, self.normals[i]).sqr_len();
      if current_normal < self.normals.len() && d < 1.0 {
        return 0;
      }
      if d < 0.0 {
        return 0;
      }
      n_dists.push(d);
    }

    if current_normal < self.normals.len() {
      let hole = 1.5;
      for n in n_dists.iter_mut() {
        if *n < hole {
          return 0;
        }
        *n -= hole;
      }
    }

    n_dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let out_r = if current_normal < self.normals.len() { 1.5 } else { 1.0 };
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

    if axis_pos.len() == 1 && current_normal == self.normals.len() && n_dists[0] > 1.5 {
      let hole_r = if r < self.groove[1] { 1.5 } else { 3.2 };
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

      let sr = if axis_pos.len() == 3 && axis_neg.len() == 0 {
        if r > self.groove[self.groove.len() - 2] + 0.2 {
          0.3f32
        } else if r > self.groove[self.groove.len() - 4] - 0.2 {
          0.05f32
        } else {
          0.15f32
        }
      } else {
        0.15f32
      };

      let mut in_sr = |a, b, d| {
        let r = sr * d;
        if a < r && b < r && sqr(r - a) + sqr(r - b) > sqr(r) {
          return true;
        }
        false
      };

      if current_normal < self.normals.len() {
        let hole = 0.02;
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
        /*
        if axis_pos.len() < 3 {
          for a in [n_dists[0], n_dists[1]] {
            let a = a * 0.03;
            if a < hole {
              return 0;
            }
            axis_pos.push((a - hole, Point::ZERO));
          }
        }*/
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
        if d <= -0.6 {
          if axis_pos.len() >= 2 {
            let d = dot(axis_pos[1].1, axis_neg[0].1);
            if d >= -0.6 {
              if in_sr(axis_pos[1].0, axis_neg[0].0, 0.5) {
                return 0;
              }
            }
          }
          if axis_neg.len() >= 2 {
            let d = dot(axis_pos[0].1, axis_neg[1].1);
            if d >= -0.6 {
              if in_sr(axis_pos[0].0, axis_neg[1].0, 0.5) {
                return 0;
              }
            }
          }
        } else {
          if in_sr(axis_pos[0].0, axis_neg[0].0, 0.5) {
            return 0;
          }
        }
      }
    }

    if current_normal == self.normals.len() {
      if axis_pos.len() == 1 {
        index += 1000;
        let a0 = axis_pos[0].1;
        let a1 = Point { x: a0.y, y: -a0.x, z: 0.0 }.norm();
        let a2 = cross(a0, a1);

        let x1 = r - (self.groove[self.groove.len() - 2] - 0.2);
        let x2 = f32::max(dot(pos, a1).abs(), dot(pos, a2).abs()) - 5.2;
        let x3 = 1.5 - n_dists[0];
        let x = f32::min(x1, f32::max(x2, x3));

        if x1 > 0.15 && (x2 > 0.12 || x3 > 0.15) {
          index += 500;
        } else if x1 > 0.0 && (x2 > 0.0 || x3 > 0.0) {
          return 0;
        }
      } else if axis_pos.len() == 2 {
        index += 2000;
      }
    }

    return index;
  }
}
