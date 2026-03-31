use gears::*;

#[macro_use]
mod gear_result;
use gear_result::*;

mod dxf_draw;
use dxf_draw::*;

mod config;
use config::*;

mod img_buffer;
use img_buffer::*;

fn compose_gear_results<C: Couple + std::fmt::Debug>(
  g1: Result<C::G1, String>,
  g2: Result<C::G2, String>,
) -> Result<C, GearError> {
  if let Ok(g1) = g1 {
    if let Ok(g2) = g2 {
      if let Ok(couple) = C::new_without_cuts(g1, g2) {
        return Ok(couple);
      } else {
        return Err(GearError { no_gear_1: false, no_gear_2: false, no_couple: true });
      }
    } else {
      return Err(GearError { no_gear_1: false, no_gear_2: true, no_couple: false });
    }
  } else {
    if g2.is_ok() {
      return Err(GearError { no_gear_1: true, no_gear_2: false, no_couple: false });
    } else {
      return Err(GearError { no_gear_1: true, no_gear_2: true, no_couple: false });
    }
  }
}

fn create_couple<C: Couple + std::fmt::Debug>(
  p1: Profile,
  p2: Profile,
  c: &Chisel,
) -> Result<C, GearError> {
  compose_gear_results(C::produce_g1(c, p1), C::produce_g2(c, p2))
}

fn print_couple_stats<C: Couple + std::fmt::Debug>(c: &C)
where
  C::G1: std::fmt::Debug,
  C::G2: std::fmt::Debug,
{
  println!("  w1={}", c.get_top_w1());
  println!("  w2={}", c.get_top_w2());
  println!("  couple length={}", c.get_couple_length());
  println!("  overlap={}", c.get_overlap());
  println!("  g1 fillet={}", c.get_g1().get_fillet_r());
  println!("  g2 fillet={}", c.get_g2().get_fillet_r());
  println!("  failed rg1={}", c.failed_raidal_gap1());
  println!("  failed rg2={}", c.failed_raidal_gap2());
  println!("  has interference1={}", c.has_g1_interference());
  println!("  has interference2={}", c.has_g2_interference());
  println!("  g1={:?}", c.get_g1());
  println!("  g2={:?}", c.get_g2());
  println!("  dist={}", c.get_dist());
  println!("  angle={}", c.get_angle().to_degrees());
  println!();
}

fn test_two_instruments<C: Couple + std::fmt::Debug>(
  x1: f32,
  x2: f32,
  new: &Chisel,
  old: &Chisel,
  config: &Config,
) -> (Result<GearResult, GearError>, Result<GearResult, GearError>)
where
  C::G1: std::fmt::Debug,
  C::G2: std::fmt::Debug,
{
  let p1 = Profile { z: config.z1, x: x1 };
  let p2 = Profile { z: config.z2, x: x2 };
  let nc = create_couple::<C>(p1, p2, new);
  let oc = create_couple::<C>(p1, p2, old);

  match nc {
    Ok(mut nc) => match oc {
      Ok(mut oc) => {
        adjust_couple(&mut nc, config);

        if config.radial_gap.is_some() {
          oc.cut1(nc.get_g1().top_r());
          oc.cut2(nc.get_g2().top_r());
        } else {
          adjust_couple(&mut oc, config);
          // make diameters equal
          nc.cut1(oc.get_g1().top_r());
          nc.cut2(oc.get_g2().top_r());
          oc.cut1(nc.get_g1().top_r());
          oc.cut2(nc.get_g2().top_r());
        }

        if config.show {
          println!("By new instrument:");
          print_couple_stats(&nc);
          println!("By old instrument:");
          print_couple_stats(&oc);

          draw_couple(&new.produce_out_instrument_couple(p1).unwrap(), "couple_new_1.dxf");
          draw_couple(&C::new_g2_couple(new, p2).unwrap(), "couple_new_2.dxf");
          draw_couple(&old.produce_out_instrument_couple(p1).unwrap(), "couple_old_1.dxf");
          draw_couple(&C::new_g2_couple(old, p2).unwrap(), "couple_old_2.dxf");

          draw_couple(&nc, "gears_by_new.dxf");
          draw_couple(&oc, "gears_by_old.dxf");
        }

        return (Ok(couple_stats(&nc)), Ok(couple_stats(&oc)));
      }
      Err(oe) => {
        adjust_couple(&mut nc, config);
        if config.radial_gap.is_some() {
          return (Ok(couple_stats(&nc)), Err(oe));
        } else {
          return (Err(oe), Err(oe));
        }
      }
    },
    Err(ne) => match oc {
      Ok(_) => return (Err(ne), Err(ne)),
      Err(oe) => return (Err(ne), Err(oe)),
    },
  }
}

