use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::FxHashMap;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

pub fn sqr(x: f32) -> f32 {
  x * x
}

pub struct RailroadCreator {}

impl RailroadCreator {
  pub fn new() -> Self {
    Self {}
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
    384
  }

  pub fn get_size() -> f32 {
    120.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if pos.x.abs() > 35.999 || pos.y.abs() > 35.999 || pos.z.abs() > 35.999 {
      return 0;
    }

    if pos.z < 0.1 && pos.z > -5.0 {
      if (pos.x + 5.0).rem_euclid(30.0) < 10.0 && pos.y.abs() < 40.0 {
        return 1;
      }
    }

    if pos.z > 0.0 && pos.z < 5.0 {
      let rw2 = 1524.0 / 43.0 * 0.5;
      if pos.y.abs() > rw2 && pos.y.abs() < rw2 + 2.0 {
        return 1;
      }
    }

    return 0;
  }
}
