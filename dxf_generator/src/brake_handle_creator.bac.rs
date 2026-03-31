use crate::points2d::*;
use crate::solid::*;

use slots_and_holes::*;

pub struct BrakeHandleCreator {
  figures: Vec<Figure>,
  connectors: Vec<Connector>,
}

impl BrakeHandleCreator {
  pub fn new() -> Self {
    let cable = Hole { position: Point { x: 0.0, y: 36.5 }, r_in: 3.5, r_out: 3.5 + 4.0 };
    let screw_r = 2.5;
    let pipe_r = 3.0;

    let axis = vec![
      Point { x: 4.4, y: 27.0 },
      Point { x: 5.4, y: 22.0 },
      Point { x: 5.2, y: 17.0 },
    ];

    let cable_slot1 = Point { x: 0.0, y: 36.5 };
    let cable_slot2 = Point { x: -15.0, y: 36.5 };
    let cable_slot3 = Point { x: -15.0, y: 21.5 };

    let pipe_holes: Vec<_> = axis
      .iter()
      .map(|&position| Hole { position, r_in: pipe_r, r_out: pipe_r + 2.0 })
      .collect();
    let screw_holes: Vec<_> = axis
      .iter()
      .map(|&position| Hole { position, r_in: screw_r, r_out: screw_r + 2.0 })
      .collect();

    let screw_holes_w: Vec<_> = axis
      .iter()
      .map(|&position| Hole {
        position: Point { x: position.x - 5.0, y: position.y - 2.0 },
        r_in: 0.0,
        r_out: 3.0,
      })
      .collect();
    let pipe_holes_w: Vec<_> = axis
      .iter()
      .map(|&position| Hole {
        position: Point { x: position.x + 5.0, y: position.y + 1.0 },
        r_in: 0.0,
        r_out: 3.0,
      })
      .collect();

    let handle_screw_far =
      Hole { position: Point { x: 101.0, y: 50.0 }, r_in: screw_r, r_out: screw_r + 2.0 };

    let get_handle_position_lo = |x: f32| {
      let mut p0 = handle_screw_far.position;
      p0.y -= 4.0;
      let p1 = pipe_holes.last().unwrap().position;
      let l = (p1 - p0).len();
      let mut result = p0 + (p1 - p0).scale(x / l);
      result.y += x * (l - x) / (l * l) * 9.0;
      result
    };

    let get_handle_position_hi = |x: f32| {
      let mut p0 = handle_screw_far.position;
      p0.y += 0.0;
      let p1 = cable.position;
      let l = (p1 - p0).len();
      let mut result = p0 + (p1 - p0).scale(x / l);
      result.y -= x * (l - x) / (l * l) * 15.0;
      result
    };

    let handle_controls_hi: Vec<Hole> = [22.0, 44.0, 66.0]
      .iter()
      .map(|&x| Hole { position: get_handle_position_hi(x), r_in: -1.0, r_out: 2.5 })
      .collect();

    let handle_controls_lo: Vec<Hole> = [22.0, 44.0, 66.0]
      .iter()
      .map(|&x| Hole { position: get_handle_position_lo(x), r_in: 0.0, r_out: 1.5 })
      .collect();

    let handle_screw_1 = Hole {
      position: get_handle_position_lo(89.0) + Point { x: 0.0, y: 3.0 },
      r_in: screw_r,
      r_out: screw_r + 3.0,
    };
    let handle_screw_2 =
      Hole { position: get_handle_position_hi(92.0), r_in: screw_r, r_out: screw_r + 3.0 };

    let filler1 = Hole {
      position: (handle_screw_1.position + handle_screw_2.position).scale(0.5),
      r_in: 0.0,
      r_out: 4.0,
    };
    let filler2 = Hole {
      position: (handle_controls_hi[2].position + handle_controls_lo[2].position).scale(0.5),
      r_in: 0.0,
      r_out: 3.0,
    };
    let filler3 = Hole {
      position: (handle_controls_hi[1].position + handle_controls_lo[1].position).scale(0.5),
      r_in: 0.0,
      r_out: 3.0,
    };
    let filler4 = Hole {
      position: (handle_controls_hi[0].position + handle_controls_lo[0].position).scale(0.5),
      r_in: 0.0,
      r_out: 3.0,
    };

    let slot_keeper3_p_hi = Point { x: -26.0, y: 6.0 };
    let slot_keeper3_p_lo = Point { x: -10.0, y: 6.0 };
    let slot_ribbon_p_hi = Point { x: -22.0, y: 8.0 };
    let slot_ribbon_p_lo = Point { x: -10.0, y: 8.0 };

    let bot1 = Hole { position: Point { x: -21.0, y: 6.0 }, r_in: 0.5, r_out: 6.5 };
    let bot2 = Hole { position: Point { x: 16.0, y: 6.0 }, r_in: 0.5, r_out: 6.5 };

    let cut1 = Point { x: 20.0, y: 0.0 };
    let cut2 = Point { x: 30.0, y: 0.0 };

    let slot_start_p_hi = Point { x: -38.5, y: 45.0 };
    let slot_start_p_hi_s = Point { x: -38.5, y: 37.0 };
    let slot_start_p_lo = Point { x: -38.5, y: 9.0 };

    let slot_start_r_p_hi = Point { x: -8.5, y: 42.0 };
    let slot_start_r_p_lo = Point { x: -8.5, y: 32.0 };

    let slot_start_r_hi =
      Hole { position: slot_start_r_p_hi + Point { x: 0.0, y: -4.0 }, r_in: 0.0, r_out: 4.0 };
    let slot_start_r_lo =
      Hole { position: slot_start_r_p_lo + Point { x: 0.0, y: 4.0 }, r_in: 0.0, r_out: 4.0 };

    let slot_start_hi = Hole { position: slot_start_p_hi_s, r_in: 0.0, r_out: 4.0 };
    let slot_start_lo = Hole { position: slot_start_p_lo, r_in: 0.0, r_out: 4.0 };
    let slot_start_pcut_hi = Point { x: -42.5, y: 54.5 };
    let slot_start_pcut_lo = Point { x: -42.5, y: 28.5 };

    let down_hole =
      Hole { position: Point { x: 15.0, y: 5.0 }, r_in: screw_r, r_out: screw_r + 2.5 };

    let main_screw =
      Hole { position: Point { x: -9.0, y: 29.5 }, r_in: screw_r, r_out: screw_r + 2.0 };
    let main_screw_2 = Hole {
      position: slot_start_p_hi + Point { x: 4.0, y: -19.0 },
      r_in: screw_r,
      r_out: screw_r + 2.0,
    };

    let main_screw_3 =
      Hole { position: Point { x: -15.5, y: 14.5 }, r_in: screw_r, r_out: screw_r + 2.0 };

    let cable_slot_x = [(3.0, 13.0), (23.0, 33.0)];
    let ribbon_slot_x = [(0.0, 13.0)];
    let keeper_slot_x = [(2.0, 19.0), (23.0, 36.0)];

    let slot_of_connectors =
      Slot::new(Point { x: 0.0, y: 0.0 }, Point { x: 4.0, y: 0.0 }, 2.0, &[(0.0, 4.0)]);
    let slot_of_border = Slot::new(slot_start_r_p_hi, slot_start_r_p_lo, 2.0, &[(0.0, 16.0)]);

    let mut figures = vec![
      Figure::new()
        .contour(&[
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
        ])
        .chain(&[handle_controls_hi[2], handle_controls_lo[2]])
        .chain(&[handle_controls_hi[1], handle_controls_lo[1]])
        .chain(&[handle_screw_1, handle_controls_hi[2], handle_controls_lo[1]])
        .slots(&[
          Slot::new(cable_slot1, cable_slot2, 2.0, &[(0.0, 15.0)]),
          Slot::new(cable_slot1, cable_slot3, 2.0, &[(0.0, 12.0)]),
          slot_of_border.clone(),
        ]),
      Figure::new(),
      Figure::new(),
      Figure::new()
        .contour(&[
          slot_start_hi,
          slot_start_lo,
          bot1,
          bot2,
          down_hole,
          screw_holes[2],
          screw_holes[1],
          screw_holes[0],
          screw_holes_w[2],
          screw_holes_w[1],
          screw_holes_w[0],
        ])
        .chain(&[main_screw, slot_start_r_lo, slot_start_r_hi])
        .chain(&[main_screw_3, main_screw, slot_start_hi])
        .chain(&[main_screw_2, slot_start_hi])
        .slots(&[
          Slot::new(slot_start_p_hi, slot_start_p_lo, 2.0, &cable_slot_x),
          Slot::new(slot_start_r_p_hi, slot_start_r_p_lo, 2.0, &[(3.0, 7.0)]),
          Slot::new(slot_keeper3_p_hi, slot_keeper3_p_lo, 2.0, &keeper_slot_x),
          Slot::new(slot_start_pcut_hi, slot_start_pcut_lo, 6.0, &[(0.0, 26.0)]),
          Slot::new(slot_ribbon_p_hi, slot_ribbon_p_lo, 2.0, &ribbon_slot_x),
          Slot::new(cut1, cut2, 40.0, &[(0.0, 10.0)]),
        ]),
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
  }

  pub fn faces(&self) -> usize {
    self.figures.len() + self.connectors.len()
  }

  pub fn get_sticker_index(&self, pos: Point, current_normal: usize) -> PartIndex {
    let mut r;
    if current_normal < self.figures.len() {
      r = self.figures[current_normal].contains(pos);

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
    } else {
      r = self.connectors[current_normal - self.figures.len()].contains(pos);
    }
    r as PartIndex
  }

  pub fn get_part_index(&self, pos: crate::points3d::Point) -> PartIndex {
    return 0;
  }
}