fn test_chisel<C: Couple + std::fmt::Debug>(
  x1: f32,
  x2: f32,
  config: &Config,
) -> Vec<ResultWithCoords>
where
  C::G1: std::fmt::Debug,
  C::G2: std::fmt::Debug,
{
  let new = Chisel::new_m1_z26();
  let old = Chisel::old_m1_z26();

  let (gnew, gold) =
    test_two_instruments::<C>(x1, x2, &new, &old, config);

  let ix1 = x1.div_euclid(0.2) as i32;
  let ix2 = x2.div_euclid(0.2) as i32;

  vec![
    ResultWithCoords { ix1, ix2, g: gnew, important: true },
    ResultWithCoords { ix1, ix2, g: gold, important: false },
  ]
}

fn test_rail(x1: f32, x2: f32, config: &Config) -> Vec<ResultWithCoords> {
  let r = Rail::new();
  let p1 = Profile { z: config.z1, x: x1 };
  let p2 = Profile { z: config.z2, x: x2 };

  let ix1 = x1.div_euclid(0.2) as i32;
  let ix2 = x2.div_euclid(0.2) as i32;

  let c = compose_gear_results::<OutCouple>(r.produce_out_gear(p1), r.produce_out_gear(p2));
  let g = match c {
    Ok(mut c) => {
      adjust_couple(&mut c, config);

      if config.show {
        println!("By rail:");
        print_couple_stats(&c);
        draw_couple(&c, "gears_by_rail.dxf");
      }

      Ok(couple_stats(&c))
    }
    Err(err) => Err(err),
  };

  vec![ResultWithCoords { ix1, ix2, g, important: true }]
}

fn test_two_instruments_for_planet<C: Couple + std::fmt::Debug>(
  p1: Profile,
  p2: Profile,
  new: &Chisel,
  old: &Chisel,
  config: &Config,
) -> (Result<C, GearError>, Result<C, GearError>) {
  let nc = create_couple::<C>(p1, p2, new);
  let oc = create_couple::<C>(p1, p2, old);

  match nc {
    Ok(mut nc) => match oc {
      Ok(mut oc) => {
        adjust_couple(&mut nc, config);
        adjust_couple(&mut oc, config);
        nc.cut1(oc.get_g1().top_r());
        nc.cut2(oc.get_g2().top_r());
        oc.cut1(nc.get_g1().top_r());
        oc.cut2(nc.get_g2().top_r());

        return (Ok(nc), Ok(oc));
      }
      Err(oe) => {
        return (Err(oe), Err(oe));
      }
    },
    Err(ne) => match oc {
      Ok(_) => return (Err(ne), Err(ne)),
      Err(oe) => return (Err(ne), Err(oe)),
    },
  }
}

