use crate::points2d::*;
use crate::solid::*;

pub struct BrakeHandleCreator {
  centers_p1: Vec<Point>,
  centers_p2: Vec<Point>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl BrakeHandleCreator {
  pub fn new() -> Self {
    let centers_p1 = vec![
      Point { x: -32.0, y: -24.0 },
      Point { x: 0.0, y: 0.0 },
      Point { x: 0.0, y: 20.0 },
    ];
    let centers_p2 = vec![
      Point { x: 0.0, y: 0.0 },
      Point { x: 0.0, y: 20.0 },
      Point { x: 100.0, y: 20.0 },
    ];
    Self { centers_p1, centers_p2 }
  }

  pub fn faces(&self) -> usize {
    2
  }

  pub fn get_sticker_index(&self, pos: Point, current_normal: usize) -> PartIndex {
    if pos.len() < 2.6 {
      return 0;
    }

    if current_normal == 0 {
      for i in 0..self.centers_p1.len() - 1 {
        let p1 = self.centers_p1[i];
        let p2 = self.centers_p1[i + 1];
        if dist_pl(pos, p1, p2) < 10.0 {
          return 1;
        }
      }
    } else {
      for i in 0..self.centers_p2.len() - 1 {
        let p1 = self.centers_p2[i];
        let p2 = self.centers_p2[i + 1];
        if dist_pl(pos, p1, p2) < 10.0 {
          return 2;
        }
      }
    }

    0
  }

  pub fn get_part_index(&self, pos: crate::points3d::Point) -> PartIndex {
    return 0;
  }
}
