use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::{FxHashMap, FxHashSet};
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct BraidCreator {
  axis: Vec<Point>,
  add_a: FxHashMap<PartIndex, Vec<(Point, u8)>>,
  sq_parts: FxHashSet<PartIndex>,
  b_parts: FxHashSet<PartIndex>,
  normals: Vec<Point>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
}

fn sqr(x: f32) -> f32 {
  x * x
}

fn reflect(p: Point, a: Point, b: Point) -> Point {
  let c = (a + b).norm();
  c.scale(2.0 * dot(c, p)) - p
}

fn find_ec(a: Point, b: Point, c: Point) -> f32 {
  let d1 = dot(a, b);
  let d2 = dot(a, c);
  let d3 = dot(b, c);
  let s1 = (1.0 - sqr(d1)).sqrt();
  let s2 = (1.0 - sqr(d2)).sqrt();
  let s3 = (1.0 - sqr(d3)).sqrt();

  let a1 = (d1 - d2 * d3) / (s2 * s3);
  let a2 = (d2 - d3 * d1) / (s3 * s1);
  let a3 = (d3 - d1 * d2) / (s1 * s2);

  let an1 = ((a2.acos() + a3.acos() - a1.acos()) * 0.5).cos();
  let w1 = -sqr(an1) + (1.0 - sqr(an1)) * d1;
  ((d1 - w1) / (1.0 - w1)).sqrt()
}

