use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::FxHashMap;
use num::iter;
use num::Float;

use std::cell::RefCell;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum MatchResult {
  Close,
  Hole,
  Far,
}

pub fn reflect(p: Point, a: Point, b: Point) -> Point {
  let c = (a - b).norm();
  p - c.scale(2.0 * dot(p, c))
}

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct BermudaOctahedronCreator {
  axis: Vec<Point>,
  perp: Vec<Vec<Point>>,
  turn_axis: Vec<Point>,
  normals_for_part: FxHashMap<PartIndex, Vec<Point>>,
  extra_cuts_close: FxHashMap<PartIndex, Vec<Point>>,
  extra_cuts_far: FxHashMap<PartIndex, Vec<Point>>,
  n_dists: RefCell<Vec<(f32, usize)>>,
  sz: f32,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
  rcenters: Vec<Point>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}
impl BermudaOctahedronCreator {
  pub fn new() -> Self {
    let axis: Vec<_> = [
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

    let rot_a = PI * 4.0 / 9.0;

    let turn_axis: Vec<_> =
      vec![axis[2], axis[2].rotate(axis[0], rot_a), axis[2].rotate(axis[0], rot_a * 2.0), axis[1]];

    let max_angle = dot(axis[0], axis[3]).acos() * 0.5;
    let edge = dot(axis[0], axis[1]);
    let cr2 = (rot_a * 0.5).cos();
    let view_a = -sqr(cr2) + (1.0 - sqr(cr2)) * edge;
    let min_c = ((edge - view_a) / (1.0 - view_a)).sqrt();

    let fa = find_factors_for_triangle(edge, min_c, min_c);
    let rcenters: Vec<Point> = turn_axis[0..3]
      .iter()
      .map(|&ta| axis[0].scale(fa.0) + ta.scale(fa.1) + cross(axis[0], ta).scale(fa.2))
      .collect();

    let mut extra_cuts_close = FxHashMap::default();
    for ta in [1, 2] {
      extra_cuts_close.insert(
        1 | 1 << (ta + 1),
        vec![
          reflect(turn_axis[ta - 1], axis[0], turn_axis[ta]),
          reflect(turn_axis[ta + 1], axis[0], turn_axis[ta]),
        ],
      );
    }

    let mut extra_cuts_far = FxHashMap::default();

    let tra = PI * 2.0 / 3.0;
    for ta in [0, 1, 2] {
      extra_cuts_far.insert(
        1 | 1 << (ta + 1) | 1 << (ta + 2),
        vec![
          axis[0].rotate(rcenters[ta], tra),
          turn_axis[ta].rotate(rcenters[ta], tra),
          turn_axis[ta + 1].rotate(rcenters[ta], tra),
          axis[0].rotate(rcenters[ta], -tra),
          turn_axis[ta].rotate(rcenters[ta], -tra),
          turn_axis[ta + 1].rotate(rcenters[ta], -tra),
        ],
      );
    }

    let mut normals_for_part = FxHashMap::default();
    normals_for_part.insert(1, vec![axis[0]]);
    normals_for_part.insert(2, vec![axis[1]]);
    let corner_normals =
      vec![axis[0], axis[0].rotate(rcenters[0], tra), axis[0].rotate(rcenters[0], -tra)];
    let side_normals = vec![axis[0], turn_axis[1]];
    normals_for_part.insert(7, corner_normals.clone());
    normals_for_part.insert(5, side_normals.clone());
    normals_for_part.insert(
      13,
      vec![axis[0], axis[0].rotate(rcenters[1], tra), axis[0].rotate(rcenters[1], -tra)],
    );
    normals_for_part.insert(9, corner_normals.iter().map(|n| n.rotate(axis[0], tra)).collect());
    normals_for_part.insert(25, side_normals.iter().map(|n| n.rotate(axis[0], tra)).collect());

    let perp0b = cross(cross(axis[0], axis[2]), axis[0]).norm();
    let mut perp = Vec::new();
    perp.push(vec![
      perp0b,
      perp0b.rotate(axis[0], rot_a),
      perp0b.rotate(axis[0], rot_a * 2.0),
      perp0b.rotate(axis[0], rot_a * 3.0),
    ]);
    let perp1b = cross(cross(axis[1], axis[0]), axis[1]).norm();
    perp.push(vec![perp1b, perp1b.rotate(axis[1], tra), perp1b.rotate(axis[1], -tra)]);

    let min_angle = min_c.acos();
    let edge_angle = edge.acos() * 0.5;

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());
    let n_dists = RefCell::new(Vec::new());

    let r = 5.0 / (max_angle - min_angle);

    let sz = 28.5;

    let groove = vec![
      (edge_angle + 5.0 / (r - 4.8)).cos(),
      r - 4.6,
      (edge_angle + 2.0 / (r - 4.8)).cos(),
      r - 2.2,
      (max_angle - 0.0 / r).cos(),
      r + 0.2,
      (max_angle - 3.0 / r).cos(),
      r + 2.6,
      (max_angle - 0.0 / r).cos(),
    ];

    Self {
      axis,
      perp,
      turn_axis,
      extra_cuts_close,
      extra_cuts_far,
      normals_for_part,
      n_dists,
      sz,
      groove,
      axis_pos,
      axis_neg,
      rcenters,
    }
  }

