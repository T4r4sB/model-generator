use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;
use num::PrimInt;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

fn sqr(x: f32) -> f32 {
  x * x
}

pub struct Pyra5Creator {
  axis: Vec<Point>,
  holes: Vec<Point>,
  normals: Vec<Point>,

  a01_l: Point,
  a01_r: Point,
  a02_l: Point,
  a02_r: Point,
  a03_l: Point,
  a03_r: Point,
  a04_l: Point,
  a04_r: Point,

  r12_l: Point,
  r12_r: Point,
  r23_l: Point,
  r23_r: Point,
  r34_l: Point,
  r34_r: Point,
  r41_l: Point,
  r41_r: Point,

  groove: Vec<f32>,
  slice_groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, usize)>>,
  axis_neg: RefCell<Vec<(f32, usize)>>,
}

impl Pyra5Creator {
  pub fn new() -> Self {
    let ball_radius = 25.0;

    let max_angle = (1.0 / 7.0).sqrt().acos();
    let max_inner_angle = (-0.5).acos() * 0.5;
    let min_inner_angle = (0.25).acos() * 0.5;
    let ahole = max_angle - max_inner_angle;
    let ahole_inner = max_inner_angle - min_inner_angle;

    let r = 6.0 / ahole_inner;
    let r = f32::max(r, 20.0);

    /*
    let groove = vec![
      (max_inner_angle - 0.0 / (r - 4.8)).cos(),
      r - 4.6,
      (max_inner_angle - 3.0 / (r - 4.8)).cos(),
     r - 2.2,
      (max_angle).cos(),
      r + 0.2,
      (max_angle - 3.0 / r).cos(),
      r + 2.6,
      (max_angle).cos(),
    ];*/

    let groove = vec![
      1.0,
      r - 5.4,
      (8.5 / (r - 5.6)).cos(),
      r - 3.4,
      (5.0 / (r - 5.6)).cos(),
      r - 1.4,
      (max_inner_angle - 0.0 / r).cos(),
      r + 0.6,
      (max_inner_angle - 4.0 / r).cos(),
      r + 2.6,
      (max_inner_angle - 0.0 / r).cos(),
    ];

    let min_core_r = (16.0) / (PI / 2.0);
    assert!(r - 9.6 >= min_core_r);

    let slice_groove = vec![
      (PI * 0.25 + 4.0 / (r - 7.6)).cos(),
      r - 7.4,
      (PI * 0.25 + 1.5 / (r - 7.6)).cos(),
      r - 5.4,
      0.0,
    ];

    println!("r={}", r);

    let tr = 0.75.sqrt();

    let mut axis = vec![
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: -tr, z: -0.5 },
      Point { x: -tr, y: 0.0, z: -0.5 },
      Point { x: 0.0, y: tr, z: -0.5 },
      Point { x: tr, y: 0.0, z: -0.5 },
      // slice_axis
      Point { x: 1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
    ];

    let mut normals = vec![
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: -tr, z: 0.5 },
      Point { x: 0.0, y: tr, z: 0.5 },
      Point { x: -tr, y: 0.0, z: 0.5 },
      Point { x: tr, y: 0.0, z: 0.5 },
    ];

    let mut holes = axis.iter().copied().map(|v| -v).collect();

    if false {
      // saurus
      for dst in [&mut axis, &mut normals, &mut holes] {
        for v in dst {
          if v.y < 0.0 {
            *v = v.rotate(Point::Y, 2.0 * PI / 3.0);
          }
        }
      }
    }

    fn reflect(p: Point, p1: Point, p2: Point) -> Point {
      let a = (p1 + p2).norm();
      a.scale(2.0 * dot(a, p)) - p
    }

    let a01_l = find_square(axis[0], axis[1]);
    let a01_r = find_square(axis[1], axis[0]);
    let a02_l = find_square(axis[0], axis[2]);
    let a02_r = find_square(axis[2], axis[0]);
    let a03_l = find_square(axis[0], axis[3]);
    let a03_r = find_square(axis[3], axis[0]);
    let a04_l = find_square(axis[0], axis[4]);
    let a04_r = find_square(axis[4], axis[0]);

