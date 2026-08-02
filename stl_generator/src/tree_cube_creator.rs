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

type Traj = Vec<Vec<u8>>;

#[derive(Copy, Clone, Debug)]
pub enum SubGroup {
  Group(usize),
  Piece(PartIndex),
}

#[derive(Copy, Clone, Debug)]
struct Group {
  splitter: (Point, usize),
  lhs: SubGroup,
  rhs: SubGroup,
}

pub struct TreeCreator {
  axis: Vec<(Point, f32)>,
  splits: Vec<Group>,
  split_start: SubGroup,
  normals: Vec<Point>,
  groove: [Vec<f32>; 3],
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
}

fn sqr(x: f32) -> f32 {
  x * x
}

fn reflect(p: Point, a: Point) -> Point {
  a.scale(dot(p, a) * 2.0) - p
}

fn find_ec_dy_d(d1: f32, d2: f32, d3: f32) -> f32 {
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

fn find_ec(a: Point, b: Point, c: Point) -> f32 {
  let d1 = dot(a, b);
  let d2 = dot(a, c);
  let d3 = dot(b, c);
  find_ec_dy_d(d1, d2, d3)
}

impl TreeCreator {
  fn in_a(&self, p: Point, a: usize) -> bool {
    dot(p, self.axis[a].0) > self.axis[a].1
  }

  fn get_traj_l(&self, p: Point, last: usize, res: &mut Vec<u8>) {
    if res.len() > 20 {
      return;
    }
    let mut handled = false;
    for i in 0..3 {
      if i == last {
        continue;
      }
      if self.in_a(p, i) {
        if handled {
          println!("warning, point falled to triangle!");
        } else {
          res.push(i as u8);
          self.get_traj_l(reflect(p, self.axis[i].0), i, res);
          handled = true;
        }
      }
    }
  }

  fn in_t(&self, p: Point) -> bool {
    self.in_a(p, 0) && self.in_a(p, 1) && self.in_a(p, 2)
  }

  fn get_p7() -> Traj {
    vec![vec![0], vec![1], vec![2]]
  }
  fn get_p01() -> Traj {
    vec![vec![0, 1, 8], vec![], vec![]]
  }
  fn get_p12() -> Traj {
    vec![vec![], vec![1, 2, 9], vec![]]
  }
  fn get_p20() -> Traj {
    vec![vec![], vec![], vec![2, 0, 4]]
  }

  fn get_traj(&self, p: Point) -> (Traj, PartIndex) {
    if self.in_t(p) {
      return (vec![], 7);
    } else if self.in_a(p, 0) && self.in_t(reflect(p, self.axis[0].0)) {
      return (vec![], 7);
    } else if self.in_a(p, 1) && self.in_t(reflect(p, self.axis[1].0)) {
      return (vec![], 7);
    } else if self.in_a(p, 2) && self.in_t(reflect(p, self.axis[2].0)) {
      return (vec![], 7);
    } else {
      let mut gr = 0;
      let mut result = Traj::new();
      for i in 0..3 {
        let mut ra = Vec::new();
        if self.in_a(p, i) {
          gr += 1 << i;
          ra.push(i as u8);
          self.get_traj_l(reflect(p, self.axis[i].0), i, &mut ra);
        }
        result.push(ra);
      }

      return (result, gr);
    }
  }

  fn split(&mut self, traj_buf: &[Traj], cur: &[usize]) -> SubGroup {
    println!("split set {cur:?}");
    if cur.len() == 0 {
      panic!("empty group!");
    } else if cur.len() == 1 {
      return SubGroup::Piece(cur[0] as PartIndex);
    }
    for j in 0..3 {
      let mut key = Vec::new();
      loop {
        let mut new_el = u8::MAX;
        for &i in cur {
          let t = &traj_buf[i][j];
          let t_el = t.get(key.len()).copied().unwrap_or(u8::MAX - 1);
          if new_el < u8::MAX {
            if new_el != t_el {
              let mut key_split = std::cmp::min(new_el, t_el) as usize;
              let mut splitter;
              if key_split > self.axis.len() {
                let ai = key_split % 3;
                let a = self.axis[key_split % 3];
                splitter = (a.0, ai);
                splitter.0 = reflect(splitter.0, self.axis[key_split / 3 - 1].0);
              } else {
                let ai = key_split;
                let a = self.axis[key_split];
                splitter = (a.0, ai);
              }
              for &key in key.iter().rev() {
                splitter.0 = reflect(splitter.0, self.axis[key as usize].0);
              }
              println!("split by {:?}", splitter.0);
              let mut left = Vec::new();
              let mut right = Vec::new();
              for &c in cur {
                let t = &traj_buf[c][j];
                let t_el = t.get(key.len()).copied().unwrap_or(u8::MAX - 1);
                if key_split as u8 == t_el {
                  left.push(c);
                } else {
                  right.push(c);
                }
              }
              let lhs = self.split(traj_buf, &left);
              let rhs = self.split(traj_buf, &right);
              let ng = self.splits.len();
              self.splits.push(Group { splitter, lhs, rhs });
              return SubGroup::Group(ng);
              // here create split
            }
          } else {
            new_el = t_el;
          }
        }

        if new_el >= u8::MAX - 1 {
          break;
        }

        key.push(new_el);
      }
    }
    panic!("Unable to split");
  }

  fn print_split(&self, start: SubGroup, indent: usize) {
    match start {
      SubGroup::Group(g) => {
        let g = &self.splits[g];
        println!("{:indent$}Split {:?}", "", g.splitter);
        if let SubGroup::Piece(p) = g.lhs {
          println!("{:indent$}LHS: Piece #{p}", "");
        } else {
          println!("{:indent$}LHS: {{", "");
          self.print_split(g.lhs, indent + 4);
          println!("{:indent$}}}", "");
        }
        if let SubGroup::Piece(p) = g.rhs {
          println!("{:indent$}RHS: Piece #{p}", "");
        } else {
          println!("{:indent$}RHS: {{", "");
          self.print_split(g.rhs, indent + 4);
          println!("{:indent$}}}", "");
        }
      }
      SubGroup::Piece(p) => println!("{:indent$}Piece #{p}", ""),
    }
  }

  pub fn new() -> Self {
    let c1 = (80.0 * PI / 180.0).cos();
    let c2 = (90.0 * PI / 180.0).cos();
    let c3 = (100.0 * PI / 180.0).cos();
    let d1 = (c1 + c2 * c3) / (1.0 - sqr(c2)).sqrt() / (1.0 - sqr(c3)).sqrt();
    let d2 = (c2 + c1 * c3) / (1.0 - sqr(c1)).sqrt() / (1.0 - sqr(c3)).sqrt();
    let d3 = (c3 + c1 * c2) / (1.0 - sqr(c1)).sqrt() / (1.0 - sqr(c2)).sqrt();

    let ca = find_ec_dy_d(d1, d2, d3);
    let sa = (1.0 - sqr(ca)).sqrt();

    let caa1 = (d1 - sqr(ca)) / (1.0 - sqr(ca));
    let caa2 = (d2 - sqr(ca)) / (1.0 - sqr(ca));
    let caa3 = (d3 - sqr(ca)) / (1.0 - sqr(ca));

    let corner = Point { x: 1.0, y: 1.0, z: 1.0 }.norm();
    // a=c,s,s d*sq3=c+s+s
    // cc+4ss+4cs=1+3ss+4cs
    // d*d2 =

    let a0 = corner.rotate(Point { x: 1.0, y: -1.0, z: 0.0 }.norm(), ca.acos());
    let a1 = a0.rotate(corner, caa1.acos());
    let a2 = a0.rotate(corner, -caa2.acos());

    let main_dot = [0.2, 0.2, 0.2];

    let mut axis = vec![(a0, main_dot[0]), (a1, main_dot[1]), (a2, main_dot[2])];
    let normals = vec![Point::X, Point::Y, Point::Z, -Point::X, -Point::Y, -Point::Z];
    let mut splits = Vec::new();
    let split_start = SubGroup::Piece(0);

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let sphere_r = 25.0;
    let maximal_angle = [main_dot[0].acos(), main_dot[1].acos(), main_dot[2].acos()];

    let groove = [
      vec![
        (maximal_angle[0] + 0.0 / sphere_r).cos(),
        sphere_r + 0.2,
        (maximal_angle[0] - 2.5 / sphere_r).cos(),
        sphere_r + 2.6,
        (maximal_angle[0] + 0.0 / sphere_r).cos(),
      ],
      vec![
        (maximal_angle[1] + 0.0 / sphere_r).cos(),
        sphere_r + 0.2,
        (maximal_angle[1] - 2.5 / sphere_r).cos(),
        sphere_r + 2.6,
        (maximal_angle[1] + 0.0 / sphere_r).cos(),
      ],
      vec![
        (maximal_angle[2] + 0.0 / sphere_r).cos(),
        sphere_r + 0.2,
        (maximal_angle[2] - 2.5 / sphere_r).cos(),
        sphere_r + 2.6,
        (maximal_angle[2] + 0.0 / sphere_r).cos(),
      ],
    ];

    let mut result = Self { axis, splits, split_start, normals, groove, axis_pos, axis_neg };
    let mut values = FxHashSet::<Traj>::default();
    let mut trajs = Vec::<Traj>::new();

    trajs.push(vec![vec![], vec![], vec![]]);
    trajs.push(vec![vec![0], vec![], vec![]]);
    trajs.push(vec![vec![], vec![1], vec![]]);
    trajs.push(vec![vec![0], vec![1], vec![]]);
    trajs.push(vec![vec![], vec![], vec![2]]);
    trajs.push(vec![vec![0], vec![], vec![2]]);
    trajs.push(vec![vec![], vec![1], vec![2]]);
    trajs.push(Self::get_p7());
    trajs.push(Self::get_p01());
    trajs.push(Self::get_p12());
    trajs.push(Self::get_p20());
    for (g, t) in trajs.iter().enumerate() {
      values.insert(t.clone());
    }

    for i in 0..90 {
      for j in 0..90 {
        let u = i as f32 / 45.0 - 1.0;
        let v = j as f32 / 45.0 - 1.0;
        let r = [
          result.get_traj(Point { x: u, y: v, z: 1.0 }.norm()),
          result.get_traj(Point { x: 1.0, y: u, z: v }.norm()),
          result.get_traj(Point { x: v, y: 1.0, z: u }.norm()),
          result.get_traj(Point { x: u, y: v, z: -1.0 }.norm()),
          result.get_traj(Point { x: -1.0, y: u, z: v }.norm()),
          result.get_traj(Point { x: v, y: -1.0, z: u }.norm()),
        ];
        for (t, g) in r {
          if t.len() > 0 && values.insert(t.clone()) {
            trajs.push(t);
          }
        }
      }
    }
    let all_trajs: Vec<_> = (0..trajs.len()).collect();
    result.split_start = result.split(&trajs, &all_trajs);
    result.print_split(result.split_start, 0);
    //  panic!("look");
    result
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
    return 0;
    let n0 = self.normals[current_normal];
    let n1 = n0.any_perp().norm();
    let n2 = cross(n0, n1);

    let last_groove = self.groove[0][self.groove[0].len() - 2];
    let sz = last_groove + 2.2;
    let p = n0.scale(sz) + n1.scale(pos.x) + n2.scale(pos.y);
    (self.get_part_index_impl(p, current_normal) > 0) as PartIndex
  }

  pub fn get_quality() -> usize {
    100
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();

    if pos.y < 0.0 {
      // return 0;
    }

    if r > self.groove[0][self.groove[0].len() - 2] + 2.0 {
      //  return 0;
    }

    let sphere_r = self.groove[0][1] - 2.2;

    if r < sphere_r {
      // return 0; // tmp
      if r > sphere_r - 0.2 || r < sphere_r - 5.2 {
        return 0;
      }
      for &(a, g) in &self.axis {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < 1.25 {
          return 0;
        }
      }
      return 63;
    }

    let mut out_core = false;
    let last_groove = self.groove[0][self.groove[0].len() - 2];
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

    let shift = [
      get_groove(r, &self.groove[0], 0.03),
      get_groove(r, &self.groove[1], 0.03),
      get_groove(r, &self.groove[2], 0.03),
    ];
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

    let mut match_axis = |a: Point, shift_in: f32, shift_out: f32| {
      let c = dot(pos, a) / r;
      let s = cross(pos, a).len();
      let check_in = c - shift_in;
      if check_in > 0.0 {
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

    let mut sg = self.split_start;
    loop {
      match sg {
        SubGroup::Piece(p) => {
          index = p;
          break;
        }
        SubGroup::Group(g) => {
          let gr = &self.splits[g];
          match match_axis(gr.splitter.0, shift[gr.splitter.1].0, shift[gr.splitter.1].1) {
            PositionRelAxis::Inside => sg = gr.lhs,
            PositionRelAxis::Between => return 0,
            PositionRelAxis::Outside => sg = gr.rhs,
          }
        }
      }
    }

    if index == 0 {
      index = 63
    }

    if index.count_ones() == 1 && r < sz + 333.0 {
      let hole_r = if r > sphere_r + 2.0 { 3.2 } else { 1.5 };
      for (i, &(a, g)) in self.axis.iter().enumerate() {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < hole_r {
          return 0;
        }
      }
    }

    let g = &self.groove[0];

    let thick = false;
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

    return index;

    if axis_pos.len() >= 2 {
      if in_sr(axis_pos[0].0, axis_pos[1].0, 1.0) {
        return 0;
      }
    }
    if axis_neg.len() >= 2 {
      if in_sr(axis_neg[0].0, axis_neg[1].0, 1.0) {
        return 0;
      }
    }
    if !shift[0].2 && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
      if in_sr(axis_pos[0].0, axis_neg[0].0, 1.0) {
        return 0;
      }
    }

    return index;
  }
}
