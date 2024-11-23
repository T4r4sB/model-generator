use crate::points2d::*;
use crate::solid::*;

use crate::slots_and_holes::*;

pub struct ClickboxCreator {
  error: f32,
  builder_1_5: Builder,
  builder_2: Builder,
}

impl ClickboxCreator {
  fn make_clickbox(&mut self) {
    // outer_pipe_length_before: 73mm
    // inner_pipe_length_before: 108mm
    let builder = &mut self.builder_2;

    let drum_pos = Point { x: 38.0, y: 22.0 };
    let drum_hole = builder.add_hole(Hole::new(drum_pos, 1.5));
    let crank1_pos = Point { x: 65.0, y: 29.0 };
    let crank2_pos = Point { x: 48.307, y: 45.0 };

    let roll1 = Point { x: -9.1920, y: -16.1036 };
    let roll2 = Point { x: 6.46, y: -12.0977 };

    fn abp(p1: Point, p2: Point) -> f32 {
      f32::atan2(cross(p1, p2), dot(p1, p2))
    }

    {
      // some calculations
      let drum_stop_points = [
        Point { x: 16.1179, y: -8.0133 },
        Point { x: 14.9118, y: 10.0816 },
        Point { x: 8.9147, y: 17.9033 },
        Point { x: -1.3283, y: 21.9599 },
        Point { x: -19.6819, y: 9.8296 },
        Point { x: -19.9438, y: -1.2494 },
        Point { x: -14.8238, y: -10.2106 },
      ];

      let ca1 = abp(drum_stop_points[1], drum_stop_points[0]);
      let ca2 = abp(drum_stop_points[3], drum_stop_points[1]);
      let ca3 = abp(drum_stop_points[4], drum_stop_points[3]);
      let ca4 = abp(drum_stop_points[6], drum_stop_points[4]);

      fn rotate(pt: Point, cnt: i32) -> Point {
        let mut a = PI / 6.0 * cnt as f32;
        if cnt == 0 {
          a -= 0.1 / 20.0
        };
        let a = Point::from_angle(a);
        complex_mul(pt, a)
      }

      let wp = |r1: Point, r2: Point| {
        let c1 = crank1_pos - drum_pos;
        let a1 = 2.0 * (1.0 / r1.len()).asin();
        let r1lo = complex_mul(r1, Point::from_angle(-a1));
        let r1hi = complex_mul(r1, Point::from_angle(a1));

        let c2 = crank2_pos - drum_pos;
        let a2 = 2.0 * (1.0 / r2.len()).asin();
        let r2lo = complex_mul(r2, Point::from_angle(-a2));
        let r2hi = complex_mul(r2, Point::from_angle(a2));

        let check1 = abp(r2lo + c2, r1lo + c1);
        println!("check1={check1}, ca1={ca1}, diffmm={}", (check1 - ca1) * 20.0);

        let check2 = abp(r2hi + c2, r1lo + c1);
        println!("check1={check2}, ca2={ca2}, diffmm={}", (check2 - ca2) * 20.0);

        let check3 = abp(r2hi + c2, r1hi + c1);
        println!("check3={check3}, ca3={ca3}, diffmm={}", (check3 - ca3) * 20.0);

        let check4 = abp(r2lo + c2, r1hi + c1);
        println!("check4={check4}, ca4={ca4}, diffmm={}", (check4 - ca4) * 20.0);

        let mut result = 0.0;

        result
      };

      wp(roll1, roll2);
    }

    let crank1_center = builder.add_hole(Hole::new(crank1_pos, 1.5));
    let crank2_center = builder.add_hole(Hole::new(crank2_pos, 1.5));
    let crank1_center_big = builder.add_hole(Hole::new(crank1_pos, 3.1));
    let crank2_center_big = builder.add_hole(Hole::new(crank2_pos, 3.1));

    let drum_r = 11.0;
    let cable_r = 2.8;
    let drum_sz = 16.5;

    let bolt_start = builder.add_hole(Hole::new(Point { x: -9.0, y: 14.0 }, 1.5));

    let a0 = -95.0 * PI / 180.0;
    let a1 = a0 - 150.0 * PI / 180.0;

    let cable_mid_center_dist = (sqr(drum_r - cable_r) + sqr(drum_sz + 9.0)).sqrt();

    let cable_mid_hole = builder.add_hole(
      Hole::new(Point::from_angle((a0 + a1) * 0.5).scale(cable_mid_center_dist) + drum_pos, 1.5)
        .border(3.0),
    );

    let cable_slot_lo_r = drum_r + cable_r + 6.0;
    let cable_slot_hi_r = drum_r + cable_r + 6.0;
    let cable_slot_dst_1 = drum_sz + 1.0;
    let cable_slot_dst_2 = drum_sz + 3.0;
    let cable_slot_dst_3 = drum_sz + 5.0;

    let cable_slot_lo_dir = Point::from_angle(a0);
    let cable_slot_hi_dir = Point::from_angle(a1);

    let cable_slot_start_lo = |dist: f32| -> Point {
      cable_slot_lo_dir.scale(cable_slot_lo_r) + cable_slot_lo_dir.perp().scale(dist) + drum_pos
    };

    let cable_slot_start_hi = |dist: f32| -> Point {
      cable_slot_hi_dir.scale(cable_slot_hi_r) - cable_slot_hi_dir.perp().scale(dist) + drum_pos
    };

    let slot_length_lo = |dist: f32| cable_slot_lo_r - dist * ((PI + a1 - a0) * 0.5).tan();
    let slot_length_hi = |dist: f32| cable_slot_hi_r - dist * ((PI + a1 - a0) * 0.5).tan();

    let slot_length_uni_lo = slot_length_lo(cable_slot_dst_1 - 1.0 - self.error) - self.error;
    let slot_length_uni_hi = slot_length_hi(cable_slot_dst_1 - 1.0 - self.error) - self.error;
    let cable_slot_uni_full_lo_p = [(4.0, slot_length_uni_lo)];
    let cable_slot_uni_full_hi_p = [(4.0, 8.0), (11.0, slot_length_uni_hi)];
    let cable_slot_lo_p = [(6.0, 6.0 + 2.0 * cable_r)];
    let cable_slot_hi_p = [(6.0, 6.0 + 2.0 * cable_r)];

    let conn_slot_p = [(2.0, 8.0)];
    let axle_slot_p = [(2.0, 12.0)];
    let axle_slot_p_small = [(3.0, 11.0)];
    let axle_slot_1_p = [(2.0, 8.0), (12.0, 22.0)];
    let bolt_slot_p = [(3.0, 13.0), (23.0, 33.0)];
    let axle_keep_slot_p = [(4.0, 9.0)];

    let create_slot_lo_at = |dst| {
      Slot::new(cable_slot_start_lo(dst), -cable_slot_lo_dir, slot_length_lo(dst), &cable_slot_lo_p)
    };

    let create_slot_hi_at = |dst| {
      Slot::new(cable_slot_start_hi(dst), -cable_slot_hi_dir, slot_length_hi(dst), &cable_slot_hi_p)
    };

    let cable_slot_lo_1 = builder.add_slot(create_slot_lo_at(cable_slot_dst_1));
    let cable_slot_lo_2 = builder.add_slot(create_slot_lo_at(cable_slot_dst_2));
    let cable_slot_lo_3 = builder.add_slot(create_slot_lo_at(cable_slot_dst_3));
    let cable_slot_hi_1 = builder.add_slot(create_slot_hi_at(cable_slot_dst_1));
    let cable_slot_hi_2 = builder.add_slot(create_slot_hi_at(cable_slot_dst_2));
    let cable_slot_hi_3 = builder.add_slot(create_slot_hi_at(cable_slot_dst_3));

    let cable_slot_lo_full = builder.add_slot(
      Slot::new(
        cable_slot_start_lo(cable_slot_dst_2),
        -cable_slot_lo_dir,
        slot_length_uni_lo + 2.0,
        &cable_slot_uni_full_lo_p,
      )
      .width(6.0),
    );
    let cable_slot_hi_full = builder.add_slot(
      Slot::new(
        cable_slot_start_hi(cable_slot_dst_2),
        -cable_slot_hi_dir,
        slot_length_uni_hi + 2.0,
        &cable_slot_uni_full_hi_p,
      )
      .width(6.0),
    );
    
    let cable_slot_hi_full_for_cuts = builder.add_slot(
      Slot::new(
        cable_slot_start_hi(cable_slot_dst_2) - cable_slot_hi_dir.scale(10.0),
        -cable_slot_hi_dir,
        slot_length_uni_hi + 2.0 - 10.0,
        &cable_slot_uni_full_hi_p,
      )
      .width(6.0),
    );

    let cable_slot_lo = builder.add_slot(
      Slot::new(
        cable_slot_start_lo(cable_slot_dst_2),
        -cable_slot_lo_dir,
        slot_length_uni_lo + 2.0,
        &cable_slot_lo_p,
      )
      .width(6.0),
    );

    let cable_slot_hi = builder.add_slot(
      Slot::new(
        cable_slot_start_hi(cable_slot_dst_2),
        -cable_slot_hi_dir,
        slot_length_uni_hi + 2.0,
        &cable_slot_hi_p,
      )
      .width(6.0),
    );

    let conn_start_1_slot = builder.add_slot(Slot::new(
      Point { x: 68.0, y: 8.0 },
      Point::from_angle(PI * 0.25),
      10.0,
      &conn_slot_p,
    ));

    let conn_start_2_slot = builder.add_slot(Slot::new(
      Point { x: 67.0, y: 36.0 },
      Point::from_angle(-PI * 0.25),
      10.0,
      &conn_slot_p,
    ));

    let axle_diam = 9.0;
    let axle_pipe_diam = 5.4;
    let floor1_h = 9.0;
    let floor2_h = 12.0;

    let slot_for_cranks_start = 20.0;
    let slot_for_cranks_length = 32.0;
    let crank_hold_slot_p = [(3.5, 13.5), (18.0, 22.0 + slot_for_cranks_length)];
    let slot_for_cranks = builder.add_slot(
      Slot::new_no_border(
        Point { x: slot_for_cranks_start, y: 1.0 },
        Point::X,
        slot_for_cranks_length,
        &[(0.0, slot_for_cranks_length)],
      )
      .width(4.4),
    );

    let conn_start_3_slot =
      builder.add_slot(Slot::new(Point { x: 30.0, y: 40.0 }, Point::X, 10.0, &conn_slot_p));
    let bolt_keep_slot =
      builder.add_slot(Slot::new(Point { x: -14.0, y: 11.0 }, Point::Y, 36.5, &bolt_slot_p));

    let crank_hold_slot_start = Point { x: 8.0, y: 8.5 };
    let crank_hold_slot_length = 24.0 + slot_for_cranks_length;
    let crank_hold_slot = builder.add_slot(
      Slot::new(crank_hold_slot_start, Point::Y, crank_hold_slot_length, &crank_hold_slot_p)
        .border(3.0),
    );

    let top_pt = crank_hold_slot_start + Point::Y.scale(crank_hold_slot_length + 1.0);
    let left_pt = Point { x: -14.0, y: 47.5 };
    let top_left_pt = top_pt - Point::X.scale(15.0);
    let cup_slot_dir = top_left_pt - left_pt;
    let cup_slot_l = cup_slot_dir.len();

    let left_filler_hole =
      builder.add_hole(Hole::new(left_pt + Point { x: 2.0, y: 0.0 }, 0.0).border(5.0));
    let top_left_hole =
      builder.add_hole(Hole::new(top_left_pt + Point { x: 2.5, y: -4.0 }, 1.5).border(5.0));

    let center_filler_hole_1 =
      builder.add_hole(Hole::new(Point { x: -2.0, y: 35.0 }, 0.0).border(10.0));
    let center_filler_hole_2 =
      builder.add_hole(Hole::new(Point { x: -2.0, y: 50.0 }, 0.0).border(10.0));

    let cup_slot =
      builder.add_slot(Slot::new(left_pt, cup_slot_dir, cup_slot_l, &[(2.0, cup_slot_l - 2.0)]));

    let cup_slot_2 = builder.add_slot(Slot::new(top_left_pt, Point::X, 17.0, &[(4.0, 12.0)]));

    let crank_hold_slot_lo =
      builder.add_slot(Slot::new(crank_hold_slot_start, Point::Y, slot_for_cranks_start, &[]));
    let crank_hold_slot_hi = builder.add_slot(Slot::new(
      crank_hold_slot_start + Point::Y.scale(slot_for_cranks_start + slot_for_cranks_length),
      Point::Y,
      crank_hold_slot_length - slot_for_cranks_start - slot_for_cranks_length,
      &[],
    ));

    let axle_start_1_slot =
      builder.add_slot(Slot::new(Point { x: -17.0, y: 10.0 }, Point::X, 24.0, &axle_slot_1_p));
    let axle_start_2_slot = builder.add_slot(
      Slot::new(Point { x: -9.0, y: 19.0 }, Point::X, 9.0, &axle_keep_slot_p).border(9.0),
    );
    let axle_start_2_slot_shifted = builder.add_slot(
      Slot::new(Point { x: -11.0, y: 19.0 }, Point::X, 9.0, &axle_keep_slot_p).border(9.0),
    );
    let axle_start_3_slot = builder
      .add_slot(Slot::new(Point { x: -7.0, y: 21.0 }, Point::X, 14.0, &axle_slot_p).border(9.0));
    let axle_start_4_slot = builder.add_slot(
      Slot::new(Point { x: -7.0, y: 23.0 }, Point::X, 14.0, &axle_slot_p_small).border(9.0),
    );

    let axle_pipe_hole_slot = builder.add_slot(
      Slot::new_no_border(Point { x: 4.0, y: 2.25 }, Point::X, 6.0, &[(0.0, 6.0)]).width(2.5),
    );

    let axle_hole =
      builder.add_hole(Hole::new_no_border(Point { x: 7.0, y: 0.0 }, axle_diam * 0.5));
    let axle_pipe_hole =
      builder.add_hole(Hole::new_no_border(Point { x: 7.0, y: 0.0 }, axle_pipe_diam * 0.5));
    let axle_hole_1 =
      builder.add_hole(Hole::new_no_border(Point { x: 17.0, y: 0.0 }, axle_diam * 0.5));
    let axle_neck_hole =
      builder.add_hole(Hole::new_no_border(Point { x: 9.0, y: 0.0 }, axle_diam * 0.5 - 0.5));
    let sidebolt_hole = builder.add_hole(Hole::new_no_border(Point { x: 8.0, y: 0.0 }, 2.1));

    let crank_conn = [(2.0, 6.0)];

    struct Crank2Result {
      roll_pos: Point,
      roll_hole: HoleID,
      angle1: f32,
      angle2: f32,
    }

    // outer crank
    let add_crank1_stuff = |builder: &mut Builder, crank2_result: Crank2Result| {
      let outer_crank1_angle1 = -7.0 / 65.0;
      let outer_crank1_angle2 = 9.0 / 65.0;
      let outer_crank1_contact = builder.add_hole(Hole::new_solid(Point { x: 0.0, y: 29.0 }, 3.0));
      let outer_crank1_intermediate0 =
        builder.add_hole(Hole::new_solid(Point { x: 5.0, y: 40.0 }, 3.0));
      let outer_crank1_intermediate1 =
        builder.add_hole(Hole::new_solid(Point { x: 15.0, y: 38.0 }, 3.0));
      let outer_crank1_intermediate2 =
        builder.add_hole(Hole::new_solid(Point { x: 25.0, y: 29.0 }, 3.0));
      let crank1_to_pipe_slot = builder.add_slot(
        Slot::new_no_border(Point { x: 0.0, y: 27.0 }, -Point::Y, 5.0, &[(0.0, 5.0)]).width(10.0),
      );
      let crank1_to_pipe_slot_clamp = builder.add_slot(
        Slot::new_no_border(
          Point { x: -10.0, y: 28.0 },
          Point::X,
          2.0,
          &[(0.0, 7.5), (12.5, 20.0)],
        )
        .width(4.0),
      );

      let outer_crank1_angle1_complex = Point::from_angle(outer_crank1_angle1);
      let crank1_center_rel =
        complex_mul(crank2_pos - crank1_pos, outer_crank1_angle1_complex) + crank1_pos;
      let crank1_roll_rel =
        complex_mul(crank2_result.roll_pos - crank1_pos, outer_crank1_angle1_complex) + crank1_pos;

      let crank2_roll_hole_rel = builder.add_hole_arc(
        HoleArc::new(
          &builder,
          crank2_result.roll_hole,
          3.5,
          crank2_pos,
          crank2_result.angle1,
          crank2_result.angle2,
        )
        .border(0.0),
      );

      let outer_crank1_hole = builder.add_hole_arc(
        HoleArc::new(
          &builder,
          drum_hole,
          1.0,
          crank1_pos,
          outer_crank1_angle1,
          outer_crank1_angle2,
        )
        .border(4.0),
      );

      let slot_rel_crank1 = |builder: &mut Builder, slot_id| {
        builder.add_slot_arc(SlotArc::new_no_border(
          &builder,
          slot_id,
          0.5,
          crank1_pos,
          outer_crank1_angle1,
          outer_crank1_angle2,
        ))
      };

      let crank1_slot_hole = slot_rel_crank1(builder, cable_slot_hi_full_for_cuts);
      let crank1_conn_start_1_hole = slot_rel_crank1(builder, conn_start_1_slot);
      let crank1_conn_start_2_hole = slot_rel_crank1(builder, conn_start_2_slot);
      let crank1_conn_start_3_hole = slot_rel_crank1(builder, conn_start_3_slot);
      let crank1_crank_slot_lo_hole = slot_rel_crank1(builder, crank_hold_slot_lo);
      let crank1_crank_slot_hi_hole = slot_rel_crank1(builder, crank_hold_slot_hi);

      let crank1_center_fat = builder.add_hole(Hole::new(crank1_pos, 1.9).border(4.0));

      let crank1_roll_pos = crank1_pos + roll1;
      let crank1_roll_hole = builder.add_hole(Hole::new(crank1_roll_pos, 2.0));
      let crank1_roll_hole_w = builder.add_hole(Hole::new(crank1_roll_pos, 0.0).border(4.0));

      let crank1_bolt_pos = crank1_pos + Point { x: -0.7608, y: -9.4018 };
      let crank1_bolt_hole = builder.add_hole(Hole::new(crank1_bolt_pos, 2.0));

      let crank1_a = PI * 35.0 / 180.0;
      let crank1_x = Point::from_angle(crank1_a);
      let crank1_y = crank1_x.perp();
      let slot1 = builder.add_slot(Slot::new(
        crank1_bolt_pos + crank1_x.scale(5.0) - crank1_y.scale(4.0),
        crank1_y,
        8.0,
        &crank_conn,
      ));
      let slot2 = builder.add_slot(Slot::new(
        crank1_bolt_pos + crank1_y.scale(5.0) - crank1_x.scale(4.0),
        crank1_x,
        8.0,
        &crank_conn,
      ));

      let crank1_a = -PI * 30.0 / 180.0;
      let crank1_x = Point::from_angle(crank1_a);
      let slot3 = builder.add_slot(Slot::new(
        crank1_bolt_pos - crank1_x.scale(2.0),
        -crank1_x,
        8.0,
        &crank_conn,
      ));

      // crank_out
      builder.add_figure(
        Figure::new(
          &builder,
          &[
            chain![
              outer_crank1_contact,
              outer_crank1_intermediate0,
              outer_crank1_intermediate1,
              outer_crank1_intermediate2,
              outer_crank1_hole,
              crank1_center_fat,
              slot1,
              slot3,
              crank1_bolt_hole,
              slot2,
              crank1_roll_hole_w,
              outer_crank1_hole,
            ],
            chain![crank1_roll_hole],
            chain![crank2_roll_hole_rel],
            chain![crank1_slot_hole],
            chain![crank1_conn_start_1_hole],
            chain![crank1_conn_start_2_hole],
            chain![crank1_conn_start_3_hole],
            chain![crank1_crank_slot_lo_hole],
            chain![crank1_crank_slot_hi_hole],
            chain![crank1_to_pipe_slot],
            chain![crank1_to_pipe_slot_clamp],
          ],
        )
        .name("crank_outer_pipe".into()),
      );

      builder.add_connector(
        Connector::new(&builder, slot1, 9.0).name("crank_outer_pipe_connector".into()).count(3),
      );

      builder.add_figure(
        Figure::new(
          &builder,
          &[
            contour![crank1_center_fat, slot1, slot3, crank1_bolt_hole, slot2, crank1_roll_hole_w,],
            chain![crank1_roll_hole],
            chain![crank2_roll_hole_rel],
            chain![crank1_conn_start_1_hole],
            chain![crank1_conn_start_2_hole],
            chain![crank1_crank_slot_lo_hole],
            chain![crank1_crank_slot_hi_hole],
          ],
        )
        .name("crank_outer_pipe_cup".into()),
      );
    };

    // inner crank
    let add_crank2_stuff = |builder: &mut Builder| -> Crank2Result {
      let outer_crank2_angle1 = -7.0 / 48.3;
      let outer_crank2_angle2 = 9.0 / 48.3;
      let outer_crank2_contact = builder.add_hole(Hole::new_solid(Point { x: 0.0, y: 45.0 }, 3.0));
      let outer_crank2_intermediate0 =
        builder.add_hole(Hole::new_solid(Point { x: 15.0, y: 45.0 }, 3.0));
      let outer_crank2_intermediate1 =
        builder.add_hole(Hole::new_solid(Point { x: 30.0, y: 46.0 }, 3.0));
      let outer_crank2_intermediate2 =
        builder.add_hole(Hole::new_solid(Point { x: 30.0, y: 33.0 }, 3.0));
      let crank2_to_pipe_slot = builder.add_slot(
        Slot::new_no_border(Point { x: 0.0, y: 43.0 }, -Point::Y, 5.0, &[(0.0, 5.0)]).width(10.0),
      );
      let crank2_to_pipe_slot_clamp = builder.add_slot(
        Slot::new_no_border(Point { x: -10.0, y: 43.0 }, Point::X, 2.0, &[(0.0, 7.5)]).width(4.0),
      );

      let outer_crank2_hole = builder.add_hole_arc(
        HoleArc::new(
          &builder,
          drum_hole,
          1.0,
          crank2_pos,
          outer_crank2_angle1,
          outer_crank2_angle2,
        )
        .border(0.0),
      );

      let slot_rel_crank2 = |builder: &mut Builder, slot_id| {
        builder.add_slot_arc(SlotArc::new_no_border(
          &builder,
          slot_id,
          0.5,
          crank2_pos,
          outer_crank2_angle1,
          outer_crank2_angle2,
        ))
      };

      let crank2_slot_hole = slot_rel_crank2(builder, cable_slot_hi_full_for_cuts);
      let crank2_conn_start_1_hole = slot_rel_crank2(builder, conn_start_1_slot);
      let crank2_conn_start_2_hole = slot_rel_crank2(builder, conn_start_2_slot);
      let crank2_conn_start_3_hole = slot_rel_crank2(builder, conn_start_3_slot);
      let crank2_crank_slot_lo_hole = slot_rel_crank2(builder, crank_hold_slot_lo);
      let crank2_crank_slot_hi_hole = slot_rel_crank2(builder, crank_hold_slot_hi);

      let crank2_center_fat = builder.add_hole(Hole::new(crank2_pos, 1.9).border(4.0));

      let crank2_roll_pos = crank2_pos + roll2;
      let crank2_roll_hole = builder.add_hole(Hole::new(crank2_roll_pos, 2.0));
      let crank2_roll_hole_w = builder.add_hole(Hole::new(crank2_roll_pos, 0.0).border(4.0));
      let crank2_bolt_pos = crank2_pos + Point { x: 6.4557, y: -3.5977 };
      let crank2_bolt_hole = builder.add_hole(Hole::new(crank2_bolt_pos, 2.0));

      let crank2_a = PI * 93.0 / 180.0;
      let crank2_x = Point::from_angle(crank2_a);
      let crank2_y = crank2_x.perp();
      let slot1 = builder.add_slot(Slot::new(
        crank2_bolt_pos + crank2_x.scale(5.0) - crank2_y.scale(4.0),
        crank2_y,
        8.0,
        &crank_conn,
      ));
      let slot2 = builder.add_slot(Slot::new(
        crank2_bolt_pos + crank2_y.scale(5.0) - crank2_x.scale(4.0),
        crank2_x,
        8.0,
        &crank_conn,
      ));

      let crank2_a = PI * 10.0 / 180.0;
      let crank2_x = Point::from_angle(crank2_a);
      let slot3 = builder.add_slot(Slot::new(
        crank2_bolt_pos - crank2_x.scale(2.0),
        -crank2_x,
        8.0,
        &crank_conn,
      ).border(3.0));

      builder.add_connector(
        Connector::new(&builder, slot1, 11.5).name("crank_inner_pipe_connector".into()).count(3),
      );
      // crank_out
      builder.add_figure(
        Figure::new(
          &builder,
          &[
            chain![
              outer_crank2_contact,
              outer_crank2_intermediate0,
              outer_crank2_intermediate1,
              crank2_center_fat,
              slot1,
              slot3,
              crank2_bolt_hole,
              slot2,
              crank2_roll_hole_w,
              outer_crank2_intermediate2,
              outer_crank2_intermediate0,
            ],
            chain![crank2_roll_hole],
            chain![outer_crank2_hole],
            chain![crank2_slot_hole],
            chain![crank2_conn_start_1_hole],
            chain![crank2_conn_start_2_hole],
            chain![crank2_conn_start_3_hole],
            chain![crank2_crank_slot_lo_hole],
            chain![crank2_crank_slot_hi_hole],
            chain![crank2_to_pipe_slot],
            chain![crank2_to_pipe_slot_clamp],
          ],
        )
        .name("crank_inner_pipe".into()),
      );

      builder.add_figure(
        Figure::new(
          &builder,
          &[
            contour![crank2_center_fat, slot1, slot3, crank2_bolt_hole, slot2, crank2_roll_hole_w,],
            chain![crank2_roll_hole],
            chain![crank2_conn_start_1_hole],
            chain![crank2_conn_start_2_hole],
            chain![crank2_crank_slot_lo_hole],
            chain![crank2_crank_slot_hi_hole],
          ],
        )
        .name("crank_inner_pipe_cup".into()),
      );

      Crank2Result {
        roll_pos: crank2_roll_pos,
        roll_hole: crank2_roll_hole,
        angle1: outer_crank2_angle1,
        angle2: outer_crank2_angle2,
      }
    };

    

    let crank2_result = add_crank2_stuff(builder);
    add_crank1_stuff(builder, crank2_result);


    
    let cable_hole = builder
      .add_hole(Hole::new_no_border(Point { x: 6.0 + cable_r, y: floor1_h * 0.5 + 6.5 }, cable_r));
    let rope_hole = builder
      .add_hole(Hole::new_no_border(Point { x: 6.0 + cable_r, y: floor1_h * 0.5 + 6.5 }, 1.0));
    let rope_slot = builder.add_slot(Slot::new_no_border(
      Point { x: 6.0 + cable_r, y: floor1_h * 0.5 + 6.5 },
      Point::Y,
      10.0,
      &[(0.0, 10.0)],
    ));

    let outer_crank_slot = builder.add_slot(Slot::new_no_border(
      Point { x: 0.0, y: (floor1_h * 0.5 + 0.5) * 0.5},
      Point::X,
      10.0,
      &[(0.0, 10.0)],
    ).width(floor1_h * 0.5 - 0.5));
    
    let plate_slot = builder.add_slot(Slot::new_no_border(
      Point { x: 0.0, y: floor1_h * 0.5 + 1.0},
      Point::X,
      10.0,
      &[(7.0, 11.0)],
    ));

    // Cable connectors stuff
    builder.add_connector(
      Connector::new(&builder, cable_slot_lo_1, floor1_h)
        .extra_layers_top(&[(floor2_h + 2.0, 4.0, slot_length_lo(cable_slot_dst_1))])
        .holes(&[rope_hole])
        .slots(&[rope_slot])
        .name("cable_connector_1".into())
        .count(1),
    );

    builder.add_connector(
      Connector::new(&builder, cable_slot_lo_2, floor1_h)
        .extra_layers_top(&[(floor2_h + 2.0, 4.0, slot_length_lo(cable_slot_dst_2))])
        .holes(&[cable_hole])
        .slots(&[rope_slot])
        .name("cable_connector_2".into())
        .count(1),
    );

    builder.add_connector(
      Connector::new(&builder, cable_slot_lo_3, floor1_h)
        .extra_layers_top(&[(floor2_h + 2.0, 4.0, slot_length_lo(cable_slot_dst_3))])
        .holes(&[cable_hole])
        .slots(&[rope_slot])
        .name("cable_connector_3".into())
        .count(1),
    );

    // Cable connectors with slot stuff
    builder.add_connector(
      Connector::new(&builder, cable_slot_hi_1, floor1_h)
        .extra_layers_top(&[(floor2_h + 2.0, 4.0, slot_length_hi(cable_slot_dst_1))])
        .holes(&[rope_hole])
        .slots(&[rope_slot, outer_crank_slot, plate_slot])
        .name("cable_connector_1_sloted".into())
        .count(1),
    );

    builder.add_connector(
      Connector::new(&builder, cable_slot_hi_2, floor1_h)
        .extra_layers_top(&[(floor2_h + 2.0, 4.0, slot_length_hi(cable_slot_dst_2))])
        .holes(&[cable_hole])
        .slots(&[rope_slot, outer_crank_slot, plate_slot])
        .name("cable_connector_2_sloted".into())
        .count(1),
    );

    builder.add_connector(
      Connector::new(&builder, cable_slot_hi_3, floor1_h)
        .extra_layers_top(&[(floor2_h + 2.0, 4.0, slot_length_hi(cable_slot_dst_3))])
        .holes(&[cable_hole])
        .slots(&[rope_slot, outer_crank_slot, plate_slot])
        .name("cable_connector_3_sloted".into())
        .count(1),
    );

    // Connectors
    builder.add_connector(
      Connector::new(&builder, conn_start_1_slot, floor1_h + floor2_h)
        .couple_size_top(4.0)
        .name("wide_part_connector".into())
        .count(3),
    );
    builder.add_connector(
      Connector::new(&builder, axle_start_1_slot, floor1_h)
        .holes(&[axle_hole_1])
        .name("axle_hold_connector".into()),
    );
    builder.add_connector(
      Connector::new(&builder, axle_start_2_slot, floor1_h)
        .holes(&[axle_neck_hole])
        .couple_size(4.0)
        .name("axle_keep_connector".into()),
    );
    builder.add_connector(
      Connector::new(&builder, axle_start_3_slot, floor1_h)
        .holes(&[axle_hole])
        .name("axle_hold_connector_inner".into()),
    );
    builder.add_connector(
      Connector::new(&builder, axle_start_4_slot, floor1_h)
        .holes(&[axle_pipe_hole])
        .slots(&[axle_pipe_hole_slot])
        .name("axle_hold_pipe_connector_inner".into()),
    );
    builder.add_connector(
      Connector::new(&builder, bolt_keep_slot, floor1_h)
        .holes(&[sidebolt_hole])
        .name("bolt_keep_connector".into()),
    );
    builder.add_connector(
      Connector::new(&builder, crank_hold_slot, floor1_h)
        .slots(&[slot_for_cranks])
        .name("connector_with_slot_for_cranks".into()),
    );

    builder
      .add_connector(Connector::new(&builder, cup_slot, floor1_h).name("cup_connector".into()));
    builder
      .add_connector(Connector::new(&builder, cup_slot_2, floor1_h).name("cup_connector_2".into()));

    /*


    //plate
    builder.add_figure(
      Figure::new(
        &builder,
        &[
          contour![
            conn_start_3_slot,
            crank2_center,
            conn_start_2_slot,
            crank1_center,
            conn_start_1_slot,
            cable_slot_lo,
            axle_start_1_slot,
            bolt_start,
            bolt_keep_slot,
            cup_slot,
            top_left_hole,
            cup_slot_2,
            crank_hold_slot,
            axle_start_3_slot,
            axle_start_4_slot,
            cable_slot_hi,
          ],
          chain![crank1_center, drum_hole, crank2_center],
          chain![left_filler_hole, center_filler_hole_1, center_filler_hole_2],
          chain![drum_hole, conn_start_1_slot],
          chain![bolt_keep_slot, axle_start_2_slot, axle_start_2_slot_shifted, cable_slot_lo,],
          contour![cable_slot_lo, drum_hole, cable_slot_hi, cable_mid_hole,],
        ],
      )
      .name("plate".into()),
    );

    //intermediate
    builder.add_figure(
      Figure::new(
        &builder,
        &[
          contour![
            cable_slot_lo_full,
            axle_start_1_slot,
            bolt_start,
            bolt_keep_slot,
            cup_slot,
            top_left_hole,
            cup_slot_2,
            crank_hold_slot,
            axle_start_3_slot,
            axle_start_4_slot,
            cable_slot_hi_full,
            cable_mid_hole,
          ],
          chain![bolt_keep_slot, axle_start_2_slot, axle_start_2_slot_shifted, cable_slot_lo_full,],
          chain![left_filler_hole, center_filler_hole_1, center_filler_hole_2],
        ],
      )
      .name("plate_left".into()),
    );

    // pre_cup
    builder.add_figure(
      Figure::new(
        &builder,
        &[chain![
          conn_start_3_slot,
          crank2_center,
          conn_start_2_slot,
          crank1_center,
          conn_start_1_slot,
        ]],
      )
      .name("cup_holder".into()),
    );

    //cup
    builder.add_figure(
      Figure::new(
        &builder,
        &[
          contour![
            conn_start_3_slot,
            crank2_center_big,
            conn_start_2_slot,
            crank1_center_big,
            conn_start_1_slot,
            cable_slot_lo,
            cable_slot_hi
          ],
          chain![crank1_center_big, drum_hole, crank2_center_big],
          chain![drum_hole, conn_start_1_slot],
          contour![cable_slot_lo, drum_hole, cable_slot_hi],
        ],
      )
      .name("cup".into()),
    );

    */
  }

