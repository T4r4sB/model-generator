use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;

use fxhash::FxHashMap;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

#[derive(Debug, Default, Copy, Clone)]
struct SideClutchInfo {
  center: Point,
  r: f32,
}

pub struct GhostPrismCreator {
  axis: Vec<Point>,
  z_axis: Point,
  sz: f32,
  edge: f32,
  a2_for_1: Point,
  maximal_angle_cos: f32,
  normals: Vec<(Point, Point, Point)>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
  axis_seq: Vec<(usize, f32)>,
  side_clutches: FxHashMap<PartIndex, Vec<SideClutchInfo>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl GhostPrismCreator {
  pub fn new() -> Self {
    let a = 65.0 * PI / 180.0;
    let b = (2.0 * PI - 2.0 * a) / 3.0;
    let x = PI * 0.5 - ((a.cos() - b.cos()) / (2.0 + a.cos() - b.cos())).sqrt().acos();

    fn from_polar_coords(longitude: f32, lattitude: f32) -> Point {
      let (s1, c1) = longitude.sin_cos();
      let (s2, c2) = lattitude.sin_cos();
      Point { x: c2 * c1, y: c2 * s1, z: s2 }
    }

    let axis: Vec<_> = [
      from_polar_coords(0.0, x),
      from_polar_coords(b, x),
      from_polar_coords(b + b, x),
      from_polar_coords(b + b + a, -x),
      from_polar_coords(b + b + a + b, -x),
    ]
    .into_iter()
    .collect();

    let edge = dot(axis[0], axis[1]);
    let long_edge = dot(axis[0], axis[3]);
    let rot_angle = ((long_edge - sqr(edge)) / (1.0 - sqr(edge))).acos();
    let rot_angle_s = ((dot(axis[0], axis[2]) - sqr(edge)) / (1.0 - sqr(edge))).acos();

    println!("x={x}, edge={edge}");

    let a2_for_1 = axis[0].rotate(axis[1], rot_angle);

    let axis_seq = vec![(2, PI * 0.5), (1, rot_angle_s), (0, -rot_angle), (4, rot_angle)];

    let normals: Vec<_> = [
      Point { x: 1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: -1.0 },
    ]
    .into_iter()
    .map(|n| {
      let n = n.norm();
      let n2 = n.any_perp().norm();
      let n3 = cross(n, n2);
      (n, n2, n3)
    })
    .collect();

    let minimal_angle = edge.acos() * 0.5;
    let maximal_angle = long_edge.acos() * 0.5;
    let maximal_angle_2 = edge.acos();
    let minimal_sphere_1 = 5.0 / (maximal_angle - minimal_angle);
    let minimal_sphere_2 = 8.0 / (maximal_angle_2 - minimal_angle);
    let sphere_r = f32::max(minimal_sphere_1, minimal_sphere_2);
    println!("min_sphere={minimal_sphere_1}, {minimal_sphere_2}");

    let minimal_angle = edge.acos() * 0.5;
    let maximal_angle = long_edge.acos() * 0.5;
    let maximal_angle_cos = maximal_angle.cos();

    let sz = 28.5;

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let groove = vec![
      (maximal_angle_2 - 3.0 / (sphere_r + 2.0)).cos(),
      sphere_r + 2.2,
      (minimal_angle + 2.0 / (sphere_r + 2.0)).cos(),
      sz - 7.3,
      (maximal_angle_2 - 4.0 / (sz - 7.5)).cos(),
      sz - 4.9,
      (maximal_angle - 0.0).cos(),
    ];

    let z_axis = Point { x: 0.0, y: 0.0, z: 1.0 };
    let side_clutches = FxHashMap::default();
    let mut result = Self {
      axis: axis.clone(),
      z_axis,
      sz,
      edge,
      a2_for_1,
      maximal_angle_cos,
      normals,
      groove,
      axis_pos,
      axis_neg,
      axis_seq,
      side_clutches,
    };

    let mut rng = StdRng::seed_from_u64(400);
    let mut best = 1;
    let mut best_ghosting = (Point { x: -0.36161304, y: -0.88137895, z: -0.30398527 }, 0.6558536);
    let mut ghosting = best_ghosting;
    let mut weight = 1.0;

    for it in 0.. {
      let mut worst = usize::MAX;
      let mut wi = 0;

      let make_ghost = |p: Point| -> Point { p.rotate(ghosting.0, ghosting.1) };

      result.axis = axis.iter().copied().map(make_ghost).collect();
      result.a2_for_1 = make_ghost(a2_for_1);
      result.z_axis = make_ghost(z_axis);

      let mut side_clutches = FxHashMap::<PartIndex, Vec<SideClutchInfo>>::default();
      for side in 0..result.normals.len() {
        let cn = result.normals[side];
        let mut ms = FxHashMap::<PartIndex, (usize, bool)>::default();
        let mut buf = [(0, 0); 1024];
        for ix in -285..=285i32 {
          let mut prev = (0, 0);
          let mut pb = 0;
          let mut cb = 0;
          for iy in -285..=285i32 {
            let fx = ix as f32 * 0.1;
            let fy = iy as f32 * 0.1;

            let base = cn.1.scale(fx) + cn.2.scale(fy);
            let pos = cn.0.scale(sz - 0.01) + base;
            let control_pos_1 = cn.0.scale(sz - 2.0) + base;
            let control_pos_2 = cn.0.scale(sz - 3.9) + base;
            let c = result.get_part_index_impl(control_pos_1, 7);
            let c2 = result.get_part_index_impl(control_pos_2, 7);
            pb = (iy + 511) as usize;
            cb = (iy + 512) as usize;
            let mut cnt = 0;
            if c != 0 && c == c2 && ix.abs() < 245 && iy.abs() < 245 {
              let dst = ms.entry(c).or_insert((0, false));
              if c == prev.0 && c == buf[cb].0 && c == buf[pb].0 {
                cnt = std::cmp::min(prev.1, std::cmp::min(buf[cb].1, buf[pb].1)) + 1;
                if cnt > dst.0 {
                  dst.0 = cnt;
                  let r = cnt as f32 * 0.05;
                  let sc = side_clutches.entry(c).or_default();
                  sc.resize_with(result.normals.len(), Default::default);
                  sc[side].center = pos - (cn.1 + cn.2).scale(r);
                  sc[side].r = r;
                }
              }
              if ix.abs() < 245 && iy.abs() < 245 || ix.abs() > 280 && iy.abs() > 280 {
                dst.1 = true;
              }
            }

            buf[pb] = prev;
            prev = (c, cnt);
          }
          buf[cb] = prev;
        }
        for (ci, (cnt, famous)) in ms {
          if famous && cnt < worst {
            worst = cnt;
            wi = ci;
          }
        }
      }
      println!("matched iter {it} worst sticker {worst} at {wi}");
      if worst > best {
        best = worst;
        best_ghosting = ghosting;
        result.side_clutches = side_clutches;
        println!("on iter {it} has better sticker {worst} ghosting={ghosting:?}");
      }

      if weight > 0.5 {
        weight -= 0.05;
      } else if weight > 0.1 {
        weight -= 0.01;
      } else {
        weight -= 0.001;
      }

      if it == 0 || weight <= 0.0 {
        let make_ghost = |p: Point| -> Point { p.rotate(best_ghosting.0, best_ghosting.1) };
        println!("on iter {it} run with worst sticker {best} ghosting={best_ghosting:?}");
        result.axis = axis.iter().copied().map(make_ghost).collect();
        result.a2_for_1 = make_ghost(a2_for_1);
        result.z_axis = make_ghost(z_axis);
        println!("result axis={:?}", result.axis);
        return result;
      }

      let gx = rng.gen_range(-1.0..1.0);
      let gy = rng.gen_range(-1.0..1.0);
      let gz = rng.gen_range(-1.0..1.0);
      let gr = rng.gen_range(-PI..PI);

      ghosting = (
        Point { x: gx, y: gy, z: gz }.scale(weight) + best_ghosting.0.scale(1.0 - weight),
        gr * weight + best_ghosting.1 * (1.0 - weight),
      );
      if ghosting.0.len() < 0.1 {
        ghosting.0 = Point::Z;
      }
      ghosting.0 = ghosting.0.norm();
    }

    unreachable!()
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
    if current_normal == 0 {
      let pos = Point { x: pos.x, y: 0.0, z: pos.y };
      self.get_part_index_impl(pos, current_normal)
    } else {
      0
    }
  }

  pub fn get_quality() -> usize {
    320
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 49.999 || pos.y.abs() > 49.999 || pos.z.abs() > 49.999 {
      return 0;
    }

    let sphere_r = self.groove[1] - 2.2;

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
    let sz = self.sz;

    let mut dists = [(f32::INFINITY, 0); 8];
    for (i, n) in self.normals.iter().enumerate() {
      let d = sz - dot(pos, n.0);
      if d < 0.0 {
        return 0;
      }
      dists[i] = (d, i);
    }
    dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    if current_normal <= 6 {
      let out_r = 2.0;
      if sqr(out_r - f32::min(dists[0].0, out_r))
        + sqr(out_r - f32::min(dists[1].0, out_r))
        + sqr(out_r - f32::min(dists[2].0, out_r))
        > sqr(out_r)
      {
        return 0;
      }
    }

    let mut index: PartIndex = 0;

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);
    let over_r = f32::min(sz - 1.0, r) - (self.groove[1] + 2.2);

    if over_r > 0.0 {
      //  shift_in = f32::max(self.maximal_angle_cos, shift_in - over_r * 0.021);
      //   shift_out = f32::max(self.maximal_angle_cos, shift_out - over_r * 0.021);
    }

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
      let a = if index & 2 != 0 && i == 2 { self.a2_for_1 } else { self.axis[i] };
      if !match_axis(&mut index, mpos, a, i) {
        return 0;
      }
    }

