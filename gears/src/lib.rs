const PI: f32 = std::f32::consts::PI;
const EPS: f32 = 0.00001;
pub const MINIMAL_RADIAL_GAP: f32 = 0.05;

include!(concat!(env!("OUT_DIR"), "/constants.rs"));

#[derive(Debug, Copy, Clone)]
pub struct Profile {
  pub z: usize,
  pub x: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct OutGear {
  pub profile: Profile,
  pub evo_in_tan: f32,
  pub r_in: f32,
  pub r_out: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct InGear {
  pub profile: Profile,
  pub evo_out_tan: f32,
  pub r_in: f32,
  pub r_out: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Rail {}

#[derive(Debug, Copy, Clone)]
pub struct Perfect {}

#[derive(Debug, Copy, Clone)]
pub struct Chisel {
  pub g: OutGear,
}

#[derive(Debug, Copy, Clone)]
pub struct OutCouple {
  pub g1: OutGear,
  pub g2: OutGear,
  pub dist: f32,
  pub angle: f32,
  pub length: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct InCouple {
  pub g1: OutGear,
  pub g2: InGear,
  pub dist: f32,
  pub angle: f32,
  pub length: f32,
}

// Free functions

fn a2t(a: f32) -> f32 {
  a.tan() - a
}

fn t2a(t: f32) -> f32 {
  let mut a0 = 0.0f32.to_radians();
  let mut a1 = 89.0f32.to_radians();
  loop {
    let am = (a0 + a1) * 0.5;
    if am == a0 || am == a1 {
      return am;
    }
    if a2t(am) > t {
      a1 = am
    } else {
      a0 = am
    }
  }
}

fn cath(h: f32, c: f32) -> f32 {
  (h * h - c * c).sqrt()
}

fn hypot(c1: f32, c2: f32) -> f32 {
  (c1 * c1 + c2 * c2).sqrt()
}

// Class implementations

impl Profile {
  pub fn basic_r(&self) -> f32 {
    self.z as f32 * 0.5 * COS0
  }

  pub fn basic_w(&self) -> f32 {
    T0 * self.basic_r() + PI * 0.25 * COS0 + self.x * SIN0
  }

  pub fn w(&self, r: f32) -> f32 {
    let a = f32::min(1.0, self.basic_r() / r).acos();
    (self.basic_w() / self.basic_r() - a2t(a)) * r
  }

  pub fn basic_w_angle(&self) -> f32 {
    T0 + (PI * 0.25 * COS0 + self.x * SIN0) / self.basic_r()
  }

  pub fn w_angle(&self, r: f32) -> f32 {
    let a = f32::min(1.0, self.basic_r() / r).acos();
    self.basic_w_angle() - a2t(a)
  }

  pub fn max_possible_r(&self) -> f32 {
    let max_t = self.basic_w_angle();
    let max_a = t2a(max_t);
    self.basic_r() / max_a.cos() - EPS
  }

  pub fn teeth_overlap_r(&self) -> f32 {
    let min_t = self.basic_w_angle() - PI / self.z as f32;
    if min_t <= 0.0 {
      return 0.0;
    }
    let min_a = t2a(min_t);
    self.basic_r() / min_a.cos() + EPS
  }
}

pub trait Gear {
  fn inner(&self) -> bool;
  fn top_r(&self) -> f32;
  fn w_at(&self, r: f32) -> f32;
  fn evo_couple_part(&self) -> f32;
  fn total_couple_part(&self) -> f32;
  fn get_fillet_r(&self) -> f32;

  fn cut_by_top_w(&mut self, w: f32);
  fn cut(&mut self, r: f32);

  fn r_in(&self) -> f32;
  fn r_out(&self) -> f32;
  fn r_evo(&self) -> f32;
  fn profile(&self) -> &Profile;

  fn top_w(&self) -> f32 {
    self.w_at(self.top_r())
  }
}

impl Gear for OutGear {
  fn inner(&self) -> bool {
    false
  }

  fn top_r(&self) -> f32 {
    self.r_out
  }
  fn r_in(&self) -> f32 {
    self.r_in
  }
  fn r_out(&self) -> f32 {
    self.r_out
  }
  fn r_evo(&self) -> f32 {
    hypot(1.0, self.evo_in_tan) * self.profile.basic_r()
  }
  fn profile(&self) -> &Profile {
    &self.profile
  }
  fn w_at(&self, r: f32) -> f32 {
    self.profile.w(r)
  }
  fn evo_couple_part(&self) -> f32 {
    self.profile.basic_r() * self.evo_in_tan
  }
  fn total_couple_part(&self) -> f32 {
    cath(self.r_out, self.profile.basic_r())
  }
  fn get_fillet_r(&self) -> f32 {
    hypot(self.evo_in_tan, 1.0) * self.profile.basic_r() - self.r_in
  }

  fn cut_by_top_w(&mut self, w: f32) {
    if self.top_w() > w {
      return;
    }

    let mut r2 = self.r_out;
    let mut r1 = f32::max(self.r_in, self.profile.basic_r());
    loop {
      let r = (r1 + r2) * 0.5;
      if r == r1 || r == r2 {
        self.r_out = f32::max(r - EPS, self.profile.basic_r());
        return;
      }
      if self.w_at(r) > w {
        r1 = r;
      } else {
        r2 = r;
      }
    }
  }

  fn cut(&mut self, r: f32) {
    self.r_out = r.clamp(f32::max(self.r_in, self.profile.basic_r() + EPS), self.r_out);
  }
}

impl Gear for InGear {
  fn inner(&self) -> bool {
    true
  }

  fn top_r(&self) -> f32 {
    self.r_in
  }
  fn r_in(&self) -> f32 {
    self.r_in
  }
  fn r_out(&self) -> f32 {
    self.r_out
  }
  fn r_evo(&self) -> f32 {
    hypot(1.0, self.evo_out_tan) * self.profile.basic_r()
  }
  fn profile(&self) -> &Profile {
    &self.profile
  }
  fn w_at(&self, r: f32) -> f32 {
    r * PI / self.profile.z as f32 - self.profile.w(r)
  }
  fn evo_couple_part(&self) -> f32 {
    self.profile.basic_r() * self.evo_out_tan
  }
  fn total_couple_part(&self) -> f32 {
    cath(self.r_in, self.profile.basic_r())
  }
  fn get_fillet_r(&self) -> f32 {
    self.r_out - hypot(self.evo_out_tan, 1.0) * self.profile.basic_r()
  }

  fn cut_by_top_w(&mut self, w: f32) {
    if self.top_w() > w {
      return;
    }

    let mut r2 = self.r_out;
    let mut r1 = self.r_in;
    loop {
      let r = (r1 + r2) * 0.5;
      if r == r1 || r == r2 {
        self.r_in = f32::min(r + EPS, self.r_out);
        return;
      }
      if self.w_at(r) > w {
        r2 = r;
      } else {
        r1 = r;
      }
    }
  }

  fn cut(&mut self, r: f32) {
    self.r_in = r.clamp(self.r_in, self.r_out);
  }
}

impl Rail {
  pub fn new() -> Self {
    Self {}
  }

  pub fn produce_out_gear(&self, profile: Profile) -> Result<OutGear, String> {
    let r = profile.z as f32 * 0.5;
    let dz = profile.x;
    let r_in = r + dz - 1.25;
    let r_out = r + dz + 1.0;

    let evo_in_tan = TAN0 - (1.0 - dz) / (r * SIN0 * COS0);

    if evo_in_tan < 0.0 {
      return Err("incorrect parameters, the leg is clipped".to_owned());
    }
    Ok(OutGear { profile, evo_in_tan, r_in, r_out })
  }
}

impl Perfect {
  pub fn new() -> Self {
    Self {}
  }
}

impl Perfect {
  pub fn produce_out_gear(&self, profile: Profile) -> Result<OutGear, String> {
    let r_in = profile.teeth_overlap_r();
    let r_out = profile.max_possible_r();
    let evo_in_tan = 0.0;
    Ok(OutGear { profile, evo_in_tan, r_in, r_out })
  }

  pub fn produce_in_gear(&self, profile: Profile) -> Result<InGear, String> {
    let r_in = f32::max(profile.teeth_overlap_r(), profile.basic_r());
    let r_out = profile.max_possible_r();
    let evo_out_tan = 0.0;
    Ok(InGear { profile, evo_out_tan, r_in, r_out })
  }
}

impl Chisel {
  fn by_profile_and_tan(profile: Profile, evo_in_tan: f32) -> Self {
    Self::by_profile_f_and_tan(profile, evo_in_tan, 1.25)
  }

  fn by_profile_f_and_tan(profile: Profile, evo_in_tan: f32, f: f32) -> Self {
    let r = profile.z as f32 * 0.5 + profile.x;
    Self { g: OutGear { profile, evo_in_tan, r_in: r - f, r_out: r + f } }
  }

  pub fn new_m1_z26() -> Self {
    Self::by_profile_and_tan(Profile { z: 26, x: 0.105 }, 0.08991)
  }

  pub fn old_m1_z26() -> Self {
    Self::by_profile_and_tan(Profile { z: 26, x: -0.540 }, 0.0)
  }
  
  pub fn new_m1_z38() -> Self {
    Self::by_profile_and_tan(Profile { z: 38, x: 0.105 }, 0.17647)
  }

  pub fn old_m1_z38() -> Self {
    Self::by_profile_and_tan(Profile { z: 38, x: -0.640 }, 0.05446)
  }

  pub fn new_m1_z50() -> Self {
    Self::by_profile_and_tan(Profile { z: 50, x: 0.11 }, 0.22207)
  }

  pub fn old_m1_z50() -> Self {
    Self::by_profile_and_tan(Profile { z: 50, x: -0.70 }, 0.12128)
  }

  pub fn new_m1_z100() -> Self {
    Self::by_profile_and_tan(Profile { z: 100, x: 1.050 }, 0.35153)
  }

  pub fn old_m1_z100() -> Self {
    Self::by_profile_and_tan(Profile { z: 100, x: 0.0 }, 0.28618)
  }

  pub fn new_m2_z50() -> Self {
    Self::by_profile_and_tan(Profile { z: 50, x: 0.577 }, 0.28033)
  }

  pub fn old_m2_z50() -> Self {
    Self::by_profile_and_tan(Profile { z: 50, x: -0.10 }, 0.19596)
  }

  pub fn new_m5_5_z22() -> Self {
    Self::by_profile_f_and_tan(Profile { z: 22, x: 0.105 }, 0.0, 1.3)
  }

  pub fn old_m5_5_z22() -> Self {
    Self::by_profile_f_and_tan(Profile { z: 22, x: -0.18 }, 0.0, 1.3)
  }

  pub fn new_m9_z14() -> Self {
    Self::by_profile_f_and_tan(Profile { z: 14, x: 0.105 }, 0.0, 1.3)
  }

  pub fn old_m9_z14() -> Self {
    Self::by_profile_f_and_tan(Profile { z: 14, x: -0.1 }, 0.0, 1.3)
  }

  pub fn produce_out_instrument_couple(&self, profile: Profile) -> Result<OutCouple, String> {
    let angle = OutCouple::couple_angle(self.g.profile, profile)?;
    let dist = OutCouple::angle_to_dist(self.g.profile, profile, angle);
    let length = OutCouple::angle_dist_to_length(angle, dist);

    let mut result = OutGear {
      profile,
      evo_in_tan: (length - self.g.total_couple_part()) / profile.basic_r(),
      r_in: dist - self.g.r_out,
      r_out: dist - self.g.r_in,
    };

    if result.evo_in_tan < 0.0 {
      return Err("incorrect parameters, the leg is clipped".to_owned());
    }
    result.cut(hypot(length - self.g.evo_couple_part(), profile.basic_r()));

    Ok(OutCouple { g1: self.g, g2: result, dist, angle, length })
  }

  pub fn produce_in_instrument_couple(&self, profile: Profile) -> Result<InCouple, String> {
    let angle = InCouple::couple_angle(self.g.profile, profile)?;
    let dist = InCouple::angle_to_dist(self.g.profile, profile, angle);
    let length = InCouple::angle_dist_to_length(angle, dist);

    let mut result = InGear {
      profile,
      evo_out_tan: (length + self.g.total_couple_part()) / profile.basic_r(),
      r_in: dist + self.g.r_in,
      r_out: dist + self.g.r_out,
    };

    if result.r_in < result.profile.basic_r() {
      result.cut(profile.basic_r());
    }

    result.cut(hypot(length + self.g.evo_couple_part(), profile.basic_r()));

    Ok(InCouple { g1: self.g, g2: result, dist, angle, length })
  }

  pub fn produce_out_gear(&self, profile: Profile) -> Result<OutGear, String> {
    Ok(self.produce_out_instrument_couple(profile)?.g2)
  }

  pub fn produce_in_gear(&self, profile: Profile) -> Result<InGear, String> {
    Ok(self.produce_in_instrument_couple(profile)?.g2)
  }
}

pub trait Couple {
  type G1: Gear;
  type G2: Gear;

  fn inner(&self) -> bool;

  fn produce_g1(instrument: &Chisel, profile: Profile) -> Result<Self::G1, String>;

  fn produce_g2(instrument: &Chisel, profile: Profile) -> Result<Self::G2, String>;

  fn new_g2_couple(instrument: &Chisel, profile: Profile) -> Result<Self, String>
  where
    Self: Sized;

  fn new_without_cuts(g1: Self::G1, g2: Self::G2) -> Result<Self, String>
  where
    Self: Sized;

  fn new(g1: Self::G1, g2: Self::G2) -> Result<Self, String>
  where
    Self: Sized;

  fn new_with_custom_radial_gap(
    g1: Self::G1,
    g2: Self::G2,
    radial_gap: f32,
  ) -> Result<Self, String>
  where
    Self: Sized;

  fn get_g1(&self) -> &Self::G1;
  fn get_g2(&self) -> &Self::G2;

  fn basic_dist(z1: usize, z2: usize) -> f32;
  fn corr_cum(x1: f32, x2: f32) -> f32;

  fn couple_angle(g1: Profile, g2: Profile) -> Result<f32, String> {
    let corr_sum = Self::corr_cum(g1.x, g2.x);
    let dist0 = Self::basic_dist(g1.z, g2.z);
    let teta = T0 + corr_sum / dist0 * TAN0;
    if teta <= 0.0 {
      return Err("incorrect couple, gears are fall inside".to_owned());
    }
    Ok(t2a(teta))
  }

  fn angle_to_dist(g1: Profile, g2: Profile, angle: f32) -> f32 {
    Self::basic_dist(g1.z, g2.z) * COS0 / angle.cos()
  }

  fn dist(g1: Profile, g2: Profile) -> Result<f32, String> {
    Ok(Self::angle_to_dist(g1, g2, Self::couple_angle(g1, g2)?))
  }

  fn angle_dist_to_length(angle: f32, dist: f32) -> f32 {
    dist * angle.sin()
  }

  fn sum_of_corrections(z1: usize, z2: usize, dist: f32) -> f32 {
    let dist0 = Self::basic_dist(z1, z2);
    let a = (dist0 * COS0 / dist).acos();
    let t = a2t(a);
    (t - T0) * dist0 / TAN0
  }

  fn get_dist(&self) -> f32;
  fn get_angle(&self) -> f32;

  fn cut1(&mut self, r: f32);
  fn cut2(&mut self, r: f32);

  fn failed_raidal_gap1(&self) -> bool;
  fn failed_raidal_gap2(&self) -> bool;
  fn cut1_by_radial_gap(&mut self, radial_gap: f32);
  fn cut2_by_radial_gap(&mut self, radial_gap: f32);

  fn has_g1_interference(&self) -> bool;
  fn cut1_by_interference(&mut self);
  fn has_g2_interference(&self) -> bool;
  fn cut2_by_interference(&mut self);

  fn get_top_w1(&self) -> f32;
  fn get_top_w2(&self) -> f32;
  fn cut1_by_top_w(&mut self, w: f32);
  fn cut2_by_top_w(&mut self, w: f32);

  fn get_l1(&self) -> f32;
  fn get_l2(&self) -> f32;

  fn adjust_radial_gap_from_deepest_part(&mut self, radial_gap: f32);

  fn cut_by_radial_gap(&mut self, radial_gap: f32) {
    self.cut1_by_radial_gap(radial_gap);
    self.cut2_by_radial_gap(radial_gap);
  }

  fn cut_by_interference(&mut self) {
    self.cut1_by_interference();
    self.cut2_by_interference();
  }

  fn cut_by_top_w(&mut self, w: f32) {
    self.cut1_by_top_w(w);
    self.cut2_by_top_w(w);
  }

  fn get_couple_length(&self) -> f32 {
    Self::angle_dist_to_length(self.get_angle(), self.get_dist())
  }

  fn get_overlap(&self) -> f32 {
    (self.get_l1() + self.get_l2()) / (PI * COS0)
  }
}

impl Couple for OutCouple {
  type G1 = OutGear;
  type G2 = OutGear;

  fn inner(&self) -> bool {
    false
  }

  fn produce_g1(instrument: &Chisel, profile: Profile) -> Result<OutGear, String> {
    instrument.produce_out_gear(profile)
  }

  fn produce_g2(instrument: &Chisel, profile: Profile) -> Result<OutGear, String> {
    instrument.produce_out_gear(profile)
  }

  fn new_g2_couple(instrument: &Chisel, profile: Profile) -> Result<Self, String> {
    instrument.produce_out_instrument_couple(profile)
  }

  fn new_without_cuts(g1: OutGear, g2: OutGear) -> Result<Self, String> {
    let angle = Self::couple_angle(g1.profile, g2.profile)?;
    let dist = Self::angle_to_dist(g1.profile, g2.profile, angle);
    let length = Self::angle_dist_to_length(angle, dist);
    Ok(Self { g1, g2, angle, dist, length })
  }

  fn new(g1: OutGear, g2: OutGear) -> Result<Self, String> {
    Self::new_with_custom_radial_gap(g1, g2, 0.25)
  }

  fn new_with_custom_radial_gap(g1: OutGear, g2: OutGear, radial_gap: f32) -> Result<Self, String> {
    let mut result = Self::new_without_cuts(g1, g2)?;
    result.cut_by_radial_gap(radial_gap);
    Ok(result)
  }

  fn get_g1(&self) -> &OutGear {
    &self.g1
  }
  fn get_g2(&self) -> &OutGear {
    &self.g2
  }

  fn get_dist(&self) -> f32 {
    self.dist
  }

  fn get_angle(&self) -> f32 {
    self.angle
  }

  fn basic_dist(z1: usize, z2: usize) -> f32 {
    (z1 + z2) as f32 * 0.5
  }

  fn corr_cum(x1: f32, x2: f32) -> f32 {
    x1 + x2
  }

  fn cut1(&mut self, r: f32) {
    self.g1.cut(r);
  }

  fn cut2(&mut self, r: f32) {
    self.g2.cut(r);
  }

  fn cut1_by_radial_gap(&mut self, radial_gap: f32) {
    self.cut1(self.dist - self.g2.r_in - radial_gap);
  }

  fn cut2_by_radial_gap(&mut self, radial_gap: f32) {
    self.cut2(self.dist - self.g1.r_in - radial_gap);
  }

  fn adjust_radial_gap_from_deepest_part(&mut self, radial_gap: f32) {
    self.g1.r_in = f32::max(self.g1.r_in, self.dist - self.g2.r_out - radial_gap);
    self.g2.r_in = f32::max(self.g2.r_in, self.dist - self.g1.r_out - radial_gap);
  }

  fn failed_raidal_gap1(&self) -> bool {
    self.g1.r_out + self.g2.r_in >= self.dist
  }

  fn failed_raidal_gap2(&self) -> bool {
    self.g2.r_out + self.g1.r_in >= self.dist
  }

  fn has_g1_interference(&self) -> bool {
    self.g1.total_couple_part() > self.length - self.g2.evo_couple_part()
  }

  fn cut1_by_interference(&mut self) {
    let l = self.length - self.g2.evo_couple_part();
    self.cut1(hypot(self.g1.profile.basic_r(), l) - MINIMAL_RADIAL_GAP);
  }

  fn has_g2_interference(&self) -> bool {
    self.g2.total_couple_part() > self.length - self.g1.evo_couple_part()
  }

  fn cut2_by_interference(&mut self) {
    let l = self.length - self.g1.evo_couple_part();
    self.cut2(hypot(self.g2.profile.basic_r(), l) - MINIMAL_RADIAL_GAP);
  }

  fn get_top_w1(&self) -> f32 {
    self.g1.top_w()
  }

  fn get_top_w2(&self) -> f32 {
    self.g2.top_w()
  }

  fn cut1_by_top_w(&mut self, w: f32) {
    self.g1.cut_by_top_w(w);
  }

  fn cut2_by_top_w(&mut self, w: f32) {
    self.g2.cut_by_top_w(w);
  }

  fn get_l1(&self) -> f32 {
    self.g1.total_couple_part() - self.g1.profile.basic_r() * self.angle.tan()
  }

  fn get_l2(&self) -> f32 {
    self.g2.total_couple_part() - self.g2.profile.basic_r() * self.angle.tan()
  }
}

impl Couple for InCouple {
  type G1 = OutGear;
  type G2 = InGear;

  fn inner(&self) -> bool {
    true
  }

  fn produce_g1(instrument: &Chisel, profile: Profile) -> Result<OutGear, String> {
    instrument.produce_out_gear(profile)
  }

  fn produce_g2(instrument: &Chisel, profile: Profile) -> Result<InGear, String> {
    instrument.produce_in_gear(profile)
  }

  fn new_g2_couple(instrument: &Chisel, profile: Profile) -> Result<Self, String> {
    instrument.produce_in_instrument_couple(profile)
  }

  fn new_without_cuts(g1: OutGear, g2: InGear) -> Result<Self, String> {
    let angle = Self::couple_angle(g1.profile, g2.profile)?;
    let dist = Self::angle_to_dist(g1.profile, g2.profile, angle);
    let length = Self::angle_dist_to_length(angle, dist);
    Ok(Self { g1, g2, angle, dist, length })
  }

  fn new(g1: OutGear, g2: InGear) -> Result<Self, String> {
    Self::new_with_custom_radial_gap(g1, g2, 0.25)
  }

  fn new_with_custom_radial_gap(g1: OutGear, g2: InGear, radial_gap: f32) -> Result<Self, String> {
    let mut result = Self::new_without_cuts(g1, g2)?;
    result.cut_by_radial_gap(radial_gap);
    Ok(result)
  }

  fn get_g1(&self) -> &OutGear {
    &self.g1
  }

  fn get_g2(&self) -> &InGear {
    &self.g2
  }

  fn get_dist(&self) -> f32 {
    self.dist
  }

  fn get_angle(&self) -> f32 {
    self.angle
  }

  fn basic_dist(z1: usize, z2: usize) -> f32 {
    (z2 - z1) as f32 * 0.5
  }

  fn corr_cum(x1: f32, x2: f32) -> f32 {
    x2 - x1
  }

  fn cut1(&mut self, r: f32) {
    self.g1.cut(r);
  }

  fn cut2(&mut self, r: f32) {
    self.g2.cut(r);
  }

  fn cut1_by_radial_gap(&mut self, radial_gap: f32) {
    self.cut1(self.g2.r_out - self.dist - radial_gap);
  }

  fn cut2_by_radial_gap(&mut self, radial_gap: f32) {
    self.cut2(self.dist + self.g1.r_in + radial_gap);
  }

  fn adjust_radial_gap_from_deepest_part(&mut self, radial_gap: f32) {
    self.g1.r_in = f32::max(self.g1.r_in, self.g2.r_in - self.dist - radial_gap);
    self.g2.r_out = f32::min(self.g2.r_out, self.dist + self.g1.r_out + radial_gap);
  }

  fn has_g1_interference(&self) -> bool {
    self.g1.total_couple_part() + self.length > self.g2.evo_couple_part()
  }

  fn cut1_by_interference(&mut self) {
    let l = self.g2.evo_couple_part() - self.length;
    self.cut1(hypot(self.g1.profile.basic_r(), l) - MINIMAL_RADIAL_GAP);
  }

  fn has_g2_interference(&self) -> bool {
    self.g2.total_couple_part() < self.length + self.g1.evo_couple_part()
  }

  fn cut2_by_interference(&mut self) {
    let l = self.length + self.g1.evo_couple_part();
    self.cut2(hypot(self.g2.profile.basic_r(), l) + MINIMAL_RADIAL_GAP);
  }

  fn get_top_w1(&self) -> f32 {
    self.g1.top_w()
  }

  fn get_top_w2(&self) -> f32 {
    self.g2.top_w()
  }

  fn cut1_by_top_w(&mut self, w: f32) {
    self.g1.cut_by_top_w(w);
  }

  fn cut2_by_top_w(&mut self, w: f32) {
    self.g2.cut_by_top_w(w);
  }

  fn failed_raidal_gap1(&self) -> bool {
    self.g2.r_out <= self.g1.r_out + self.dist
  }

  fn failed_raidal_gap2(&self) -> bool {
    self.g2.r_in <= self.g1.r_in + self.dist
  }

  fn get_l1(&self) -> f32 {
    self.g1.total_couple_part() - self.g1.profile.basic_r() * self.angle.tan()
  }

  fn get_l2(&self) -> f32 {
    self.g2.profile.basic_r() * self.angle.tan() - self.g2.total_couple_part()
  }

  fn get_overlap(&self) -> f32 {
    (self.get_l1() + self.get_l2()) / (PI * COS0)
  }
}

#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[test]
  fn test_dist() {
    let g1 = Profile { z: 10, x: 0.6 };
    let g2 = Profile { z: 26, x: 0.12 };
    let dist = OutCouple::dist(g1, g2).unwrap();
    let expected = 18.642;
    assert!((dist - expected).abs() < 0.001, "Expected {expected}, actual {dist}");
  }

  #[test]
  fn test_sum_corr() {
    let corr_sum = OutCouple::sum_of_corrections(10, 26, 18.642);
    let expected = 0.72;
    assert!((corr_sum - expected).abs() < 0.01, "Expected {expected}, actual {corr_sum}");
  }

  #[test]
  fn test_rail_leg_cut() {
    let r = Rail {};
    let g = r.produce_out_gear(Profile { z: 17, x: 0.01 });
    assert!(g.is_ok());

    let g = r.produce_out_gear(Profile { z: 16, x: 0.06 });
    assert!(g.is_err());

    let g = r.produce_out_gear(Profile { z: 16, x: 0.07 });
    assert!(g.is_ok());
  }

  #[test]
  fn test_complete_profiles() {
    let g1 = Profile { z: 10, x: 0.6 };
    let g2 = Profile { z: 26, x: 0.12 };
    let r = Rail {};
    let g1 = r.produce_out_gear(g1).unwrap();
    let g2 = r.produce_out_gear(g2).unwrap();
    let couple = OutCouple::new(g1, g2).unwrap();
    let expected = 6.522;
    assert!(
      (couple.g1.r_out - expected).abs() < 0.001,
      "Expected {expected}, actual {}",
      couple.g1.r_out
    );
    let expected = 14.042;
    assert!(
      (couple.g2.r_out - expected).abs() < 0.001,
      "Expected {expected}, actual {}",
      couple.g2.r_out
    );
  }

  #[test]
  fn test_top_width() {
    let g1 = Profile { z: 12, x: 0.8 };
    let g2 = Profile { z: 14, x: 0.4 };
    let r = Rail {};

    let g1 = r.produce_out_gear(g1).unwrap();
    let g2 = r.produce_out_gear(g2).unwrap();
    let couple = OutCouple::new(g1, g2).unwrap();
    let w1 = couple.g1.top_w();
    let expected = 0.21;
    assert!((w1 - expected).abs() < 0.01, "Expected {expected}, actual {w1}");
  }

  #[test]
  fn test_overlap() {
    let g1 = Profile { z: 12, x: 0.4 };
    let g2 = Profile { z: 15, x: 0.4 };
    let r = Rail {};

    let g1 = r.produce_out_gear(g1).unwrap();
    let g2 = r.produce_out_gear(g2).unwrap();
    let couple = OutCouple::new(g1, g2).unwrap();
    let overlap = couple.get_overlap();
    let expected = 1.19;
    assert!((overlap - expected).abs() < 0.01, "Expected {expected}, actual {overlap}");
  }

  #[test]
  fn test_overlap_extremal_case() {
    let g1 = Profile { z: 12, x: 0.6 };
    let g2 = Profile { z: 34, x: 0.6 };
    let r = Rail {};

    let g1 = r.produce_out_gear(g1).unwrap();
    let g2 = r.produce_out_gear(g2).unwrap();
    let couple = OutCouple::new(g1, g2).unwrap();
    let overlap = couple.get_overlap();
    let expected = 1.2;
    assert!((overlap - expected).abs() < 0.01, "Expected {expected}, actual {overlap}");
  }

  #[test]
  fn test_has_interference() {
    let g1 = Profile { z: 12, x: 0.4 };
    let g2 = Profile { z: 42, x: 1.8 };
    let r = Rail {};

    let g1 = r.produce_out_gear(g1).unwrap();
    let g2 = r.produce_out_gear(g2).unwrap();
    let couple = OutCouple::new(g1, g2).unwrap();
    let i1 = couple.has_g1_interference();
    let i2 = couple.has_g2_interference();
    assert!(i1);
    assert!(!i2);
  }

  #[test]
  fn test_no_interference() {
    let g1 = Profile { z: 12, x: 0.4 };
    let g2 = Profile { z: 42, x: 1.6 };
    let r = Rail {};

    let g1 = r.produce_out_gear(g1).unwrap();
    let g2 = r.produce_out_gear(g2).unwrap();
    let couple = OutCouple::new(g1, g2).unwrap();
    let i1 = couple.has_g1_interference();
    let i2 = couple.has_g2_interference();
    assert!(!i1);
    assert!(!i2);
  }

  #[test]
  fn test_extremal_interference() {
    let g1 = Profile { z: 12, x: 0.3 };
    let g2 = Profile { z: 34, x: -0.8 };
    let r = Rail {};

    let g1 = r.produce_out_gear(g1).unwrap();
    let g2 = r.produce_out_gear(g2).unwrap();
    let couple = OutCouple::new(g1, g2).unwrap();
    let i1 = couple.has_g1_interference();
    let i2 = couple.has_g2_interference();
    assert!(!i1);
    assert!(i2);
  }

  #[test]
  fn test_extremal_no_interference() {
    let g1 = Profile { z: 12, x: 0.3 };
    let g2 = Profile { z: 34, x: -0.6 };
    let r = Rail {};

    let g1 = r.produce_out_gear(g1).unwrap();
    let g2 = r.produce_out_gear(g2).unwrap();
    let couple = OutCouple::new(g1, g2).unwrap();
    let i1 = couple.has_g1_interference();
    let i2 = couple.has_g2_interference();
    assert!(!i1);
    assert!(!i2);
  }

  #[test]
  fn test_cut_for_w() {
    let g1 = Profile { z: 17, x: 0.6 };
    let g2 = Profile { z: 42, x: -0.6 };
    let r = Rail {};

    let g1 = r.produce_out_gear(g1).unwrap();
    let g2 = r.produce_out_gear(g2).unwrap();
    let mut couple = OutCouple::new(g1, g2).unwrap();
    assert!(couple.g1.top_w() < 0.2);
    assert!(couple.g2.top_w() > 0.2);
    couple.cut_by_top_w(0.2);
    assert!(couple.g1.top_w() > 0.2);
    assert!(couple.g2.top_w() > 0.2);
  }

  #[test]
  fn test_cut_for_interference() {
    let g1 = Profile { z: 17, x: 0.2 };
    let g2 = Profile { z: 42, x: -0.8 };
    let r = Rail {};

    let g1 = r.produce_out_gear(g1).unwrap();
    let g2 = r.produce_out_gear(g2).unwrap();
    let mut couple = OutCouple::new(g1, g2).unwrap();
    let i1 = couple.has_g1_interference();
    let i2 = couple.has_g2_interference();
    assert!(!i1);
    assert!(i2);
    couple.cut_by_interference();
    let i1 = couple.has_g1_interference();
    let i2 = couple.has_g2_interference();
    assert!(!i1);
    assert!(!i2);
  }

  #[test]
  fn test_round_leg_cut() {
    let r = Chisel::new_m1_z100();
    let g = r.produce_out_gear(Profile { z: 18, x: 0.02 });
    assert!(g.is_err());
    let g = r.produce_out_gear(Profile { z: 18, x: 0.1 });
    assert!(g.is_ok());
  }
}