fn test_planet(x1: f32, x2: f32, config: &Config) -> Vec<ResultWithCoords> {
  let p1 = Profile { z: config.z1, x: x1 };
  let p2 = Profile { z: config.z2, x: x2 };
  let ix1 = x2.div_euclid(0.2) as i32;
  let ix2 = x1.div_euclid(0.2) as i32;

  let new = Chisel::new_m1_z26();
  let old = Chisel::old_m1_z26();

  let (onc, ooc) = test_two_instruments_for_planet::<OutCouple>(p2, p1, &new, &old, config);

  match onc {
    Ok(mut onc) => {
      let dist = onc.get_dist();
      let sum_corr = InCouple::sum_of_corrections(config.z2, config.z3, dist);
      let x3 = sum_corr + x2;
      let p3 = Profile { z: config.z3, x: x3 };
      let ix3 = x3.div_euclid(0.2) as i32;

      let (inc, ioc) = test_two_instruments_for_planet::<InCouple>(p2, p3, &new, &old, config);

      let mut ooc = ooc.unwrap();

      match inc {
        Ok(mut inc) => {
          let mut ioc = ioc.unwrap();
          onc.cut1(inc.get_g1().top_r());
          inc.cut1(onc.get_g1().top_r());
          ooc.cut1(ioc.get_g1().top_r());
          ioc.cut1(ooc.get_g1().top_r());

          if config.show {
            println!("Outer by new instrument:");
            print_couple_stats(&onc);
            println!("Outer by old instrument:");
            print_couple_stats(&ooc);
            println!("Inner by new instrument:");
            print_couple_stats(&inc);
            println!("Inner by old instrument:");
            print_couple_stats(&ioc);

            draw_couple(&new.produce_out_instrument_couple(p1).unwrap(), "couple_new_1.dxf");
            draw_couple(&new.produce_out_instrument_couple(p2).unwrap(), "couple_new_2.dxf");
            draw_couple(&new.produce_in_instrument_couple(p3).unwrap(), "couple_new_3.dxf");
            draw_couple(&old.produce_out_instrument_couple(p1).unwrap(), "couple_old_1.dxf");
            draw_couple(&old.produce_out_instrument_couple(p2).unwrap(), "couple_old_2.dxf");
            draw_couple(&old.produce_in_instrument_couple(p3).unwrap(), "couple_old_3.dxf");

            draw_planet_couple(&onc, &inc, "planet_gears_by_new.dxf");
            draw_planet_couple(&ooc, &ioc, "planet_gears_by_old.dxf");
          }

          return vec![
            ResultWithCoords { ix1, ix2, g: Ok(couple_stats(&onc)), important: true },
            ResultWithCoords { ix1, ix2, g: Ok(couple_stats(&ooc)), important: false },
            ResultWithCoords { ix1, ix2: ix3, g: Ok(couple_stats(&inc)), important: true },
            ResultWithCoords { ix1, ix2: ix3, g: Ok(couple_stats(&ioc)), important: false },
          ];
        }
        Err(ine) => {
          let ioe = ioc.unwrap_err();
          return vec![
            ResultWithCoords { ix1, ix2, g: Ok(couple_stats(&onc)), important: true },
            ResultWithCoords { ix1, ix2, g: Ok(couple_stats(&ooc)), important: false },
            ResultWithCoords { ix1, ix2: ix3, g: Err(ine), important: true },
            ResultWithCoords { ix1, ix2: ix3, g: Err(ioe), important: false },
          ];
        }
      }
    }
    Err(one) => {
      let ooe = ooc.unwrap_err();
      return vec![
        ResultWithCoords { ix1, ix2, g: Err(one), important: true },
        ResultWithCoords { ix1, ix2, g: Err(ooe), important: false },
        ResultWithCoords { ix1, ix2, g: Err(one), important: true },
        ResultWithCoords { ix1, ix2, g: Err(ooe), important: false },
      ];
    }
  }
}

fn test(x1: f32, x2: f32, config: &Config) -> Vec<ResultWithCoords> {
  // test_chisel::<InCouple>(x1, x2, config)
 // test_chisel::<OutCouple>(x1, x2, config)
  test_planet(x1, x2, config)
 // test_rail(x1, x2, &config)
}

