use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::FxHashMap;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

const MAGNET: Point = Point { x: 8.4, y: 15.4, z: 25.2 };

pub fn sqr(x: f32) -> f32 {
  x * x
}

pub struct ClickerCreator {}

impl ClickerCreator {
  pub fn new() -> Self {
    Self {}
  }

  pub fn faces(&self) -> usize {
    1
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
    self.get_part_index_impl(Point { x: pos.x, y: pos.y, z: 0.0 }, 0)
  }

  pub fn get_quality() -> usize {
    200
  }

  pub fn get_size() -> f32 {
    120.0
  }

  pub fn get_part_index_impl(&self, mut pos: Point, current_normal: usize) -> PartIndex {
    if sqr(pos.x) + sqr(pos.y) < sqr(1.6) {
      return 0;
    }

    let match_disk = || {
      let r = 9.8 + 2.39 - 1.5;
      for i in 0..7 {
        let a = i as f32 * PI / 6.0;
        let (s, c) = a.sin_cos();
        let bc = Point { x: c * r, y: s * r, z: 0.0 };
        if (pos - bc).sqr_len() < sqr(2.49) {
          return 0;
        }
      }
      if sqr(pos.x) + sqr(pos.y) < sqr(9.8) && pos.z.abs() < 2.3 {
        return 2;
      }

      return 0;
    };

    let match_keeper = || {
      if pos.z.abs() < 4.6 && pos.y.abs() < 5.0 && pos.x > -23.0 && pos.x < 4.5 {
        if pos.x > -21.0 && sqr(pos.y) + sqr(pos.z) < sqr(2.6) {
          return 0;
        }
        if pos.x > -10.2 && pos.z.abs() < 2.6 {
          return 0;
        }

        return 1;
      }

      return 0;
    };

    fn update(a: &mut PartIndex, b: impl FnOnce() -> PartIndex) {
      if *a == 0 {
        *a = b()
      }
    }

    let mut result = 0;
    update(&mut result, match_disk);
    update(&mut result, match_keeper);

    result
  }
}
