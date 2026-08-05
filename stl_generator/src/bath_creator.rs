use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::FxHashMap;
use num::Float;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

const MAGNET: Point = Point { x: 8.4, y: 15.4, z: 25.2 };

pub fn sqr(x: f32) -> f32 {
  x * x
}

pub struct BathCreator {
  tiles: Vec<u8>,
}

impl BathCreator {
  pub fn new() -> Self {
    let mut rng = rngs::StdRng::seed_from_u64(20);
    let mut tiles = Vec::with_capacity(10000);
    for i in 0..10000 {
      tiles.push(rng.gen_range(0..100));
      let x = (i % 1000) % 20;
      let y = (i % 1000) / 20;
      if y >= 5 && y <= 6 && (x < 9 || x > 12) && (x + y) % 2 == 0 {
       // *tiles.last_mut().unwrap() = 255
      };
    }
    Self { tiles }
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
    150
  }

  pub fn get_size() -> f32 {
    35.0
  }

  fn get_tile(pos: (f32, f32), tile_size: (f32, f32), stride: usize) -> usize {
    let tile_x = f32::max(pos.0 / tile_size.0, 0.0);
    let tile_y = f32::max(pos.1 / tile_size.1, 0.0);
    if tile_x.fract() * tile_size.0 < 1.0 || tile_y.fract() * tile_size.0 < 1.0 {
      return usize::MAX;
    }
    let tile_x = f32::max(tile_x.floor(), 0.0) as usize;
    let tile_y = f32::max(tile_y.floor(), 0.0) as usize;
    tile_x + tile_y * stride
  }

