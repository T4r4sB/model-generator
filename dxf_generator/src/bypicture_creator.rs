use common::points2d::*;
use common::solid::*;
use num::Float;

use common::slots_and_holes::*;

pub struct ByPictureCreator {
  mat: Vec<Vec<f32>>,
}

impl ByPictureCreator {
  pub fn new() -> Self {
    let image = bmp::open("C:\\Users\\user\\Pictures\\puzzle.bmp").unwrap();
    let mut mat = Vec::new();

    let s = 4;
    for y in 0..image.get_height() {
      for _ in 0..s {
        let mut row = Vec::new();
        for x in 0..image.get_width() {
          for _ in 0..s {
            row.push(if image.get_pixel(x, image.get_height() - 1 - y).r > 128 {
              0.0
            } else {
              1.0
            });
          }
        }
        mat.push(row);
      }
    }

    for _ in 0..90 {
      let mut mat2 = Vec::new();
      for y in 0..mat.len() {
        mat2.push(vec![0.0; mat[y].len()]);
      }

      for y in 1..mat.len() - 1 {
        for x in 1..mat[y].len() - 1 {
          mat2[y - 1][x] += mat[y][x] * 0.125;
          mat2[y][x - 1] += mat[y][x] * 0.125;
          mat2[y + 1][x] += mat[y][x] * 0.125;
          mat2[y][x + 1] += mat[y][x] * 0.125;
          mat2[y][x] += mat[y][x] * 0.5;
        }
      }

      mat = mat2;
    }

    Self { mat }
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
    100.0
  }

  pub fn get_name(&self, part_index: usize) -> Option<&str> {
    None
  }

  pub fn get_sticker_index(&self, pos: Point, part_index: usize) -> PartIndex {
    let x = (pos.x * 56.0 + 10.0) * 0.5;
    let y = (pos.y * 56.0 + 10.0) * 0.5;
    if y >= 0.0 && (y as usize) < self.mat.len() {
      let row = &self.mat[y as usize];
      if x >= 0.0 && (x as usize) < self.mat.len() {
        return (row[x as usize] > 0.5) as PartIndex;
      }
    }

    return 0;
  }

  pub fn get_part_index(&self, pos: common::points3d::Point) -> PartIndex {
    0
  }
}