impl BraidCreator {
  pub fn new() -> Self {
    let ca = -0.743;
    let sa = ((1.0 - sqr(ca)) * 0.5).sqrt();

    let corner = Point { x: 1.0, y: 1.0, z: 1.0 }.norm();

    let a0 = Point { x: ca, y: sa, z: sa }.rotate(corner, 0.0);
    let a1 = Point { x: sa, y: ca, z: sa }.rotate(corner, 0.0);
    let a2 = Point { x: sa, y: sa, z: ca }.rotate(corner, 0.0);

    let mut axis = vec![a0, a1, a2];

    let mut add_a = FxHashMap::<PartIndex, Vec<(Point, u8)>>::default();
    let mut add_for = |a: &[PartIndex], p: (Point, u8)| {
      for &a in a {
        add_a.entry(a).or_default().push(p)
      }
    };
    let mut sq_parts = FxHashSet::default();
    let mut add_sq = |a: &[PartIndex]| {
      for &a in a {
        sq_parts.insert(a);
      }
    };
    let mut b_parts = FxHashSet::default();
    let mut add_b = |a: &[PartIndex]| {
      for &a in a {
        b_parts.insert(a);
      }
    };

    let a0r = reflect(a0, a1, a2);
    let a1r = reflect(a1, a2, a0);
    let a2r = reflect(a2, a0, a1);

    let a0p1 = reflect(a2, a2r, a0);
    let a0m1 = reflect(a2r, a2, a0);
    let a2s1 = reflect(a0, a2r, a0p1);
    let a2q1 = reflect(a0, a2, a0m1);
    let a0p2 = reflect(a2r, a0p1, a2s1);
    let a0m2 = reflect(a2, a0m1, a2q1);

    let a1p1 = reflect(a0, a0r, a1);
    let a1m1 = reflect(a0r, a0, a1);
    let a0s1 = reflect(a1, a0r, a1p1);
    let a0q1 = reflect(a1, a0, a1m1);
    let a1p2 = reflect(a0r, a1p1, a0s1);
    let a1m2 = reflect(a0, a1m1, a0q1);

    let a2p1 = reflect(a1, a1r, a2);
    let a2m1 = reflect(a1r, a1, a2);
    let a1s1 = reflect(a2, a1r, a2p1);
    let a1q1 = reflect(a2, a1, a2m1);
    let a2p2 = reflect(a1r, a2p1, a1s1);
    let a2m2 = reflect(a1, a2m1, a1q1);

    add_for(&[1, 5], (a2r, 3));
    add_for(&[9, 13], (a0p1, 4));
    add_for(&[5, 13, 29], (a0m1, 5));
    add_for(&[37, 45], (a2q1, 6));
    add_for(&[25, 29], (a2s1, 6));
    add_for(&[101], (a0m2, 7));
    add_for(&[89], (a0p2, 7));
    add_sq(&[61, 93, 109, 217, 229]);
    add_b(&[13, 25, 37]);

    add_for(&[2, 3], (a0r, 3));
    add_for(&[10, 11], (a1p1, 4));
    add_for(&[3, 11, 27], (a1m1, 5));
    add_for(&[35, 43], (a0q1, 6));
    add_for(&[26, 27], (a0s1, 6));
    add_for(&[99], (a1m2, 7));
    add_for(&[90], (a1p2, 7));
    add_sq(&[59, 91, 107, 218, 227]);
    add_b(&[11, 26, 35]);

    add_for(&[4, 6], (a1r, 3));
    add_for(&[12, 14], (a2p1, 4));
    add_for(&[6, 14, 30], (a2m1, 5));
    add_for(&[38, 46], (a1q1, 6));
    add_for(&[28, 30], (a1s1, 6));
    add_for(&[102], (a2m2, 7));
    add_for(&[92], (a2p2, 7));
    add_sq(&[62, 94, 110, 220, 230]);
    add_b(&[14, 28, 38]);

    let normals = vec![Point::X, Point::Y, Point::Z, -Point::X, -Point::Y, -Point::Z];

    let ce = dot(axis[0], axis[1]);
    let ra = (ce / (1.0 + ce)).acos();
    let d2c = dot(axis[0], corner);

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let square_angle = find_ec(a2, a0p1, a0m1).acos();
    let bt_angle = find_ec(a0, a2, a2r).acos();
    let edge_angle = ce.acos() * 0.5;

    let maximal_angle = d2c.acos();
    let sphere_r = 3.5 / (maximal_angle - square_angle);
    let maximal_angle_2 = dot(a2s1, a0m1).acos() * 0.5;
    let sphere_r_2 = 5.0 / (maximal_angle_2 - square_angle);
    let maximal_angle_3 = dot(a2s1, a0).acos() * 0.5;
    let sphere_r_3 = 5.0 / (maximal_angle_3 - bt_angle) + 4.8;

    println!("sphere_r={sphere_r}, sphere_r_2={sphere_r_2}, sphere_r_3={sphere_r_3}");
    let sphere_r = f32::max(sphere_r, sphere_r_2);
    let sphere_r = f32::max(sphere_r, sphere_r_3);
    let sphere_r = f32::max(sphere_r, 25.2);

    let groove = vec![
      (bt_angle + 3.5 / (sphere_r - 4.8)).cos(),
      sphere_r - 4.6,
      (bt_angle + 1.5 / (sphere_r - 4.8)).cos(),
      // sphere_r - 4.6,
      // (bt_angle + 4.0 / (sphere_r - 7.2)).cos(),
      sphere_r - 2.2,
      (maximal_angle + 0.5 / sphere_r).cos(),
      sphere_r + 0.2,
      (maximal_angle - 2.5 / sphere_r).cos(),
      sphere_r + 2.6,
      (maximal_angle + 0.5 / sphere_r).cos(),
    ];

    Self { axis, add_a, sq_parts, b_parts, normals, groove, axis_pos, axis_neg }
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
    320
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();

    if pos.y < 0.0 {
      // return 0;
    }

    if r > self.groove[self.groove.len() - 6] + 1.0 {
      // return 0;
    }

    let sphere_r = self.groove[1] - 2.2;

    if r < sphere_r {
      // return 0; // tmp
      if r > sphere_r - 0.2 || r < sphere_r - 5.2 {
        return 0;
      }
      for &a in &self.axis {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < 1.25 {
          return 0;
        }
      }
      return 31;
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
      let d = sz - dot(pos, self.normals[i]);
      if current_normal < self.normals.len() && d < 1.0 {
        return 0;
      }
      if d < 0.0 {
        return 0;
      }
      n_dists.push(d);
    }

    n_dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let out_r = 3.0;
    if sqr(out_r - f32::min(n_dists[0], out_r))
      + sqr(out_r - f32::min(n_dists[1], out_r))
      + sqr(out_r - f32::min(n_dists[2], out_r))
      > sqr(out_r)
    {
      return 0;
    }

    let mut index: PartIndex = 0;

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);
    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();

