use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;
use num::PrimInt;

use std::cell::RefCell;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;

fn sqr(x: f32) -> f32 {
  x * x
}

struct TrivialTip {
  axis: Point,
  shift: Point,
  part: PartIndex,
  dist: f32,
}

pub struct GhostPyraCreator {
  axis: Vec<Point>,
  trivials: Vec<TrivialTip>,
  normals: Vec<(Point, f32)>,

  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
  n_dists: RefCell<Vec<(f32, usize)>>,
}

impl GhostPyraCreator {
  pub fn new() -> Self {
    let r = 15.0;
    let edge = (-1.0 / 3.0).acos();
    let min_angle = edge * 0.5;
    let max_angle = (1.0 / 3.0).acos();

    let groove = vec![(min_angle + 5.0 / r).cos(), r + 0.2, (min_angle + 2.0 / r).cos()];

    let mut axis: Vec<Point> = [
      Point { x: -1.0, y: -1.0, z: 1.0 },
      Point { x: -1.0, y: 1.0, z: -1.0 },
      Point { x: 1.0, y: -1.0, z: -1.0 },
      Point { x: 1.0, y: 1.0, z: 1.0 },
    ]
    .into_iter()
    .map(|p| p.norm().rotate(Point { x: 1.0, y: 2.0, z: 3.0 }.norm(), 0.48))
    .collect();

    let normals: Vec<_> = [
      Point { x: 1.0, y: 1.0, z: -1.0 },
      Point { x: 1.0, y: -1.0, z: 1.0 },
      Point { x: -1.0, y: 1.0, z: 1.0 },
      Point { x: -1.0, y: -1.0, z: -1.0 },
    ]
    .into_iter()
    .map(|p| p.norm())
    .collect();
    let dists: Vec<f32> =
      normals.iter().zip(&axis).map(|(n, a): (&Point, &Point)| -dot(*n, *a)).collect();
    let edge = dists.iter().copied().map(f32::recip).sum::<f32>() * (1.5).sqrt();
    let mind = dists.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied().unwrap();
    println!("dist={dists:?}");
    let osz = groove[1] + 4.0;
    let normals = normals.into_iter().zip(dists).map(|(p, d)| (p, d * osz / mind)).collect();
    let a0 = axis[0];

    println!("osz={osz}, edge={}", edge * osz);

    for a in &mut axis[1..] {
      *a = a.rotate(a0, PI / 3.0)
    }

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());
    let n_dists = RefCell::new(Vec::new());

    let trivials = vec![
      TrivialTip {
        axis: Point { x: 1.0, y: 3.0, z: 2.0 }.norm(),
        shift: Point { x: 11.0, y: 0.0, z: 5.0 },
        part: 12,
        dist: 33.5,
      },
      TrivialTip {
        axis: Point { x: -2.0, y: 1.0, z: -1.0 }.norm(),
        shift: Point { x: 15.0, y: 0.0, z: -6.0 },
        part: 10,
        dist: 35.0,
      },
    ];

    Self { axis, trivials, normals, groove, axis_pos, axis_neg, n_dists }
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    1.5
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    1
  }

  pub fn get_name(&self, current_normal: usize) -> Option<String> {
    None
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    if pos.x.abs() > 99.0 || pos.y.abs() > 99.0 {
      return 0;
    }
    let n = self.normals[current_normal];
    let sz = n.1;
    let n = n.0;
    let n1 = n.any_perp().norm();
    let n2 = cross(n, n1);

    let pos = n.scale(sz - 0.01) + n1.scale(pos.x) + n2.scale(pos.y);
    self.get_part_index_impl(pos, current_normal)
  }

  pub fn faces(&self) -> usize {
    self.normals.len()
  }

  pub fn get_quality() -> usize {
    320
  }

  pub fn get_size() -> f32 {
    120.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if pos.x.abs() > 59.0 || pos.y.abs() > 59.0 || pos.z.abs() > 59.0 {
      return 0;
    }

    let r = pos.len();

    let inner_r = self.groove[1] - 2.2;
    if r < inner_r {
      if r > inner_r - 0.3 {
        return 0;
      }

      for (i, &a) in self.axis.iter().enumerate() {
        let d = dot(pos, a);
        let s = cross(pos, a).len();
        if s < 1.5 || d < inner_r - 3.0 && s < 3.2 {
          return 0;
        }
      }

      return 33;
    }

    let mut n_dists = self.n_dists.borrow_mut();
    let n_dists = n_dists.deref_mut();
    n_dists.clear();

    for (i, n) in self.normals.iter().enumerate() {
      let d = dot(pos, n.0);
      if d > n.1 {
        return 0;
      }
      n_dists.push((n.1 - d, i));
    }

    n_dists.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    let mut fd;
    if current_normal == self.faces() {
      let out_r = 3.0;
      fd = out_r
        - (sqr(out_r - f32::min(n_dists[0].0, out_r))
          + sqr(out_r - f32::min(n_dists[1].0, out_r))
          + sqr(out_r - f32::min(n_dists[2].0, out_r)))
        .sqrt();

      if fd < 0.0 {
        return 0;
      }

    } else {
      fd = n_dists[0].0;
      if n_dists[1].0 > 3.0 { return 0; }
      return 1;
    }

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);
    let f = (self.groove[self.groove.len() - 2] + 1.0) / r;
    if f < 1.0 {
      shift_out *= f;
      shift_in *= f;
    }

    let mut axis_pos = self.axis_pos.borrow_mut();
    let axis_pos = axis_pos.deref_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    let axis_neg = axis_neg.deref_mut();

    axis_pos.clear();
    axis_neg.clear();

    let mut index: PartIndex = 0;

    let mut pos = pos;

    for (i, a) in self.axis.iter().enumerate() {
      let d = dot(pos, *a) / r;

      let check_in = d - shift_in;
      if check_in > 0.0 {
        index |= (1 << i);
        axis_pos.push((check_in, *a));
        if i == 0 {
          pos = pos.rotate(*a, PI / 3.0);
        }
      } else {
        let check_out = shift_out - d;
        if check_out > 0.0 {
          axis_neg.push((check_out, *a));
        } else {
          return 0;
        }
      }
    }

    for t in &self.trivials {
      if index != t.part {
        continue;
      }
      let d = dot(pos, t.axis);
      let c = cross(pos - t.shift, t.axis).len();

      if d > t.dist {
        index += 1 << self.axis.len();
        if c < 1.2 && fd > 2.0 {
          return 0;
        }
      } else if d > t.dist - 2.0 {
        if c < 1.5 {
          return 0;
        }
      } else if d > 0.0 {
        if c < 3.2 {
          return 0;
        }
      }
    }

    if current_normal < self.faces() {
      return index;
    }

    if index.count_ones() == 1 && fd > 2.0 {
      for &a in &self.axis {
        if dot(pos, a) > 0.0 && cross(pos, a).len() < 1.2 {
          return 0;
        }
      }
    }

    let mut rr = 0.2f32;

    axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    let mut in_sr = |a, b, r| {
      if a < r && b < r {
        return (sqr(r - a) + sqr(r - b)) > sqr(r);
      }
      return false;
    };

    if axis_pos.len() >= 2 {
      if in_sr(axis_pos[0].0, axis_pos[1].0, rr) {
        return 0;
      }
    }
    if axis_neg.len() >= 2 {
      if in_sr(axis_neg[0].0, axis_neg[1].0, rr) {
        return 0;
      }
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
      if in_sr(axis_pos[0].0, axis_neg[0].0, rr * 0.5) {
        return 0;
      }
    }

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.axis.len())
  }
}
