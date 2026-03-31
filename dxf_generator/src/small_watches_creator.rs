use common::points2d::*;
use common::solid::*;

use common::slots_and_holes::*;

pub struct WatchesCreator {
  builder: Builder,
}

impl WatchesCreator {
  pub fn new() -> Self {
    let error = 0.1;
    let mut builder = Builder::new(4.0, 2.0, error);

    let d1 = 40.5;
    let d2 = 36.5;
    let d3 = 41.5;
    let d4 = 42.5;

    let c0 = Point { x: 0.0, y: -50.0 };
    let c1 = c0 + Point::from_angle(PI * 0.125).scale(d1);
    let c2 = c1 + Point::from_angle(PI * 0.375).scale(d2);
    let c3 = c2 + Point::from_angle(PI * 0.625).scale(d3);
    let c4 = c3 + Point::from_angle(PI * 0.875).scale(d4);

    let bear_r = 6.0;
    let axle_r = 2.0;

    let bear_to_bolt = 10.0;

    let hole_with_slots = |builder: &mut Builder,
                           pos: Point,
                           dir: Point,
                           bot_border: f32|
     -> (HoleID, SlotID, SlotID) {
      let dir = dir.norm();
      let bolt_hole = builder.add_hole(Hole::new(pos, 2.0).border(bot_border));

      let a45 = Point::from_angle(PI / 4.0);
      let dir1 = complex_mul(dir, a45);
      let dir2 = complex_mul(dir, a45.conj());

      let slot1 = builder.add_slot(Slot::new(
        pos - dir1.scale(6.0) + dir2.scale(5.0),
        dir1,
        10.0,
        &[(2.0, 8.0)],
      ));

      let slot2 = builder.add_slot(Slot::new(
        pos - dir2.scale(6.0) + dir1.scale(5.0),
        dir2,
        10.0,
        &[(2.0, 8.0)],
      ));

      (bolt_hole, slot1, slot2)
    };

    let bear_with_holes =
      |builder: &mut Builder, pos: Point, dir: Point| -> (HoleID, HoleID, HoleID, SlotID, HoleID) {
        let dir = dir.norm();
        let bear_hole = builder.add_hole(Hole::new(pos, bear_r));
        let bolt_hole_1 = builder.add_hole(Hole::new(pos + dir.perp().scale(bear_to_bolt), 2.0));
        let bolt_hole_2 = builder.add_hole(Hole::new(pos - dir.perp().scale(bear_to_bolt), 2.0));
        let last_assemple_hole =
          builder.add_hole(Hole::new_no_border(pos + dir.scale(20.0), bear_r));
        let slot_for_assemble =
          builder.add_slot(Slot::new_no_border(pos, dir, 20.0, &[(0.0, 20.0)]).width(bear_r * 2.0));
        (bear_hole, bolt_hole_1, bolt_hole_2, slot_for_assemble, last_assemple_hole)
      };

    let (h0, h01, h02, s0, lh0) = bear_with_holes(&mut builder, c0, Point::from_angle(0.0));
    let (h1, h11, h12, s1, lh1) = bear_with_holes(&mut builder, c1, Point::from_angle(PI * 0.25));
    let (h2, h21, h22, s2, lh2) = bear_with_holes(&mut builder, c2, Point::from_angle(PI * 0.5));
    let (h3, h31, h32, s3, lh3) = bear_with_holes(&mut builder, c3, Point::from_angle(PI * 0.75));
    let (h4, h41, h42, s4, lh4) = bear_with_holes(&mut builder, c4, Point::from_angle(PI * 1.0));
    let (k0, s01, s02) = hole_with_slots(&mut builder, Point { x: -40.0, y: 30.0 }, Point::X, 5.0);
    let (k1, s11, s12) = hole_with_slots(&mut builder, Point { x: -40.0, y: -30.0 }, Point::X, 5.0);
    let (k2, s21, s22) = hole_with_slots(&mut builder, Point { x: 0.0, y: 0.0 }, Point::X, 5.0);

    let bear_c = builder.add_hole(Hole::new(Point::ZERO, bear_r));
    let bear_keep = builder.add_hole(Hole::new(Point::ZERO, bear_r - 1.0));
    let bear_h1 = builder.add_hole(Hole::new(Point::X.scale(bear_to_bolt), axle_r));
    let bear_h2 = builder.add_hole(Hole::new(Point::X.scale(-bear_to_bolt), axle_r));

    builder.add_figure(
      Figure::new(&builder, &[chain![bear_h1, bear_keep, bear_h2]])
        .name("bear_keep".into())
        .count(10),
    );

    builder.add_figure(
      Figure::new(&builder, &[chain![bear_h1, bear_c, bear_h2]]).name("bear_frame".into()).count(5),
    );

    builder.add_figure(
      Figure::new(
        &builder,
        &[
          chain![h01, h0, h02, h11, h1, h12, h21, h2, h22, h31, h3, h32, h41, h4, h42],
          chain![h1, k0],
          chain![h3, k1],
          chain![k0, s01, s02],
          chain![k1, s11, s12],
          chain![k2, s21, s22],
          chain![s0, s1, s2, s3, s4, lh0, lh1, lh2, lh3, lh4],
        ],
      )
      .name("small_watches_cup".into())
      .count(2),
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
    self.builder.contains(pos, current_normal) as PartIndex
  }

  pub fn aabb(&self, part_index: usize) -> Option<AABB> {
    Some(self.builder.aabb(part_index))
  }
}
