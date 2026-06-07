use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;

fn sqr(x: f32) -> f32 {
  x * x
}

pub struct WasherCreator {}

impl WasherCreator {
  pub fn new() -> Self {
    Self {}
  }

  pub fn faces(&self) -> usize {
    0
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

  pub fn get_quality() -> usize {
    512
  }

  pub fn get_size() -> f32 {
    25.0
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    
    if pos.z < 5.5 {
     // return 0; //tmp
      
      if pos.z < 0.0 {
        return 0;
      }
      if pos.z > 1.2 {
        return 0;
      }
      let r = (sqr(pos.x) + sqr(pos.y)).sqrt();
      if r > 2.5 {
        return 0;
      }
      if r < 1.5 {
        return 0;
      }
      if pos.z < 0.5 {
        return 1;
      }
      if r < 2.0 && pos.z > 0.4 + (2.1 - r) {
        return 0;
      }
      return 1;
      
    }

    return 0;

    if pos.z < 10.0 {
      let r = (sqr(pos.x) + sqr(pos.y)).sqrt();
      if r < 2.8 || r > 5.7 {
        return 0;
      }
      return 2;
    }

    return 0;
  }
}