    let color_r = self.groove[1] + 0.4;
    let screw_r = sphere_r + 4.4;
    let hole_r = if r < screw_r {
      1.5
    } else if r < color_r {
      3.2
    } else {
      3.2
    };

    if r < self.groove[self.groove.len() - 2] - 0.2 {
      for (i, &a) in self.axis.iter().enumerate() {
        let c = dot(mpos, a) / r;
        let s = cross(mpos, a).len();
        if c > 0.0 {
          if s < hole_r {
            return 0;
          }
          if index.count_ones() == 1 && r > screw_r && s < hole_r + 0.0 {
            return 0;
          }
        }
      }
    }

    let mut thick = false;

    if !thick {
      if spiral {
        //  return 0;
      }

      let mut in_sr = |a, b, d: f32| {
        let r = 0.1 * d;
        if a < r && b < r && sqr(r - a) + sqr(r - b) > sqr(r) {
          return true;
        }
        false
      };

      axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
      axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

      let mut i = 0;
      while i + 1 < axis_pos.len() && dot(axis_pos[i].1, axis_pos[i + 1].1) > 0.99 {
        i += 1;
      }
      axis_pos.drain(..i);

      let mut i = 0;
      while i + 1 < axis_neg.len() && dot(axis_neg[i].1, axis_neg[i + 1].1) > 0.99 {
        i += 1;
      }
      axis_neg.drain(..i);

      if axis_pos.len() >= 2 {
        let d = dot(axis_pos[0].1, axis_pos[1].1);
        if d > -0.99 && in_sr(axis_pos[0].0, axis_pos[1].0, 1.0) {
          return 0;
        }
      }
      if axis_neg.len() >= 2 {
        let d = dot(axis_neg[0].1, axis_neg[1].1);
        if in_sr(axis_neg[0].0, axis_neg[1].0, 1.0) {
          return 0;
        }
      }
      if !inter {
        for p in axis_pos.iter() {
          for n in axis_neg.iter() {
            let d = dot(p.1, n.1);
            if d > self.edge - 0.001 && in_sr(p.0, n.0, 1.0) {
              return 0;
            }
          }
        }
      }
    }

