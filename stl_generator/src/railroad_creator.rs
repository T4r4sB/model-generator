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
    330
  }

  pub fn get_size() -> f32 {
    120.0
  }

  pub fn get_part_index_impl(&self, mut pos: Point, current_normal: usize) -> PartIndex {
    if pos.z > 30.0 {
      pos.z -= 30.0;

      let x1 = -pos.x - MAGNET.x * 0.5 - 5.0;
      let x2 = pos.x - MAGNET.x * 0.5 + 5.0;
      let y1 = -pos.y - MAGNET.z * 0.5;
      let y2 = pos.y - MAGNET.z * 0.5;
      let z1 = -pos.z + 3.5;
      let z2 = pos.z - MAGNET.y - 3.5;
      let mx = f32::max(x1, x2);
      let my = f32::max(y1, y2);

      if mx < 0.1 && my < 0.1 && z1 < 0.1 && z2 < 0.1 {
        return 0;
      }

      if sqr(pos.x - 20.0) + sqr(pos.y) < sqr(1.6) {
        return 0;
      }
      if sqr(pos.x + 20.0) + sqr(pos.y) < sqr(1.6) {
        return 0;
      }
      if sqr(f32::max(pos.x.abs() - 10.0, 0.0)) + sqr(pos.y) > sqr(18.0) {
        return 0;
      }

      if x1 < 1.4 && x2 < 26.4 && my < 1.4 && z1 < -0.1 && z2 < 1.4 && (z2 < 2.0 - x2 || z1 > -2.4)
      {
        let to_spike = (pos.x.abs() - 23.5).abs();
        if to_spike < 3.0 && z1 < -1.4 - to_spike {
          return 0;
        }
        return 1000;
      }

      if mx < 2.4 && my < 2.4 && z1 < -0.1 && z1 > -3.4 + f32::max(mx, my) {
        return 1000;
      }

      if pos.x.abs() < 28.0 && my < 4.9 && z1 < 2.4 && z1 > 0.1 {
        if z1 < 1.1 - (pos.x + 23.5).abs() {
          return 0;
        }

        return 1001;
      }

      if (x1 > 2.6 || my > 2.6) && mx < 4.9 && my < 4.9 && z1 < 1.4 && z1 > -3.0 {
        return 1001;
      }

      if (x1 > 1.6 || my > 1.6) && mx < 4.9 && my < 4.9 && z1 < -3.6 + f32::max(x1, my) && z1 > -3.0
      {
        return 1001;
      }
    } else if pos.z > 0.0 {
      let x1 = -pos.x - 20.0;
      let x2 = pos.x - 20.0;
      let y1 = -pos.y - 14.5;
      let y2 = pos.y - 14.5;
      let z1 = -pos.z + 3.5;
      let z2 = pos.z - 13.5;
      let mx = f32::max(x1, x2);
      let my = f32::max(y1, y2);

      let mz = (z2 - z1) * 0.5 + 0.0;

      if mx < -0.1 && z1 < -0.1 && z2 < -0.1 && y1 < -2.9 && y1 > -14.9 {
        if sqr(pos.x.rem_euclid(5.0) - 2.5) + sqr(mz) < sqr(1.0) {
          return 0;
        }
        return 2003;
      }

      if mx < 0.1 && y1 < -0.9 && y2 < 0.1 && z1 < 0.1 && z2 < -0.1 {
        return 0;
      }

      if mx < 1.4 && my < 1.4 && z1 < 1.4 && z2 < -0.1 {
        if sqr(pos.x.rem_euclid(5.0) - 2.5) + sqr(y1 - 1.4) < sqr(1.0) {
          return 0;
        }

        return 2001;
      }

      let hz = (z2 - z1) * 0.5 + 4.0;

      if x2 > 1.3
        && x2 < 15.0
        && sqr(f32::min(x2 - 6.3, 0.0)) + sqr(pos.y.abs() - 8.0) > sqr(4.9)
        && pos.y.abs() < 8.0
        && hz.abs() < 5.0
        && z1 < 1.4 + (x2 - 1.3)
      {
        if sqr(hz) + sqr(x2 - 10.0) < sqr(2.6) {
          return 0;
        }

        if sqr(hz) + sqr(f32::max(x2 - 10.0, 0.0)) > sqr(4.9) {
          return 0;
        }
        return 2001;
      }

      
      if mx < 2.9 && my < 2.9 && z2 < 1.4 {
        if z2 > 0.1 {
          return 2002;
        }
        if (mx > 1.6 || my > 1.6) && z2 > -3.9 {
          return 2002;
        }
      }
    }

    0
  }
}
