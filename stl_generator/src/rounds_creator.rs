use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

#[derive(Debug, Default, Clone)]
struct FLFunc {
  points: Vec<(f32, f32)>,
}

impl FLFunc {
  fn get(&self, x: f32) -> f32 {
    if x < self.points[0].0 {
      return self.points[0].1;
    }

    for i in 1..self.points.len() {
      if x < self.points[i].0 {
        return self.points[i - 1].1
          + (x - self.points[i - 1].0) * (self.points[i].1 - self.points[i - 1].1)
            / (self.points[i].0 - self.points[i - 1].0);
      }
    }

    return self.points[self.points.len() - 1].1;
  }
}

pub struct RoundsCreator {
  r_in: FLFunc,
  r_out: FLFunc,
  orange: Vec<(f32, f32)>,
  blue: Vec<(f32, f32)>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl RoundsCreator {
  pub fn new() -> Self {
    let t = 0.12;
    let r_in = FLFunc {
      points: vec![
        (-4.0, -0.2),
        (-3.8, -0.0),
        (-2.9 - t * 2.0, 0.0),
        (-2.1 - t * 2.0, 0.8),
        (-0.5 - t * 2.0, -0.8),
        (-0.5 + t * 2.0, -0.8),
        (1.1 + t * 2.0, 0.8),
        (1.9 + t * 2.0, 0.0),
        (3.8, 0.0),
        (4.0, -0.2),
      ],
    };
    let r_out = FLFunc {
      points: vec![
        (-4.0, -0.2),
        (-3.8, -0.0),
        (-2.9 - t * 4.0, 0.0),
        (-2.1 - t * 4.0, 0.8),
        (-2.1, 0.8),
        (-0.5, -0.8),
        (1.1, 0.8),
        (1.1 + t * 4.0, 0.8),
        (1.9 + t * 4.0, 0.0),
        (3.8, 0.0),
        (4.0, 0.2),
      ],
    };
    let orange = vec![
      (10.0, 0.0),
      (7.0, 12.0),
      (3.0, 16.0),
      (-10.0, 21.0),
      (6.0, 22.0),
      (2.0, 26.0),
      (20.0, 17.0),
      (20.0, 12.0),
    ];

    let blue = vec![(-1.0, 0.0)];
    Self { r_in, r_out, orange, blue }
  }

  pub fn faces(&self) -> usize {
    0
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
    0
  }

  pub fn get_quality() -> usize {
    512
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();

    if pos.z < -4.0 || pos.z > 5.0 {
      return 0;
    }

    let oval_r = sqr(pos.x) + sqr(pos.y * 0.8) + sqr(pos.z);
    if oval_r > sqr(28.0) {
      return 0;
    }

    let oval_r = sqr(pos.x) + sqr(pos.y * 0.8);

    let dr = 0.15;
    let main_r = 15.0;

    let r_in = main_r + self.r_in.get(pos.z) - dr;
    let r_out = main_r + self.r_out.get(pos.z) + dr;

    let mut dists = [0.0; 5];

    let hd = 10.0;
    for (i, p) in
      [(hd, 0.0), (-hd, 0.0), (0.0, hd * 3.0.sqrt()), (0.0, -hd * 3.0.sqrt())].iter().enumerate()
    {
      let r = (sqr(pos.x - p.0) + sqr(pos.y - p.1)).sqrt();

      if r < r_in {
        dists[i] = r_in - r;
      } else if r > r_out {
        dists[i] = r - r_out;
      } else {
        return 0;
      }
    }

    dists[4] = f32::INFINITY;

    dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rr = 2.0;
    if dists[0] < rr && dists[1] < rr && sqr(rr - dists[0]) + sqr(rr - dists[1]) > sqr(rr) {
      return 0;
    }

    dists[4] = 27.0 - oval_r.sqrt();
    dists.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let b = 0.7 - f32::clamp(pos.z - 3.8, 0.0, 0.2);
    if pos.z > 3.0 {
      let mut dist = dists[0] - b;

      let rr = 1.5;
      if (dists[0] - b) < rr && (dists[1] - b) < rr {
        dist = rr - (sqr(rr - (dists[0] - b)) + sqr(rr - (dists[1] - b))).sqrt();
      }

      if dist > 0.3 + f32::max(0.0, pos.z - 4.0) {
        let mut min_orange = f32::INFINITY;
        for &o in &self.orange {
          min_orange = f32::min(min_orange, (sqr(pos.x - o.0) + sqr(pos.y - o.1)).sqrt());
        }
        let mut min_blue = f32::INFINITY;
        for &b in &self.blue {
          min_blue = f32::min(min_blue, (sqr(pos.x - b.0) + sqr(pos.y - b.1)).sqrt());
        }
        if min_orange > 10.3 {
          return 2;
        } else if min_orange < 10.0 {
          return 3;
        } else {
          return 0;
        }
      }

      if dist > 0.0 {
        return 0;
      }

      if pos.z > 4.0 {
        return 0;
      }
    }

    return 0; // tmp
    return 1;
  }
}
