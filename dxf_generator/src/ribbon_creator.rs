use crate::points2d::*;
use crate::solid::*;

use crate::slots_and_holes::*;

pub struct RibbonCreator {
  builder: Builder,
}

impl RibbonCreator {
  pub fn new() -> Self {
    let error = 0.1;
    let mut builder = Builder::new(3.5, 1.0, error);

    let h05 = builder.add_hole(Hole::new(Point::ZERO, 2.5).gauntlet_width(5.0));
    let h06 = builder.add_hole(Hole::new(Point::ZERO, 3.0).gauntlet_width(5.0));
    let h56 = builder.add_hole(Hole::new(Point::X.scale(15.0 * 5.0), 3.0).gauntlet_width(5.0));
    let h65 = builder.add_hole(Hole::new(Point::X.scale(15.0 * 6.0), 2.5).gauntlet_width(5.0));

    builder.add_figure(
      Figure::new(&builder, &[chain![h06, h56]])
        .name("ribbon_6h".into())
        .count(2),
    );

    builder.add_figure(
      Figure::new(&builder, &[chain![h05, h65]])
        .name("ribbon_7h".into())
        .count(4),
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