    axis_pos.clear();
    axis_neg.clear();

    let mut spiral = false;

    #[derive(PartialEq, Eq)]
    enum PositionRelAxis {
      Inside,
      Between,
      Outside,
    };

    let mut match_axis =
      |index: &mut PartIndex, a: Point, i: usize, shift_in: f32, shift_out: f32| {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        let check_in = c - shift_in;
        if check_in > 0.0 {
          *index |= (1 << i);
          axis_pos.push((check_in, a));
          return PositionRelAxis::Inside;
        } else {
          let check_out = shift_out - c;
          if check_out > 0.0 {
            axis_neg.push((check_out, a));
            return PositionRelAxis::Outside;
          } else {
            return PositionRelAxis::Between;
          }
        }
      };

    for (i, &a) in self.axis.iter().enumerate() {
      if match_axis(&mut index, a, i, shift_in, shift_out) == PositionRelAxis::Between {
        return 0;
      }
    }

    'geta: while let Some(add_a) = self.add_a.get(&index) {
      for (add_a, add_i) in add_a {
        match match_axis(&mut index, *add_a, *add_i as usize, shift_in, shift_out) {
          PositionRelAxis::Inside => continue 'geta,
          PositionRelAxis::Between => return 0,
          PositionRelAxis::Outside => {}
        }
      }
      break;
    }

    let sq_part = self.sq_parts.contains(&index);
    let b_part = self.b_parts.contains(&index);
    if sq_part {
      axis_neg.clear();
    }

    if index == 0 {
      index = 63
    }

    if index.count_ones() == 1 && r < sz + 333.0 {
      let hole_r = if r > sphere_r + 2.0 { 3.2 } else { 1.5 };
      for (i, &a) in self.axis.iter().enumerate() {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < hole_r {
          return 0;
        }
      }
    }

    let thick = if sq_part {
      r < self.groove[self.groove.len() - 2] + 0.2 && r > self.groove[self.groove.len() - 4] - 0.2
        || r < self.groove[self.groove.len() - 6] + 0.2
    } else if index.count_ones() == 1 {
      false
    } else if b_part {
      r < self.groove[self.groove.len() - 6] - 0.2 && r > self.groove[self.groove.len() - 8] - 0.2
    } else {
      r < self.groove[self.groove.len() - 2] + 0.2 && r > self.groove[self.groove.len() - 4] - 0.2
    };
    let rr: f32 = if thick { 0.03 } else { 0.1 };


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


    let mut in_sr = |a, b, d| {
      let r = rr * d;
      if a < r && b < r && sqr(r - a) + sqr(r - b) > sqr(r) {
        return true;
      }
      false
    };

    if axis_pos.len() >= 2 {

      let dd = if b_part
        && r > self.groove[self.groove.len() - 6] - 0.2
        && r < self.groove[self.groove.len() - 6] + 0.2
      {
        0.0
      } else {
        1.0
      };

      if in_sr(axis_pos[0].0, axis_pos[1].0, dd) {
        return 0;
      }
    }
    if axis_neg.len() >= 2 {
      if in_sr(axis_neg[0].0, axis_neg[1].0, 1.0) {
        return 0;
      }
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
      if in_sr(axis_pos[0].0, axis_neg[0].0, 1.0) {
        return 0;
      }
    }

    return index;
  }
}