  pub fn get_part_index_impl(&self, mut pos: Point, current_normal: usize) -> PartIndex {
    /*
    colors

        9 => 0xEEF8FF,
        10 => 0x604000,
        11 => 0x704000,
        12 => 0x87C5D4,
        13 => 0xFFFFFF,
        14 => 0xAFC863,

        12..14 Berac Axima tile


    */

    pos = pos.scale(10.0);
    pos.y = -pos.y;
    let x1 = 250.0;
    let x2 = 310.0;
    let y1 = 120.0;
    let y2 = 20.0;
    let z1 = 300.0;
    let z2 = 240.0;
    pos += Point { x: x2 * 0.5, y: y1 * 0.5, z: z1 * 0.5 };
    let wall_tile = (20.0, 20.0);
    let choose_tile = |t: u8| -> PartIndex {
      if t == 255 {
        return 3;
      }
      (t % 3) as PartIndex
    };
    let texture = |pos: (f32, f32)| -> bool {
      true
      //(((pos.0 + pos.1) / 3.0) as i32) % 2 == 0
    };

    if pos.z > -5.0
      && pos.z < 1.0
      && pos.x > -5.0
      && pos.x < x2 + 5.0
      && pos.y > -5.0
      && pos.y < y1 + 5.0
    {
      // floor
      let tile_index = if pos.y < y1 {
        Self::get_tile((pos.x + 20.0, pos.y), (30.0, 30.0), 20)
      } else {
        usize::MAX
      };
      if tile_index == usize::MAX {
        if pos.z < 0.0 {
          return 1;
        }
      } else {
        let select_tile = (self.tiles[tile_index] % 2) as PartIndex;
        return (10 + select_tile) * 100000 + tile_index as PartIndex;
      }
    }

    if pos.z > 0.0 && pos.z < z1 && pos.x > 0.0 && pos.x < x1 && pos.y > -5.0 && pos.y < 1.0 {
      // wall1

      let tile_index = if pos.z < z2 && pos.y > -2.0 {
        Self::get_tile((pos.x, pos.z), wall_tile, 20)
      } else {
        usize::MAX
      };
      if tile_index == usize::MAX {
        if pos.y < 0.0 {
          return 1;
        }
      } else if pos.y < 0.1 || texture((pos.x, pos.z)) {
        let tile_index = tile_index + 1000;
        let select_tile = choose_tile(self.tiles[tile_index]);
        return (12 + select_tile) * 100000 + tile_index as PartIndex;
      }
    }

    if pos.z > 0.0
      && pos.z < z1
      && pos.x > x1 - 1.0
      && pos.x < x2 + 5.0
      && pos.y > -5.0
      && pos.y < y2 + 1.0
    {
      // wall_vent
      if sqr(pos.x - (x1 + x2) * 0.5) + sqr(pos.z - 260.0) < sqr(10.0) {
        return 0;
      }

      if x1 - pos.x > pos.y - y2 {
        let tile_index = if pos.z < z2 && pos.x < x1 + 2.0 {
          Self::get_tile((pos.y, pos.z), wall_tile, 20)
        } else {
          usize::MAX
        };
        if tile_index == usize::MAX {
          if pos.x > x1 {
            return 1;
          }
        } else if pos.x > x1 - 0.1 || texture((pos.y, pos.z)) {
          let tile_index = tile_index + 2000;
          let select_tile = choose_tile(self.tiles[tile_index]);
          return (12 + select_tile) * 100000 + tile_index as PartIndex;
        }
      } else {
        let tile_index = if pos.z < z2 && pos.y > y2 - 2.0 {
          Self::get_tile((pos.x, pos.z), wall_tile, 20)
        } else {
          usize::MAX
        };
        if tile_index == usize::MAX {
          if pos.y < y2 {
            return 1;
          }
        } else if pos.y < y2 + 0.1 || texture((pos.x, pos.z)) {
          let tile_index = tile_index + 3000;
          let select_tile = choose_tile(self.tiles[tile_index]);
          return (12 + select_tile) * 100000 + tile_index as PartIndex;
        }
      }
    }

    if pos.z > 0.0 && pos.z < z1 && pos.x > x2 - 1.0 && pos.x < x2 + 5.0 && pos.y > y2 && pos.y < y1
    {
      // wall 2
      let tile_index = if pos.z < z2 && pos.x < x2 + 2.0 {
        Self::get_tile((pos.y, pos.z), wall_tile, 20)
      } else {
        usize::MAX
      };
      if tile_index == usize::MAX {
        if pos.x > x2 {
          return 1;
        }
      } else if pos.x > x2 - 0.1 || texture((pos.y, pos.z)) {
        let tile_index = tile_index + 4000;
        let select_tile = choose_tile(self.tiles[tile_index]);
        return (12 + select_tile) * 100000 + tile_index as PartIndex;
      }
    }

    if pos.z > 100.0
      && pos.z < 200.0
      && pos.x > x1 - 60.0
      && pos.x < x1 - 10.0
      && pos.y > 0.0
      && pos.y < 5.0
    {
      // mirror
      return 900000;
    }

    if pos.z > 70.0
      && pos.z < 85.0
      && pos.x > x1 - 60.0
      && pos.x < x1 - 10.0
      && pos.y > 0.0
      && pos.y < 45.0
    {
      // hand washer
      if pos.z > 75.0 && pos.x > x1 - 55.0 && pos.x < x1 - 15.0 && pos.y > 15.0 && pos.y < 40.0 {
        return 0;
      }

      return 600000;
    }

    if pos.z > 0.0
      && pos.z < 85.0
      && pos.x > x1
      && pos.x < x1 + 50.0
      && pos.y > y1 - 65.0
      && pos.y < y1 - 5.0
    {
      // washing machine
      if pos.x < x1 + 5.0 + (pos.z - 70.0) * 0.3 {
        return 0;
      }

      if sqr(pos.z - 30.0) + sqr(pos.y - (y1 - 35.0)) > sqr(20.0) && pos.x < x1 + 5.0 {
        return 0;
      }

      if sqr(pos.z - 30.0) + sqr(pos.y - (y1 - 35.0)) < sqr(13.0) {
        return 3;
      }
      return 600001;
    }

    if pos.z > 0.0 && pos.z < z1 && sqr(pos.x - (x2 - 10.0)) + sqr(pos.y - (y2 + 10.0)) < sqr(5.0) {
      // unknown pipe
      return 300000;
    }

    if pos.z > 5.0 && pos.z < 71.0 &&  sqr(pos.x - (x1 - 35.0)) + sqr(pos.y - 5.0) < sqr(2.5) {
      // pipe1
      return 300001;
    }

    if pos.x > 150.0 && pos.x < x1 - 5.0 && sqr(pos.y - 5.0) + sqr(pos.z - 5.0) < sqr(2.5) {
      // pipe2
      return 300001;
    }

    if pos.y > 5.0 && pos.y < y2 + 5.0 && sqr(pos.x - (x1 - 5.0)) + sqr(pos.z - 5.0) < sqr(2.5) {
      // pipe3
      return 300001;
    }

    if pos.x > x1 - 5.0 && pos.x < x2 - 20.0 && sqr(pos.y - (y2 + 5.0)) + sqr(pos.z - 5.0) < sqr(2.5) {
      // pipe4
      return 300001;
    }

    if pos.y > y2 + 5.0 && pos.y < y2 + 20.0 && sqr(pos.x - (x2 - 20.0)) + sqr(pos.z - 5.0) < sqr(2.5) {
      // pipe5
      return 300001;
    }

    if pos.x > x2 - 20.0 && pos.x < x2 && sqr(pos.y - (y2 + 20.0)) + sqr(pos.z - 5.0) < sqr(2.5) {
      // pipe6
      return 300001;
    }

    if pos.y > 5.0 && pos.y < 42.5 && sqr(pos.x - 150.0) + sqr(pos.z - 5.0) < sqr(2.5) {
      // pipe7
      return 300001;
    }

    if pos.z > 5.0 && pos.z < 20.5 && sqr(pos.x - 150.0) + sqr(pos.y - 42.5) < sqr(2.5) {
      // pipe8
      return 300001;
    }


    if pos.z > 10.0
      && pos.z < 50.0
      && pos.x > 0.0 + 0.5 * (50.0 - pos.z)
      && pos.x < 170.0 - 0.3 * (50.0 - pos.z)
      && pos.y > 5.0 + 0.3 * (50.0 - pos.z)
      && pos.y < 75.0 - 0.3 * (50.0 - pos.z)
    {
      // bath
      if pos.z > 15.0
        && pos.x > 5.0 + 0.5 * (50.0 - pos.z)
        && pos.x < 165.0 - 0.3 * (50.0 - pos.z)
        && pos.y > 10.0 + 0.3 * (50.0 - pos.z)
        && pos.y < 70.0 - 0.3 * (50.0 - pos.z)
      {
        return 0;
      }

      return 600002;
    }

    for (x, y) in [(25.0, 20.0), (145.0, 20.0), (25.0, 60.0), (145.0, 60.0)] {
      if pos.z > 0.0 && pos.z < 14.0 && sqr(pos.x - x) + sqr(pos.y - y) < sqr(2.0) {
        return 5;
      }
    }

    0
  }
}