  pub fn make_handle(&mut self) {
    let builder = &mut self.builder_1_5;
    let a = PI * 135.0 / 180.0;
    let r = 30.0 / a;

    let ap = Point::from_angle(a);

    let drum_mid = builder.add_hole(Hole::new(Point::ZERO, 3.1).border(r - 1.0));
    let cable_imitation = builder.add_hole(Hole::new_no_border(Point { x: r + 3.0, y: 0.0 }, 3.5));
    let cable_trajectory =
      builder.add_hole_arc(HoleArc::new(builder, cable_imitation, 0.0, Point::ZERO, 0.0, a + PI));
    let slot_start_1 = builder.add_slot(
      Slot::new_no_border(Point { x: r + 1.0, y: 0.0 }, -Point::Y, 10.0, &[(0.0, 10.0)]).width(6.5),
    );
    let slot_start_2 = builder.add_slot(
      Slot::new_no_border(-ap.scale(r + 1.0), ap.perp(), 10.0, &[(0.0, 10.0)]).width(6.5),
    );

    let slot_start_1_b = builder.add_slot(
      Slot::new(Point { x: r, y: 2.0 }, -Point::Y, 10.0, &[(2.0, 7.0)]).width(4.0).border(3.0),
    );
    let slot_start_2_b = builder.add_slot(
      Slot::new(-ap.scale(r) - ap.perp().scale(2.0), ap.perp(), 10.0, &[(2.0, 7.0)])
        .width(4.0)
        .border(3.0),
    );

    let bolt = builder.add_hole(Hole::new(Point { x: 6.0, y: 6.0 }, 1.25));
    let bolt_op = builder.add_hole(Hole::new(Point { x: -6.0, y: -6.0 }, 1.25));

    let bolt_far = builder.add_hole(Hole::new_solid(Point { x: 10.0, y: -48.0 }, 4.5));
    let bolt_far_keep = builder.add_hole(Hole::new_solid(Point { x: 30.0, y: -48.0 }, 5.5));

    let cup_mid = builder.add_hole(Hole::new(Point::ZERO, 2.5).border(2.0));
    let cup_mid_h = builder.add_hole(Hole::new(Point::ZERO, 2.5).border(8.0));
    let slot_bor_cables = builder.add_slot(Slot::new(
      Point { x: -r - 7.0, y: r + 13.0 },
      Point::X,
      2.0 * (r + 7.0),
      &[(2.0, 12.0), (r + 3.0, r + 11.0), (r * 2.0 + 2.0, r * 2.0 + 12.0)],
    ));

    let slot_bor_cables_1 = builder.add_slot(
      Slot::new_no_border(Point { x: -r - 17.0, y: r + 15.0 }, Point::X, 24.0, &[(0.0, 24.0)])
        .width(2.5),
    );

    let slot_bor_cables_2 = builder.add_slot(
      Slot::new_no_border(Point { x: r + 17.0, y: r + 15.0 }, -Point::X, 24.0, &[(0.0, 24.0)])
        .width(2.5),
    );

    let slot_for_keeper =
      builder.add_slot(Slot::new(Point { x: r + 8.0, y: -23.0 }, Point::Y, 46.0, &[(12.0, 34.0)]));

    let slot_for_ribbon = builder.add_slot(
      Slot::new(Point { x: r + 6.25, y: -8.5 }, Point::Y, 27.0, &[(2.0, 15.0)]).width(2.0),
    );

    let cable_slot_hole1 = builder.add_hole(Hole::new(Point { x: 7.0, y: 0.0 }, 5.0));
    let cable_slot_rope1 = builder.add_slot(Slot::new_no_border(
      Point { x: 7.0, y: 0.0 },
      -Point::X,
      10.0,
      &[(0.0, 10.0)],
    ));
    let cable_slot_hole2 = builder.add_hole(Hole::new(Point { x: 7.0 + r * 2.0, y: 0.0 }, 5.0));
    let cable_slot_rope2 = builder.add_slot(Slot::new_no_border(
      Point { x: 7.0 + r * 2.0, y: 0.0 },
      Point::X,
      10.0,
      &[(0.0, 10.0)],
    ));

    builder.add_connector(
      Connector::new(builder, slot_bor_cables, 10.0)
        .holes(&[cable_slot_hole1, cable_slot_hole2])
      //  .slots(&[cable_slot_rope1, cable_slot_rope2])
        .name("handle_slot_for_cables".into()),
    );
    builder.add_connector(
      Connector::new(builder, slot_for_keeper, 10.0).name("handle_slot_for_keeper".into()),
    );

    builder.add_figure(
      Figure::new(
        &builder,
        &[
          chain![bolt_op, drum_mid, bolt],
          chain![cable_trajectory],
          chain![slot_start_1],
          chain![slot_start_2],
        ],
      )
      .name("handle_drum".into()),
    );

    builder.add_figure(
      Figure::new(
        &builder,
        &[
          chain![bolt_op, drum_mid, bolt],
          chain![slot_start_1_b, bolt_far, bolt_far_keep],
          chain![slot_start_2_b],
        ],
      )
      .name("handle_drum_keep_with_handle".into()),
    );

    builder.add_figure(
      Figure::new(
        &builder,
        &[chain![bolt_op, drum_mid, bolt], chain![slot_start_1_b], chain![slot_start_2_b]],
      )
      .name("handle_drum_keep".into()),
    );

    builder.add_figure(
      Figure::new(
        &builder,
        &[
          contour![
            cup_mid,
            slot_bor_cables.begin(),
            slot_bor_cables,
            slot_for_ribbon,
            slot_for_keeper,
            slot_for_keeper.begin(),
          ],
          chain![cup_mid, slot_for_keeper.end()],
          chain![slot_bor_cables_1],
          chain![slot_bor_cables_2],
          chain![cup_mid_h],
        ],
      )
      .count(2)
      .name("handle_cup".into()),
    );
  }

