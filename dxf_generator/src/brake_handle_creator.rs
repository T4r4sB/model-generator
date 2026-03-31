use common::points2d::*;
use common::solid::*;

use common::slots_and_holes::*;

pub struct BrakeHandleCreator {
  builder: Builder,
}

impl BrakeHandleCreator {
  pub fn new() -> Self {
    let error = 0.05;
    let mut builder = Builder::new(2.0, 2.0, error);
    let cable_pos = Point { x: 0.0, y: 36.5 };
    let cable = builder.add_hole(Hole::new(cable_pos, 3.5).border(4.0));
    let screw_r = 2.5;
    let pipe_r = 3.0;

    let axis =
      vec![Point { x: 4.4, y: 27.0 }, Point { x: 5.4, y: 22.0 }, Point { x: 5.2, y: 17.0 }];

    let cable_slot1 =
      builder.add_slot(Slot::new_no_border(cable_pos, Point::from_angle(PI), 20.0, &[(0.0, 20.0)]));
    let cable_slot2 = builder.add_slot(Slot::new_no_border(
      cable_pos,
      Point::from_angle(-PI * 0.75),
      20.0,
      &[(0.0, 20.0)],
    ));

    let pipe_holes: Vec<_> = axis
      .iter()
      .map(|&position| builder.add_hole(Hole::new(position, pipe_r).border(2.0)))
      .collect();
    let screw_holes: Vec<_> = axis
      .iter()
      .map(|&position| builder.add_hole(Hole::new(position, screw_r).border(2.0)))
      .collect();

    let screw_holes_w: Vec<_> = axis
      .iter()
      .map(|&position| {
        builder.add_hole(Hole::new_solid(Point { x: position.x - 5.0, y: position.y - 4.0 }, 3.0))
      })
      .collect();
    let pipe_holes_p: Vec<_> =
      axis.iter().map(|&position| Point { x: position.x + 5.0, y: position.y + 1.0 }).collect();

    let pipe_holes_w: Vec<_> = pipe_holes_p
      .iter()
      .map(|&position| builder.add_hole(Hole::new_solid(position, 3.0)))
      .collect();

    let handle_screw_far_pos = Point { x: 101.0, y: 50.0 };
    let handle_screw_far = builder.add_hole(Hole::new(handle_screw_far_pos, screw_r).border(2.0));

    let get_handle_position_lo = |x: f32| {
      let mut p0 = handle_screw_far_pos;
      p0.y -= 4.0;
      let p1 = *pipe_holes_p.last().unwrap();
      let l = (p1 - p0).len();
      let mut result = p0 + (p1 - p0).scale(x / l);
      result.y += x * (l - x) / (l * l) * 9.0;
      result
    };

    let get_handle_position_hi = |x: f32| {
      let mut p0 = handle_screw_far_pos;
      p0.y += 0.0;
      let p1 = cable_pos;
      let l = (p1 - p0).len();
      let mut result = p0 + (p1 - p0).scale(x / l);
      result.y -= x * (l - x) / (l * l) * 15.0;
      result
    };

    let handle_controls_hi_pos: Vec<_> =
      [22.0, 44.0, 66.0].iter().map(|&x| get_handle_position_hi(x)).collect();
    let handle_controls_lo_pos: Vec<_> =
      [22.0, 44.0, 66.0].iter().map(|&x| get_handle_position_lo(x)).collect();

    // r_in negative?
    let handle_controls_hi: Vec<_> = handle_controls_hi_pos
      .iter()
      .map(|&position| builder.add_hole(Hole::new_solid(position, 2.5).oval_width(1.5)))
      .collect();

    let handle_controls_lo: Vec<_> = handle_controls_lo_pos
      .iter()
      .map(|&position| builder.add_hole(Hole::new_solid(position, 1.5)))
      .collect();

    let handle_screw_1_pos = get_handle_position_lo(89.0) + Point { x: 0.0, y: 3.0 };
    let handle_screw_2_pos = get_handle_position_hi(92.0);

    let handle_screw_1 = builder.add_hole(Hole::new(handle_screw_1_pos, screw_r).border(4.0));
    let handle_screw_2 = builder.add_hole(Hole::new(handle_screw_2_pos, screw_r).border(4.0));

    let filler1 =
      builder.add_hole(Hole::new_solid((handle_screw_1_pos + handle_screw_2_pos).scale(0.5), 4.0));
    let filler2 = builder.add_hole(Hole::new_solid(
      (handle_controls_hi_pos[2] + handle_controls_lo_pos[2]).scale(0.5),
      3.0,
    ));
    let filler3 = builder.add_hole(Hole::new_solid(
      (handle_controls_hi_pos[1] + handle_controls_lo_pos[1]).scale(0.5),
      3.0,
    ));
    let filler4 = builder.add_hole(Hole::new_solid(
      (handle_controls_hi_pos[0] + handle_controls_lo_pos[0]).scale(0.5),
      3.0,
    ));

    let slot_keeper3_p_hi = Point { x: -26.0, y: 6.0 };
    let slot_keeper3_p_lo = Point { x: -10.0, y: 6.0 };
    let slot_ribbon_p_hi = Point { x: -22.0, y: 8.0 };
    let slot_ribbon_p_lo = Point { x: -10.0, y: 8.0 };

    // r_in 0.5 ???
    let bot1 = builder.add_hole(Hole::new_solid(Point { x: -21.0, y: 5.5 }, 7.5).oval_width(5.5));
    let bot2 = builder.add_hole(Hole::new_solid(Point { x: 16.0, y: 5.5 }, 7.5).oval_width(5.5));

    let cut1 = Point { x: 20.0, y: 0.0 };
    let cut2 = Point { x: 30.0, y: 0.0 };

    //let cable_hole_x = -38.5;
    let cable_hole_x = -19.5;

    let slot_start_p_hi = Point { x: cable_hole_x, y: 45.0 };
    let slot_start_p_hi_s = Point { x: cable_hole_x, y: 37.0 };
    let slot_start_p_lo = Point { x: cable_hole_x, y: 9.0 };

    let slot_start_r_p_hi = Point { x: -8.5, y: 42.0 };
    let slot_start_r_p_lo = Point { x: -8.5, y: 32.0 };

    let slot_start_r_hi =
      builder.add_hole(Hole::new_solid(slot_start_r_p_hi + Point { x: 0.0, y: -4.0 }, 4.0));
    let slot_start_r_lo =
      builder.add_hole(Hole::new_solid(slot_start_r_p_lo + Point { x: 0.0, y: 4.0 }, 4.0));

    let slot_start_hi = builder.add_hole(Hole::new_solid(slot_start_p_hi_s, 4.0));
    let slot_start_lo = builder.add_hole(Hole::new_solid(slot_start_p_lo, 4.0));
    let slot_start_pcut_hi = Point { x: cable_hole_x - 4.0, y: 54.5 };
    let slot_start_pcut_lo = Point { x: cable_hole_x - 4.0, y: 28.5 };

    let down_hole = builder.add_hole(Hole::new(Point { x: 15.0, y: 5.0 }, screw_r).border(2.5));

    let main_screw = builder.add_hole(Hole::new_solid(Point { x: -9.0, y: 29.5 }, 2.0));
    let main_screw_2 = builder
      .add_hole(Hole::new(slot_start_p_hi + Point { x: 4.0, y: -19.0 }, screw_r).border(2.0));

    let main_screw_3 =
      builder.add_hole(Hole::new(Point { x: -12.5, y: 14.5 }, screw_r).border(2.0));

    let cable_slot_x = [(3.0, 13.0), (23.0, 33.0)];
    let ribbon_slot_x = [(0.0, 13.0)];
    let keeper_slot_x = [(2.0, 19.0), (23.0, 36.0)];

    //let slot_of_connectors =
    //  Slot::new(Point { x: 0.0, y: 0.0 }, Point { x: 4.0, y: 0.0 }, 2.0, &[(0.0, 4.0)]);
    //let slot_of_border = Slot::new(slot_start_r_p_hi, slot_start_r_p_lo, 2.0, &[(0.0, 16.0)]);

    let cable_slot = builder.add_slot(Slot::new_no_border(
      slot_start_p_hi,
      -Point::Y,
      (slot_start_p_hi - slot_start_p_lo).len(),
      &cable_slot_x,
    ));
    let react_slot = builder.add_slot(Slot::new_no_border(
      slot_start_r_p_hi,
      -Point::Y,
      (slot_start_r_p_hi - slot_start_r_p_lo).len(),
      &[(3.0, 7.0)],
    ));
    let keeper_slot = builder.add_slot(Slot::new_no_border(
      slot_keeper3_p_hi,
      Point::X,
      (slot_keeper3_p_hi - slot_keeper3_p_lo).len(),
      &keeper_slot_x,
    ));
    let pcut_slot = builder.add_slot(
      Slot::new_no_border(
        slot_start_pcut_hi,
        -Point::Y,
        (slot_start_pcut_hi - slot_start_pcut_lo).len(),
        &[(0.0, 26.0)],
      )
      .width(6.0),
    );
    let ribbon_slot = builder.add_slot(Slot::new_no_border(
      slot_ribbon_p_hi,
      Point::X,
      (slot_ribbon_p_hi - slot_ribbon_p_lo).len(),
      &ribbon_slot_x,
    ));
    let cut_slot = builder.add_slot(
      Slot::new_no_border(cut1, Point::X, (cut1 - cut2).len(), &[(0.0, 10.0)]).width(40.0),
    );

    builder.add_figure(
      Figure::new(
        &builder,
        &[
          contour![
            cable,
            handle_screw_2,
            handle_controls_hi[2],
            handle_controls_hi[1],
            handle_controls_hi[0],
            handle_screw_far,
            handle_controls_lo[0],
            handle_controls_lo[1],
            handle_controls_lo[2],
            handle_screw_1,
            pipe_holes[2],
            pipe_holes[1],
            pipe_holes[0],
            pipe_holes_w[2],
            pipe_holes_w[1],
            pipe_holes_w[0],
          ],
          chain![handle_controls_hi[2], handle_controls_lo[2]],
          chain![handle_controls_hi[1], handle_controls_lo[1]],
          chain![handle_screw_1, handle_controls_hi[2], handle_controls_lo[1]],
          chain![cable_slot1, cable_slot2],
        ],
      )
      .count(4)
      .name("handle_top".to_owned()),
    );

    builder.add_figure(
      Figure::new(
        &builder,
        &[
          contour![
            slot_start_hi,
            slot_start_lo,
            bot1,
            bot2,
            down_hole,
            screw_holes_w[2],
            screw_holes_w[1],
            screw_holes_w[0],
            screw_holes[2],
            screw_holes[1],
            screw_holes[0],
          ],
          chain![main_screw, slot_start_r_lo, slot_start_r_hi],
          chain![main_screw_3, screw_holes[0], main_screw_2],
          chain![main_screw_2, slot_start_hi],
          chain![cable_slot, react_slot, keeper_slot, pcut_slot, ribbon_slot, cut_slot],
        ],
      )
      .count(4)
      .name("plate".to_owned()),
    );
    /*
      Figure::new()
        .chain(&[
          Hole { position: Point { x: -32.0, y: 0.0 }, r_in: 2.5, r_out: 4.5 },
          Hole { position: Point { x: 32.0, y: 0.0 }, r_in: 3.0, r_out: 5.0 },
        ])
        .chain(&[
          Hole { position: Point { x: -32.0, y: 0.0 }, r_in: 0.0, r_out: 4.5 },
          Hole { position: Point { x: 32.0, y: 0.0 }, r_in: 0.0, r_out: 5.0 },
        ]),
      Figure::new()
        .chain(&[
          Hole { position: Point { x: 32.0, y: 0.0 }, r_in: 2.5, r_out: 4.5 },
          Hole { position: Point { x: -32.0, y: 0.0 }, r_in: 3.0, r_out: 5.0 },
        ])
        .chain(&[
          Hole { position: Point { x: 32.0, y: 0.0 }, r_in: 0.0, r_out: 4.5 },
          Hole { position: Point { x: -32.0, y: 0.0 }, r_in: 0.0, r_out: 5.0 },
        ]),
      Figure::new()
        .contour(&[
          Hole { position: Point { x: 0.0, y: -7.5 }, r_in: 0.0, r_out: 4.0 },
          Hole { position: Point { x: 17.0, y: -7.5 }, r_in: 0.0, r_out: 4.0 },
          Hole { position: Point { x: 17.0, y: 7.5 }, r_in: 0.0, r_out: 4.0 },
          Hole { position: Point { x: 0.0, y: 7.5 }, r_in: 0.0, r_out: 4.0 },
        ])
        .slots(&[Slot::new(
          Point { x: 0.0, y: 0.0 },
          Point { x: 17.0, y: 0.0 },
          15.0,
          &[(0.0, 17.0)],
        )]),
      Figure::new().chain(&[
        Hole { position: Point { x: 0.0, y: -10.0 }, r_in: 2.5, r_out: 5.5 },
        Hole { position: Point { x: 0.0, y: 0.0 }, r_in: 2.5, r_out: 5.5 },
        Hole { position: Point { x: 0.0, y: 10.0 }, r_in: 2.5, r_out: 5.5 },
      ]),
    ];
    let connectors = vec![
      Connector::new(10.0, 2.0, 36.0, &cable_slot_x)
        .holes(&[Hole { position: Point { x: 8.0, y: 0.0 }, r_in: 5.0, r_out: 7.0 }])
        .slots(&[slot_of_connectors.clone()]),
      Connector::new(10.0, 2.0, 10.0, &[(3.0, 7.0)])
        .holes(&[Hole { position: Point { x: 5.0, y: 0.0 }, r_in: 2.0, r_out: 3.0 }])
        .slots(&[slot_of_connectors]),
      Connector::new(10.0, 2.0, 38.0, &keeper_slot_x),
    ];

    figures[1] = figures[0].clone();
    figures[1].slots = vec![slot_of_border.clone()];

    if true {
      figures[1] = Figure::new()
        .contour(&[
          cable,
          handle_screw_2,
          handle_controls_hi[2],
          handle_screw_1,
          pipe_holes[2],
          pipe_holes[1],
          pipe_holes[0],
          pipe_holes_w[2],
          pipe_holes_w[1],
          pipe_holes_w[0],
        ])
        .chain(&[
          filler1,
          handle_controls_hi[2],
          handle_controls_hi[1],
          handle_controls_hi[0],
          handle_screw_far,
        ])
        .chain(&[pipe_holes_w[0], handle_screw_1])
        .slots(&[slot_of_border]);
    } else {
      figures[1] = figures[1].clone().chain(&[filler1, handle_controls_hi[2]]);
    }

    figures[2] = figures[1].clone();

    Self { figures, connectors }
    */
    Self { builder }
  }

  pub fn faces(&self) -> usize {
    self.builder.contour_count()
  }

  pub fn aabb(&self, part_index: usize) -> Option<AABB> {
    Some(self.builder.aabb(part_index))
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
    let mut r = self.builder.contains(pos, current_normal);

    const C: f32 = 0.962;
    const S: f32 = 0.274;

    if current_normal == 1 {
      let pos = pos - Point { x: 20.0, y: 33.0 };
      let pos = Point { x: pos.x * C + pos.y * S, y: pos.y * C - pos.x * S };
      if in_riga_logo(pos) {
        r = false;
      }
    } else if current_normal == 2 {
      let mut pos = pos - Point { x: 20.0, y: 33.0 };
      let pos = Point { x: -pos.x * C - pos.y * S, y: pos.y * C - pos.x * S };
      if in_riga_logo(pos) {
        r = false;
      }
    } else if current_normal == 4 || current_normal == 5 {
      if in_riga_logo(pos) {
        r = false;
      }
    }
    r as PartIndex
  }
}
