use crate::points2d::*;
use crate::solid::*;

use crate::slots_and_holes::*;

pub struct BrakeToolCreator {
  builder: Builder,
}

impl BrakeToolCreator {
  pub fn new() -> Self {
    let error = 0.1;
    let mut builder = Builder::new(10.0, 3.0, error);

    let h1 = builder.add_hole(Hole::new_solid(Point { x: 80.0, y: 30.0 }, 8.0).gauntlet_width(5.0));
    let h2 = builder.add_hole(Hole::new_solid(Point { x: 120.0, y: 30.0 }, 8.0).gauntlet_width(5.0));

    let h1p = builder.add_hole(Hole::new_no_border(Point { x: 80.0, y: 36.0 }, 4.0));
    let h2p = builder.add_hole(Hole::new_no_border(Point { x: 120.0, y: 36.0 }, 4.0));

    let pt = Point::from_angle(PI * 32.0 / 180.0);
    let sq = 7.0;
    let w = 10.0;
    let slot_start = pt.scale(sq * 0.5);

    let slot = builder.add_slot(
      Slot::new(slot_start, -pt, sq + w * 2.0, &[(w, sq + w)])
        .width(sq)
        .border(w),
    );

    builder.add_figure(
      Figure::new(&builder, &[chain![slot, h1, h2], chain![h1p], chain![h2p]])
        .name("brake_tool".into())
        .count(3),
    );

    Self { builder }
  }

  pub fn get_quality() -> usize {
    1
  }

  pub fn get_size() -> f32 {
    1.0
  }

  pub fn faces(&self) -> usize {
    self.builder.contour_count()
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    self.builder.get_material_thickness(current_normal)
  }

  pub fn get_name(&self, current_normal: usize) -> Option<&str> {
    self.builder.get_name(current_normal)
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    self.builder.get_count(current_normal)
  }

  pub fn get_sticker_index(&self, pos: Point, current_normal: usize) -> PartIndex {
    self.builder.contains(current_normal, pos) as PartIndex
  }

  pub fn get_part_index(&self, pos: crate::points3d::Point) -> PartIndex {
    return 0;
  }
}
