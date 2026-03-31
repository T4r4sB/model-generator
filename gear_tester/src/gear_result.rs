use crate::config::*;
use gears::*;

#[derive(Debug, Copy, Clone)]
pub struct GearError {
  pub no_gear_1: bool,
  pub no_gear_2: bool,
  pub no_couple: bool,
}

#[derive(Debug)]
pub struct GearResult {
  pub tw1: i32,
  pub tw2: i32,
  pub overlap: i32,
  pub dl: i32,
  pub interf1: bool,
  pub interf2: bool,
  pub danger_fillet1: bool,
  pub danger_fillet2: bool,
  pub radial_gap: bool,
}

impl GearResult {
  pub fn incompatible(&self) -> bool {
    self.tw1 == 0
      || self.tw2 == 0
      || self.overlap == 0
      || self.interf1
      || self.interf2
      || self.radial_gap
  }
}

#[derive(Debug)]
pub struct ResultWithCoords {
  pub ix1: i32,
  pub ix2: i32,
  pub g: Result<GearResult, GearError>,
  pub important: bool,
}

impl ResultWithCoords {
  pub fn leg_cut1(&self) -> bool {
    match &self.g {
      Err(e) => e.no_gear_1,
      _ => false,
    }
  }

  pub fn leg_cut2(&self) -> bool {
    match &self.g {
      Err(e) => e.no_gear_2,
      _ => false,
    }
  }

  pub fn no_couple(&self) -> bool {
    match &self.g {
      Err(e) => e.no_couple,
      _ => false,
    }
  }

  pub fn unwrap(&self) -> &GearResult {
    match &self.g {
      Ok(g) => g,
      _ => panic!("unwrapping erroneous GearResult"),
    }
  }
}

pub fn adjust_couple<C: Couple>(c: &mut C, config: &Config) {
  if let Some(radial_gap) = config.radial_gap {
    c.cut_by_radial_gap(radial_gap);
  } else {
    c.cut_by_radial_gap(MINIMAL_RADIAL_GAP);
    c.cut_by_interference();
    c.cut_by_top_w(0.2);
  }
}

pub fn couple_stats(c: &(impl Couple + std::fmt::Debug)) -> GearResult {
  fn overlap_to_index(w: f32) -> i32 {
    return if w <= 1.0 {
      0
    } else if w <= 1.2 {
      1
    } else {
      2
    };
  }

  fn top_w_to_index(w: f32) -> i32 {
    if w <= 0.0 {
      return 0;
    } else if w <= 0.125 {
      return 1;
    } else if w <= 0.2 {
      return 2;
    } else {
      return 3;
    }
  }

  fn l_to_index(l1: f32, l2: f32) -> i32 {
    if l2 <= 0.0 {
      return -2;
    } else if l2 <= l1 {
      return -1;
    } else if l1 > 0.0 {
      return 1;
    } else {
      return 2;
    }
  }

  GearResult {
    tw1: top_w_to_index(c.get_top_w1()),
    tw2: top_w_to_index(c.get_top_w2()),
    overlap: overlap_to_index(c.get_overlap()),
    interf1: c.has_g1_interference(),
    interf2: c.has_g2_interference(),
    danger_fillet1: c.get_g1().get_fillet_r() < 0.001,
    danger_fillet2: c.get_g2().get_fillet_r() < 0.001,
    radial_gap: c.failed_raidal_gap1() || c.failed_raidal_gap2(),
    dl: l_to_index(c.get_l1(), c.get_l2()),
  }
}


  
  pub fn has_diff<T, C: Eq>(
    c1: &T,
    c2: &T,
    c3: &T,
    c4: &T,
    f: impl Fn(&T) -> C,
  ) -> bool {
    let f1 = f(c1);
    let f2 = f(c2);
    let f3 = f(c3);
    let f4 = f(c4);
    f1 != f2 || f1 != f3 || f1 != f4
  }

  pub fn has_any<T>(
    c1: &T,
    c2: &T,
    c3: &T,
    c4: &T,
    f: impl Fn(&T) -> bool,
  ) -> bool {
    let f1 = f(c1);
    let f2 = f(c2);
    let f3 = f(c3);
    let f4 = f(c4);
    f1 || f2 || f3 || f4
  }