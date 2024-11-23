use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::FxHashSet;
use num::Float;
use num::PrimInt;
use tinyvec::*;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

fn sqr(x: f32) -> f32 {
  x * x
}

struct SpacerInfo {
  center: Point,
  c1: Point,
  c2: Point,
  cdot: f32,
  rdot: f32,
}

pub struct SemiosnikCreator {
  axis: Vec<Point>,
  normals: Vec<Point>,

  petals: FxHashSet<PartIndex>,
  add1: Vec<(PartIndex, Point)>,
  add2: Vec<(PartIndex, Point, Point)>,

  groove: Vec<f32>,
  inner_groove: Vec<f32>,

  spacer_info: Option<SpacerInfo>,

  core_cos1: f32,
  core_cos2: f32,
}

impl SemiosnikCreator {
  pub fn new() -> Self {
    let e1cos = 1.0 / 3.0;
    let e2cos = -1.0 / 3.0;
    let e1halfcos = ((e1cos + 1.0) * 0.5).sqrt();
    let e2halfcos = ((e2cos + 1.0) * 0.5).sqrt();
    let hbig = e2cos / e1halfcos;
    let hsmall = e2halfcos / e1halfcos;
    let ahole = hbig.acos() - hsmall.acos() - e2halfcos.acos();
    let r = 8.0 / ahole;

    let min_angle = e2halfcos.acos();

    let inner_groove = vec![
      (min_angle - 0.0 / (r - 6.8)).cos(),
      r - 9.0,
      (min_angle - 2.0 / (r - 6.8)).cos(),
      r - 6.6,
      (min_angle - 0.0 / (r - 6.8)).cos(),
      r - 4.2,
      (min_angle + 4.0 / r).cos(),
    ];

    let groove = vec![
      r - 3.0,
      (min_angle + 4.0 / r).cos(),
      r + 0.1,
      (min_angle + 1.0 / r).cos(),
      r + 3.2,
      (min_angle + 4.0 / r).cos(),
    ];

    let phi = (5.0.sqrt() + 1.0) * 0.5;
    let p = 2.0 - phi;

    let axis: Vec<_> = [
      // 1.0
      /*
      Point { x: -1.0, y: 1.0, z: -1.0 },
      Point { x: -1.0, y: -1.0, z: -1.0 },
      Point { x: 1.0, y: 1.0, z: -1.0 },
      Point { x: 1.0, y: -1.0, z: -1.0 },
      Point { x: -1.0, y: -1.0, z: 1.0 },
      Point { x: 1.0, y: -1.0, z: 1.0 },
      Point { x: 0.0, y: 1.0, z: p },
      */
      /*  // 2.0
       Point { x: -1.0, y: -1.0, z: -1.0 },
       Point { x: -1.0, y: -1.0, z: 1.0 },
       Point { x: -1.0, y: 1.0, z: -1.0 },
       Point { x: 1.0, y: -1.0, z: -1.0 },
       Point { x: 1.0, y: p, z: 0.0 },
       Point { x: 0.0, y: 1.0, z: p },
       Point { x: p, y: 0.0, z: 1.0 },
       */
      // 3.0
      Point { x: -1.0, y: 1.0, z: -1.0 },
      Point { x: -1.0, y: -1.0, z: -1.0 },
      Point { x: 1.0, y: 1.0, z: -1.0 },
      Point { x: 1.0, y: -1.0, z: -1.0 },
      Point { x: p, y: 0.0, z: 1.0 },
      Point { x: -1.0, y: -1.0, z: 1.0 },
      Point { x: 0.0, y: 1.0, z: p },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let mut normals = Vec::new();
    let mut add_normal = |p: Point| {
      if !normals.iter().any(|&n| dot(n, p) > 0.99) {
        normals.push(p);
      }
    };

    let mut add1 = Vec::new();
    let mut add2 = Vec::new();
    let mut petals = FxHashSet::default();

    let ise1 = |p1, p2| (dot(p1, p2) - e1cos).abs() < 0.001;
    let ise2 = |p1, p2| (dot(p1, p2) - e2cos).abs() < 0.001;

    let mut spacer_info = None;

    for i1 in 0..axis.len() {
      let a1 = axis[i1];
      for i2 in i1 + 1..axis.len() {
        let a2 = axis[i2];
        if ise2(a1, a2) {
          petals.insert(1 << i1 | 1 << i2);
          add_normal((a1 + a2).norm());
          let cr = cross(a1, a2).scale(0.75.sqrt());
          let mut has1 = None;
          let mut has2 = None;

          for i3 in 0..axis.len() {
            let a3 = axis[i3];
            if ise1(a1, a3) && ise1(a2, a3) {
              if dot(a3, cr) > 0.0 {
                has1 = Some(i3);
              } else {
                has2 = Some(i3);
              }
            }
          }

          let p1 = (a1 + a2).scale(0.5) + cr;
          let p2 = (a1 + a2).scale(0.5) - cr;

          if let Some(has1) = has1 {
            if has2.is_none() {
              add1.push((1 << has1, p2));
            }
          } else {
            if let Some(has2) = has2 {
              add1.push((1 << has2, p1));
            } else {
              add2.push((1 << i1 | 1 << i2, p1, p2));
            }
          }
        }

        if ise1(a1, a2) {
          for i3 in i2 + 1..axis.len() {
            let a3 = axis[i3];
            if ise1(a1, a3) && ise1(a2, a3) {
              add_normal((a1 + a2 + a3).norm());
            }
          }

          let cr = cross(a1, a2).scale((15.0 / 16.0).sqrt());
          let mut has1 = 0;
          let mut has2 = 0;

          for i3 in 0..axis.len() {
            let a3 = axis[i3];
            let tar = if dot(a3, cr) > 0.0 { &mut has1 } else { &mut has2 };
            if ise1(a1, a3) && ise1(a2, a3)
              || ise1(a1, a3) && ise2(a2, a3)
              || ise2(a1, a3) && ise1(a2, a3)
            {
              *tar = std::cmp::max(*tar, 2);
            } else if ise2(a1, a3) && ise2(a2, a3) {
              if spacer_info.is_none() {
                let f = (3.0 / 35.0).sqrt();
                let center = a3.scale(3.0 * f) + a1.scale(2.0 * f) + a2.scale(2.0 * f);
                let m1 = (a1 + a3).scale(0.75.sqrt());
                let m2 = (a2 + a3).scale(0.75.sqrt());
                let c1 = (m1 + center.scale(0.5)).norm();
                let c2 = (m2 + center.scale(0.5)).norm();
                let cdot = dot(c1, center);
                let rdot = 1.0 + (dot(c1, m1) - 1.0) * 0.5;
                spacer_info = Some(SpacerInfo { center, c1, c2, cdot, rdot });
              }

              *tar = std::cmp::max(*tar, 1);
            }
          }

          let mut reg = |p| {
            add1.push((1 << i1 | 1 << i2, p));
          };

          if has1 == 1 {
            reg((a1 + a2).scale(0.25) + cr);
          }
          if has2 == 1 {
            reg((a1 + a2).scale(0.25) - cr);
          }
        }
      }
    }

    let core_a = e2halfcos.acos() * 0.45;

    let core_cos1 = core_a.cos();
    let core_cos2 = (core_a - 0.3 / (r - 4.5)).cos();

    Self {
      axis,
      normals,
      groove,
      inner_groove,
      petals,
      add1,
      add2,
      spacer_info,
      core_cos1,
      core_cos2,
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

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn faces(&self) -> usize {
    self.normals.len()
  }

  pub fn get_quality() -> usize {
    512
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if pos.x.abs() > 49.0 || pos.y.abs() > 49.0 || pos.z.abs() > 49.0 {
      return 0;
    }

    let r = pos.len();
    let inner_r = self.inner_groove[1] - 2.4;
    if r < inner_r {
      return 0;
      if r > inner_r - 0.3 {
        return 0;
      }
      if r < inner_r - 5.0 {
        return 0;
      }
      for &a in &self.axis {
        if dot(pos, a) > 0.0 && cross(pos, a).len() < 1.5 {
          return 0;
        }
      }
      let mut first = true;
      for &n in &self.normals {
        if dot(pos, n) > r * self.core_cos2 {
          //return if first { 62 } else { 0 };
          return 0;
        } else if dot(pos, n) > r * self.core_cos1 {
          return 0;
        }
        first = false;
      }

      return 63;
    }

    let m = self.groove[self.groove.len() - 2] + 0.8;
    let mut cup = false;
    let mut hcup = true;
    let surface_k = 0.006;

    let mut nheights = ArrayVec::<[(f32, usize); 10]>::default();

    let core;
    if r < m - 2.0 {
      core = true;
    } else {
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
        let k = surface_k;
        let ur = (m * m + r * r - 2.0 * d * m) * k + 2.0 * (d - m);
        let height = ur / ((f32::max(0.0, k * ur + 1.0)).sqrt() + 1.0);

        if height > 1.0 {
          return 0;
        }

        nheights.push((height, ni));
      }

      nheights.sort_by(|(a, _), (b, _)| b.partial_cmp(a).unwrap());
      core = nheights[0].0 < -2.0;
    }

    if core {
      for &a in &self.axis {
        let d = dot(pos, a);
        let s = cross(pos, a).len();
        if d > 0.0 && s < 1.25 {
          return 0;
        }
      }
    }

    let get_minimal = |pos: Point| -> (PartIndex, f32) {
      let r = pos.len();

      let (mut shift_out, mut shift_in, inter) = if r > self.groove[0] {
        get_diag_groove(r, &self.groove)
      } else {
        get_groove(r, &self.inner_groove, 0.03)
      };

      let mut axis_pos = ArrayVec::<[(f32, Point); 10]>::default();
      let mut axis_neg = ArrayVec::<[(f32, Point); 10]>::default();

      axis_pos.clear();
      axis_neg.clear();

      let mut index: PartIndex = 0;

      let mut match_all = || -> Option<()> {
        let mut match_axis =
          |pos: Point, index: &mut PartIndex, bit: usize, axis: Point| -> Option<()> {
            let d = dot(pos, axis) / r;

            let check_in = d - shift_in;
            if check_in > 0.0 {
              *index |= (1 << bit);
              axis_pos.push((check_in, axis));
            } else {
              let check_out = shift_out - d;
              if check_out > 0.0 {
                axis_neg.push((check_out, axis));
              } else {
                return None;
              }
            }

            return Some(());
          };
        for i in 0..self.axis.len() {
          match_axis(pos, &mut index, i, self.axis[i])?;
        }

        let mut ai = 0;
        for &(i1, a) in &self.add1 {
          if index & i1 == i1 {
            match_axis(pos, &mut index, 7 + ai, a)?;
            ai += 1;
          }
        }

        for &(i2, a1, a2) in &self.add2 {
          if index & i2 == i2 {
            match_axis(pos, &mut index, 7, a1)?;
            match_axis(pos, &mut index, 8, a2)?;
          }
        }

        Some(())
      };

      if match_all().is_none() {
        return (0, -1.0);
      }

      let mut in_sr = |a, b, d| {
        let r = 0.096 * d;
        if a < r && b < r {
          return r - (sqr(r - a) + sqr(r - b)).sqrt();
        }
        return f32::INFINITY;
      };

      let petal = self.petals.iter().any(|&p| index & p == p);
      let add_petal =
        index >= (1 << self.axis.len()) && self.add1.iter().any(|&(p, _)| index & p == p);
      let petal = petal || add_petal;

      let thick: f32 =
        if petal { f32::clamp(((r - self.groove[2]).abs() - 0.1) / 3.0, 0.0, 1.0) } else { 1.0 };
      let rounded = index.count_ones() == 4;

      let mut rr = 0.3 + thick * 0.7;

      axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
      axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

      let mut minimal = f32::INFINITY;
      if axis_pos.len() >= 1 {
        minimal = f32::min(minimal, axis_pos[0].0);
      }
      if axis_neg.len() >= 1 {
        minimal = f32::min(minimal, axis_neg[0].0);
      }

      let mut k = 0;
      if axis_pos.len() >= 2 {
        let r = in_sr(axis_pos[0].0, axis_pos[1].0, rr);
        if r < minimal {
          k = 1;
          minimal = r;
        }
      }
      if axis_neg.len() >= 2 {
        let r = in_sr(axis_neg[0].0, axis_neg[1].0, rr);
        if r < minimal {
          k = 2;
          minimal = r;
        }
      }
      if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
        let mut ok = true;
        if r < self.groove[0] {
          ok = ok && dot(axis_pos[0].1, axis_neg[0].1) > 0.0;
        } else {
          if dot(axis_pos[0].1, axis_neg[0].1) < 0.0 {
            rr *= 0.3;
          }
        }
        if axis_pos.len() >= 2 && axis_pos[1].0 < 0.2 {
          if dot(axis_pos[0].1, axis_pos[1].1) < 0.0 {
            ok = ok
              && dot(axis_pos[0].1, axis_neg[0].1) > 0.0
              && dot(axis_pos[1].1, axis_neg[0].1) > 0.0;
          }
        }
        if axis_neg.len() >= 2 && axis_neg[1].0 < 0.2 {
          if dot(axis_neg[0].1, axis_neg[1].1) < 0.0 {
            if r > self.groove[0] {
              ok = ok
                && (dot(axis_pos[0].1, axis_neg[0].1) > 0.0
                  && dot(axis_pos[0].1, axis_neg[1].1) > 0.0);
            }
          }
        }

        if ok {
          let r = in_sr(axis_pos[0].0, axis_neg[0].0, rr);
          minimal = f32::min(minimal, r);
        }
      }

      if let Some(spacer_info) = &self.spacer_info {
        if index == 0 {
          if dot(pos, spacer_info.center) > r * spacer_info.cdot
            || dot(pos, spacer_info.c1) > r * spacer_info.rdot
            || dot(pos, spacer_info.c2) > r * spacer_info.rdot
          {
            index = 61
          }
        }
      }

      return (index, minimal);
    };

    let (index, minimal) = get_minimal(pos);

    if minimal < 0.0 {
      return 0;
    }

    if core {
      return 0;
      return index;
    }

    let h0 = nheights[0].0;

    if h0 < -1.0 {
      return 0;
      return index;
    }

    let nn = self.normals[nheights[0].1];
    let ndir = (pos.scale(surface_k) - nn.scale(m * surface_k - 1.0)).norm();
    let corr_pos = pos + ndir.scale(-h0);
    let (_, corr_minimal) = get_minimal(corr_pos);

    if corr_minimal > 0.022 {
      if nheights.len() > 2 && (nheights[0].0 - nheights[1].0).abs() < 0.1 {
        return 0;
      }
      if corr_minimal > 0.022 + 0.022 * h0 {
        return index + ((nheights[0].1 + 1) as PartIndex * 1024);
      }
      return 0;
    } else if corr_minimal > 0.016 {
      return 0;
    } else if h0 > 0.0 {
      return 0;
    }

    return 0;
    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.axis.len())
  }
}
