use crate::points2d::*;
use crate::solid::*;

use crate::slots_and_holes::*;

pub struct WatchesCreator {
  builder: Builder,
}

impl WatchesCreator {
  pub fn new() -> Self {
    let error = 0.1;
    let mut builder = Builder::new(4.0, 2.0, error);

    let lo_dist = 76.5;
    let lo_dist_shift = 20.0;
    let lo_dist_shifted = (sqr(lo_dist) - sqr(lo_dist_shift)).sqrt();
    let mid_dist = 77.5;
    let hi_dist = 64.0;

    let bear_r = 6.0;
    let axle_r = 2.0;

    let bear_to_bolt = 10.0;

    let top_pos = Point { x: 0.0, y: hi_dist };
    let mid_pos = Point::ZERO;
    let bottom_h = (sqr(mid_dist) - sqr(lo_dist_shifted * 0.5)).sqrt();
    let left_pos = Point { x: -lo_dist_shifted * 0.5, y: -bottom_h };
    let right_pos = Point { x: lo_dist_shifted * 0.5, y: -bottom_h - lo_dist_shift };

    let left_bottom_pos = left_pos + Point { x: -30.0, y: -76.0 - lo_dist_shift };
    let right_bottom_pos = right_pos + Point { x: 30.0, y: -76.0 };

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
      |builder: &mut Builder, pos: Point, dir: Point| -> (HoleID, HoleID, HoleID, SlotID) {
        let dir = dir.norm();
        let bear_hole = builder.add_hole(Hole::new(pos, bear_r));
        let bolt_hole_1 = builder.add_hole(Hole::new(pos + dir.perp().scale(bear_to_bolt), 2.0));
        let bolt_hole_2 = builder.add_hole(Hole::new(pos - dir.perp().scale(bear_to_bolt), 2.0));
        let slot_for_assemble =
          builder.add_slot(Slot::new_no_border(pos, dir, 40.0, &[(0.0, 40.0)]).width(bear_r * 2.0));
        (bear_hole, bolt_hole_1, bolt_hole_2, slot_for_assemble)
      };

    let (left_bottom_hole, left_bottom_slot_1, left_bottom_slot_2) =
      hole_with_slots(&mut builder, left_bottom_pos, Point { x: -1.0, y: -1.0 }, 7.0);
    let (right_bottom_hole, right_bottom_slot_1, right_bottom_slot_2) =
      hole_with_slots(&mut builder, right_bottom_pos, Point { x: 1.0, y: -1.0 }, 7.0);

    let (left_top_hole, left_top_slot_1, left_top_slot_2) =
      hole_with_slots(&mut builder, Point { x: -70.0, y: 9.0 }, Point { x: 1.0, y: -1.0 }, 4.0);
    let (right_top_hole, right_top_slot_1, right_top_slot_2) =
      hole_with_slots(&mut builder, Point { x: 70.0, y: 9.0 }, Point { x: -1.0, y: -1.0 }, 4.0);

    let (mid_bear_hole, mid_bolt_hole_1, mid_bolt_hole_2, mid_axle_slot) =
      bear_with_holes(&mut builder, mid_pos, Point::Y);
    let (left_bear_hole, left_bolt_hole_1, left_bolt_hole_2, left_axle_slot) =
      bear_with_holes(&mut builder, left_pos, -Point::X);
    let (right_bear_hole, right_bolt_hole_1, right_bolt_hole_2, right_axle_slot) =
      bear_with_holes(&mut builder, right_pos, Point::X);

    let big_ring = builder.add_hole(Hole::new(Point{x:5.0,y:-47.0}, 37.0));

    let left_couple = Point { x: -65.0, y: 25.0 };
    let right_couple = Point { x: 65.0, y: 25.0 };

    let left_upper_couple_slot = builder.add_slot(Slot::new(
      left_couple + Point::Y.scale(10.0),
      -Point::X,
      10.0,
      &[(2.0, 8.0)],
    ));
    let left_upper_couple_slot_cut = builder.add_slot(
      Slot::new_no_border(left_couple + Point::Y.scale(13.0), -Point::X, 10.0, &[(0.0, 10.0)])
        .width(4.0),
    );
    let left_upper_couple_bolt =
      builder.add_hole(Hole::new(left_couple + Point::X.scale(2.0), 2.0));

    let right_upper_couple_slot = builder.add_slot(Slot::new(
      right_couple + Point::Y.scale(10.0),
      Point::X,
      10.0,
      &[(2.0, 8.0)],
    ));
    let right_upper_couple_slot_cut = builder.add_slot(
      Slot::new_no_border(right_couple + Point::Y.scale(13.0), Point::X, 10.0, &[(0.0, 10.0)])
        .width(4.0),
    );
    let right_upper_couple_bolt =
      builder.add_hole(Hole::new(right_couple - Point::X.scale(2.0), 2.0));

    let (balance_bear_hole, balance_bolt_hole_1, balance_bolt_hole_2, balance_axle_slot) =
      bear_with_holes(&mut builder, top_pos, Point::Y);
    let (left_balance_hole, left_balance_slot_1, left_balance_slot_2) =
      hole_with_slots(&mut builder, Point { x: -50.0, y: 40.0 }, Point { x: -1.0, y: 1.0 }, 4.0);
    let (right_balance_hole, right_balance_slot_1, right_balance_slot_2) =
      hole_with_slots(&mut builder, Point { x: 50.0, y: 40.0 }, Point { x: 1.0, y: 1.0 }, 4.0);

    let left_upper_couple_balance_slot = builder.add_slot(
      Slot::new(left_couple + Point::Y.scale(10.0), -Point::X, 10.0, &[(1.9, 8.1)]).width(6.0),
    );
    let left_down_couple_balance_cut =
      builder.add_slot(Slot::new(left_couple, -Point::X, 10.0, &[(2.0, 8.0)]));
    let left_down_couple_bolt_1 =
      builder.add_hole(Hole::new(left_couple + Point { x: 2.0, y: -2.0 }, 2.0));
    let left_down_couple_bolt_m =
      builder.add_slot(Slot::new_no_border(left_couple, Point::X, 4.0, &[(0.0, 4.0)]).width(4.0));
    let left_down_couple_bolt_2 =
      builder.add_hole(Hole::new(left_couple + Point { x: 2.0, y: 2.0 }, 2.0));

    let right_upper_couple_balance_slot = builder.add_slot(
      Slot::new(right_couple + Point::Y.scale(10.0), Point::X, 10.0, &[(1.9, 8.1)]).width(6.0),
    );
    let right_down_couple_balance_cut =
      builder.add_slot(Slot::new(right_couple, Point::X, 10.0, &[(2.0, 8.0)]));
    let right_down_couple_bolt_1 =
      builder.add_hole(Hole::new(right_couple + Point { x: -2.0, y: -2.0 }, 2.0));
    let right_down_couple_bolt_m =
      builder.add_slot(Slot::new_no_border(right_couple, -Point::X, 4.0, &[(0.0, 4.0)]).width(4.0));
    let right_down_couple_bolt_2 =
      builder.add_hole(Hole::new(right_couple + Point { x: -2.0, y: 2.0 }, 2.0));

    let hole2 = builder.add_hole(Hole::new_no_border(Point { x: 5.0, y: 0.0 }, 2.0));

    builder.add_connector(
      Connector::new(&builder, left_bottom_slot_1, 25.8)
        .name("watches_connector_balance".into())
        .count(4),
    );
    builder.add_connector(
      Connector::new(&builder, left_bottom_slot_1, 30.0)
        .name("watches_connector_wide".into())
        .count(8),
    );
    builder.add_connector(
      Connector::new(&builder, left_bottom_slot_1, 25.8)
        .holes(&[hole2])
        .name("watches_connector_balande_holed".into())
        .count(2),
    );
    builder.add_connector(
      Connector::new(&builder, left_bottom_slot_1, 25.8)
        .holes(&[hole2])
        .couple_size(4.0)
        .name("watches_connector_balande_holed_widened".into())
        .count(2),
    );

    builder.add_figure(
      Figure::new(
        &builder,
        &[
          chain![
            left_down_couple_balance_cut,
            left_down_couple_bolt_1,
            left_down_couple_bolt_m,
            left_down_couple_bolt_2,
            left_upper_couple_balance_slot,
            left_balance_hole,
            balance_bear_hole,
            right_balance_hole,
            right_upper_couple_balance_slot,
            right_down_couple_bolt_2,
            right_down_couple_bolt_m,
            right_down_couple_bolt_1,
            right_down_couple_balance_cut,
          ],
          chain![balance_axle_slot],
          chain![balance_bolt_hole_1, balance_bear_hole, balance_bolt_hole_2],
          chain![left_balance_slot_1, left_balance_hole, left_balance_slot_2],
          chain![
            right_balance_slot_1,
            right_balance_hole,
            right_balance_slot_2
          ],
        ],
      )
      .name("watches_balance_cup".into())
      .count(2),
    );

    builder.add_figure(
      Figure::new(
        &builder,
        &[
          contour![
            mid_bear_hole,
            left_upper_couple_slot,
            left_upper_couple_bolt,
            left_top_hole,
            left_bear_hole,
            left_bottom_hole,
            right_bottom_hole,
            right_bear_hole,
            right_top_hole,
            right_upper_couple_bolt,
            right_upper_couple_slot,
          ],
          // necessary stuff to connect bear with holes, make slots etcs
          chain![big_ring],
          chain![left_upper_couple_slot_cut],
          chain![right_upper_couple_slot_cut],
          chain![mid_axle_slot],
          chain![left_axle_slot],
          chain![right_axle_slot],
          chain![left_bolt_hole_1, left_bear_hole, left_bolt_hole_2],
          chain![right_bolt_hole_1, right_bear_hole, right_bolt_hole_2],
          chain![mid_bolt_hole_1, mid_bear_hole, mid_bolt_hole_2],
          chain![left_bottom_slot_1, left_bottom_hole, left_bottom_slot_2],
          chain![right_bottom_slot_1, right_bottom_hole, right_bottom_slot_2],
          chain![left_top_slot_1, left_top_hole, left_top_slot_2],
          chain![right_top_slot_1, right_top_hole, right_top_slot_2],
        ],
      )
      .name("watches_cup".into())
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
    self.builder.contains(current_normal, pos) as PartIndex
  }

  pub fn get_part_index(&self, pos: crate::points3d::Point) -> PartIndex {
    return 0;
  }
}
