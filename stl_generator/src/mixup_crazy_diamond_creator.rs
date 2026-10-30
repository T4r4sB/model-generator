use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::FxHashMap;
use num::complex::ComplexFloat;
use num::Float;

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum MatchResult {
  Close,
  Hole,
  Far,
}

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct CutInfo {
  axis: Vec<Point>,
  groove: Vec<f32>,
  extra_splits: FxHashMap<PartIndex, Vec<Point>>,
  extra_cuts_close: FxHashMap<PartIndex, Vec<Point>>,
  extra_cuts_far: FxHashMap<PartIndex, Vec<Point>>,
}

pub struct MixupCrazyDiamondCreator {
  cut_info0: CutInfo,
  cut_info1: CutInfo,
  normals: Vec<Point>,
  small_t_ir_sz: f32,
  holes: Vec<Point>,
  holes_shift: Vec<Point>,
  inner_splits: FxHashMap<PartIndex, Point>,
  inner_groove: Vec<f32>,
  inner_screw_dirs: Vec<Point>,
  n_dists: RefCell<Vec<(f32, usize)>>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl MixupCrazyDiamondCreator {
  pub fn new() -> Self {
    let tc = (PI * 6.0 / 7.0).cos();
    let edge = tc / (1.0 - tc);

    let hra = (2.0 * PI / 7.0).cos();
    let va = -sqr(hra) + (1.0 - sqr(hra)) * edge;
    let base_r = ((edge - va) / (1.0 - va)).sqrt();
    let small_edge = sqr(base_r) + (1.0 - sqr(base_r)) * hra;
    let small_edge_h = ((small_edge + 1.0) / 2.0).sqrt();
    let tan_r = base_r / small_edge_h;
    let tr_or = ((edge * 2.0 + 1.0) / 3.0).sqrt();
    let small_t_or = ((small_edge * 2.0 + 1.0) / 3.0).sqrt();
    let small_t_h = small_edge / small_edge_h;
    let small_t_2h = 2.0 * sqr(small_t_h) - 1.0;
    let corner_to_intersection5 =
      small_t_2h * small_t_or + (1.0 - sqr(small_t_2h)).sqrt() * (1.0 - sqr(small_t_or)).sqrt();
    let fake_edge = 1.5 * sqr(corner_to_intersection5) - 0.5;
    let small_t_ir_sz = tr_or;

    fn get_v(edge: f32) -> f32 {
      //edge=n(1,1,v)*n(1,v,1)=(2v+1)/(v^2+2)
      //2v+1=v^2*e+2e
      let a = edge;
      let p = -1.0;
      let c = 2.0 * edge - 1.0;
      (-p + (p * p - a * c).sqrt()) / a
    }
    let nv = get_v(fake_edge);
    let normals: Vec<_> = [
      Point { x: nv, y: 1.0, z: 1.0 },
      Point { x: 1.0, y: nv, z: 1.0 },
      Point { x: 1.0, y: 1.0, z: nv },
      Point { x: -nv, y: -1.0, z: -1.0 },
      Point { x: -1.0, y: -nv, z: -1.0 },
      Point { x: -1.0, y: -1.0, z: -nv },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let tan_c = tan_r.acos();
    let min_c = ((edge + 1.0) / 2.0).sqrt().acos();
    let max_c = tr_or.acos();
    let req_r = 3.0 / (tan_c - min_c);
    let r = 57.0 / 2.0 - 8.0;
    assert!(r >= req_r);

    let v = get_v(edge);
    let basic_axis: Vec<_> = [
      Point { x: v, y: 1.0, z: 1.0 },
      Point { x: 1.0, y: v, z: 1.0 },
      Point { x: 1.0, y: 1.0, z: v },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let axis = basic_axis.clone();

    println!("r={r}, edge={edge}, {}", dot(axis[0], axis[1]));

    let mut extra_splits = FxHashMap::default();
    let mut extra_cuts_close = FxHashMap::default();
    let mut extra_cuts_far = FxHashMap::default();

    for t in 0..3 {
      extra_splits.insert(
        1 << t | 1 << ((t + 1) % 3 + 3),
        vec![
          axis[t].rotate(axis[(t + 2) % 3], -2.0 * PI / 7.0),
          axis[t].rotate(axis[(t + 2) % 3], 2.0 * PI / 7.0),
          axis[t].rotate(axis[(t + 2) % 3], 4.0 * PI / 7.0),
        ],
      );
      extra_splits.insert(
        1 << t | 1 << ((t + 2) % 3 + 3),
        vec![
          axis[t].rotate(axis[(t + 1) % 3], 2.0 * PI / 7.0),
          axis[t].rotate(axis[(t + 1) % 3], -2.0 * PI / 7.0),
          axis[t].rotate(axis[(t + 1) % 3], -4.0 * PI / 7.0),
        ],
      );
    }

    extra_cuts_close.insert(
      15,
      vec![
        axis[1].rotate(axis[0], PI * 2.0 / 7.0),
        axis[2].rotate(axis[1], PI * 2.0 / 7.0),
        axis[0].rotate(axis[2], PI * 2.0 / 7.0),
      ],
    );

    let groove = vec![
      tan_c.cos(),
      r - 1.8,
      (max_c - 2.0 / (r - 2.0)).cos(),
      r + 0.2,
      (tan_c + 1.5 / r).cos(),
      r + 2.2,
      (tan_c - 1.5 / r).cos(),
      r + 4.2,
      (tan_c + 1.5 / r).cos(),
      r + 6.2,
      tan_c.cos(),
    ];

    extra_cuts_far.insert(31, vec![axis[0], axis[1], axis[2]]);
    let mut cut_info0 = CutInfo { axis, groove, extra_splits, extra_cuts_close, extra_cuts_far };

    let axis: Vec<_> = basic_axis.iter().map(|&p| -p).collect();

    let mut extra_splits = FxHashMap::default();
    let mut extra_cuts_close = FxHashMap::default();
    let mut extra_cuts_far = FxHashMap::default();

    extra_cuts_close.insert(
      31,
      vec![
        axis[0].rotate(axis[1], PI * 2.0 / 7.0),
        axis[1].rotate(axis[2], PI * 2.0 / 7.0),
        axis[2].rotate(axis[0], PI * 2.0 / 7.0),
      ],
    );

    let groove = vec![
      tan_c.cos(),
      r - 1.8,
      (max_c - 2.0 / (r - 2.0)).cos(),
      r + 0.2,
      (tan_c - 1.5 / r).cos(),
      r + 2.2,
      (tan_c + 1.5 / r).cos(),
      r + 4.2,
      (tan_c - 1.5 / r).cos(),
      r + 6.2,
      tan_c.cos(),
    ];

    extra_cuts_far.insert(15, vec![axis[0], axis[1], axis[2]]);
    let mut cut_info1 = CutInfo { axis, groove, extra_splits, extra_cuts_close, extra_cuts_far };

    cut_info1.extra_cuts_close.entry(15).or_default().extend(&[
      cut_info1.axis[1].rotate(basic_axis[0], -PI * 2.0 / 7.0),
      cut_info1.axis[0].rotate(basic_axis[1], PI * 2.0 / 7.0),
      cut_info1.axis[2].rotate(basic_axis[1], -PI * 2.0 / 7.0),
      cut_info1.axis[1].rotate(basic_axis[2], PI * 2.0 / 7.0),
      cut_info1.axis[0].rotate(basic_axis[2], -PI * 2.0 / 7.0),
      cut_info1.axis[2].rotate(basic_axis[0], PI * 2.0 / 7.0),
    ]);
    cut_info0.extra_cuts_close.entry(31).or_default().extend(&[
      cut_info0.axis[1].rotate(basic_axis[0], -PI * 2.0 / 7.0),
      cut_info0.axis[0].rotate(basic_axis[1], PI * 2.0 / 7.0),
      cut_info0.axis[2].rotate(basic_axis[1], -PI * 2.0 / 7.0),
      cut_info0.axis[1].rotate(basic_axis[2], PI * 2.0 / 7.0),
      cut_info0.axis[0].rotate(basic_axis[2], -PI * 2.0 / 7.0),
      cut_info0.axis[2].rotate(basic_axis[0], PI * 2.0 / 7.0),
    ]);

    for t in 0..3 {
      let t0 = t;
      let t1 = (t + 1) % 3;
      let t2 = (t + 2) % 3;

      cut_info1.extra_cuts_close.insert(
        1 << (t0 + 3) | 1 << t1 | 1 << t2,
        vec![cut_info1.axis[t0].rotate(basic_axis[t1], -PI * 2.0 / 7.0)],
      );
      cut_info0.extra_cuts_close.insert(
        1 << t0 | 1 << (t1 + 3) | 1 << (t2 + 3),
        vec![cut_info0.axis[t0].rotate(basic_axis[t1], -PI * 2.0 / 7.0)],
      );
    }

    let mut reuse_cuts = |src: PartIndex, dst: PartIndex, center: usize, angle: f32| {
      for info in [&mut cut_info0, &mut cut_info1] {
        for ec in [&mut info.extra_cuts_close, &mut info.extra_cuts_far] {
          if let Some(src) = ec.get(&src) {
            ec.insert(dst, src.iter().map(|p| p.rotate(basic_axis[center], angle)).collect());
          }
        }
      }
    };

    for t in 0..3 {
      let t0 = t;
      let t1 = (t + 1) % 3;
      let t2 = (t + 2) % 3;

      reuse_cuts(15, 1 << t0, t1, PI * 6.0 / 7.0);
      reuse_cuts(15, 1 << t0 | 1 << (t1 + 3) | 64 | 128, t2, -PI * 4.0 / 7.0);
      reuse_cuts(15, 1 << t0 | 1 << (t1 + 3) | 128 | 256, t2, -PI * 2.0 / 7.0);
      reuse_cuts(15, 1 << t0 | 1 << (t2 + 3) | 64 | 128, t1, PI * 4.0 / 7.0);
      reuse_cuts(15, 1 << t0 | 1 << (t2 + 3) | 128 | 256, t1, PI * 2.0 / 7.0);

      reuse_cuts(31, 1 << (t0 + 3), t2, -PI * 6.0 / 7.0);
      reuse_cuts(31, 1 << t0 | 1 << (t1 + 3) | 64, t2, PI * 2.0 / 7.0);
      reuse_cuts(31, 1 << t0 | 1 << (t1 + 3) | 128, t2, PI * 4.0 / 7.0);
      reuse_cuts(31, 1 << t0 | 1 << (t2 + 3) | 64, t1, -PI * 2.0 / 7.0);
      reuse_cuts(31, 1 << t0 | 1 << (t2 + 3) | 128, t1, -PI * 4.0 / 7.0);
    }

    let mut inner_splits = FxHashMap::default();
    let a0r = cut_info0.axis[2].rotate(cut_info0.axis[1], PI * 6.0 / 7.0);
    let a1r = cut_info0.axis[0].rotate(cut_info0.axis[2], PI * 6.0 / 7.0);
    let a2r = cut_info0.axis[1].rotate(cut_info0.axis[0], PI * 6.0 / 7.0);
    inner_splits.insert(3, a2r);
    inner_splits.insert(6, a0r);
    inner_splits.insert(5, a1r);
    inner_splits.insert(4, a2r);
    inner_splits.insert(1, a0r);
    inner_splits.insert(2, a1r);

    let mut holes = vec![
      (basic_axis[0] + a0r).norm(),
      (basic_axis[1] + a1r).norm(),
      (basic_axis[2] + a2r).norm(),
    ];

    let a0 = max_c;
    let a1 = PI - max_c;

    let hedge = dot(cut_info0.axis[0], holes[1]);
    let inner_groove = vec![
      (PI - tr_or.acos() + 0.2 / (r - 8.0)).cos(),
      r - 7.8,
      (tr_or.acos() + 1.0 / (r - 8.0)).cos(),
      r - 5.8,
      (PI - tr_or.acos() + 0.2 / (r - 8.0)).cos(),
      r - 3.8,
      0.0f32.cos(),
    ];

    let n_dists = RefCell::new(Vec::new());
    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let corner = Point { x: 1.0, y: 1.0, z: 1.0 };

    let mut holes_shift = Vec::new();
    for &h in &holes {
      holes_shift.push(cross(h, cross(h, corner)).norm());
    }

    let mut inner_screw_dirs = Vec::new();

    for i in 0..3 {
      let a = PI * 2.0 * i as f32 / 3.0;

      for i in -20..20 {
        for j in 0..2 {
          let h = (1.0 - sqr(i as f32 / 20.0)) * 0.15;
          let h = if j == 0 { h } else { -h };
          let (s, c) = (a + h).sin_cos();
          let v = Point { x: c, y: s, z: 0.0 };
          let (ds, dc) = (i as f32 * 0.02).sin_cos();
          let dir = v.scale(dc) + Point::Z.scale(ds);
          inner_screw_dirs.push(dir);
        }
      }
    }

    Self {
      normals,
      small_t_ir_sz,
      inner_splits,
      inner_groove,
      holes,
      holes_shift,
      inner_screw_dirs,
      cut_info0,
      cut_info1,
      n_dists,
      axis_pos,
      axis_neg,
    }
  }

  pub fn faces(&self) -> usize {
    5
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
    match current_normal {
      0 => self.get_part_index(Point { x: pos.x, y: pos.y, z: 0.0 }),
      1 => {
        let sq2 = 0.5f32.sqrt();
        self.get_part_index(Point { x: pos.x * sq2, y: pos.x * sq2, z: pos.y })
      }
      2 => self.get_part_index(Point { x: pos.x, y: 0.0, z: pos.y }),
      3 => {
        let v1 = self.holes[0];
        let v2 = self.cut_info0.axis[1];
        let v2 = cross(v1, cross(v1, v2)).norm();
        self.get_part_index(v1.scale(pos.x) + v2.scale(pos.y))
      }
      4 => {
        let v1 = self.holes[0];
        let v2 = self.cut_info0.axis[1];
        let v2 = cross(v1, v2).norm();
        self.get_part_index(v1.scale(pos.x) + v2.scale(pos.y))
      }
      _ => 0,
    }
  }

  pub fn get_quality() -> usize {
    184
  }

  pub fn get_size() -> f32 {
    100.0
  }

  fn get_part_index_impl(&self, pos: Point, current_normal: usize, side_check: bool) -> PartIndex {
    let index = self.inner_part(pos, current_normal, side_check);
    if index != 0 {
      return index;
    }

    // let index = self.outer_part(pos, current_normal, side_check);
    if index != 0 {
      return index;
    }
    0
  }

  fn inner_part(&self, pos: Point, current_normal: usize, side_check: bool) -> PartIndex {
    let rxy = (sqr(pos.x) + sqr(pos.y)).sqrt();
    if pos.z > 40.0 && pos.z < 44.0 && rxy > 1.25 && rxy < 2.8 {
      if pos.x > 2.3 {
        return 0;
      }
      return 127;
    }

    // return 0;

    let r = pos.len();
    if r > self.cut_info0.groove[1] - 2.2 {
      return 0;
    }
    let ir = self.inner_groove[1] - 2.0;
    if r < ir {
      // return 0; // tmp
      if r > ir - 0.2 {
        return 0;
      }

      if r < ir - 3.7 {
        if r < ir - 3.9 && r > ir - 5.9 {
          if sqr(pos.y) + sqr(pos.z) < sqr(1.5) {
            return 0;
          }

          if pos.x > r * 0.7 {
            return 130;
          }
        }

        return 0;
      }

      for i in 0..3 {
        let a = PI * 2.0 * i as f32 / 3.0;
        let (s, c) = a.sin_cos();
        let v = Point { x: c, y: s, z: 0.0 };
        if dot(pos, v) < 0.0 && cross(pos, v).len() < 1.25 {
          return 0;
        }
      }

      for &dir in &self.inner_screw_dirs {
        let d = dot(pos, dir);
        let c = cross(pos, dir).sqr_len();
        let hr = 1.5; //(5.0 - d).clamp(0.0, 1.1) + 1.5;

        if d > 0.0 && c < sqr(hr) {
          return 0;
        }
      }

      let r = if pos.z > ir - 2.2 {
        2.6
      } else if pos.z > 0.0 {
        1.5
      } else {
        1.2
      };
      if rxy < r {
        return 0;
      }

      let d1 = pos.x.abs();
      let d2 = (-pos.x * 0.5 + pos.y * 0.75f32.sqrt()).abs();
      let d3 = (-pos.x * 0.5 - pos.y * 0.75f32.sqrt()).abs();

      let m = f32::max(1.0 - f32::min(d1, f32::min(d2, d3)), 0.0);

      if pos.z + m > 0.11 {
        return 128;
      } else if pos.z + m < -0.11 {
        return 129;
      }
      return 0;
    }

    let mut index: PartIndex = 0;
    let a = &self.cut_info0.axis;

    let max_r = self.cut_info0.groove[1] - 2.2;
    let small_t_ir_sz = self.small_t_ir_sz;
    let depth = small_t_ir_sz * max_r;

    let mut axis_pos = self.axis_pos.borrow_mut();
    let axis_pos = axis_pos.deref_mut();
    axis_pos.clear();
    let mut axis_neg = self.axis_neg.borrow_mut();
    let axis_neg = axis_neg.deref_mut();
    axis_neg.clear();
    let mut g = get_groove(r, &self.inner_groove, 0.03);
    /*
    let dt = r / (self.inner_groove[1] + 0.2) * 1.0 - 0.0;
    if dt < 1.0 {
      g.0 *= dt;
      g.1 *= dt;
    }*/

    let mut match_axis = |a: Point, shift_in: f32, shift_out: f32| -> MatchResult {
      let c = dot(pos, a) / r;
      let check_in = c - shift_in;
      if check_in > 0.0 {
        axis_pos.push((check_in, a));
        return MatchResult::Close;
      } else {
        let check_out = shift_out - c;
        if check_out > 0.0 {
          axis_neg.push((check_out, a));
          return MatchResult::Far;
        } else {
          return MatchResult::Hole;
        }
      }
    };

    for (i, &a) in self.cut_info0.axis.iter().enumerate() {
      match match_axis(a, g.1, g.0) {
        MatchResult::Close => index += 1 << i,
        MatchResult::Far => {}
        MatchResult::Hole => {
          return 0;
        }
      }
    }

    if index.count_ones() != 1 && r < ir + 0.2 {
      return 0;
    }

    if r > self.cut_info0.groove[1] - 4.8 {
      if index.count_ones() == 2 {
        let a1 = index.ilog2() as usize;
        let a2 = (index - (1 << a1)).ilog2() as usize;
        let a3 = 3 - a1 - a2;
        if cross(pos, self.holes[a3]).sqr_len() > sqr(3.0) {
          if dot(pos, a[a1]) > self.cut_info0.groove[0] * r - 0.27
            && dot(pos, a[a2]) > self.cut_info0.groove[0] * r - 0.27
          {
            return 0;
          }
        }
      }
      if index.count_ones() == 1 {
        let a3 = index.ilog2() as usize;
        let a1 = (a3 + 1) % 3;
        let a2 = (a1 + 1) % 3;
        if cross(pos, self.holes[a3]).sqr_len() > sqr(3.0) {
          if dot(pos, a[a1]) < -self.cut_info0.groove[0] * r + 0.27
            && dot(pos, a[a2]) < -self.cut_info0.groove[0] * r + 0.27
          {
            return 0;
          }
        }
      }
    }

    if let Some(&s) = self.inner_splits.get(&index) {
      match match_axis(s, g.1, g.0) {
        MatchResult::Close => {}
        MatchResult::Far => index += 1 << 3,
        MatchResult::Hole => {
          return 0;
        }
      }
    }
    let mut hole_r = 1.5;
    let mut hole_rn = 1.2;
    for &h in &self.holes {
      let hole_r = if dot(pos, h) < 0.0 { hole_rn } else { hole_r };
      if cross(pos, h).len() < hole_r {
        return 0;
      }
    }
    let mut rr = 0.1f32;

    if r < self.inner_groove[1] + 0.2
      || r > self.inner_groove[3] - 0.2 && r < self.inner_groove[5] + 0.2
    {
      if index == 0 || index == 9 || index == 10 || index == 12 {
        rr = 0.05f32;
      }
    }

    if r > self.inner_groove[1] - 0.2 && r < self.inner_groove[3] + 0.2
      || r > self.inner_groove[5] - 0.2
    {
      if index == 3 || index == 5 || index == 6 || index == 7 {
        rr = 0.05f32;
      }
    }

    let mut in_sr = |a, b, d, sqrd: &mut f32| {
      let r = rr * d;
      if a < r && b < r {
        *sqrd = f32::min(*sqrd, r - (sqr(r - a) + sqr(r - b)).sqrt());
      }
    };

    let mut sqrd = f32::INFINITY;
    axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    for (i, a1) in axis_pos.iter().enumerate() {
      sqrd = f32::min(sqrd, a1.0);
      for a2 in &axis_pos[i + 1..] {
        in_sr(a1.0, a2.0, 1.0, &mut sqrd);
        if sqrd < 0.0 {
          return 0;
        }
      }
    }
    for (i, a1) in axis_neg.iter().enumerate() {
      sqrd = f32::min(sqrd, a1.0);
      for a2 in &axis_neg[i + 1..] {
        in_sr(a1.0, a2.0, 1.0, &mut sqrd);
        if sqrd < 0.0 {
          return 0;
        }
      }
    }
    if !g.2 && index.count_ones() != 2 {
      for a1 in &*axis_pos {
        for a2 in &*axis_neg {
          in_sr(a1.0, a2.0, 1.0, &mut sqrd);
          if sqrd < 0.0 {
            return 0;
          }
        }
      }
    }
    index += 200;
    index
  }

  fn outer_part(&self, pos: Point, current_normal: usize, side_check: bool) -> PartIndex {
    let r = pos.len();

    if r < self.cut_info0.groove[1] - 4.2 {
      return 0;
    }
    if r < self.cut_info0.groove[1] - 1.8 {
      for &h in &self.holes {
        if cross(pos, h).len() < 3.4 {
          return 0;
        }
      }
    }

    let last_groove = self.cut_info0.groove[self.cut_info0.groove.len() - 2];
    let sz = last_groove + 1.8;

    let mut axis_pos = self.axis_pos.borrow_mut();
    let axis_pos = axis_pos.deref_mut();
    axis_pos.clear();
    let mut axis_neg = self.axis_neg.borrow_mut();
    let axis_neg = axis_neg.deref_mut();
    axis_neg.clear();
    let mut n_dists = self.n_dists.borrow_mut();
    let n_dists = n_dists.deref_mut();
    n_dists.clear();

    for (i, &n) in self.normals.iter().enumerate() {
      n_dists.push((sz - dot(pos, n), i));
    }
    n_dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    if n_dists[0].0 < 0.0 {
      return 0;
    }

    let g0 = get_groove(r, &self.cut_info0.groove, 0.03);
    let g1 = get_groove(r, &self.cut_info1.groove, 0.03);

    let mut match_axis = |a: Point, shift_in: f32, shift_out: f32| -> MatchResult {
      let c = dot(pos, a) / r;
      let check_in = c - shift_in;
      if check_in > 0.0 {
        axis_pos.push((check_in, a));
        return MatchResult::Close;
      } else {
        let check_out = shift_out - c;
        if check_out > 0.0 {
          axis_neg.push((check_out, a));
          return MatchResult::Far;
        } else {
          return MatchResult::Hole;
        }
      }
    };

    let mut index = 0;
    /*

    for (i, &a) in self.cut_info1.extra_cuts_far[&1].iter().enumerate() {
      match match_axis(a, g0.1, g0.0) {
        MatchResult::Close => index += 1 << (i +0),
        MatchResult::Far => {}
        MatchResult::Hole => {
          return 0;
        }
      }
    }*/

    for (c, p, g) in [(&self.cut_info0, 0, &g0), (&self.cut_info1, 3, &g1)] {
      for (i, &a) in c.axis.iter().enumerate() {
        match match_axis(a, g.1, g.0) {
          MatchResult::Close => index += 1 << (i + p),
          MatchResult::Far => {}
          MatchResult::Hole => {
            return 0;
          }
        }
      }
    }

    if index == 0 {
      if pos.x + pos.y + pos.z > 0.0 {
        index = 15
      } else {
        index = 31
      }
    }

    for (c, g) in [(&self.cut_info0, &g0), (&self.cut_info1, &g1)] {
      if let Some(ec) = c.extra_splits.get(&index) {
        for (i, &ec) in ec.iter().enumerate() {
          match match_axis(ec, g.1, g.0) {
            MatchResult::Close => index += 1 << (i + 6),
            MatchResult::Far => {}
            MatchResult::Hole => {
              return 0;
            }
          }
        }
      }
    }

    let mut has_cut = false;
    for (c, g) in [(&self.cut_info0, &g0), (&self.cut_info1, &g1)] {
      if c.extra_cuts_close.contains_key(&index) || c.extra_cuts_far.contains_key(&index) {
        has_cut = true;
        break;
      }
    }
    if !has_cut {
      return 0;
    }
    let mut is_center = (index & 63).count_ones() == 3;
    if !is_center && r < self.cut_info0.groove[1] - 1.8 {
      return 0;
    }

    for (c, g) in [(&self.cut_info0, &g0), (&self.cut_info1, &g1)] {
      if let Some(ec) = c.extra_cuts_close.get(&index) {
        for &ec in ec {
          if match_axis(ec, g.1, g.0) != MatchResult::Close {
            return 0;
          }
        }
      }
      if let Some(ec) = c.extra_cuts_far.get(&index) {
        for &ec in ec {
          if match_axis(ec, g.1, g.0) != MatchResult::Far {
            return 0;
          }
        }
      }
    }

    let mut rr = 0.1f32;
    let mut in_sr = |a, b, d, sqrd: &mut f32| {
      let r = rr * d;
      if a < r && b < r {
        *sqrd = f32::min(*sqrd, r - (sqr(r - a) + sqr(r - b)).sqrt());
      }
    };
    let mut sqrd = f32::INFINITY;
    axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    for (i, a1) in axis_pos.iter().enumerate() {
      sqrd = f32::min(sqrd, a1.0);
      for a2 in &axis_pos[i + 1..] {
        if dot(a1.1, a2.1) < 0.6 {
          in_sr(a1.0, a2.0, 1.0, &mut sqrd);
          if sqrd < 0.0 {
            return 0;
          }
        }
      }
    }
    for (i, a1) in axis_neg.iter().enumerate() {
      sqrd = f32::min(sqrd, a1.0);
      for a2 in &axis_neg[i + 1..] {
        if dot(a1.1, a2.1) < 0.6 {
          in_sr(a1.0, a2.0, 1.0, &mut sqrd);
          if sqrd < 0.0 {
            return 0;
          }
        }
      }
    }
    for a1 in &*axis_pos {
      sqrd = f32::min(sqrd, a1.0);
      for a2 in &*axis_neg {
        /*
        in_sr(a1.0, a2.0, 1.0, &mut sqrd);
        if sqrd < 0.0 {
          return 0;
        }*/
      }
    }

    let mut nd = 0.0;
    if is_center || index == 31 || index == 15 {
      let out_r = 3.0;
      let dst_to_face = out_r
        - (sqr(out_r - f32::min(n_dists[0].0, out_r))
          + sqr(out_r - f32::min(n_dists[1].0, out_r))
          + sqr(out_r - f32::min(n_dists[2].0, out_r)))
        .sqrt();
      if dst_to_face < 0.0 {
        return 0;
      }

      if n_dists[0].0 < 1.0 || n_dists[0].0 + n_dists[1].0 < 6.0 {
        if n_dists[0].0 + 0.15 > n_dists[1].0 {
          return 0;
        }
        if n_dists[0].0 < 0.8 || n_dists[0].0 + n_dists[1].0 < 5.7 {
          index += (n_dists[0].1 as PartIndex + 1) * 100000;
          is_center = false;
        } else {
          return 0;
        }
      }
      nd = n_dists[0].0;
    } else {
      let mut mid = Point::ZERO;
      for p in axis_pos {
        mid += p.1
      }
      let mut bn = self.normals.len();
      let mut bd = -1.0;
      for (i, &n) in self.normals.iter().enumerate() {
        let d = dot(n, mid);
        if d > bd {
          bd = d;
          bn = i;
        }
      }
      nd = sz - dot(pos, self.normals[bn]);
      index += (bn as PartIndex + 1) * 100000;
    }

    sqrd *= 100.0;
    let frr = 3.0;
    if nd < frr && sqrd < frr && sqr(frr - nd) + sqr(frr - sqrd) > sqr(frr) {
      return 0;
    }

    if is_center {
      let bhr = self.cut_info0.groove[self.cut_info0.groove.len() - 2] + 1.5;
      for &h in &self.holes {
        let d = dot(pos, h).abs();

        let hr = if d < bhr { 1.5 } else { 3.5 };
        if cross(pos, h).len() < hr {
          return 0;
        }
      }

      if r > bhr + 1.0 {
        let mut hd = f32::INFINITY;
        let center_kind = (index & 7).count_ones() == 1;
        let cf = if center_kind { 1.0 } else { -1.0 };
        for (&h, &s) in self.holes.iter().zip(&self.holes_shift) {
          hd = f32::min(hd, cross(pos - s.scale(7.0 * cf), h).len() - 2.0);
          hd = f32::min(hd, cross(pos + s.scale(7.0 * cf), h).len() - 2.0);
          hd = f32::min(hd, cross(pos + s.scale(8.5 * cf), h).len() - 1.75);
          hd = f32::min(hd, cross(pos + s.scale(10.0 * cf), h).len() - 1.5);
          hd = f32::min(hd, cross(pos + s.scale(11.5 * cf), h).len() - 1.25);
          hd = f32::min(hd, cross(pos + s.scale(13.0 * cf), h).len() - 1.0);
        }
        if hd > 0.13 && r > bhr + 1.2 || r > bhr + 4.0 {
          index += 400;
        } else if hd > 0.0 || r > bhr + 3.5 {
          return 0;
        }
      }
    }

    index
  }
}
