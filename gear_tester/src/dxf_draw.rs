use gears::*;

fn decart_to_dxfp(p: (f32, f32)) -> dxf::Point {
  dxf::Point { x: p.0 as f64, y: p.1 as f64, z: 0.0 }
}

fn decart_to_dxfv(p: (f32, f32)) -> dxf::entities::Vertex {
  dxf::entities::Vertex::new(decart_to_dxfp(p))
}

fn write_line_to_dxf(dxf: &mut dxf::Drawing, pos1: (f32, f32), pos2: (f32, f32)) {
  use dxf::entities::*;
  dxf.add_entity(Entity::new(EntityType::Line(Line::new(
    decart_to_dxfp(pos1),
    decart_to_dxfp(pos2),
  ))));
}

fn write_gear_to_dxf(dxf: &mut dxf::Drawing, g: &dyn Gear, pos: (f32, f32), phase: f32) {
  const PI: f32 = std::f32::consts::PI;
  let r_in = g.r_in();
  let r_out = g.r_out();

  let w_in = g.profile().w_angle(r_in);
  let w_out = g.profile().w_angle(r_out);
  let tooth_angle = 2.0 * PI / g.profile().z as f32;

  use dxf::entities::*;
  use dxf::*;

  let mut pl = Polyline::default();
  pl.set_is_closed(true);
  for z in 0..g.profile().z {
    let da = (phase + z as f32) * tooth_angle;

    fn fill_curve(
      pl: &mut Polyline,
      dxf: &mut Drawing,
      f: &dyn Fn(f32) -> (f32, f32),
      r1: f32,
      r2: f32,
      p1: (f32, f32),
      p2: (f32, f32),
    ) {
      let rm = (r1 + r2) * 0.5;
      let pm = f(rm);
      let cross = (p1.0 - pm.0) * (p2.1 - pm.1) - (p1.1 - pm.1) * (p2.0 - pm.0);
      let sq_cross = cross * cross;
      let sq_len = (p1.0 - pm.0) * (p1.0 - pm.0) + (p2.1 - pm.1) * (p2.1 - pm.1);
      if sq_cross > sq_len * 1.0e-6 {
        fill_curve(pl, dxf, f, r1, rm, p1, pm);
        pl.add_vertex(dxf, decart_to_dxfv(pm));
        fill_curve(pl, dxf, f, rm, r2, pm, p2);
      }
    }

    let evo = |r: f32| -> (f32, f32) {
      let (s, c) = (da - g.profile().w_angle(r)).sin_cos();
      (c * r + pos.0, s * r + pos.1)
    };

    let p1 = evo(r_in);
    let p2 = evo(r_out);
    pl.add_vertex(dxf, decart_to_dxfv(p1));
    fill_curve(&mut pl, dxf, &evo, r_in, r_out, p1, p2);
    pl.add_vertex(dxf, decart_to_dxfv(p2));

    let circle = |a: f32| -> (f32, f32) {
      let (s, c) = a.sin_cos();
      (c * r_out + pos.0, s * r_out + pos.1)
    };

    let a1 = da - w_out;
    let a2 = da + w_out;
    let p1 = circle(a1);
    let p2 = circle(a2);
    fill_curve(&mut pl, dxf, &circle, a1, a2, p1, p2);

    let evo = |r: f32| -> (f32, f32) {
      let (s, c) = (da + g.profile().w_angle(r)).sin_cos();
      (c * r + pos.0, s * r + pos.1)
    };

    let p1 = evo(r_out);
    let p2 = evo(r_in);
    pl.add_vertex(dxf, decart_to_dxfv(p1));
    fill_curve(&mut pl, dxf, &evo, r_out, r_in, p1, p2);
    pl.add_vertex(dxf, decart_to_dxfv(p2));

    let circle = |a: f32| -> (f32, f32) {
      let (s, c) = a.sin_cos();
      (c * r_in + pos.0, s * r_in + pos.1)
    };

    let a1 = da + w_in;
    let a2 = da - w_in + tooth_angle;
    let p1 = circle(a1);
    let p2 = circle(a2);
    fill_curve(&mut pl, dxf, &circle, a1, a2, p1, p2);
  }
  dxf.add_entity(Entity::new(EntityType::Polyline(pl)));
  dxf.add_entity(Entity::new(EntityType::Circle(Circle::new(
    decart_to_dxfp(pos),
    g.top_r() as f64,
  ))));
  dxf.add_entity(Entity::new(EntityType::Circle(Circle::new(
    decart_to_dxfp(pos),
    g.profile().basic_r() as f64,
  ))));
  dxf.add_entity(Entity::new(EntityType::Circle(Circle::new(
    decart_to_dxfp(pos),
    g.r_evo() as f64,
  ))));
}

pub fn draw_center_line(drawing: &mut dxf::Drawing, c: &(impl Couple + std::fmt::Debug)) {
   let d = c.get_dist();
  write_line_to_dxf(drawing, (d, 0.0), (0.0, 0.0));
}

pub fn draw_couple_lines(drawing: &mut dxf::Drawing, c: &(impl Couple + std::fmt::Debug)) {
  let mut l = c.get_couple_length();
  if c.inner() {
    l += c.get_g1().total_couple_part();
  }

  let r2 = c.get_g2().profile().basic_r();
  let (s, c) = c.get_angle().sin_cos();
  let x1 = r2 * c;
  let y1 = r2 * s;
  let dx = s;
  let dy = c;
  let x2 = x1 + dx * l;
  let y2 = y1 - dy * l;

  write_line_to_dxf(drawing, (x1, y1), (x2, y2));
  write_line_to_dxf(drawing, (x1, -y1), (x2, -y2));
}

pub fn draw_couple_gears(drawing: &mut dxf::Drawing, c: &(impl Couple + std::fmt::Debug)) {
  let d = c.get_dist();
  let mut phase_2 = c.get_g1().profile().z as f32 * 0.5 + 0.5;
  if c.inner() {
    phase_2 = 0.0;
  }
  write_gear_to_dxf(drawing, c.get_g1(), (d, 0.0), 0.0);
  write_gear_to_dxf(drawing, c.get_g2(), (0.0, 0.0), phase_2);
}

pub fn draw_couple(c: &(impl Couple + std::fmt::Debug), filename: &str) {
  let mut drawing = dxf::Drawing::new();

  draw_couple_gears(&mut drawing, c);
  draw_center_line(&mut drawing, c);
  draw_couple_lines(&mut drawing, c);

  drawing.save_file(filename).unwrap();
}


pub fn draw_planet_couple_gears(drawing: &mut dxf::Drawing, oc: &OutCouple, ic: &InCouple) {
  let d = oc.get_dist();
  let phase_2 = oc.get_g1().profile().z as f32 * 0.5 + 0.5;
  write_gear_to_dxf(drawing, oc.get_g1(), (d, 0.0), 0.0);
  write_gear_to_dxf(drawing, oc.get_g2(), (0.0, 0.0), phase_2);
  write_gear_to_dxf(drawing, ic.get_g2(), (0.0, 0.0), 0.0);
}

pub fn draw_planet_couple(oc: &OutCouple, ic: &InCouple, filename: &str) {
  let mut drawing = dxf::Drawing::new();

  draw_planet_couple_gears(&mut drawing, oc, ic);
  draw_center_line(&mut drawing, oc);
  draw_couple_lines(&mut drawing, oc);
  draw_couple_lines(&mut drawing, ic);

  drawing.save_file(filename).unwrap();
}