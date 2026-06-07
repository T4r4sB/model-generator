use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::FxHashMap;
use num::complex::ComplexFloat;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

type SideMap = FxHashMap<PartIndex, Vec<(Point, f32)>>;

pub struct BeginnerJumblerCreator {
  axis: Vec<Point>,
  axis1: Vec<Point>,
  axis2: Vec<Point>,
  normals: Vec<Point>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
  axis_seq: Vec<(usize, f32)>,
  sz: Vec<f32>,
  side_maps: Vec<SideMap>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl BeginnerJumblerCreator {
  pub fn new() -> Self {
    fn from_polar_coords(longitude: f32, lattitude: f32) -> Point {
      let (s1, c1) = longitude.sin_cos();
      let (s2, c2) = lattitude.sin_cos();
      Point { x: c2 * c1, y: c2 * s1, z: s2 }
    }

    fn make_ghost(p: Point) -> Point {
      p.rotate(Point { x: 1.0, y: 0.0, z: 0.0 }.norm(), PI)
    }

    let x = 0.15;
    let a0 = 0.0 * PI / 180.0;
    let a = 2.0 * PI / 3.0;

    let axis: Vec<_> =
      [from_polar_coords(a0, x), from_polar_coords(a0 + a, x), from_polar_coords(a0 + a + a, x)]
        .map(make_ghost)
        .into_iter()
        .collect();

    let edge = dot(axis[0], axis[1]);
    let rot_angle = (edge / (1.0 + edge)).acos();

    let axis_seq = vec![(1, rot_angle), (0, rot_angle)];

    let normals: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let maximal_angle = x.acos() + 0.015;
    let minimal_angle = edge.acos() * 0.5;
    let minimal_r = 3.0 / (maximal_angle - minimal_angle);
    let sz = minimal_r + 4.4;
    println!("sz = {sz}");
    /*
    let minimal_angle = x.acos() + 0.015;
    let maximal_angle_1 =  edge.acos();
    let minimal_r_1 = 4.0 / (maximal_angle_1 - minimal_angle);
    let maximal_angle_2 =  (-x).acos();
    let minimal_r_2 = 3.0 / (maximal_angle_2 - minimal_angle);
    let minimal_r = f32::max(minimal_r_1, minimal_r_2);
    let sz = minimal_r + 2.4;
    */

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let bsz = 28.5;
    println!("sz = {sz}");

    let groove = vec![
      (maximal_angle + 0.0 / minimal_r).cos(),
      minimal_r + 0.2,
      (maximal_angle - 2.0 / minimal_r).cos(),
      minimal_r + 2.6,
      (maximal_angle + 0.0 / minimal_r).cos(),
    ];

    let sz = [sz, sz, bsz, bsz, bsz, bsz].to_vec();

    let mut axis1 = Vec::new();
    let mut axis2 = Vec::new();
    for &a in &axis {
      let a1 = a.any_perp();
      let a2 = cross(a, a1);
      axis1.push(a1);
      axis2.push(a2);
    }

    let mut side_maps = Vec::new();

    let mut result =
      Self { axis, axis1, axis2, normals, groove, axis_pos, axis_neg, axis_seq, side_maps, sz };

    //result.check_side_points();
    result
  }

  fn check_side_points(&mut self) {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let mut side_maps: Vec<SideMap> = Vec::new();
    side_maps.resize(6, FxHashMap::default());
    let mut rng = StdRng::seed_from_u64(0);
    const ROUNDS: usize = 12;
    let mut round = [common::points2d::Point::ZERO; ROUNDS];
    for i in 0..ROUNDS {
      let a = i as f32 / ROUNDS as f32 * 2.0 * PI;
      round[i] = common::points2d::Point::from_angle(a);
    }
    let sz = self.sz[5];
    for side in 0..6 {
      let pn = self.normals[side];
      let pp1 = pn.any_perp().norm();
      let pp2 = cross(pp1, pn).norm();
      for iy0 in 0..=800 {
        let y0 = (iy0 as f32 * sz * 2.0 / 800.0) - sz;
        for ix0 in 0..=800 {
          let x0 = (ix0 as f32 * sz * 2.0 / 800.0) - sz;
          'genp: for ir in 0..2 {
            let r = [1.5, 3.05][ir];
            let pos = pn.scale(self.sz[side]) + pp1.scale(x0) + pp2.scale(y0);
            let nd = self.get_ndists(pos, self.faces());
            let n = self.normals[nd[0].1];
            let pos0 = pos + n.scale(nd[0].0 - 2.0);
            let i0 = self.get_part_index_for_check(pos0, nd[0].1);
            if i0 == 0 {
              continue 'genp;
            }
            let pos4 = pos + n.scale(nd[0].0 - 4.0);
            let p1 = n.any_perp().norm();
            let p2 = cross(p1, n).norm();
            for rv in &round {
              let rv = (p1.scale(rv.x) + p2.scale(rv.y)).scale(r + 0.5);
              if self.get_part_index_for_check(pos0 + rv, nd[0].1) != i0 {
                continue 'genp;
              }
              if self.get_part_index_for_check(pos4 + rv, nd[0].1) != i0 {
                continue 'genp;
              }
            }
            side_maps[nd[0].1].entry(i0).or_default().push((pos0, r));
          }
        }
      }
      println!("checked {}/6", side + 1);
    }
    // filter keypoints only

    for (ni, sm) in side_maps.iter_mut().enumerate() {
      let n = self.normals[ni];
      let p1 = n.any_perp().norm();
      let p2 = cross(p1, n).norm();
      let mut dv = [Point::ZERO; ROUNDS];
      for i in 0..round.len() {
        dv[i] = p1.scale(round[i].x) + p2.scale(round[i].y);
      }
      for (_, v) in sm {
        let mut top = [(v.len(), f32::NEG_INFINITY); ROUNDS];
        let mut btop = [(v.len(), f32::NEG_INFINITY); ROUNDS];
        for (pi, &(pos, r)) in v.iter().enumerate() {
          for i in 0..round.len() {
            let d = dot(pos, dv[i]);
            if d > top[i].1 {
              top[i].0 = pi;
              top[i].1 = d;
            }
            if r > 3.0 && d > btop[i].1 {
              btop[i].0 = pi;
              btop[i].1 = d;
            }
          }
        }
        let mut indices = Vec::new();
        for (i, _) in top {
          if i < v.len() {
            indices.push(i);
          }
        }
        for (i, _) in btop {
          if i < v.len() {
            indices.push(i);
          }
        }
        indices.sort();
        let mut nv = Vec::new();
        for i in 0..indices.len() {
          if i == 0 || indices[i] != indices[i - 1] {
            nv.push(v[indices[i]]);
          }
        }
        *v = nv;
      }
    }

    self.side_maps = side_maps;
  }