    let mut sum_a = Point::ZERO;
    for i in 0..self.axis.len() {
      if index & (1 << i) != 0 {
        sum_a += self.axis[i];
      }
    }

    if index == 0 {
      if dot(mpos, self.z_axis) > 0.0 {
        index = 29;
      } else {
        index = 30;
      }
    }

    // if index != 1 { return 0; }

    // return index; // if dont need colors

    if dists[0].0 < 4.0 {
      let index_for_check = if index.count_ones() == 1 && dot(mpos, self.z_axis) > 0.0 {
        index + 5000
      } else {
        index
      };

      if current_normal <= 6 {
        let mut hdist = 10.0;
        if let Some(clutches) = self.side_clutches.get(&index_for_check) {
          let clutch = clutches[dists[0].1];
          let ni = self.normals[dists[0].1];
          let proj = pos + ni.0.scale(dists[0].0 - 0.01);
          let dproj = proj - clutch.center;
          let dx = dot(dproj, ni.1);
          let dy = dot(dproj, ni.2);
          hdist = f32::max(dx.abs() - clutch.r, dy.abs() - clutch.r);
        }

        if dists[0].0 < 2.0 || dists[0].0 < 4.0 && hdist < -1.0 {
          if dists[0].0 < 1.8 || dists[0].0 < 3.5 && hdist < -1.12 {
            if dists[0].0 + 0.2 / 2.0.sqrt() > dists[1].0 {
              return 0;
            }
            index = index_for_check + (dists[0].1 as PartIndex + 1) * 10000;
          } else {
            return 0;
          }
        }
      } else {
        index = index_for_check;
      }
    }

    return index;
  }
}
