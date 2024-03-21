use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;

pub struct OctoMixupCreator {
  axis: Vec<Point>,
}

const PI: f32 = std::f32::consts::PI;

fn sqr(x: f32) -> f32 {
  x * x
}

impl OctoMixupCreator {
  pub fn new() -> Self {
    let axis = vec![
      Point { x: 1.0, y: 1.0, z: 1.0 }.norm(),
      Point { x: 1.0, y: 1.0, z: -1.0 }.norm(),
      Point { x: 1.0, y: -1.0, z: -1.0 }.norm(),
      Point { x: 1.0, y: -1.0, z: 1.0 }.norm(),
      Point { x: -1.0, y: -1.0, z: 1.0 }.norm(),
      Point { x: -1.0, y: 1.0, z: 1.0 }.norm(),
      Point { x: -1.0, y: 1.0, z: -1.0 }.norm(),
      Point { x: -1.0, y: -1.0, z: -1.0 }.norm(),
    ];

    Self { axis }
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn faces(&self) -> usize {
    0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if sqr(pos.x) + sqr(pos.y) < sqr(1.25) {
      return 0;
    }
    if sqr(pos.x) + sqr(pos.z) < sqr(1.25) {
      return 0;
    }
    if sqr(pos.z) + sqr(pos.y) < sqr(1.25) {
      return 0;
    }

    let r = pos.len();

    if r > 30.0 {
      return 0;
    }
    if r < 24.0 {
      return 0xffffffff;
    }

    const CUR_COS: f32 = 0.45;

    let split_cos;
    if r > 26.0 && r < 28.0 {
      split_cos = [CUR_COS + 1.5 / 26.0, CUR_COS - 1.5 / 26.0];
    } else {
      split_cos = [CUR_COS - 1.5 / 26.0, CUR_COS + 1.5 / 26.0];
    }

    if sqr(pos.x) + sqr(pos.y) < sqr(1.25) {
      return 0;
    }

    let mut index = 0;
    for i in 0..self.axis.len() {
      let a = self.axis[i];
      if dot(pos, a) > split_cos[i % 2] * r {
        index += (1 << i) as PartIndex;
      }
    }

   // return index;

    if index.count_ones() == 2 {
      let a1 = index.ilog2();
      let a2 = (index - (1 << a1) as PartIndex).ilog2();

      let c = (self.axis[a1 as usize] + self.axis[a2 as usize]).norm();

      for i in 1..=2 {
        let mut index2 = 0;
        let pos2 = pos.rotate(c, PI * i as f32 * 2.0 / 3.0);
        for i in 0..self.axis.len() {
          let a = self.axis[i];
          let mask = (1 << i) as PartIndex;
          let control = if dot(pos2, a) > split_cos[i % 2] * r {
            mask
          } else {
            0
          };
          if index & mask != control {
            return 0;
          }
        }
      }
    } else if index.count_ones() != 4 {
      return 0;
    }

    return index;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, 8)
  }
}