  pub fn faces(&self) -> usize {
    3
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
    /*
    let n = self.axis[current_normal];
    let n1 = n.any_perp();
    let n2 = cross(n, n1);
    let pos = n.scale(29.9) + n1.scale(pos.x) + n2.scale(pos.y);
    (result > 0) as PartIndex*/
    let sq2 = 0.5.sqrt();
    let pos = match current_normal {
      0 => Point { x: pos.x * sq2, y: pos.x * sq2, z: pos.y },
      1 => Point { x: pos.x * sq2, y: pos.y, z: pos.x * sq2 },
      _ => Point { x: pos.y, y: pos.x * sq2, z: pos.x * sq2 },
    };
    self.get_part_index_impl(pos, current_normal)
  }

  pub fn get_quality() -> usize {
    384
  }

  pub fn get_size() -> f32 {
    90.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if pos.x.abs() > 44.999 || pos.y.abs() > 44.999 || pos.z.abs() > 44.999 {
      return 0;
    }
    let r = pos.len();

    for &rcenter in &self.rcenters {
      if dot(pos, rcenter) > 0.0 && cross(pos, rcenter).len() < 2.0 {
        //return 0;
      }
    }

    let sphere_r = self.groove[1] - 6.2;

    //if pos.y < 0.0 { return 0; }

    if r < sphere_r {
      // return 0; // tmp
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

    let mut axis_pos = self.axis_pos.borrow_mut();
    let axis_pos = axis_pos.deref_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    let axis_neg = axis_neg.deref_mut();
    let mut n_dists = self.n_dists.borrow_mut();
    let n_dists = n_dists.deref_mut();

    let shaft_r = f32::min(1.5, self.groove[1] - 0.9 - r);

    let mut get_index = |pos, r| {
      axis_pos.clear();
      axis_neg.clear();
      let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);
      let mut index: PartIndex = 0;
      let mut match_a = |a: Point| -> MatchResult {
        let c = dot(pos, a);
        let check_in = c - shift_in * r;
        if check_in > 0.0 {
          axis_pos.push((check_in, a));
          return MatchResult::Close;
        } else {
          let check_out = shift_out * r - c;
          if check_out > 0.0 {
            axis_neg.push((check_out, a));
            return MatchResult::Far;
          } else {
            return MatchResult::Hole;
          }
        }
      };

      let mut it = self.axis.iter();
      let mut i = 0;
      while let Some(&a) = it.next() {
        match match_a(a) {
          MatchResult::Close => {
            index += 1 << i;
            if i == 0 {
              it = self.turn_axis.iter();
            }
          }
          MatchResult::Hole => return 0,
          MatchResult::Far => {}
        }
        i += 1;
      }

      if let Some(ec) = self.extra_cuts_close.get(&index) {
        for &ec in ec {
          if match_a(ec) != MatchResult::Far {
            return 0;
          }
        }
      }
      if let Some(ec) = self.extra_cuts_far.get(&index) {
        for &ec in ec {
          if match_a(ec) != MatchResult::Close {
            return 0;
          }
        }
      }
      return index;
    };

    let mut index = get_index(pos, r);
    let normals =
      if let Some(normals) = self.normals_for_part.get(&index) { normals } else { return 0 };

    let sz = self.sz;
    let orr = 2.5;
    let mut get_fd = |sz, orr| {
      n_dists.clear();
      for (i, &n) in normals.iter().enumerate() {
        let dn = sz - dot(pos, n);
        n_dists.push((dn, i));
      }
      n_dists.sort_by(|(a, _), (b, _)| a.partial_cmp(&b).unwrap());

      n_dists.resize(3, (f32::INFINITY, usize::MAX));
      let fd = orr
        - (sqr(f32::max(orr - n_dists[0].0, 0.0))
          + sqr(f32::max(orr - n_dists[1].0, 0.0))
          + sqr(f32::max(orr - n_dists[2].0, 0.0)))
        .sqrt();
      return fd;
    };
    let fd = get_fd(sz + 2.0, orr);
    if fd < 0.0 {
      return 0;
    }

    if index.count_ones() > 2 && r < self.groove[self.groove.len() - 6] + 0.2 {
      return 0;
    }

    let mut thick = false;
    if index.count_ones() == 2 {
      thick = r > self.groove[self.groove.len() - 8] - 0.2
        && r < self.groove[self.groove.len() - 6] + 0.2;
    } else if index.count_ones() == 3 {
      thick = r > self.groove[self.groove.len() - 4] - 0.2
        && r < self.groove[self.groove.len() - 2] + 0.2;
    }
    let rr = if thick { 1.0 } else { 2.0 };

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

    axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    let mut cd = f32::INFINITY;
    if axis_pos.len() >= 1 {
      cd = f32::min(cd, axis_pos[0].0);
    }
    if axis_neg.len() >= 1 {
      cd = f32::min(cd, axis_neg[0].0);
    }

    for a in [&axis_pos, &axis_neg] {
      let mut i1 = a.iter();
      while let Some(ap1) = i1.next() {
        let mut i2 = i1.clone();
        while let Some(ap2) = i2.next() {
          let d = dot(ap1.1, ap2.1);
          if d < 0.8 {
            cd = f32::min(cd, in_sr(ap1.0, ap2.0));
            if cd < 0.0 {
              return 0;
            }
          }
        }
      }
    }

    for ap1 in &*axis_pos {
      for ap2 in &*axis_neg {
        let d = dot(ap1.1, ap2.1);
        if d > 0.0 {
          cd = f32::min(cd, in_sr(ap1.0, ap2.0));
          if cd < 0.0 {
            return 0;
          }
        }
      }
    }

    if fd < orr && cd < orr && sqr(orr - fd) + sqr(orr - cd) > sqr(orr) {
      return 0;
    }

    if index.count_ones() != 1 {
      let orr = 2.0;
      let fd = get_fd(sz, orr);
      if fd < 0.0 || fd < orr && cd < orr && sqr(orr - fd) + sqr(orr - cd) > sqr(orr) {
        let fd = get_fd(sz + 0.3, orr);
        if fd < 0.0 || fd < orr && cd < orr && sqr(orr - fd) + sqr(orr - cd) > sqr(orr) {
          if n_dists[0].0 + 0.12 > n_dists[1].0 {
            return 0;
          }
          if n_dists[0].0 > 1.0 { return 0; }
          index += 100000 * (n_dists[0].1 as PartIndex + 1);
        } else {
          return 0;
        }
      }
    } else {
      let ai = index.ilog2() as usize;
      let d2 = self.perp[ai]
        .iter()
        .map(|&p| dot(pos, p))
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
      let lr = self.groove[self.groove.len() - 2] - 0.2;
      if r > lr || r > sz - 6.0 && d2 < 5.0 {
        if r > lr + 0.2 || r > sz - 5.5 && d2 < 4.88 {
          index += 100000;
        } else {
          return 0;
        }
      }

      if r < sz - 1.0 {
        let hole_r = if r < sphere_r + 2.0 { 1.5 } else { 3.2 };
        for (i, &a) in self.axis.iter().enumerate() {
          let s = cross(pos, a).len();
          let d = dot(pos, a);
          if d > 0.0 && s < hole_r {
            return 0;
          }
        }
      }
    }

    return index;
  }
}
