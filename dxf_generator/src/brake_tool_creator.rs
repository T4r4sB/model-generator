use common::points2d::*;
use common::solid::*;

use common::slots_and_holes::*;

pub struct BrakeToolCreator {
  builder: Builder,
}

impl BrakeToolCreator {
  pub fn new() -> Self {
    let error = 0.1;
    let mut builder = Builder::new(10.0, 3.0, error);

    let h1 = builder.add_hole(Hole::new_solid(Point { x: 80.0, y: 30.0 }, 8.0).oval_width(5.0));
    let h2 = builder.add_hole(Hole::new_solid(Point { x: 130.0, y: 30.0 }, 8.0).oval_width(5.0));

    let h1p = builder.add_hole(Hole::new_no_border(Point { x: 80.0, y: 36.0 }, 4.0));
    let h2p = builder.add_hole(Hole::new_no_border(Point { x: 130.0, y: 36.0 }, 4.0));

    let pt = Point::from_angle(PI * 32.0 / 180.0);
    let sq = 6.95;
    let w = 10.0;
    let slot_start = pt.scale(sq * 0.5);

    let slot = builder
      .add_slot(Slot::new(slot_start, -pt, sq + w * 2.0, &[(w, sq + w)]).width(sq).border(w));

    builder.add_figure(
      Figure::new(&builder, &[chain![slot, h1, h2], chain![h1p], chain![h2p]])
        .name("brake_tool".into())
        .count(3),
    );

    let axle_hole = builder.add_hole(Hole::new(Point { x: 230.81, y: 16.75 }, 11.0).border(3.0));
    let protr_hole = builder.add_hole(Hole::new(Point { x: 260.5, y: 28.0 }, 5.5).border(3.0));

    builder
      .add_figure(Figure::new(&builder, &[chain![axle_hole, protr_hole]]).name("axle_part".into()));

    let cargo_axle_hole =
      builder.add_hole(Hole::new(Point { x: 0.0, y: 0.0 }, 5.1).border(2.0).ring());
    let cargo_axle_hole_2 =
      builder.add_hole(Hole::new(Point { x: 6.0, y: 0.0 }, 5.1).border(2.0).ring());
    let cargo_bottom_hole = builder.add_hole(Hole::new(Point { x: 0.0, y: 50.0 }, 3.0).border(4.0));
    let cargo_top_hole = builder.add_hole(Hole::new(Point { x: 0.0, y: 71.0 }, 3.0).border(4.0));
    let keeper_slot = builder.add_slot(
      Slot::new(Point { x: -10.0, y: 0.0 }, Point::X, 40.0, &[(2.0, 38.0)]).width(8.0).border(0.0),
    );

    builder.add_figure(
      Figure::new(
        &builder,
        &[chain![cargo_axle_hole, cargo_axle_hole_2, cargo_bottom_hole, cargo_top_hole]],
      )
      .name("cargo_axle_part".into())
      .thickness(2.0),
    );

    Self { builder }
  }

  pub fn aabb(&self, part_index: usize) -> Option<AABB> {
    Some(self.builder.aabb(part_index))
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
    self.builder.contains(pos, current_normal) as PartIndex
  }
}
