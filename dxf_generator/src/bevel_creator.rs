use common::points2d::*;
use common::solid::*;
use num::Float;

use common::slots_and_holes::*;

pub struct BevelCreator {}

impl BevelCreator {
  pub fn new() -> Self {
    Self {}
  }

  pub fn get_count(&self, part_index: usize) -> usize {
    1
  }

  pub fn aabb(&self, part_index: usize) -> Option<AABB> {
    None
  }

  pub fn faces(&self) -> usize {
    1
  }

  pub fn get_height(&self, part_index: usize) -> f32 {
    2.0
  }

  pub fn get_quality() -> usize {
    128
  }

  pub fn get_size() -> f32 {
    120.0
  }

  pub fn get_name(&self, part_index: usize) -> Option<&str> {
    Some("bevel")
  }

  pub fn get_sticker_index(&self, pos: Point, part_index: usize) -> PartIndex {
    if pos.y < -38.0 || pos.x < -46.0 || pos.y > 9.0 || pos.x > 60.0 {
      return 0;
    }

    if (pos - Point { x: 51.0, y: -29.0 }).len() < 5.0 { return 0; }

    let size = 27.0 * PI / 2.0;

    if pos.x < -1.0 && pos.x > 1.0 - size && pos.y < -6.0 && pos.y > -34.0 {
      return 0;
    }
    if pos.x < -1.0 && pos.x > 1.0 - size && pos.y < 4.0 && pos.y > -4.0 {
      return 0;
    }
    {
      let pos = pos - Point { x: 57.0, y: 6.0 };

      if pos.len() < 27.0 && pos.x < -1.0 && pos.y < -1.0 {
        return 0;
      }
    }

    {
      let pos = pos - Point { x: 1.0, y: -35.0 };
      let a1 = PI / 2.0 * 4.0 / 5.0;
      let p = Point::from_angle(PI * 0.5 - a1).scale(40.0);

      if pos.len() < 40.0 && dist_pl(pos, Point::ZERO, p) > 1.0 && pos.x > 1.0 && pos.y > 1.0 {
        return 0;
      }
    }

    return 1;
  }

  pub fn get_part_index(&self, pos: common::points3d::Point) -> PartIndex {
    0
  }
}