  pub fn new() -> Self {
    let error = 0.1;
    let mut builder_1_5 = Builder::new(2.0, 1.5, error);
    let mut builder_2 = Builder::new(2.0, 2.0, error);
    let mut result = Self { error, builder_1_5, builder_2 };
    result.make_handle();
   // result.make_clickbox();
    result
  }

  pub fn get_quality() -> usize {
    1
  }

  pub fn get_size() -> f32 {
    1.0
  }

  pub fn faces(&self) -> usize {
    self.builder_1_5.contour_count() + self.builder_2.contour_count()
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    let c1 = self.builder_1_5.contour_count();
    if current_normal < c1 {
      self.builder_1_5.get_material_thickness(current_normal)
    } else {
      self.builder_2.get_material_thickness(current_normal - c1)
    }
  }

  pub fn get_name(&self, current_normal: usize) -> Option<&str> {
    let c1 = self.builder_1_5.contour_count();
    if current_normal < c1 {
      self.builder_1_5.get_name(current_normal)
    } else {
      self.builder_2.get_name(current_normal - c1)
    }
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    let c1 = self.builder_1_5.contour_count();
    if current_normal < c1 {
      self.builder_1_5.get_count(current_normal)
    } else {
      self.builder_2.get_count(current_normal - c1)
    }
  }

  pub fn get_sticker_index(&self, pos: Point, current_normal: usize) -> PartIndex {
    let c1 = self.builder_1_5.contour_count();
    if current_normal < c1 {
      self.builder_1_5.contains(current_normal, pos) as PartIndex
    } else {
      self.builder_2.contains(current_normal - c1, pos) as PartIndex
    }
  }

  pub fn get_part_index(&self, pos: crate::points3d::Point) -> PartIndex {
    return 0;
  }
}