  fn get_ndists(&self, pos: Point, current_normal: usize) -> [(f32, usize); 6] {
    let mut result = [(f32::INFINITY, 0); 6];
    /*
    for (i, &n) in self.normals.iter().enumerate() {
      if i == current_normal {
        result[i] = (f32::INFINITY, i);
      } else {
        result[i] = (self.sz[i] - dot(pos, n), i);
      }
    }*/

    let pp = (pos.x.abs().powi(4) + pos.y.abs().powi(4)).powf(0.25);

    let h = self.sz[0] * f32::max(0.0, 1.0 - sqr(pp / self.sz[2])).sqrt();
    result[0] = (h - 0.0 - pos.z, 0);
    result[1] = (h + 0.0 + pos.z, 1);
    result.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    result
  }

  pub fn faces(&self) -> usize {
    self.normals.len() + 1
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.faces(), false)
  }

  pub fn get_part_index_for_check(&self, pos: Point, current_normal: usize) -> PartIndex {
    self.get_part_index_impl(pos, current_normal, true)
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
      0
    } else {
      let pos = Point { x: pos.x, y: 0.0, z: pos.y };
      self.get_part_index_impl(pos, current_normal, false)
    }
  }

  pub fn get_quality() -> usize {
    320
  }

  pub fn get_size() -> f32 {
    100.0
  }

  fn get_part_index_impl(&self, pos: Point, current_normal: usize, side_check: bool) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 49.999 || pos.y.abs() > 49.999 || pos.z.abs() > 49.999 {
      return 0;
    }

    let sphere_r = self.groove[1] - 2.0;
    if r < sphere_r {
      if r > sphere_r - 0.2 {
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

    let mut ndists = self.get_ndists(pos, current_normal);
    if ndists[0].0 < 0.0 {
      return 0;
    }
    /*
    if !side_check {
      let out_r = 2.0;
      if sqr(out_r - f32::min(ndists[0].0, out_r))
        + sqr(out_r - f32::min(ndists[1].0, out_r))
        + sqr(out_r - f32::min(ndists[2].0, out_r))
        > sqr(out_r)
      {
        return 0;
      }
    }*/

    let mut index: PartIndex = 0;

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);
    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();

    axis_pos.clear();
    axis_neg.clear();

    let mut spiral = false;

    enum PositionRelAxis {
      Inside(f32),
      Between,
      Outside(f32),
    };

    let get_pos_rel_axis = |pos: Point, a: Point, i: usize| {
      let c = dot(pos, a) / r;
      let s = cross(pos, a).len();
      let check_in = c - shift_in;
      if check_in > 0.0 {
        return PositionRelAxis::Inside(check_in);
      } else {
        let check_out = shift_out - c;
        if check_out > 0.0 {
          return PositionRelAxis::Outside(check_out);
        } else {
          return PositionRelAxis::Between;
        }
      }
    };

    let mut match_axis = |index: &mut PartIndex, pos: Point, a: Point, i: usize| {
      match get_pos_rel_axis(pos, a, i) {
        PositionRelAxis::Inside(check_in) => {
          *index |= (1 << i);
          axis_pos.push((check_in, self.axis[i]));
        }
        PositionRelAxis::Outside(check_out) => {
          axis_neg.push((check_out, self.axis[i]));
        }
        PositionRelAxis::Between => return false,
      }

      true
    };

    let mut mpos = pos;

    for &(i, ra) in &self.axis_seq {
      match get_pos_rel_axis(mpos, self.axis[i], i) {
        PositionRelAxis::Inside(_) => {
          mpos = mpos.rotate(self.axis[i], ra);
        }
        _ => {}
      }
    }

    for (i, &a) in self.axis.iter().enumerate() {
      let a = self.axis[i];
      if !match_axis(&mut index, mpos, a, i) {
        return 0;
      }
    }

    let hole_r = if r < self.groove[1] - 0.2 { 1.5 } else { 3.2 };

    let mut in_sr = |a, b, d: f32| {
      let r = 0.1 * d;
      if a < r && b < r && sqr(r - a) + sqr(r - b) > sqr(r) {
        return true;
      }
      false
    };

    axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    for p1 in 0..axis_pos.len() {
      for p2 in p1 + 1..axis_pos.len() {
        let d = dot(axis_pos[p1].1, axis_pos[p2].1);
        if in_sr(axis_pos[p1].0, axis_pos[p2].0, 1.0) {
          return 0;
        }
      }
    }

    for p1 in 0..axis_neg.len() {
      for p2 in p1 + 1..axis_neg.len() {
        let d = dot(axis_neg[p1].1, axis_neg[p2].1);
        if in_sr(axis_neg[p1].0, axis_neg[p2].0, 1.0) {
          return 0;
        }
      }
    }

    if axis_pos.len() < 2 {
      for p1 in 0..axis_pos.len() {
        for p2 in p1 + 0..axis_neg.len() {
          let d = dot(axis_pos[p1].1, axis_neg[p2].1);
          if in_sr(axis_pos[p1].0, axis_neg[p2].0, 1.0) {
            return 0;
          }
        }
      }
    }

    if index == 0 {
      index = 29;
    }

    if side_check {
      return 0;
    }

    if !side_check {
      let br = if index.count_ones() == 1 { self.groove[3] + 0.2 } else { self.groove[1] };
      let pz = if index == 5 { -pos.z } else { pos.z };

      for pp in [
        (20.0, -7.0),
        (20.0, 7.0),
        (12.0, 21.5),
        (7.0, 16.5),
        (-3.0, 20.0),
        (-19.0, 19.0),
        (-22.0, 0.0),
        (-15.0, 0.0),
        (-19.0, -19.0),
        (-3.0, -20.0),
        (11.0, -18.5),
        (18.0, -20.0),
      ] {
        let rr = (sqr(pos.x - pp.0) + sqr(pos.y - pp.1)).sqrt();
        if pos.z.abs() < 4.0 && rr < 3.0 {
          if pos.z.abs() < 3.6 && rr < 2.88 {
            index += 2000;
          } else {
            return 0;
          }
        }
      }

      if index < 2000 {
        if pz < 0.1 && r > br - 0.2 {
          if pz > -0.1 || r < br + 0.0 {
            return 0;
          }
          index += 1000;
        }
        if index == 5 || index == 1005 {
          index = 1010 - index;
        }

        if ndists[0].0 > 4.0 {
          for (i, &a) in self.axis.iter().enumerate() {
            let c = dot(pos, a) / r;
            let s = cross(pos, a).len();
            if c > 0.0 {
              if s < hole_r {
                return 0;
              }
            }
          }
        }
      }
    }

    return index;
  }
}
