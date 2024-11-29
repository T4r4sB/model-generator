use common::points2d::*;
use common::solid::*;
use num::*;

use common::slots_and_holes::*;

#[derive(Default)]
struct ClickboxParams {
  error: f32,

  y_out: f32,
  y_in: f32,
  drum_radius: f32,
  depth: f32,

  drum_pos: Point,
  roll_radius: f32,
  step_out: f32,
  step_in: f32,

  axle_r: f32,
  braid_r: f32,
  axle_end: f32,
  pipe_r: f32,
  nut_y: f32,

  cable_slot_1_y: f32,
  cable_slot_2_y: f32,

  height: f32,

  f_in: (Point, Vec<Point>),
  f_out: (Point, Vec<Point>),
  angle_between: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct MainPoint {
  center: Point,
  r: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Semiplane {
  n: Point,
  c: f32,
}

fn find_pp(p1: Point, d1: f32, p2: Point, d2: f32) -> Point {
  let delta = p2 - p1;
  let d = delta.len();
  let sub = (sqr(d1) - sqr(d2)) / d;
  let x1 = (sub + d) * 0.5;
  p1 + delta.scale(x1 / d) + delta.perp().scale((sqr(d1) - sqr(x1)).sqrt() / d)
}

fn find_ll(s1: Semiplane, s2: Semiplane, w: f32) -> Point {
  let det = (s1.n.x * s2.n.y - s2.n.x * s1.n.y).recip();
  let x = (s1.c + w) * s2.n.y - (s2.c + w) * s1.n.y;
  let y = s1.n.x * (s2.c + w) - s2.n.x * (s1.c + w);
  Point { x: x * det, y: y * det }
}

fn find_far(p: &[MainPoint], n: Point) -> Semiplane {
  let mut c = -f32::INFINITY;
  for &p in p {
    c = f32::max(c, dot(p.center, n) + p.r);
  }

  Semiplane { n, c }
}

struct LeftSideSlots {
  big: SlotID,
  bot_left: SlotID,
  bot_mid: SlotID,
  top_mid: SlotID,
  top_right: SlotID,
}

struct AxleEndSlots {
  axle_pipe_slot: SlotID,
  axle_end_slot: SlotID,
  axle_hold_slot: SlotID,
  rail_bottom_slot: SlotID,
  rail_mid_slot: SlotID,
  rail_mid_slot_cup: SlotID,
}

impl ClickboxParams {
  fn generate_left_side_slots(&self, builder: &mut Builder, start: Point) -> LeftSideSlots {
    let y1 = self.nut_y;
    let y2 = self.drum_pos.y + (self.drum_radius - self.depth - 1.0) - (self.braid_r + 2.0);
    let y3 = self.drum_pos.y + (self.drum_radius - self.depth - 1.0) + (self.braid_r + 2.0);
    let l1 = y3 - y1;
    let l2 = 2.0 * (self.braid_r + 2.0);
    let l4 = self.axle_end + 5.0 - y1;
    let start_to_screw = self.axle_end - 3.0 - y1;

    let l3 = start.y - y2;

    let cable_real_1_y = self.drum_pos.y - (self.drum_radius - self.depth - 1.0);

    let cable_center_1 = Point { x: cable_real_1_y - y1, y: 12.8 - self.height * 0.5 };
    let cable_center_1t = Point { x: self.cable_slot_1_y - y1, y: self.height * 0.5 };
    let cable_center_1m =
      Point { x: cable_center_1t.x, y: cable_center_1.y + (cable_center_1t.x - cable_center_1.x) };

    let s1m = cable_center_1m - cable_center_1;
    let s1t = cable_center_1m - cable_center_1t;
    let l_add_45_degree = (0.75 + self.error) * (2.0.sqrt() - 1.0) - self.error;
    let s1ml = s1m.len() + l_add_45_degree;
    let s1tl = s1t.len() + l_add_45_degree;

    let cable_center_2 = Point { x: self.cable_slot_2_y - y1, y: 10.8 - self.height * 0.5 };
    let cable_center_2t = Point { x: self.cable_slot_2_y - y1, y: self.height * 0.5 + 2.0 };
    let cable_center_2_rel_mid = cable_center_2 + Point::X.scale(y1 - y2);
    let s2 = cable_center_2t - cable_center_2;
    let s2l = s2.len();

    let cable_hole_1 = builder.add_hole(Hole::new_no_border(cable_center_1, self.braid_r));
    let cable_hole_1c = builder.add_hole(Hole::new_no_border(cable_center_1, 0.75));

    let axle_bolt_center = Point { x: self.axle_end - 3.0 - y1, y: 6.2 - self.height * 0.5 };
    let axle_bolt_hole = builder.add_hole(Hole::new(axle_bolt_center, 2.5));
    let axle_bolt_hole_screw = builder.add_hole(Hole::new(axle_bolt_center, 2.1));

    let cable_slot_1m = builder
      .add_slot(Slot::new_no_border(cable_center_1, s1m.norm(), s1ml, &[(0.0, s1ml)]).width(1.5));
    let cable_slot_1t = builder
      .add_slot(Slot::new_no_border(cable_center_1t, s1t.norm(), s1tl, &[(0.0, s1tl)]).width(1.5));

    let cable_hole_2 = builder.add_hole(Hole::new_no_border(cable_center_2, self.braid_r));
    let cable_hole_2_rel_mid =
      builder.add_hole(Hole::new_no_border(cable_center_2_rel_mid, self.braid_r));
    let cable_hole_2_rel_midc = builder.add_hole(Hole::new_no_border(cable_center_2_rel_mid, 0.75));

    let cable_slot_2 = builder
      .add_slot(Slot::new_no_border(cable_center_2, s2.norm(), s2l, &[(0.0, s2l)]).width(1.5));
    let cable_slot_2_rel_mid = builder.add_slot(
      Slot::new_no_border(cable_center_2_rel_mid, s2.norm(), s2l, &[(0.0, s2l)]).width(1.5),
    );

    let big = builder.add_slot(
      Slot::new(
        Point { x: -self.axle_r - 4.0, y: y1 },
        Point::Y,
        l1,
        &[(2.0, 4.0), (start_to_screw - 2.0, start_to_screw + 2.0), (l1 - l2 + 2.0, l1 - 2.0)],
      )
      .border(2.0),
    );

    let bot_left = builder.add_slot(Slot::new(
      Point { x: -self.axle_r - 8.0, y: y1 },
      Point::Y,
      l4,
      &[(2.0, 4.0), (l4 - 4.0, l4 - 2.0)],
    ));
    let bot_mid = builder.add_slot(Slot::new(
      Point { x: -self.axle_r - 6.0, y: y1 },
      Point::Y,
      l4,
      &[(2.0, 4.0), (l4 - 4.0, l4 - 2.0)],
    ));
    let top_mid = builder.add_slot(Slot::new(
      Point { x: -self.axle_r - 2.0, y: y2 },
      Point::Y,
      l2,
      &[(2.0, l2 - 2.0)],
    ));
    let top_right = builder.add_slot(Slot::new(
      Point { x: -self.axle_r, y: y2 },
      Point::Y,
      l3,
      &[(2.0, l2 - 2.0), (l3 - 6.0, l3 - 2.0)],
    ));

    builder.add_connector(
      Connector::new(builder, bot_left, self.height)
        .holes(&[cable_hole_1, axle_bolt_hole])
        .slots(&[cable_slot_1m, cable_slot_1t])
        .name("bottom_left".to_owned())
        .count(2),
    );
    builder.add_connector(
      Connector::new(builder, big, self.height)
        .holes(&[cable_hole_1c, axle_bolt_hole_screw, cable_hole_2])
        .slots(&[cable_slot_1m, cable_slot_1t, cable_slot_2])
        .name("big_conn".to_owned()),
    );
    builder.add_connector(
      Connector::new(builder, top_mid, self.height)
        .holes(&[cable_hole_2_rel_mid])
        .slots(&[cable_slot_2_rel_mid])
        .name("top_mid".to_owned()),
    );
    builder.add_connector(
      Connector::new(builder, top_right, self.height)
        .holes(&[cable_hole_2_rel_midc])
        .slots(&[cable_slot_2_rel_mid])
        .name("top_right".to_owned()),
    );

    LeftSideSlots { big, bot_left, bot_mid, top_mid, top_right }
  }

  fn generate_axle_end_slots(&self, builder: &mut Builder, top_y: f32) -> AxleEndSlots {
    let axle_pipe_slot = builder.add_slot(Slot::new(
      Point { x: -self.pipe_r - 3.0, y: self.axle_end + 1.0 },
      Point::X,
      2.0 * (self.pipe_r + 3.0),
      &[(3.0, self.pipe_r * 2.0 + 3.0)],
    ));
    let axle_end_slot = builder.add_slot(Slot::new(
      Point { x: -self.axle_r - 2.0, y: self.axle_end - 1.0 },
      Point::X,
      2.0 * (self.axle_r + 2.0),
      &[(2.0, self.axle_r * 2.0 + 2.0)],
    ));
    let axle_hold_slot = builder.add_slot(Slot::new_no_border(
      Point { x: -self.axle_r - 2.0, y: self.axle_end - 3.0 },
      Point::X,
      self.axle_r + 3.0,
      &[(3.0, self.axle_r + 3.0)],
    ));
    let rail_bottom_slot = builder.add_slot(Slot::new(
      Point { x: 0.0, y: self.axle_end + 2.0 },
      Point::Y,
      6.0,
      &[(0.0, 6.0)],
    ));
    let rail_mid_slot_y = self.cable_slot_2_y + 0.75 + 4.0;
    let rail_mid_slot = builder.add_slot(Slot::new(
      Point { x: 0.0, y: rail_mid_slot_y },
      Point::Y,
      6.0,
      &[(0.0, 6.0)],
    ));
    let rail_mid_slot_cup = builder.add_slot(
      Slot::new(Point { x: 0.0, y: rail_mid_slot_y - 4.0 }, Point::Y, 10.0, &[(2.0, 8.0)])
        .border(2.0),
    );

    let axle_pipe_hole = builder.add_hole(Hole::new_no_border(
      Point { x: self.pipe_r + 3.0, y: 6.2 - self.height * 0.5 },
      self.pipe_r,
    ));
    let axle_pipe_outer_crank_slot = builder.add_slot(
      Slot::new_no_border(
        Point { x: self.pipe_r, y: 3.8 - self.height * 0.5 },
        Point::X,
        6.0,
        &[(0.0, 6.0)],
      )
      .width(2.8),
    );
    let axle_pipe_outer_rail_slot = builder.add_slot(Slot::new_no_border(
      Point { x: self.pipe_r + 2.0, y: -1.0 - self.height * 0.5 },
      Point::X,
      2.0,
      &[(0.0, 2.0)],
    ));
    let axle_pipe_connector = builder.add_connector(
      Connector::new(builder, axle_pipe_slot, self.height)
        .holes(&[axle_pipe_hole])
        .slots(&[axle_pipe_outer_crank_slot, axle_pipe_outer_rail_slot])
        .name("pipe_connector".to_owned()),
    );

    let axle_end_hole = builder.add_hole(Hole::new_no_border(
      Point { x: self.axle_r + 2.0, y: 6.2 - self.height * 0.5 },
      self.axle_r,
    ));
    let axle_end_connector = builder.add_connector(
      Connector::new(builder, axle_end_slot, self.height)
        .holes(&[axle_end_hole])
        .name("axle_end_connector".to_owned()),
    );

    let axle_hold_slot_for_conn = builder.add_slot(Slot::new(
      Point { x: -self.axle_r - 2.0, y: self.axle_end - 3.0 },
      Point::X,
      self.axle_r + 2.0,
      &[(4.0, self.axle_r + 2.0)],
    ));
    let axle_hold_hole = builder.add_hole(Hole::new_no_border(
      Point { x: self.axle_r + 2.0, y: 6.2 - self.height * 0.5 },
      self.axle_r - 0.5,
    ));
    let axle_hold_connector = builder.add_connector(
      Connector::new(builder, axle_hold_slot_for_conn, self.height - 0.4)
        .holes(&[axle_hold_hole])
        .name("axle_hold_connector".to_owned())
        .couple_size_bottom(4.0)
        .couple_size_top(6.0),
    );

    let rsl = top_y + 2.0 - self.axle_end;
    let rail_slot_base =
      builder.add_slot(Slot::new(Point { x: 0.0, y: self.axle_end }, Point::Y, rsl, &[]));
    let rail_slot_couple_top = builder.add_slot(Slot::new_no_border(
      Point { x: rsl - 1.0, y: -self.height * 0.5 },
      Point::Y,
      self.height,
      &[(0.0, 2.0), (self.height - 4.0, self.height)],
    ));

    let irsl = rsl - 2.0;
    let rail_inner_pipe_slot = builder.add_slot(
      Slot::new_no_border(
        Point { x: 0.0, y: 6.2 - self.height * 0.5 },
        Point::X,
        irsl,
        &[(0.0, irsl)],
      )
      .width(2.8),
    );

    let orsl = rsl + (self.y_out + self.step_out + 3.0) - (self.y_in + self.step_in);
    let rail_outer_pipe_slot = builder.add_slot(Slot::new_no_border(
      Point { x: 0.0, y: 3.8 - self.height * 0.5 },
      Point::X,
      orsl,
      &[(0.0, orsl)],
    ));

    let rail_conn_bot_slot = builder.add_slot(Slot::new_no_border(
      Point { x: 1.0, y: -self.height * 0.5 },
      Point::Y,
      6.2,
      &[(0.0, 6.2)],
    ));

    let mid_slot_y = rail_mid_slot_y - self.axle_end - 2.0;
    let rail_conn_top_slot = builder.add_slot(Slot::new_no_border(
      Point { x: mid_slot_y - 2.0, y: self.height * 0.5 + 1.0 },
      Point::X,
      4.0,
      &[(0.0, 4.0)],
    ));

    let cutl = self.cable_slot_2_y + 0.75 - self.axle_end;
    let rail_conn_cable_cut = builder.add_slot(
      Slot::new_no_border(Point { x: 0.0, y: self.height * 0.5 }, Point::X, cutl, &[(0.0, cutl)])
        .width(self.height * 2.0 - 6.2),
    );

    let mut rhomb = |p: Point, r: f32| {
      let dir = Point::from_angle(PI * 0.25);
      let start = p - dir.scale(r);
      builder.add_slot(Slot::new(start, dir, r * 2.0, &[(0.0, r * 2.0)]))
    };

    let diag1 = rhomb(Point { x: cutl, y: 7.6 - self.height * 0.5 }, 1.0);
    let diag2 = rhomb(Point { x: orsl, y: 4.8 - self.height * 0.5 }, 1.0);

    let rail_connector = builder.add_connector(
      Connector::new(builder, rail_slot_base, self.height)
        .extra_layers_bottom(&[(2.0, 0.0, 8.0)])
        .extra_layers_top(&[(4.0, mid_slot_y, mid_slot_y + 6.0)])
        .slots(&[
          rail_slot_couple_top,
          rail_inner_pipe_slot,
          rail_outer_pipe_slot,
          rail_conn_bot_slot,
          rail_conn_top_slot,
          rail_conn_cable_cut,
          diag1,
          diag2,
        ])
        .name("rail_connector".to_owned()),
    );

    AxleEndSlots {
      axle_pipe_slot,
      axle_end_slot,
      axle_hold_slot,
      rail_bottom_slot,
      rail_mid_slot,
      rail_mid_slot_cup,
    }
  }

  fn find_crank_center(&self, y: f32, step: f32, ed: f32) -> (Point, Vec<Point>) {
    let a = 1.0 - sqr(self.depth / step);
    let p = self.drum_pos.x;
    let c = sqr(self.drum_pos.x) + sqr(self.drum_pos.y - y)
      - sqr(self.drum_radius + self.roll_radius - ed);
    let x = (p + (sqr(p) - a * c).sqrt()) / a;

    let crank_center = Point { x, y };
    let crank_length = x * self.depth / step;
    let roll_center_1 = find_pp(
      self.drum_pos,
      self.drum_radius + self.roll_radius - self.depth,
      crank_center,
      crank_length,
    );
    let roll_center_2 =
      find_pp(self.drum_pos, self.drum_radius + self.roll_radius, crank_center, crank_length);
    let roll_center_3 = find_pp(
      self.drum_pos,
      self.drum_radius + self.roll_radius + self.depth,
      crank_center,
      crank_length,
    );
    let roll_center_o = find_pp(
      self.drum_pos,
      self.drum_radius + self.roll_radius + self.depth + 0.3,
      crank_center,
      crank_length,
    );
    (crank_center, vec![roll_center_1, roll_center_2, roll_center_3, roll_center_o])
  }

  fn get_angle_between(a: Point, b: Point) -> f32 {
    f32::atan2(a.x * b.y - a.y * b.x, a.x * b.x + a.y * b.y)
  }

  fn make_clickbox(&self, builder: &mut Builder) {
    let d = MainPoint { center: self.drum_pos, r: self.drum_radius + self.depth + 0.5 };
    let cr_in = MainPoint { center: self.f_in.0, r: 4.0 };
    let r0_in = MainPoint { center: self.f_in.1[0], r: 4.0 };
    let r1_in = MainPoint { center: self.f_in.1[1], r: 4.0 };
    let r2_in = MainPoint { center: self.f_in.1[2], r: 4.0 };
    let r3_in = MainPoint { center: self.f_in.1[3], r: 4.0 };
    let cr_out = MainPoint { center: self.f_out.0, r: 4.0 };
    let r0_out = MainPoint { center: self.f_out.1[0], r: 4.0 };
    let r1_out = MainPoint { center: self.f_out.1[1], r: 4.0 };
    let r2_out = MainPoint { center: self.f_out.1[2], r: 4.0 };
    let r3_out = MainPoint { center: self.f_out.1[3], r: 4.0 };
    let top = MainPoint {
      center: Point { x: -self.axle_r + 1.0, y: self.y_in + self.step_in + 5.0 },
      r: 0.0,
    };
    let left = MainPoint { center: Point { x: -self.axle_r - 3.0, y: self.nut_y + 4.0 }, r: 0.0 };

    let p1 = find_far(&[d, r0_out, r1_out, r2_out, r3_out, cr_out], Point::from_angle(-PI * 0.5));
    let p2 = find_far(&[d, r0_out, r1_out, r2_out, r3_out, cr_out], Point::from_angle(-PI * 0.3));
    let p3 = find_far(
      &[r0_out, r1_out, r2_out, r3_out, cr_out, r0_in, r1_in, r2_in, r3_in, cr_in],
      Point::from_angle(-PI * 0.1),
    );
    let p4 = find_far(&[cr_out, r0_in, r1_in, r2_in, r3_in, cr_in], Point::from_angle(PI * 0.1));
    let p5 =
      find_far(&[cr_out, r0_in, r1_in, r2_in, r3_in, cr_in, top], Point::from_angle(PI * 0.3));
    let p6 =
      find_far(&[cr_out, r0_in, r1_in, r2_in, r3_in, cr_in, top], Point::from_angle(PI * 0.4));
    let p7 = find_far(&[top], Point::from_angle(PI * 0.5));
    let p8 = find_far(&[top], Point::from_angle(PI));
    let p9 = find_far(&[left], Point::from_angle(-PI * 0.5));
    let p10 = find_far(&[d], Point::from_angle(-PI * 0.7));

    let sps = [p1, p2, p3, p4, p5, p6, p7, p8, p9, p10];
    let corners: Vec<Point> = (0..sps.len())
      .map(|i| {
        let cur = sps[i];
        let next = sps[(i + 1) % sps.len()];
        find_ll(cur, next, 1.0)
      })
      .collect();

    let holes: Vec<_> = (0..corners.len())
      .map(|i| {
        let cur = corners[i];
        builder.add_hole(Hole::new_solid(cur, 2.0))
      })
      .collect();

    let dots: Vec<f32> = (0..corners.len())
      .map(|i| {
        let cur = corners[i];
        let prev = corners[(i + corners.len() - 1) % corners.len()];
        let next = corners[(i + 1) % corners.len()];
        dot((prev - cur).norm(), (next - cur).norm())
      })
      .collect();

    let mut left_side_slots = None;

    let mut slots = Vec::new();
    let mut slots_no_border = Vec::new();

    for i in 0..corners.len() {
      let cur = corners[i];
      let next = corners[(i + 1) % corners.len()];

      let dir = (next - cur).norm();

      let curd = dots[i];
      let nextd = dots[(i + 1) % dots.len()];

      let mut start =
        if curd < -0.75 { cur } else { cur + dir.scale(1.0 / (1.0 - curd * curd).sqrt()) };
      let finish = if nextd < -0.75 { next } else { next + dir };

      let bottom_axle_slot = start.x < 0.0
        && start.y < self.nut_y + 4.0
        && finish.x > 0.0
        && finish.y < self.nut_y + 4.0;

      if bottom_axle_slot {
        start.x -= 4.0;
      }
      let top_slot =
        start.x > 0.0 && start.y > self.axle_end && finish.x < 0.0 && finish.y > self.axle_end;

      let l = (finish - start).len();

      let prot_very_short = [(2.2, l - 2.2)];
      let prot_short = [(2.0, l - 2.0)];
      let prot_long = [(2.0, 8.0), (l - 8.0, l - 2.0)];
      let prot = if l > 17.0 {
        prot_long.as_slice()
      } else if l > 7.55 {
        prot_short.as_slice()
      } else {
        prot_very_short.as_slice()
      };

      let pipe_keep_slot = cur.x < 0.0 && next.x < 0.0;

      let border = if bottom_axle_slot { 2.0 } else { 1.0 };

      if pipe_keep_slot {
        let lss = self.generate_left_side_slots(builder, start);
        slots.push(lss.big);
        left_side_slots = Some(lss);
      } else {
        println!("slot {i} len={l}");
        let s = Slot::new(start, dir, l, prot).border(border);
        let result = builder.add_slot(s.clone());
        slots.push(result);
        slots_no_border.push(builder.add_slot(s.border(0.0)));

        if bottom_axle_slot {
          let h = builder
            .add_hole(Hole::new(Point { x: -start.x, y: 6.2 - self.height * 0.5 }, self.axle_r));
          builder.add_connector(
            Connector::new(builder, result, self.height).holes(&[h]).name(format!("conn_axle_{i}")),
          );
        } else if top_slot {
          let l = self.height - 4.0;
          let s = builder.add_slot(Slot::new(
            Point { x: start.x, y: 2.0 - self.height * 0.5 },
            Point::Y,
            l,
            &[(0.0, l)],
          ));
          builder.add_connector(
            Connector::new(builder, result, self.height).slots(&[s]).name(format!("conn_rail_{i}")),
          );
        } else {
          builder
            .add_connector(Connector::new(builder, result, self.height).name(format!("conn_{i}")));
        }
      }
    }

    let left_side_slots = left_side_slots.unwrap();
    let axle_end_slots = self.generate_axle_end_slots(builder, top.center.y);

    let cable_slots_x = -self.axle_r - 10.0;
    let cable_slots_l = self.drum_pos.x - cable_slots_x;

    let cable_slot_1 = builder.add_slot(
      Slot::new_no_border(
        Point { x: cable_slots_x, y: self.cable_slot_1_y },
        Point::X,
        cable_slots_l,
        &[(0.0, cable_slots_l)],
      )
      .width(1.5),
    );
    let cable_slot_2 = builder.add_slot(
      Slot::new_no_border(
        Point { x: cable_slots_x, y: self.cable_slot_2_y },
        Point::X,
        cable_slots_l,
        &[(0.0, cable_slots_l)],
      )
      .width(1.5),
    );

    let cable_slot_1_cup = builder.add_slot(
      Slot::new(
        Point { x: -self.axle_r - 9.0, y: self.cable_slot_1_y },
        Point::X,
        cable_slots_l,
        &[],
      )
      .width(0.0)
      .border(5.0),
    );
    let cable_slot_2_cup = builder.add_slot(
      Slot::new(
        Point { x: -self.axle_r - 3.0, y: self.cable_slot_2_y },
        Point::X,
        cable_slots_l,
        &[],
      )
      .width(0.0)
      .border(5.0),
    );

    let dh = builder.add_hole(Hole::new(self.drum_pos, 1.5));
    let cr1h = builder.add_hole(Hole::new(self.f_in.0, 1.5));
    let cr2h = builder.add_hole(Hole::new(self.f_out.0, 1.5));

    let cr1h_cup = builder.add_hole(Hole::new(self.f_in.0, 3.0).border(3.0));
    let cr2h_cup = builder.add_hole(Hole::new(self.f_out.0, 3.0).border(3.0));
    let cr1h_view_cup = builder.add_hole(Hole::new_no_border(self.f_in.0, 3.0));
    let cr2h_view_cup = builder.add_hole(Hole::new_no_border(self.f_out.0, 3.0));

    let ri0 = builder.add_hole(Hole::new(self.f_in.1[0], 3.5));
    let ri1 = builder.add_hole(Hole::new(self.f_in.1[1], 3.5));
    let ri2 = builder.add_hole(Hole::new(self.f_in.1[2], 3.5));
    let ri3 = builder.add_hole(Hole::new(self.f_in.1[3], 3.5));
    let ro0 = builder.add_hole(Hole::new(self.f_out.1[0], 3.5));
    let ro1 = builder.add_hole(Hole::new(self.f_out.1[1], 3.5));
    let ro2 = builder.add_hole(Hole::new(self.f_out.1[2], 3.5));
    let ro3 = builder.add_hole(Hole::new(self.f_out.1[3], 3.5));

    let cup_keeper_top_place =
      find_pp(self.f_out.0, 7.0, self.drum_pos, self.drum_radius + self.depth + 5.0);
    let cup_keeper_top_hole = builder.add_hole(Hole::new(cup_keeper_top_place, 1.5).border(3.0));

    let drum_place =
      builder.add_hole(Hole::new(self.drum_pos, self.drum_radius + self.depth + 0.5 - 2.0));

    let drum_place_cup =
      builder.add_hole(Hole::new(self.drum_pos, 1.5).border(self.drum_radius + self.depth + 1.0));
    let drum_place_view_cup = builder.add_hole(Hole::new(self.drum_pos, 1.5).border(3.0));

    let mid_keep_bolt_hole =
      builder.add_hole(Hole::new(Point { x: -self.axle_r - 1.5, y: self.axle_end + 3.5 }, 1.5));

    let top_keep_bolt_hole = builder
      .add_hole(Hole::new(Point { x: -self.axle_r - 1.5, y: self.nut_y + 7.5 }, 1.5).border(2.0));

    let top_hole = builder.add_hole(
      Hole::new(
        Point {
          x: -self.axle_r - 2.5,
          y: self.drum_pos.y + (self.drum_radius - self.depth - 1.0) + self.braid_r + 3.5,
        },
        1.5,
      )
      .border(2.0),
    );

    let mut add_crank_in_stuff = |builder: &mut Builder, err: f32| {
      let rot_center = self.f_in.0;
      let roll_center = self.f_in.1[1];
      let crank_len = (roll_center - rot_center).len();
      let crank_llen = rot_center.x;
      let erra = err / crank_llen;

      let contact_pos = Point::Y.scale(self.y_in);
      let contact = builder
        .add_hole(Hole::new_solid(contact_pos + Point::Y.scale(3.0), 7.0).rotate(rot_center, erra));
      let contact_down = builder
        .add_hole(Hole::new_solid(contact_pos - Point::Y.scale(7.0), 7.0).rotate(rot_center, erra));

      let rot_hole = builder.add_hole(Hole::new(rot_center, 1.95).border(3.0));
      let roll_hole = builder.add_hole(Hole::new(roll_center, 1.5).border(3.0));

      let a0 = -(self.depth + 0.3) / crank_len;
      let a1 = (self.depth + 0.0) / crank_len;
      let a1s = a1 + erra;

      let inner_pipe = builder.add_slot(
        Slot::new_no_border(
          Point { x: -0.5, y: self.y_in - 0.4 - err },
          -Point::Y,
          20.0,
          &[(0.0, 20.0)],
        )
        .width(2.7),
      );

      let inner_pipe = builder
        .add_slot_arc(SlotArc::new_no_border(builder, inner_pipe, 0.4, rot_center, 0.0, a1s));

      let outer_pipe = builder.add_slot(
        Slot::new_no_border(
          Point::Y.scale(self.y_out + self.step_out + 1.0 - err),
          -Point::Y,
          20.0,
          &[(0.0, 20.0)],
        )
        .width(self.pipe_r),
      );

      let outer_pipe =
        builder.add_slot_arc(SlotArc::new_no_border(builder, outer_pipe, 1.0, rot_center, a0, a1s));
      let rot_hole = builder.add_hole(Hole::new(rot_center, 1.95).border(3.0));
      let roll_hole = builder.add_hole(Hole::new(roll_center, 1.5).border(3.0));

      let drum_hole =
        builder.add_hole(Hole::new_no_border(self.drum_pos, self.drum_radius - self.depth + 0.4));
      let drum_arc =
        builder.add_hole_arc(HoleArc::new(builder, drum_hole, 0.0, rot_center, a0, a1));

      let screw_hole = builder.add_hole(Hole::new_no_border(cup_keeper_top_place, 1.9));
      let screw_arc =
        builder.add_hole_arc(HoleArc::new(builder, screw_hole, 0.0, rot_center, a0, a1));

      let s3b =
        builder.add_slot_arc(SlotArc::new_no_border(builder, slots[3], 0.0, rot_center, a0, a1s));
      let s4b =
        builder.add_slot_arc(SlotArc::new_no_border(builder, slots[4], 0.4, rot_center, a0, a1s));
      let s5b =
        builder.add_slot_arc(SlotArc::new_no_border(builder, slots[5], 0.0, rot_center, a0, a1s));
      let s6b = builder.add_slot_arc(SlotArc::new_no_border(
        builder,
        left_side_slots.top_right,
        0.4,
        rot_center,
        a0,
        a1s,
      ));

      let dir = complex_mul((contact_pos - rot_center).norm(), Point::from_angle(erra - 0.1));

      let alt_hole_center = rot_center + dir.scale(9.5);
      let alt_hole_data = Hole::new_no_border(alt_hole_center, 3.5);
      let alt_hole = builder.add_hole(Hole::new(alt_hole_center, 1.5).border(4.0));

      let alt_slot_1 = builder
        .add_slot(Slot::new(rot_center + dir.scale(2.0), dir, 6.0, &[(1.0, 5.0)]).border(4.0));
      let alt_slot_2 = builder.add_slot(
        Slot::new(
          rot_center + dir.scale(12.0) - dir.perp().scale(5.0),
          dir.perp(),
          10.0,
          &[(3.0, 7.0)],
        )
        .border(4.0),
      );

      let dir = (rot_center - roll_center).norm();

      let alt_slot_3 = builder.add_slot(
        Slot::new(
          roll_center + dir.scale(5.0) - dir.perp().scale(5.0),
          dir.perp(),
          10.0,
          &[(3.0, 7.0)],
        )
        .border(4.0),
      );

      builder.add_figure(
        Figure::new(
          builder,
          &[
            chain![roll_hole, alt_slot_3, rot_hole, alt_hole, contact, contact_down],
            chain![alt_slot_1, alt_slot_2],
            chain![inner_pipe, outer_pipe],
            chain![drum_arc, screw_arc],
            chain![s3b, s4b, s5b, s6b],
          ],
        )
        .name(format!("crank_in_err{}", err)),
      );
      builder.add_figure(
        Figure::new(
          builder,
          &[
            chain![roll_hole, alt_slot_3, rot_hole, alt_hole],
            chain![alt_slot_1, alt_slot_2],
            chain![drum_arc, screw_arc],
            chain![s3b, s4b, s5b, s6b],
          ],
        )
        .name(format!("crank_in_cup_err{}", err)),
      );
      builder.add_connector(
        Connector::new(builder, alt_slot_1, 3.3).count(3).name("crank_in_connector".into()),
      );

      builder.add_hole(alt_hole_data.rotate(rot_center, a1))
    };

    let mut add_crank_out_stuff = |builder: &mut Builder, err: f32, in_result: HoleID| {
      let rot_center = self.f_out.0;
      let roll_center = self.f_out.1[1];
      let crank_len = (roll_center - rot_center).len();
      let crank_llen = rot_center.x;
      let erra = err / crank_llen;
      let contact_pos = Point::Y.scale(self.y_out);
      let contact = builder
        .add_hole(Hole::new_solid(contact_pos + Point::Y.scale(8.5), 8.5).rotate(rot_center, erra));
      let contact_up = builder.add_hole(
        Hole::new_solid(contact_pos + Point::Y.scale(18.5), 6.5).rotate(rot_center, erra),
      );
      let contact_s1 = builder.add_hole(
        Hole::new_no_border(contact_pos - Point::Y.scale(1.0), 1.0).rotate(rot_center, erra),
      );

      let contact_s1 = builder.add_hole_arc(HoleArc::new(
        builder,
        contact_s1,
        0.0,
        contact_pos + Point::Y.scale(10.0),
        -0.5,
        0.2,
      ));

      let a0 = -(self.depth + 0.3) / crank_len;
      let a1 = (self.depth + 0.0) / crank_len;
      let a1s = a1 + erra;

      let contact_s2_l = builder.add_slot(Slot::new_no_border(
        Point { x: -self.pipe_r - 5.0, y: self.axle_end + 1.0 - err },
        Point::X,
        5.0,
        &[(0.0, 5.0)],
      ));
      let contact_s2_l = builder.add_slot_arc(SlotArc::new_no_border(
        builder,
        contact_s2_l,
        0.4,
        rot_center,
        a0,
        a1,
      ));
      let contact_s2_r = builder.add_slot(Slot::new_no_border(
        Point { x: self.pipe_r + 5.0, y: self.axle_end + 1.0 - err },
        -Point::X,
        5.0,
        &[(0.0, 5.0)],
      ));
      let contact_s2_r = builder.add_slot_arc(SlotArc::new_no_border(
        builder,
        contact_s2_r,
        0.4,
        rot_center,
        a0,
        a1,
      ));

      let contact_s2_i = builder.add_hole(
        Hole::new_solid(contact_pos + Point { x: 7.15, y: 5.15 }, 3.0).rotate(rot_center, erra),
      );

      let rot_hole = builder.add_hole(Hole::new(rot_center, 1.95).border(3.0));
      let roll_hole = builder.add_hole(Hole::new(roll_center, 1.5).border(3.0));

      let drum_hole =
        builder.add_hole(Hole::new_no_border(self.drum_pos, self.drum_radius - self.depth + 0.4));
      let drum_arc =
        builder.add_hole_arc(HoleArc::new(builder, drum_hole, 0.0, rot_center, a0, a1));

      let drum_screw_hole = builder.add_hole(Hole::new_no_border(self.drum_pos, 2.4));
      let drum_screw_arc = builder
        .add_hole_arc(HoleArc::new(builder, drum_screw_hole, 0.0, rot_center, a0, a1).border(3.0));

      let screw_hole = builder.add_hole(Hole::new_no_border(cup_keeper_top_place, 1.9));
      let screw_arc =
        builder.add_hole_arc(HoleArc::new(builder, screw_hole, 0.0, rot_center, a0, a1));

      let s0b =
        builder.add_slot_arc(SlotArc::new_no_border(builder, slots[0], 0.0, rot_center, a0, a1s));
      let s1b =
        builder.add_slot_arc(SlotArc::new_no_border(builder, slots[1], 0.0, rot_center, a0, a1s));
      let s2b =
        builder.add_slot_arc(SlotArc::new_no_border(builder, slots[2], 0.4, rot_center, a0, a1s));
      let s3b =
        builder.add_slot_arc(SlotArc::new_no_border(builder, slots[3], 0.4, rot_center, a0, a1s));
      let s4b =
        builder.add_slot_arc(SlotArc::new_no_border(builder, slots[4], 0.4, rot_center, a0, a1s));
      let s5b =
        builder.add_slot_arc(SlotArc::new_no_border(builder, slots[5], 0.4, rot_center, a0, a1s));
      let s9b =
        builder.add_slot_arc(SlotArc::new_no_border(builder, slots[9], 0.0, rot_center, a0, a1s));
      let s6b = builder.add_slot_arc(SlotArc::new_no_border(
        builder,
        left_side_slots.top_right,
        0.4,
        rot_center,
        a0,
        a1s,
      ));
      let s_rail_rest = builder.add_slot(
        Slot::new_no_border(
          Point { x: -1.0, y: self.y_out + self.step_out + 8.0 },
          Point::Y,
          10.0,
          &[(0.0, 10.0)],
        )
        .width(4.0),
      );
      let srb = builder.add_slot_arc(SlotArc::new_no_border(
        builder,
        s_rail_rest,
        0.4,
        rot_center,
        a0,
        a1s,
      ));
      let srin = builder.add_hole_arc(HoleArc::new(builder, in_result, 0.0, rot_center, a0, a1s));

      let dir = complex_mul((roll_center - rot_center).norm(), Point::from_angle(0.0));

      let alt_hole = builder.add_hole(Hole::new(rot_center + dir.scale(9.5), 1.5).border(3.0));
      let alt_slot_1 = builder
        .add_slot(Slot::new(rot_center + dir.scale(2.0), dir, 6.0, &[(1.0, 5.0)]).border(3.0));
      let alt_slot_2 = builder.add_slot(
        Slot::new(
          rot_center + dir.scale(12.0) - dir.perp().scale(5.0),
          dir.perp(),
          10.0,
          &[(3.0, 7.0)],
        )
        .border(4.0),
      );

      builder.add_figure(
        Figure::new(
          builder,
          &[
            contour![rot_hole, alt_hole, roll_hole, drum_screw_arc.end(), drum_screw_arc.begin()],
            chain![drum_screw_arc.begin(), contact_s2_i, contact, contact_up],
            chain![alt_slot_1, alt_slot_2],
            chain![contact_s1, contact_s2_l, contact_s2_r],
            chain![screw_arc],
            chain![s9b, s0b, s1b, s2b, s3b, s4b, s5b, s6b, srb, srin],
          ],
        )
        .name(format!("crank_out_err{}", err)),
      );
      builder.add_figure(
        Figure::new(
          builder,
          &[
            chain![rot_hole, alt_hole, roll_hole,],
            chain![alt_slot_1, alt_slot_2],
            chain![screw_arc, drum_arc],
            chain![s0b, s1b, s2b, s6b, srb],
          ],
        )
        .name(format!("crank_out_cup_err{}", err)),
      );
      builder.add_connector(
        Connector::new(builder, alt_slot_1, 5.7).count(2).name("crank_out_connector".into()),
      );
    };

    let in_result = add_crank_in_stuff(builder, 0.0);
    add_crank_out_stuff(builder, 0.0, in_result);
    let in_result = add_crank_in_stuff(builder, 1.0);
    add_crank_out_stuff(builder, 1.0, in_result);

    builder.add_figure(
      Figure::new(
        builder,
        &[
          filled![
            holes[0], holes[1], holes[2], holes[3], holes[4], holes[5], holes[6], holes[7],
            holes[8], holes[9]
          ],
          chain![mid_keep_bolt_hole, top_keep_bolt_hole, top_hole, axle_end_slots.rail_bottom_slot],
          chain![
            dh,
            cr1h,
            cr2h,
            left_side_slots.top_mid,
            left_side_slots.top_right,
            left_side_slots.bot_left,
            left_side_slots.bot_mid,
            axle_end_slots.axle_pipe_slot,
            axle_end_slots.axle_end_slot,
            axle_end_slots.axle_hold_slot,
            cup_keeper_top_hole
          ],
          chain![
            slots[0], slots[1], slots[2], slots[3], slots[4], slots[5], slots[6], slots[7],
            slots[8], slots[9]
          ],
        ],
      )
      .name("plate_bottom".into()),
    );

    builder.add_figure(
      Figure::new(
        builder,
        &[
          filled![
            holes[0], holes[1], holes[2], holes[3], holes[4], holes[5], holes[6], holes[7],
            holes[8], holes[9]
          ],
          chain![mid_keep_bolt_hole, top_keep_bolt_hole, top_hole, axle_end_slots.rail_mid_slot],
          chain![drum_place, cable_slot_1, cable_slot_2],
          chain![
            dh,
            cr1h,
            cr2h,
            left_side_slots.top_mid,
            left_side_slots.top_right,
            left_side_slots.bot_left,
            left_side_slots.bot_mid,
            axle_end_slots.axle_pipe_slot,
            axle_end_slots.axle_end_slot,
            axle_end_slots.axle_hold_slot,
            cup_keeper_top_hole
          ],
          chain![
            slots[0], slots[1], slots[2], slots[3], slots[4], slots[5], slots[6], slots[7],
            slots[8], slots[9]
          ],
        ],
      )
      .name("plate_mid".into()),
    );

    builder.add_figure(
      Figure::new(
        builder,
        &[
          chain![top_keep_bolt_hole, cable_slot_1_cup.begin(),],
          chain![cable_slot_2_cup.begin(), top_hole, axle_end_slots.rail_mid_slot_cup],
          chain![axle_end_slots.axle_hold_slot],
          chain![cr1h_cup, cr2h_cup, drum_place_cup, cup_keeper_top_hole],
        ],
      )
      .name("plate_cup".into()),
    );

    builder.add_figure(
      Figure::new(
        builder,
        &[
          chain![
            top_keep_bolt_hole,
            drum_place_view_cup,
            top_hole,
            axle_end_slots.rail_mid_slot_cup
          ],
          chain![axle_end_slots.axle_hold_slot],
          chain![drum_place_view_cup, cup_keeper_top_hole],
          chain![cr1h_view_cup, cr2h_view_cup],
        ],
      )
      .name("plate_view_cup".into()),
    );

    let h1 = builder.add_hole(Hole::new(-Point::X.scale(10.0), 2.5).border(3.0));
    let h2 = builder.add_hole(Hole::new(-Point::X.scale(0.0), 2.5).border(3.0));
    let h3 = builder.add_hole(Hole::new(Point::X.scale(10.0), 2.5).border(3.0));
    builder.add_figure(Figure::new(builder, &[chain![h1, h2, h3]]).name("cabler".into()));
  }

  fn init(&mut self, error: f32) {
    self.error = error;

    self.y_out = 27.5;
    self.y_in = 43.2;
    self.drum_radius = 14.5;
    self.depth = 2.0;

    self.drum_pos =
      Point { x: self.drum_radius + self.depth + 7.1, y: self.drum_radius - self.depth + 11.0 };
    self.roll_radius = 3.5;
    self.step_out = 5.5;
    self.step_in = 6.0;

    self.axle_r = 4.6;
    self.braid_r = 2.9;
    self.axle_end = 22.0;
    self.pipe_r = 2.6;
    self.nut_y = 7.0;

    self.cable_slot_1_y = self.nut_y + 6.75;
    self.cable_slot_2_y = self.drum_pos.y + (self.drum_radius - self.depth - 1.0);

    self.height = 17.2;

    self.f_in = self.find_crank_center(self.y_in, self.step_in, 0.1);
    self.f_out = self.find_crank_center(self.y_out, self.step_out, 2.0);
    self.angle_between =
      Self::get_angle_between(self.f_in.1[0] - self.drum_pos, self.f_out.1[0] - self.drum_pos)
        .abs();

    println!("f_in={:?}", self.f_in);
    println!("f_out={:?}", self.f_out);
    println!("crank_in_l = {}", (self.f_in.1[0] - self.f_in.0).len());
    println!("crank_out_l = {}", (self.f_out.1[0] - self.f_out.0).len());
    println!("angle={}", self.angle_between.abs().to_degrees());
  }

  fn make_drum(&self, drum: &mut DrumCreator) {
    let crank_rel_drum = self.f_in.0 - self.drum_pos;
    let roll_rel_drum = self.f_in.1[1] - self.drum_pos;
    let l = (crank_rel_drum - roll_rel_drum).len();

    drum.roll_r = self.roll_radius;
    drum.max_r = self.drum_radius + self.depth + 0.3;
    drum.cable_surface_r = self.drum_radius - self.depth - 1.5;

    let max_a = 2.0 * PI - self.angle_between;
    let a0 = 0.0;
    let a1 = self.angle_between * 0.42;
    let a2 = self.angle_between;
    let a3 = max_a - self.angle_between * 0.84;
    let a4 = max_a - self.angle_between * 0.42;
    let a5 = max_a;

    println!(
      "total angle={}, length of 5th speed = {}",
      max_a.to_degrees(),
      (a3 - a2 - self.angle_between).to_degrees()
    );

    let dt01 = 4.4;
    let dt12 = 3.1;
    let dt34 = 4.2;
    let dt45 = 4.4;

    let cable_r = self.drum_radius - self.depth - 1.0;

    let l_near_nut = 3.5 / (cable_r - 3.5);

    let sum_a = max_a - 4.0 / cable_r;
    let bot_a = l_near_nut;
    let top_a = sum_a - bot_a;

    drum.error = self.error - 0.05;
    drum.nut_top = Point::from_angle(top_a).scale(cable_r - 3.5);
    drum.nut_bottom = Point::from_angle(bot_a).scale(cable_r - 3.5);
    drum.screw1 = Point::from_angle(-PI * 0.25).scale(cable_r - 3.5);
    drum.screw2 = Point::from_angle(PI - PI * 0.25).scale(cable_r - 3.5);

    for i in 0..3600 {
      let a = i as f32 * PI / 1800.0 - self.angle_between;
      let ca = Point::from_angle(a);
      let depth;
      if a < a0 {
        depth = -self.depth;
      } else if a < a1 {
        depth = f32::min(0.3, f32::min((a - a0) * dt01 - self.depth, (a1 - a) * dt01));
      } else if a < a2 {
        depth = f32::min(self.depth + 0.3, f32::min((a - a1) * dt12, (a2 - a) * dt12 + self.depth));
      } else if a < a3 {
        depth = self.depth;
      } else if a < a4 {
        depth = f32::min(self.depth + 0.3, f32::min((a - a3) * dt34 + self.depth, (a4 - a) * dt34));
      } else if a < a5 {
        depth = f32::min(0.3, f32::min((a - a4) * dt45, (a5 - a) * dt45 - self.depth));
      } else {
        depth = 0.0;
      }
      let r =
        complex_mul(roll_rel_drum - crank_rel_drum, Point::from_angle(depth / l)) + crank_rel_drum;
      drum.roll_traj.push(complex_mul(r, ca));
    }
  }
}

#[derive(Default)]
struct DrumCreator {
  error: f32,
  max_r: f32,
  cable_surface_r: f32,
  roll_r: f32,
  roll_traj: Vec<Point>,
  nut_top: Point,
  nut_bottom: Point,
  screw1: Point,
  screw2: Point,
}

impl DrumCreator {
  pub fn faces(&self) -> usize {
    5
  }

  pub fn get_height(&self, part_index: usize) -> f32 {
    if part_index == 4 {
      2.5
    } else {
      2.0
    }
  }

  pub fn get_name(&self, part_index: usize) -> Option<&str> {
    match part_index {
      0 => Some("drum_program"),
      1 => Some("drum_bottom_cable"),
      2 => Some("drum_top_cable"),
      3 => Some("drum_cup"),
      _ => Some("bear_simulator"),
    }
  }

  pub fn get_count(&self, part_index: usize) -> usize {
    1
  }

  pub fn aabb(&self,  part_index: usize) -> AABB {
    AABB::around_zero(self.max_r)
  }

  pub fn get_sticker_index(&self, pos: Point, part_index: usize) -> PartIndex {
    fn near_nut(p: Point, nut: Point, error: f32) -> bool {
      let p = p - nut;
      let nut = nut.norm();
      let x = dot(p, nut);
      let y = cross(p, nut);

      let p1 = x;
      if p1.abs() > 2.8 + error {
        return false;
      }
      let p2 = y * 0.75.sqrt() - x * 0.5;
      if p2.abs() > 2.8 + error {
        return false;
      }
      let p3 = -y * 0.75.sqrt() - x * 0.5;
      if p3.abs() > 2.8 + error {
        return false;
      }
      return true;
    }

    match part_index {
      0 => {
        if pos.len() < 3.5 + self.error || pos.len() > self.max_r {
          return 0;
        }

        for i in 1..self.roll_traj.len() {
          let prev = self.roll_traj[i - 1];
          let cur = self.roll_traj[i];
          if dist_pl(pos, prev, cur) < self.roll_r + self.error {
            return 0;
          }
        }

        if near_nut(pos, self.nut_bottom, self.error)
          || (pos - self.nut_top).len() < 1.5 + self.error
          || (pos - self.screw1).len() < 1.5 + self.error
          || (pos - self.screw2).len() < 1.5 + self.error
        {
          return 0;
        }

        return 1;
      }
      1 => {
        let d1 = self.cable_surface_r - pos.len();
        if pos.len() < 2.5 + self.error || d1 < 0.0 {
          return 0;
        }

        let d_nut = (pos - self.nut_bottom).len() - (3.5 + self.error);

        if d_nut < 0.0
          || (pos - self.nut_top).len() < 1.5 + self.error
          || (pos - self.screw1).len() < 1.5 + self.error
          || (pos - self.screw2).len() < 1.5 + self.error
        {
          return 0;
        }

        let nbl = self.nut_bottom.len();
        let cr = cross(pos, self.nut_bottom);
        let md = nbl * (nbl - 3.5 - self.error);

        if dot(pos, self.nut_bottom) > md {
          let d2 = if cr < 0.0 { d_nut } else { cr.abs() / nbl - (3.5 + self.error) };

          if d2 < 0.0 {
            return 0;
          }

          let mr = if cr < 0.0 { 4.4 } else { 1.0 };
          if (d1 < mr && d2 < mr && sqr(mr - d1) + sqr(mr - d2) > sqr(mr)) {
            return 0;
          }
        }

        return 1;
      }
      2 => {
        let d1 = self.cable_surface_r - pos.len();
        if pos.len() < 2.5 + self.error || d1 < 0.0 {
          return 0;
        }

        let d_nut = (pos - self.nut_top).len() - (3.5 + self.error);

        if d_nut < 0.0
          || (pos - self.nut_bottom).len() < 1.5 + self.error
          || (pos - self.screw1).len() < 1.5 + self.error
          || (pos - self.screw2).len() < 1.5 + self.error
        {
          return 0;
        }

        let nbl = self.nut_top.len();
        let cr = cross(pos, self.nut_top);
        let md = nbl * (nbl - 3.5 - self.error);

        if dot(pos, self.nut_top) > md {
          let d2 = if cr > 0.0 { d_nut } else { cr.abs() / nbl - (3.5 + self.error) };

          if d2 < 0.0 {
            return 0;
          }

          let mr = if cr > 0.0 { 4.4 } else { 1.0 };
          if (d1 < mr && d2 < mr && sqr(mr - d1) + sqr(mr - d2) > sqr(mr)) {
            return 0;
          }
        }

        return 1;
      }
      3 => {
        if pos.len() < 3.5 + self.error || pos.len() > self.cable_surface_r + 1.5 {
          return 0;
        }

        if near_nut(pos, self.nut_top, self.error)
          || (pos - self.nut_bottom).len() < 1.5 + self.error
          || (pos - self.screw1).len() < 1.5 + self.error
          || (pos - self.screw2).len() < 1.5 + self.error
        {
          return 0;
        }

        return 1;
      }
      _ => {
        if pos.len() < 2.0 + self.error * 0.5 || pos.len() > 3.5 - self.error {
          return 0;
        }
        return 1;
      }
    }
    0
  }
}

pub struct ClickboxCreator {
  builder: Builder,
  drum: DrumCreator,
}

impl ClickboxCreator {
  pub fn new() -> Self {
    let error = 0.2;

    let mut params = ClickboxParams::default();
    params.init(error);

    let mut builder = Builder::new(1.0, 2.0, error);
    let mut drum = DrumCreator::default();

    params.make_clickbox(&mut builder);
    params.make_drum(&mut drum);

    Self { builder, drum }
  }

  pub fn faces(&self) -> usize {
    self.builder.contour_count() + self.drum.faces()
  }

  pub fn get_height(&self, part_index: usize) -> f32 {
    if part_index < self.builder.contour_count() {
      self.builder.get_material_thickness(part_index)
    } else {
      self.drum.get_height(part_index - self.builder.contour_count())
    }
  }

  pub fn get_name(&self, part_index: usize) -> Option<&str> {
    if part_index < self.builder.contour_count() {
      self.builder.get_name(part_index)
    } else {
      self.drum.get_name(part_index - self.builder.contour_count())
    }
  }

  pub fn get_count(&self, part_index: usize) -> usize {
    if part_index < self.builder.contour_count() {
      self.builder.get_count(part_index)
    } else {
      self.drum.get_count(part_index - self.builder.contour_count())
    }
  }

  pub fn get_sticker_index(&self, pos: Point, part_index: usize) -> PartIndex {
    if part_index < self.builder.contour_count() {
      self.builder.contains(part_index, pos) as PartIndex
    } else {
      self.drum.get_sticker_index(pos, part_index - self.builder.contour_count())
    }
  }

  pub fn aabb(&self, part_index: usize) -> Option<AABB> {
    if part_index < self.builder.contour_count() {
      Some(self.builder.aabb(part_index))
    } else {
      Some(self.drum.aabb(part_index))
    }
  }
}
