use gears::*;

fn write_gear_to_dxf(dxf: &mut dxf::DxfR12Writer, g: &dyn Gear, pos: (f32, f32), phase: f32) {
  const PI: f32 = std::f32::consts::PI;
  let r_in = g.r_in();
  let r_out = g.r_out();

  let w_in = g.profile().w_angle(r_in);
  let w_out = g.profile().w_angle(r_out);
  let tooth_angle = 2.0 * PI / g.profile().z as f32;

  let mut pl = Vec::<(f32, f32)>::new();
  for z in 0..g.profile().z {
    let da = (phase + z as f32) * tooth_angle;

    fn fill_curve(
      pl: &mut Vec<(f32, f32)>,
      dxf: &mut dxf::DxfR12Writer,
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
        pl.push(pm);
        fill_curve(pl, dxf, f, rm, r2, pm, p2);
      }
    }

    let evo = |r: f32| -> (f32, f32) {
      let (s, c) = (da - g.profile().w_angle(r)).sin_cos();
      (c * r + pos.0, s * r + pos.1)
    };

    let p1 = evo(r_in);
    let p2 = evo(r_out);
    pl.push(p1);
    fill_curve(&mut pl, dxf, &evo, r_in, r_out, p1, p2);
    pl.push(p2);

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
    pl.push(p1);
    fill_curve(&mut pl, dxf, &evo, r_out, r_in, p1, p2);
    pl.push(p2);

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
  dxf.add_polyline(pl.into_iter().map(|(x, y)| (x as f64, y as f64)), true).unwrap();
  dxf.add_circle(pos.0 as f64, pos.1 as f64, g.top_r() as f64).unwrap();
  dxf.add_circle(pos.0 as f64, pos.1 as f64, g.profile().basic_r() as f64).unwrap();
  dxf.add_circle(pos.0 as f64, pos.1 as f64, g.r_evo() as f64).unwrap();
}

pub fn draw_center_line<C: Couple>(dxf: &mut dxf::DxfR12Writer, c: &C) {
  let d = c.get_dist();
  dxf.add_line(d as f64, 0.0, 0.0, 0.0).unwrap();
}

pub fn draw_couple_lines<C: Couple>(dxf: &mut dxf::DxfR12Writer, c: &C) {
  let mut l = c.get_couple_length();
  if C::inner() {
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

  dxf.add_line(x1 as f64, y1 as f64, x2 as f64, y2 as f64).unwrap();
  dxf.add_line(x1 as f64, -y1 as f64, x2 as f64, -y2 as f64).unwrap();
}

pub fn draw_couple_gears<C: Couple>(dxf: &mut dxf::DxfR12Writer, c: &C) {
  let d = c.get_dist();
  let mut phase_2 = c.get_g1().profile().z as f32 * 0.5 + 0.5;
  if C::inner() {
    phase_2 = 0.0;
  }
  write_gear_to_dxf(dxf, c.get_g1(), (d, 0.0), 0.0);
  write_gear_to_dxf(dxf, c.get_g2(), (0.0, 0.0), phase_2);
}

pub fn draw_couple(c: &(impl Couple + std::fmt::Debug), filename: &std::path::Path) {
  let mut dxf = dxf::DxfR12Writer::new(filename).unwrap();

  draw_couple_gears(&mut dxf, c);
  draw_center_line(&mut dxf, c);
  draw_couple_lines(&mut dxf, c);
  dxf.close().unwrap();
}

pub fn draw_planet_couple_gears(dxf: &mut dxf::DxfR12Writer, oc: &OutCouple, ic: &InCouple) {
  let d = oc.get_dist();
  let phase_2 = oc.get_g1().profile().z as f32 * 0.5 + 0.5;
  write_gear_to_dxf(dxf, oc.get_g1(), (d, 0.0), 0.0);
  write_gear_to_dxf(dxf, oc.get_g2(), (0.0, 0.0), phase_2);
  write_gear_to_dxf(dxf, ic.get_g2(), (0.0, 0.0), 0.0);
}

pub fn draw_planet_couple(oc: &OutCouple, ic: &InCouple, filename: &std::path::Path) {
  let mut dxf = dxf::DxfR12Writer::new(filename).unwrap();

  draw_planet_couple_gears(&mut dxf, oc, ic);
  draw_center_line(&mut dxf, oc);
  draw_couple_lines(&mut dxf, oc);
  draw_couple_lines(&mut dxf, ic);
  dxf.close().unwrap();
}
