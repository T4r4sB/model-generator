use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::FxHashMap;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

#[derive(Debug, Default, Clone)]
struct TriangleInfo {
  split_is_0: bool,
  center: Point,
  split: Point,
  cut0: Vec<Point>,
  cut1: Vec<Point>,
  cut_for_petal: Vec<Point>,
}

#[derive(Debug, Default, Clone)]
struct EdgeInfo {
  split: Point,
  cut: Vec<Point>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum MatchResult {
  Close,
  Hole,
  Far,
}

pub struct ZmeyGorynychCreator {
  axis: Vec<Point>,
  normals: Vec<Point>,
  groove0: Vec<f32>,
  groove1: Vec<f32>,
  groove0_inner: Vec<f32>,
  groove1_inner: Vec<f32>,
  sz: f32,
  core_r: f32,
  centers: FxHashMap<PartIndex, TriangleInfo>,
  edges0: FxHashMap<PartIndex, Vec<EdgeInfo>>,
  edges1: FxHashMap<PartIndex, Vec<EdgeInfo>>,
  pins: Vec<Vec<Point>>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl ZmeyGorynychCreator {
  pub fn new() -> Self {
    let a = 0.48;
    let b = ((4.0 / 9.0) / (a * a + 1.0 / 3.0) - 1.0 / 3.0).sqrt();
    let sa = (1.0 - a * a).sqrt();
    let sb = (1.0 - b * b).sqrt();

    //(a*a+1/3)(b*b+1/3)=4/9

    let sq2 = 0.5.sqrt();

    let axis: Vec<_> = [
      Point { x: sa, y: 0.0, z: a },
      Point { x: 0.0, y: sb, z: b },
      Point { x: -sa, y: 0.0, z: a },
      Point { x: 0.0, y: -sb, z: b },
      Point { x: 0.0, y: sa, z: -a },
      Point { x: -sb, y: 0.0, z: -b },
      Point { x: 0.0, y: -sa, z: -a },
      Point { x: sb, y: 0.0, z: -b },
    ]
    .into_iter()
    .map(|p| Point { x: (p.x + p.y) * sq2, y: (p.x - p.y) * sq2, z: p.z })
    .collect();

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

    println!("dots top={} and {}", dot(axis[0], axis[2]), dot(axis[1], axis[3]));
    println!("dots side={} and {}", dot(axis[0], axis[4]), dot(axis[1], axis[7]));

    let maxc0 = dot(axis[0], axis[4]);
    let maxc1 = dot(axis[1], axis[3]);
    let minc0 = dot(axis[1], axis[7]);
    let minc1 = dot(axis[0], axis[2]);
    println!("maxc0={maxc0}, maxc1={maxc1}, minc0={minc0}, minc1={minc1}");

    fn conj(a: f32) -> f32 {
      (1.0 - sqr(a)).sqrt()
    }

    let base = dot(axis[0], axis[1]);
    let abase = base.acos();
    let a0 = (maxc0 - sqr(base)) / (1.0 - sqr(base));
    let a1 = (maxc1 - sqr(base)) / (1.0 - sqr(base));
    let ha0 = ((a0 + 1.0) * 0.5).sqrt();
    let ha1 = ((a1 + 1.0) * 0.5).sqrt();
    println!("angles={} and {}", ha0.acos().to_degrees(), ha1.acos().to_degrees());
    let ba = -ha0 * ha1 + conj(ha0) * conj(ha1) * base;
    let fudging_angle = ba.acos() - PI / 2.0;

    println!("ba={}, fa={}", ba.acos().to_degrees(), fudging_angle.to_degrees());
    let d0 = conj(conj(base) * conj(ha0) / conj(ba));
    let d1 = conj(conj(base) * conj(ha1) / conj(ba));
    println!("d0={}, d0={}", d0, d1);
    let hmin0 = ((minc0 + 1.0) * 0.5).sqrt();
    let hmin1 = ((minc1 + 1.0) * 0.5).sqrt();
    let hmax0 = ((maxc0 + 1.0) * 0.5).sqrt();
    let hmax1 = ((maxc1 + 1.0) * 0.5).sqrt();

    println!("hmin0={hmin0}, hmin1={hmin1}");

    let r0 = 5.0 / (hmin1.acos() - d0.acos());
    let r1 = 5.0 / (hmin0.acos() - d1.acos());

    println!("r0={r0}, r1={r1}");
    println!("r0c={}, r1c={}", r0.cos(), r1.cos());
    println!("check0={}, check1={}", d0 / hmax0, d1 / hmax1);

    let mut centers = FxHashMap::default();
    let mut edges0 = FxHashMap::<PartIndex, Vec<EdgeInfo>>::default();
    let mut edges1 = FxHashMap::<PartIndex, Vec<EdgeInfo>>::default();
    let cf0 = find_factors_for_triangle(maxc0, d0, d0);
    let cf1 = find_factors_for_triangle(maxc1, d1, d1);

    for (i0, &a0) in axis.iter().enumerate() {
      for (i1, &a1) in axis.iter().enumerate() {
        for (i2, &a2) in axis.iter().enumerate() {
          if (dot(a0, a1) - maxc0).abs() < 0.001
            && (dot(a0, a2) - base).abs() < 0.001
            && (dot(a2, a1) - base).abs() < 0.001
            && dot(a2, cross(a0, a1)) > 0.0
          {
            let center = a0.scale(cf0.0) + a1.scale(cf0.1) + cross(a0, a1).scale(cf0.2);
            centers.insert(
              1 << i0 | 1 << i1 | 1 << i2,
              TriangleInfo {
                split_is_0: false,
                center,
                split: a2.rotate(center, PI),
                cut0: vec![
                  a0.rotate(center, -fudging_angle),
                  a0.rotate(center, -2.0 * fudging_angle),
                  a1.rotate(center, fudging_angle),
                  a1.rotate(center, 2.0 * fudging_angle),
                ],
                cut1: vec![
                  a2.rotate(center, fudging_angle),
                  a2.rotate(center, -fudging_angle),
                  a2.rotate(center, PI + fudging_angle),
                  a2.rotate(center, PI - fudging_angle),
                ],
                cut_for_petal: vec![
                  a2.rotate(center, PI - 2.0 * fudging_angle),
                  a2.rotate(center, PI + 2.0 * fudging_angle),
                ],
              },
            );
            edges1.entry(1 << i0 | 1 << i2).or_default().push(EdgeInfo {
              split: a2.rotate(center, PI + 2.0 * fudging_angle),
              cut: vec![
                a1.rotate(center, 2.0 * fudging_angle),
                a1.rotate(center, 4.0 * fudging_angle),
              ],
            });
            edges1.entry(1 << i1 | 1 << i2).or_default().push(EdgeInfo {
              split: a2.rotate(center, PI - 2.0 * fudging_angle),
              cut: vec![
                a0.rotate(center, -2.0 * fudging_angle),
                a0.rotate(center, -4.0 * fudging_angle),
              ],
            });
          }

          if (dot(a0, a1) - maxc1).abs() < 0.001
            && (dot(a0, a2) - base).abs() < 0.001
            && (dot(a2, a1) - base).abs() < 0.001
            && dot(a2, cross(a0, a1)) > 0.0
          {
            let center = a0.scale(cf1.0) + a1.scale(cf1.1) + cross(a0, a1).scale(cf1.2);
            centers.insert(
              1 << i0 | 1 << i1 | 1 << i2,
              TriangleInfo {
                split_is_0: true,
                center,
                split: a2.rotate(center, PI),
                cut0: vec![
                  a2.rotate(center, fudging_angle),
                  a2.rotate(center, -fudging_angle),
                  a2.rotate(center, PI + fudging_angle),
                  a2.rotate(center, PI - fudging_angle),
                ],
                cut1: vec![
                  a0.rotate(center, -fudging_angle),
                  a0.rotate(center, -2.0 * fudging_angle),
                  a1.rotate(center, fudging_angle),
                  a1.rotate(center, 2.0 * fudging_angle),
                ],
                cut_for_petal: vec![
                  a2.rotate(center, PI - 2.0 * fudging_angle),
                  a2.rotate(center, PI + 2.0 * fudging_angle),
                ],
              },
            );
            edges0.entry(1 << i0 | 1 << i2).or_default().push(EdgeInfo {
              split: a2.rotate(center, PI + 2.0 * fudging_angle),
              cut: vec![
                a1.rotate(center, 2.0 * fudging_angle),
                a1.rotate(center, 4.0 * fudging_angle),
              ],
            });
            edges0.entry(1 << i1 | 1 << i2).or_default().push(EdgeInfo {
              split: a2.rotate(center, PI - 2.0 * fudging_angle),
              cut: vec![
                a0.rotate(center, -2.0 * fudging_angle),
                a0.rotate(center, -4.0 * fudging_angle),
              ],
            });
          }
        }
      }
    }

    let max_angle0 = hmin1.acos();
    let max_angle1 = hmin0.acos();

    let (r0, r1) = if r0 > r1 { (r0, r0 - 1.0) } else { (r1 - 1.0, r1) };
    let sz = f32::max(r0, r1) + 3.5;
    let gr = f32::min(r0, r1) - 3.0;

    let groove0 = vec![
      gr,
      (max_angle0 - 0.0 / r0).cos(),
      r0 + 0.1,
      (max_angle0 - 3.0 / r0).cos(),
      r0 + 3.2,
      (max_angle0 - 0.0 / r0).cos(),
    ];

    let groove1 = vec![
      gr,
      (max_angle1 - 0.0 / r1).cos(),
      r1 + 0.1,
      (max_angle1 - 3.0 / r1).cos(),
      r1 + 3.2,
      (max_angle1 - 0.0 / r1).cos(),
    ];

    let groove0_inner = vec![
      (abase * 0.5 + 7.0 / (gr - 6.0)).cos(),
      gr - 5.6,
      (abase * 0.5 + 4.0 / (gr - 6.0)).cos(),
      gr - 2.8,
      (max_angle0 - 0.0 / r0).cos(),
    ];
    let groove1_inner = vec![
      (abase * 0.5 + 7.0 / (gr - 6.0)).cos(),
      gr - 5.6,
      (abase * 0.5 + 4.0 / (gr - 6.0)).cos(),
      gr - 2.8,
      (max_angle1 - 0.0 / r1).cos(),
    ];

    let core_r = gr - 8.0;

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let mut pins = Vec::new();
    pins.resize_with(6, || Vec::default());
    for (a, b) in [
      (sz - 20.0, sz - 20.0),
      (sz - 6.0, sz - 14.4),
      (sz - 14.4, sz - 6.0),
      (sz - 6.0, -(sz - 6.0)),
      (sz - 10.0, -(sz - 10.0)),
      (sz - 9.0, 9.0),
      (sz - 9.0, -13.0),
      (sz - 19.0, -2.0),
      (9.0, sz - 9.0),
      (-13.0, sz - 9.0),
      (-2.0, sz - 19.0),
    ] {
      pins[0].push(Point { x: a, y: b, z: -sz });
      pins[0].push(Point { x: -a, y: -b, z: -sz });
      pins[1].push(Point { x: b, y: -a, z: sz });
      pins[1].push(Point { x: -b, y: a, z: sz });
    }

    for (a, b) in [
      (sz - 18.0, sz - 22.0),
      (sz - 10.0, sz - 8.0),
      (-(sz - 11.0), sz - 6.0),
      (-(sz - 13.0), sz - 6.0),
      (sz - 9.0, 7.0),
      (sz - 9.0, -20.0),
      (sz - 19.0, -5.0),
      (9.0, sz - 9.0),
      (-9.0, sz - 9.0),
      (0.0, sz - 19.0),
    ] {
      pins[2].push(Point { x: sz, y: a, z: b });
      pins[2].push(Point { x: sz, y: -a, z: -b });
      pins[3].push(Point { x: -sz, y: -a, z: b });
      pins[3].push(Point { x: -sz, y: a, z: -b });
      pins[4].push(Point { x: a, y: sz, z: b });
      pins[4].push(Point { x: -a, y: sz, z: -b });
      pins[5].push(Point { x: -a, y: -sz, z: b });
      pins[5].push(Point { x: a, y: -sz, z: -b });
    }

    Self {
      axis,
      normals,
      groove0,
      groove1,
      groove0_inner,
      groove1_inner,
      centers,
      edges0,
      edges1,
      sz,
      core_r,
      pins,
      axis_pos,
      axis_neg,
    }
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
    let pt = Point { x: pos.x, y: pos.y, z: self.sz - 6.0 };
    (self.get_part_index_impl(pt, current_normal) > 0) as PartIndex
  }

  pub fn get_quality() -> usize {
    384
  }

  pub fn get_size() -> f32 {
    120.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 39.999 || pos.y.abs() > 39.999 || pos.z.abs() > 39.999 {
      return 0;
    }

    //if pos.x+pos.y>0.0 { return 0; }
    //if r > self.groove0_inner[1] - 1.0 { return 0; }

    if r < self.core_r {
      if r > self.core_r - 0.2 {
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

    let mut n_dists = Vec::new();
    for i in 0..self.normals.len() {
      if i == current_normal {
        continue;
      }
      let d = self.sz - dot(pos, self.normals[i]);
      if d < 0.0 {
        return 0;
      }
      n_dists.push((i, d));
    }

    n_dists.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
    let out_r = 3.0;
    if sqr(out_r - f32::min(n_dists[0].1, out_r))
      + sqr(out_r - f32::min(n_dists[1].1, out_r))
      + sqr(out_r - f32::min(n_dists[2].1, out_r))
      > sqr(out_r)
    {
      return 0;
    }

    // if r > self.sz-3.5 || r < self.sz - 4.0 { return 0; }

    let mut index: PartIndex = 0;

    let (shift_out0, shift_in0, inter0);
    let (shift_out1, shift_in1, inter1);

    if r > self.groove0[0] {
      (shift_out0, shift_in0, inter0) = get_diag_groove(r, &self.groove0);
      (shift_out1, shift_in1, inter1) = get_diag_groove(r, &self.groove1);
    } else {
      (shift_out0, shift_in0, inter0) = get_groove(r, &self.groove0_inner, 0.03);
      (shift_out1, shift_in1, inter1) = get_groove(r, &self.groove1_inner, 0.03);
    }

    let hole_r = if r < self.core_r + 2.0 { 1.5 } else { 3.2 };

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();

    axis_pos.clear();
    axis_neg.clear();

    let mut spiral = false;

    let mut match_axis = |index: Option<(&mut PartIndex, usize)>,
                          a: Point,
                          shift_in: f32,
                          shift_out: f32|
     -> MatchResult {
      let c = dot(pos, a) / r;
      let s = cross(pos, a).len();
      let check_in = c - shift_in;
      if check_in > 0.0 {
        if let Some((index, i)) = index {
          *index |= (1 << i);
        }
        axis_pos.push((check_in, a));
        return MatchResult::Close;
      }
      let check_out = shift_out - c;
      if check_out > 0.0 {
        axis_neg.push((check_out, a));
        return MatchResult::Far;
      }
      return MatchResult::Hole;
    };

    for (i, &a) in self.axis.iter().enumerate() {
      let shift_in = if i % 2 == 0 { shift_in0 } else { shift_in1 };
      let shift_out = if i % 2 == 0 { shift_out0 } else { shift_out1 };
      if match_axis(Some((&mut index, i)), a, shift_in, shift_out) == MatchResult::Hole {
        return 0;
      }
    }

    if index == 0 {
      return 0;
    }

    if index.count_ones() == 1 {
      if n_dists[0].1 > 2.0 {
        for (i, &a) in self.axis.iter().enumerate() {
          let c = dot(pos, a) / r;
          let s = cross(pos, a).len();
          if c > 0.0 && s < hole_r {
            return 0;
          }
        }
      }
    }

    if index.count_ones() == 2 {
      let i0 = index.ilog2();
      let i1 = (index - (1 << i0)).ilog2();
      if dot(self.axis[i0 as usize], self.axis[i1 as usize]) < 0.0 {
        return 0;
      }
      if let Some(info) = self.edges0.get(&index) {
        for (i, info) in info.iter().enumerate() {
          match match_axis(Some((&mut index, i + 8)), info.split, shift_in0, shift_out0) {
            MatchResult::Hole => return 0,
            MatchResult::Close => {
              for &cut in &info.cut {
                if match_axis(None, cut, shift_in1, shift_out1) != MatchResult::Far {
                  return 0;
                }
              }
            }
            MatchResult::Far => {}
          }
        }
      }
      if let Some(info) = self.edges1.get(&index) {
        for (i, info) in info.iter().enumerate() {
          match match_axis(Some((&mut index, i + 10)), info.split, shift_in1, shift_out1) {
            MatchResult::Hole => return 0,
            MatchResult::Close => {
              for &cut in &info.cut {
                if match_axis(None, cut, shift_in0, shift_out0) != MatchResult::Far {
                  return 0;
                }
              }
            }
            MatchResult::Far => {}
          }
        }
      }
    } else if index.count_ones() == 3 {
      let info = &self.centers[&index];
      let shift_in = if info.split_is_0 { shift_in0 } else { shift_in1 };
      let shift_out = if info.split_is_0 { shift_out0 } else { shift_out1 };

      match match_axis(Some((&mut index, 8)), info.split, shift_in, shift_out) {
        MatchResult::Hole => return 0,
        MatchResult::Close => {
          for &cut0 in &info.cut0 {
            if match_axis(None, cut0, shift_in0, shift_out0) != MatchResult::Close {
              return 0;
            }
          }
          for &cut1 in &info.cut1 {
            if match_axis(None, cut1, shift_in1, shift_out1) != MatchResult::Close {
              return 0;
            }
          }
        }
        MatchResult::Far => {
          for &cp in &info.cut_for_petal {
            if match_axis(None, cp, shift_in, shift_out) != MatchResult::Far {
              return 0;
            }
          }
        }
      }
    }

    if index.count_ones() > 2 && r < self.groove0_inner[self.groove0_inner.len() - 2] + 0.2 {
      //    return 0;
    }

    axis_pos.retain(|(d, _)| *d < 0.1);
    axis_neg.retain(|(d, _)| *d < 0.1);

    let idot = -0.4;

    axis_neg.retain(|(_, p)| {
      for a in axis_pos.iter() {
        if dot(a.1, *p) < idot {
          return false;
        }
      }
      return true;
    });

    let th = if index.count_ones() >= 3 {
      f32::min(
        1.0,
        (r - (self.groove0[self.groove0.len() - 4] + self.groove1[self.groove1.len() - 4]) * 0.5)
          .abs()
          * 0.3
          + 0.3,
      )
    } else {
      1.0
    };

    let mut in_sr = |a, b, d| {
      let r = 0.1 * f32::min(0.7, 1.0 - d) * th;
      if a < r && b < r && sqr(r - a) + sqr(r - b) > sqr(r) {
        return true;
      }
      false
    };

    axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    for i1 in 0..axis_pos.len() {
      for i2 in i1 + 1..axis_pos.len() {
        let d = dot(axis_pos[i1].1, axis_pos[i2].1);
        if in_sr(axis_pos[i1].0, axis_pos[i2].0, d) {
          return 0;
        }
      }
    }
    for i1 in 0..axis_neg.len() {
      for i2 in i1 + 1..axis_neg.len() {
        let d = dot(axis_neg[i1].1, axis_neg[i2].1);
        if in_sr(axis_neg[i1].0, axis_neg[i2].0, d) {
          return 0;
        }
      }
    }
    if !inter0 && !inter1 {
      for i1 in 0..axis_pos.len() {
        for i2 in 0..axis_neg.len() {
          let d = dot(axis_pos[i1].1, axis_neg[i2].1);
          if in_sr(axis_pos[i1].0, axis_neg[i2].0, d) {
            return 0;
          }
        }
      }
    }

    let color_index = 10000 * (n_dists[0].0 as PartIndex + 1);

    if index.count_ones() < 3 {
      if n_dists[0].1 < 4.0 {
        let check_p = pos + self.normals[n_dists[0].0].scale(n_dists[0].1);
        let mut pr = n_dists[0].1;
        for &p in &self.pins[n_dists[0].0] {
          pr = f32::min(pr, (p - check_p).len());
        }

        if pr < 2.0 {
          if pr > 1.87 || n_dists[0].1 > n_dists[1].1 - 0.2 || n_dists[0].1 > 3.5 {
            return 0;
          } else {
            index += color_index;
          }
        }
      }
    } else {
      index += color_index;
    }

    return index;
  }
}