    let r12_l = reflect(axis[3], axis[1], axis[2]);
    let r12_r = reflect(axis[4], axis[1], axis[2]);
    let r23_l = reflect(axis[4], axis[2], axis[3]);
    let r23_r = reflect(axis[1], axis[2], axis[3]);
    let r34_l = reflect(axis[1], axis[3], axis[4]);
    let r34_r = reflect(axis[2], axis[3], axis[4]);
    let r41_l = reflect(axis[2], axis[4], axis[1]);
    let r41_r = reflect(axis[3], axis[4], axis[1]);

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    Self {
      axis,
      holes,
      normals,

      a01_l,
      a01_r,
      a02_l,
      a02_r,
      a03_l,
      a03_r,
      a04_l,
      a04_r,

      r12_l,
      r12_r,
      r23_l,
      r23_r,
      r34_l,
      r34_r,
      r41_l,
      r41_r,

      groove,
      slice_groove,
      axis_pos,
      axis_neg,
    }
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    0.9
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    1
  }

  pub fn get_name(&self, current_normal: usize) -> Option<String> {
    None
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    if current_normal == 0 {
      return self.get_part_index_impl(Point { x: 0.0, y: pos.x, z: pos.y }, 5);
    }
    if current_normal == 1 {
      return self.get_part_index_impl(Point { x: pos.x, y: 0.0, z: pos.y }, 5);
    }
    if current_normal == 2 {
      return self
        .get_part_index_impl(Point { x: pos.x * 0.5.sqrt(), y: pos.x * 0.5.sqrt(), z: pos.y }, 5);
    }
    if current_normal == 3 {
      return self.get_part_index_impl(Point { x: pos.x, y: pos.y, z: 0.0 }, 5);
    }

    let n = self.normals[current_normal];

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

    let k = 0.006;
    let k2 = k * 2.0;
    let last_groove = self.groove[self.groove.len() - 2];
    let max = last_groove + 2.2;

    let r = pos.len();
    let a = r * k2;

    let n1 = n.any_perp().norm();
    let n2 = cross(n, n1).norm();

    let pos = pos.scale(sinc(a));
    let pos = n.scale(max - r * versinc(a)) + n1.scale(pos.x) + n2.scale(pos.y);

    let r = pos.len();
    let d = dot(pos, n);

    let control_c = n.scale(max - k2.recip());
    let delta = d + (max * max + r * r) * k - (2.0 * d * k + 1.0) * max;

    let result = self.get_part_index_impl(pos, current_normal);

    (result > 0) as PartIndex
  }

  pub fn faces(&self) -> usize {
    self.normals.len() * 1
  }

  pub fn get_quality() -> usize {
    320
  }

  pub fn get_size() -> f32 {
    110.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if pos.x.abs() > 59.0 || pos.y.abs() > 59.0 || pos.z.abs() > 59.0 {
      return 0;
    }

    let r = pos.len();

    if r > self.groove[3] + 1.2 {
      // return 0;
    }

    let inner_r = self.slice_groove[1] - 7.4;
    
    /*if r < inner_r - 0.2 {
      for a in 5..=6 {
        if dot(pos, self.axis[a]) > 0.0 && cross(pos, self.axis[a]).len() < 1.2 {
          return 0;
        }
      }

      return 31;
    }*/

    let m = self.groove[self.groove.len() - 2] + 2.2;
    let mut cup = false;
    let mut near_another_wall = false;
    let mut near_wall = false;
    let mut near_walls = 0;
    for ni in 0..self.normals.len() {
      if ni == current_normal {
        cup = true;
        continue;
      }

      let mut m = m;

      if current_normal < self.normals.len() {
        m -= 2.0;
      }

      let d = dot(pos, self.normals[ni]);
      let k = 0.004;
      if d + (m * m + r * r) * k > (2.0 * d * k + 1.0) * m {
        return 0;
      }
      {
        let m = m - 2.0;
        if d + (m * m + r * r) * k > (2.0 * d * k + 1.0) * m {
          cup = true;
        }
      }
      {
        let m = m - 1.0;
        if d + (m * m + r * r) * k > (2.0 * d * k + 1.0) * m {
          near_walls += 1;
          if current_normal < self.normals.len() {
            near_another_wall = true;
          }
        }
      }
      {
        let m = m - 0.5;
        if d + (m * m + r * r) * k > (2.0 * d * k + 1.0) * m {
          near_wall = true;
        }
      }
    }

    /*
    if !cup {
      for &a in &self.axis {
        let d = dot(pos, a);
        let s = cross(pos, a).len();
        if d > 0.0 && s < 1.20 {
          return 0;
        }
      }
    }
    */

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);
    let (mut s_shift_out, mut s_shift_in, s_inter) = get_groove(r, &self.slice_groove, 0.03);

    let rat = (self.slice_groove[1] - 0.2) / r;
    if rat > 1.0 {
      s_shift_out *= rat;
      s_shift_in *= rat;
    }

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();

    axis_pos.clear();
    axis_neg.clear();

    let mut index: PartIndex = 0;

    let mut match_all = || -> Option<()> {
      let mut match_axis = |pos: Point,
                            index: &mut PartIndex,
                            bit: usize,
                            axis: Point,
                            factor: f32,
                            shift_in: f32,
                            shift_out: f32|
       -> Option<()> {
        let d = dot(pos, axis) / r;

        let check_in = d - shift_in;
        if check_in > 0.0 {
          *index |= (1 << bit);
          axis_pos.push((check_in * factor, bit));
        } else {
          let check_out = shift_out - d;
          if check_out > 0.0 {
            axis_neg.push((check_out * factor, bit));
          } else {
            return None;
          }
        }

        return Some(());
      };
      for i in 0..self.axis.len() {
        if i < 5 {
          if r > self.groove[1] - 0.2 {
            match_axis(pos, &mut index, i, self.axis[i], 1.0, shift_in, shift_out)?;
          }
        } else {
          if i == 5 || index & 31 != 1 {
            match_axis(pos, &mut index, i, self.axis[i], r / 30.0, s_shift_in, s_shift_out)?;
          }
        }
      }
      if index != 1 {
        //return None;
      }

      Some(())
    };

    if match_all().is_none() {
      return 0;
    }

    if index.count_ones() != 1 {
      // return 0; // tmp
    }

    if index & 31 == 0 {
      let b1 = r - self.groove[self.groove.len() - 4];
      let x = pos.x.signum();
      let y = pos.y.signum();
      let b2 = dot(pos, Point { x, y: -y, z: 0.0 }.norm()).abs();
      let b3 = dot(pos, Point { x, y, z: -3.3 }.norm()).abs();
    
      if b1 > 2.2 || (b2 > 2.1 || b3 > 2.1) && b1 > 0.2 {
        index += 128
      } else if b1 > 2.0 || (b2 > 1.9 || b3 > 1.9) && b1 > 0.0 {
        return 0;
      }
    }

    if index == 0 {
      index = 31;
    } else if r < inner_r + 0.2 || true{ // true: tmp
      return 0;
    }

    if r < self.groove[self.groove.len() - 2] {
      for a in 5..=6 {
        let d = dot(pos, self.axis[a]);
        let hole_r = if d < inner_r + 2.0 { 1.5 } else { 3.2 };
        if d > 0.0 && cross(pos, self.axis[a]).len() < hole_r {
          return 0;
        }
      }
    }

    for a in 5..=6 {
      if index == 1 << a {
        let a = self.axis[a as usize];
        if r < self.slice_groove[1] && dot(pos, a) < 7.6 && cross(pos, a).len() > 9.8 {
          return 0;
        }
      }
    }

    let thick = (index & 31).count_ones() == 3
      && r > self.groove[self.groove.len() - 4] - 0.2
      && r < self.groove[self.groove.len() - 2] + 0.2;

    let outc = (index & 31).count_ones() == 1 
      && r > self.groove[self.groove.len() - 8] - 0.2
      && r < self.groove[self.groove.len() - 6] + 0.2;


    let mut rr = if (index & 31).count_ones() == 4 {
      if thick {
        0.1f32
      } else {
        0.3f32
      }
    } else {
      if thick || outc {
        0.3f32
      } else {
        1.0f32
      }
    };

    axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    let mut minimal = axis_pos
      .iter()
      .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
      .map(|(a, _)| *a)
      .unwrap_or(f32::INFINITY);
    minimal = f32::min(
      minimal,
      axis_neg
        .iter()
        .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
        .map(|(a, _)| *a)
        .unwrap_or(f32::INFINITY),
    );

    let mut in_sr = |a, b, d| {
      let r = 0.096 * d;
      if a < r && b < r {
        return r - (sqr(r - a) + sqr(r - b)).sqrt();
      }
      return f32::INFINITY;
    };

    let outer_edge = (index & 31).count_ones() == 2;

    for p0 in 0..axis_pos.len() {
      if outer_edge && axis_pos[p0].1 >= 5 {
        continue;
      }
      if r < self.groove[1] - 0.2 && axis_pos[p0].1 < 5 {
        continue;
      }
      for p1 in p0 + 1..axis_pos.len() {
        if outer_edge && axis_pos[p1].1 >= 5 {
          continue;
        }
        if r < self.groove[1] - 0.2 && axis_pos[p0].1 < 5 {
          continue;
        }
        let d = dot(self.axis[axis_pos[p0].1], self.axis[axis_pos[p1].1]);
        if !thick || d > -0.01 {
          minimal = f32::min(minimal, in_sr(axis_pos[p0].0, axis_pos[p1].0, rr));
        }
      }
    }

    if !outer_edge {
      for n0 in 0..axis_neg.len() {
        for n1 in n0 + 1..axis_neg.len() {
          let d = dot(self.axis[axis_neg[n0].1], self.axis[axis_neg[n1].1]);
          if !thick || d > -0.01 {
            minimal = f32::min(minimal, in_sr(axis_neg[n0].0, axis_neg[n1].0, rr));
          }
        }
      }

      if !s_inter {
        for p0 in 0..axis_pos.len() {
          for n0 in 0..axis_neg.len() {
            let d = dot(self.axis[axis_pos[p0].1], self.axis[axis_neg[n0].1]);
            if d > -0.01 {
              minimal = f32::min(minimal, in_sr(axis_pos[p0].0, axis_neg[n0].0, rr));
            }
          }
        }
      }
    }

    if minimal < 0.0 {
      return 0;
    }

    if near_wall && near_walls == 1 && minimal > 0.04 {
      return 0;
    }

    if current_normal < self.normals.len() {
      if minimal < 0.03 {
        return 0;
      }
      if !near_another_wall && minimal > 0.07 {
        return 0;
      }
    }

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.axis.len())
  }
}