fn main() {
  // 23|20|61 => R=36.3; D=72.6
  // 23|22|64 => R=38.1; D=76.2

  let mut config = Config { z1: 26, z2: 14, z3: 55, show: false, radial_gap: None };

  config.show = true;

  if config.show {
    test(0.7, 0.9, &config);
    println!("Couple printed");
    return;
  }

  let size_x = 1000usize;
  let size_y = 2000usize;

  let coords = |i: usize, j: usize| -> (f32, f32) {
    ((i as f32) / 200.0 - 1.0, (size_y as f32 - j as f32) / 200.0 - 2.0)
  };

  let mut prev = Vec::new();
  let mut cur = Vec::new();

  print!("Start...");
  for i in 0..=size_x {
    let (x1, x2) = coords(i, 0);
    prev.push(test(x1, x2, &config));
  }

  let mut img = ImgBuffer::new(size_x, size_y);
  let white = (255, 255, 255);
  for j in 1..=size_y {
    cur.clear();
    let (x1, x2) = coords(0, j);
    cur.push(test(x1, x2, &config));

    print!("\r {j} of {size_y}");

    for i in 1..=size_x {
      let (x1, x2) = coords(i, j);
      cur.push(test(x1, x2, &config));
      let mut color = white;

      for k in 0..cur[i].len() {
        let c1 = &cur[i][k];
        let c2 = &cur[i - 1][k];
        let c3 = &prev[i][k];
        let c4 = &prev[i - 1][k];
        if has_any(c1, c2, c3, c4, |c| c.g.is_err())
          || has_any(c1, c2, c3, c4, |c| c.unwrap().incompatible())
        {
          if c1.important {
            color = (240, 240, 240);
          } else if color == white {
            color = (255, 255, 240);
          }
        }
      }

      for k in 0..cur[i].len() {
        let c1 = &cur[i][k];
        let c2 = &cur[i - 1][k];
        let c3 = &prev[i][k];
        let c4 = &prev[i - 1][k];

        if has_diff(c1, c2, c3, c4, |c| c.ix1 < 0) || has_diff(c1, c2, c3, c4, |c| c.ix2 < 0) {
          color = (0, 0, 0);
        } else if has_diff(c1, c2, c3, c4, |c| c.ix1.div_euclid(5))
          || has_diff(c1, c2, c3, c4, |c| c.ix2.div_euclid(5))
        {
          color = (180, 180, 180);
        } else if has_diff(c1, c2, c3, c4, |c| c.ix1) || has_diff(c1, c2, c3, c4, |c| c.ix2) {
          color = (230, 230, 230);
        }
      }

      for k in 0..cur[i].len() {
        let c1 = &cur[i][k];
        let c2 = &cur[i - 1][k];
        let c3 = &prev[i][k];
        let c4 = &prev[i - 1][k];

        if has_diff(c1, c2, c3, c4, |c| c.leg_cut1())
          || has_diff(c1, c2, c3, c4, |c| c.leg_cut2())
          || has_diff(c1, c2, c3, c4, |c| c.no_couple())
        {
          color = (255, 0, 0);
        } else if has_any(c1, c2, c3, c4, |c| c.g.is_err()) {
          // skip
        } else {
          let c1 = c1.unwrap();
          let c2 = c2.unwrap();
          let c3 = c3.unwrap();
          let c4 = c4.unwrap();

          if has_diff(c1, c2, c3, c4, |c| c.overlap == 0) {
            color = (255, 128, 0);
          } else if has_diff(c1, c2, c3, c4, |c| c.overlap == 1) {
            color = (128, 96, 0);
          } else if has_diff(c1, c2, c3, c4, |c| c.dl == -2)
            || has_diff(c1, c2, c3, c4, |c| c.dl == 2)
          {
            color = (192, 192, 128);
          } else if has_diff(c1, c2, c3, c4, |c| c.dl > 0) {
            color = (224, 224, 0);
          } else if has_diff(c1, c2, c3, c4, |c| c.tw1 == 0)
            || has_diff(c1, c2, c3, c4, |c| c.tw2 == 0)
          {
            color = (128, 255, 64);
          } else if has_diff(c1, c2, c3, c4, |c| c.tw1 == 1)
            || has_diff(c1, c2, c3, c4, |c| c.tw2 == 1)
          {
            color = (64, 255, 0);
          } else if has_diff(c1, c2, c3, c4, |c| c.tw1 == 2)
            || has_diff(c1, c2, c3, c4, |c| c.tw2 == 2)
          {
            color = (32, 128, 0);
          } else if has_diff(c1, c2, c3, c4, |c| c.interf1)
            || has_diff(c1, c2, c3, c4, |c| c.interf2)
          {
            color = (0, 64, 128);
          } else if has_diff(c1, c2, c3, c4, |c| c.radial_gap) {
            color = (64, 0, 128);
          }

          if has_any(c1, c2, c3, c4, |c| c.danger_fillet1 || c.danger_fillet2) {
            color.1 -= color.1 / 4;
            color.2 -= color.2 / 4;
          }
        }

        if color != white {
          img.put(i - 1, j - 1, color);
        }
      }
    }
    std::mem::swap(&mut prev, &mut cur);
  }

  println!("\rWork complete");
  img.save("img.png").unwrap();
  println!("Graph saved");
}
