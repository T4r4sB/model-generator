use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points2d;
use common::points3d::*;
use common::solid::*;
use lazy_static::*;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

lazy_static! {
  static ref RAYS3: [points2d::Point; 3] = [
    points2d::Point::from_angle(0.0),
    points2d::Point::from_angle(2.0 * PI / 3.0),
    points2d::Point::from_angle(4.0 * PI / 3.0),
  ];
}

fn sqr(x: f32) -> f32 {
  x * x
}

fn xy(p: Point) -> points2d::Point {
  points2d::Point { x: p.x, y: p.y }
}

/*
   69        63    |  69
   22 13  |  16 22 |  20
   24 33  |  30 24 |  27

Input 100
Gear widths: 2.52 3.17 3.72 2.71 8.78
Bearing loading: 1344.09 1984.13 4684.53
Sun gear torques: 25.81 44.73 47.62 27.71
*/

struct GearProfile {
  teeth_count: f32,
  phase: f32,
  inner: bool,

  ir: f32,
  r_min: f32,
  r_max: f32,
  tan0: f32,
  a1: f32,
}

struct GearProfileParams {
  teeth_count: i32,
  phase: f32,
  module: f32,
  correction: f32,
  inner: bool,
}

impl GearProfileParams {
  fn create(self) -> GearProfile {
    let teeth_count = self.teeth_count as f32;

    let ga = 2.0 * PI / teeth_count;
    let ir = 20.0.to_radians().cos() * teeth_count * self.module * 0.5;

    let mr = (teeth_count * 0.5 + self.correction) * self.module;

    let a0 = (ir / mr).acos();
    let tan0 = mr * a0.sin();
    let a1 = a0 - ga * 0.25;

    let inner = self.inner;

    let r_min;
    let r_max;
    if inner {
      r_min = mr - self.module;
      r_max = mr + self.module * 1.25;
    } else {
      r_min = mr - self.module * 1.25;
      r_max = mr + self.module;
    }

    let phase = self.phase;

    GearProfile { teeth_count, phase, inner, ir, r_min, r_max, tan0, a1 }
  }
}

impl GearProfile {
  fn contains(&self, pos: points2d::Point, err: f32) -> bool {
    let r = pos.len();
    if self.inner {
      if r <= self.r_min + err {
        return false;
      } else if r >= self.r_max + err {
        return true;
      }
    } else {
      if r <= self.r_min - err {
        return true;
      } else if r >= self.r_max - err {
        return false;
      }
    }
    let a = f32::atan2(pos.y, pos.x);
    let ga = 2.0 * PI / self.teeth_count;
    let a = a + self.phase * ga;
    let a = a.rem_euclid(ga);
    let a = if a > ga * 0.5 { ga - a } else { a };
    let da = f32::min(1.0, self.ir / r).acos();
    let tanl = r * da.sin();

    if self.inner {
      tanl - self.tan0 > (da - a - self.a1) * self.ir + err
    } else {
      tanl - self.tan0 + err < (da - a - self.a1) * self.ir
    }
  }
}

struct GearParams {
  profile: GearProfile,
  z1: f32,
  z2: f32,
  index: PartIndex,
}

impl GearParams {
  fn create(self) -> Gear {
    Gear {
      profile: self.profile,
      shift: points2d::Point::ZERO,
      z1: self.z1,
      z2: self.z2,
      index: self.index,
      r_in: 0.0,
      r_out: f32::INFINITY,
      labeled_top: false,
      labeled_bottom: false,
      label_phase: 0,
      axle_clutch: None,
    }
  }
}

struct GearAxleClutchParams {
  groove1: f32,
  clutch1: f32,
  groove2: f32,
  clutch2: f32,
  clutch_count: i32,
}

struct Gear {
  profile: GearProfile,
  shift: points2d::Point,
  z1: f32,
  z2: f32,
  index: PartIndex,
  r_in: f32,
  r_out: f32,
  label_phase: usize,
  labeled_top: bool,
  labeled_bottom: bool,
  axle_clutch: Option<GearAxleClutchParams>,
}

impl Gear {
  fn shift(mut self, shift: points2d::Point) -> Self {
    self.shift = shift;
    self
  }
  fn r_in(mut self, r_in: f32) -> Self {
    self.r_in = r_in;
    self
  }
  fn r_out(mut self, r_out: f32) -> Self {
    self.r_out = r_out;
    self
  }
  fn labeled_top(mut self, labeled_top: bool, label_phase: usize) -> Self {
    self.labeled_top = labeled_top;
    self.label_phase = label_phase;
    self
  }
  fn labeled_bottom(mut self, labeled_bottom: bool, label_phase: usize) -> Self {
    self.labeled_bottom = labeled_bottom;
    self.label_phase = label_phase;
    self
  }
  fn axle_clutch(mut self, axle_clutch: GearAxleClutchParams) -> Self {
    self.axle_clutch = Some(axle_clutch);
    self
  }

  fn contains(&self, pos: Point, err: f32, g_err: f32) -> bool {
    let z = pos.z;
    if z < self.z1 || z > self.z2 {
      return false;
    }
    let pos = xy(pos) - self.shift;
    let r = pos.len();
    if r < self.r_in + err || r > self.r_out - err {
      return false;
    }

    if self.labeled_bottom {
      if z < self.z1 + 1.5
        && (pos - RAYS3[self.label_phase].scale(self.profile.r_min - 2.5)).sqr_len() < 1.0
      {
        return false;
      }
    }
    if self.labeled_top {
      if z > self.z2 - 1.5
        && (pos - RAYS3[self.label_phase].scale(self.profile.r_min - 2.5)).sqr_len() < 1.0
      {
        return false;
      }
    }
    if let Some(axle_clutch) = &self.axle_clutch {
      if z < self.z1 + axle_clutch.groove1 || z > self.z2 - axle_clutch.groove2 {
        if r < self.r_in + err + 2.0 {
          return false;
        }
      }
      if z < self.z1 + axle_clutch.groove1 + axle_clutch.clutch1
        || z > self.z2 - (axle_clutch.groove2 + axle_clutch.clutch2)
      {
        if r < self.r_in + err + 2.0 {
          if pos.x.abs() < 2.5 + err || axle_clutch.clutch_count > 2 && pos.y.abs() < 2.5 + err {
            return false;
          }
        }
      }
    }

    self.profile.contains(pos, g_err)
  }
}

#[derive(Clone, Copy)]
struct SatHolesInfo {
  sat_r: f32,
  axle_r: f32,
}

#[derive(Clone, Copy)]
struct SpringInfo {
  shift: f32,
  spring_r: f32,
  axle_r: f32,
}

struct CylinderParams {
  z1: f32,
  z2: f32,
  r_in: f32,
  r_out: f32,
  index: PartIndex,
}

impl CylinderParams {
  fn create(self) -> Cylinder {
    Cylinder {
      z1: self.z1,
      z2: self.z2,
      r_in: self.r_in,
      r_out: self.r_out,
      index: self.index,
      shift: points2d::Point::ZERO,
      holes_pos: None,
      holes_neg: None,
      spring_info: None,
    }
  }
}

struct Cylinder {
  z1: f32,
  z2: f32,
  r_in: f32,
  r_out: f32,
  shift: points2d::Point,
  holes_pos: Option<SatHolesInfo>,
  holes_neg: Option<SatHolesInfo>,
  spring_info: Option<SpringInfo>,
  index: PartIndex,
}

impl Cylinder {
  fn shift(mut self, shift: points2d::Point) -> Self {
    self.shift = shift;
    self
  }

  fn holes_pos(mut self, holes_pos: SatHolesInfo) -> Self {
    self.holes_pos = Some(holes_pos);
    self
  }

  fn holes_neg(mut self, holes_neg: SatHolesInfo) -> Self {
    self.holes_neg = Some(holes_neg);
    self
  }

  fn spring_info(mut self, spring_info: SpringInfo) -> Self {
    self.spring_info = Some(spring_info);
    self
  }

  fn contains(&self, pos: Point, err: f32) -> bool {
    let z = pos.z;
    if z < self.z1 || z > self.z2 {
      return false;
    }
    let pos = xy(pos) - self.shift;
    let r = pos.len();
    if r > self.r_in + err && r < self.r_out - err {
      if let Some(holes) = &self.holes_pos {
        for i in 0..3 {
          if (pos - RAYS3[i].scale(holes.sat_r)).sqr_len() < sqr(holes.axle_r + err) {
            return false;
          }
        }
      }
      if let Some(holes) = &self.holes_neg {
        for i in 0..3 {
          if (pos + RAYS3[i].scale(holes.sat_r)).sqr_len() < sqr(holes.axle_r + err) {
            return false;
          }
        }
      }
      if let Some(spring_info) = &self.spring_info {
        let dz = z - (self.z1 + spring_info.shift);
        let dr = f32::min(0.0, r - spring_info.axle_r);
        if sqr(dz) + sqr(dr) < sqr(spring_info.spring_r + err) {
          return false;
        }
      }
      return true;
    }
    false
  }
}

struct Spring {
  z1: f32,
  z2: f32,
  r_in: f32,
  r_out: f32,
  index: PartIndex,
}

impl Spring {
  fn contains(&self, pos: Point, err: f32) -> bool {
    let z = pos.z;
    if z < self.z1 || z > self.z2 {
      return false;
    }
    let pos = xy(pos);
    let r = pos.len();
    if r > self.r_in + err && r < self.r_out - err {
      if z < self.z1 + 1.0 || z > self.z2 - 1.0 {
        return true;
      }

      let a = f32::atan2(pos.y, pos.x) / (2.0 * PI) * 6.0 * self.r_out - z;
      let a = a.rem_euclid(self.r_out);
      if a > 3.0 {
        return false;
      }

      return true;
    }
    false
  }
}

struct Cam {
  z1: f32,
  z2: f32,
  r_in: f32,
  r_out: f32,
  index: PartIndex,
}

impl Cam {
  fn contains(&self, pos: Point, err: f32) -> bool {
    let z = pos.z;
    if z < self.z1 || z > self.z2 {
      return false;
    }
    let pos = xy(pos);
    let r = pos.len();
    if r > self.r_in + err && r < self.r_out - err {
      if pos.x.abs() < 1.9 - err {
        return true;
      }
    }
    false
  }
}

struct CarrierUnitParams {
  wall1: Cylinder,
  wall2: Cylinder,
  body_r_in: f32,
  body_r_out: f32,
  sat_holes_info: SatHolesInfo,
  body_width: f32,
}

impl CarrierUnitParams {
  fn create(self) -> (Cylinder, Cylinder, CarrierUnitConnector) {
    let conn_holes_info = SatHolesInfo {
      sat_r: (self.body_r_in + self.body_r_out) * 0.5,
      axle_r: f32::min(self.body_r_out - self.body_r_in, self.body_width) * 0.5 - 3.0,
    };

    let body = CylinderParams {
      z1: self.wall1.z2,
      z2: self.wall2.z1,
      r_in: self.body_r_in,
      r_out: self.body_r_out,
      index: self.wall1.index,
    }
    .create();

    let connector = CarrierUnitConnector {
      body,
      body_width: self.body_width,
      connector_h: self.wall2.z2 - self.wall2.z1,
      connector_r: conn_holes_info.sat_r,
      connector_spike_r: conn_holes_info.axle_r,
    };

    let half_1 = self.wall1.holes_pos(self.sat_holes_info);
    let half_2 = self.wall2.holes_pos(self.sat_holes_info).holes_neg(conn_holes_info);

    (half_1, half_2, connector)
  }
}

struct CarrierUnitConnector {
  body: Cylinder,
  body_width: f32,
  connector_spike_r: f32,
  connector_r: f32,
  connector_h: f32,
}

impl CarrierUnitConnector {
  fn contains(&self, pos: Point, err: f32) -> bool {
    if pos.z > self.body.z1 && pos.z < self.body.z2 {
      if self.body.contains(pos, err) {
        let pos = xy(pos);
        for i in 0..3 {
          if points2d::dot(pos, RAYS3[i]) < 0.0 {
            if points2d::cross(pos, RAYS3[i]).abs() < self.body_width * 0.5 - err {
              return true;
            }
          }
        }
      }
      return false;
    }

    if pos.z > self.body.z1 - self.connector_h && pos.z < self.body.z2 + self.connector_h {
      let pos = xy(pos);
      for i in 0..3 {
        if (pos + RAYS3[i].scale(self.connector_r)).sqr_len() < sqr(self.connector_spike_r - err) {
          return true;
        }
      }

      return false;
    }

    return false;
  }
}

struct PawlInfo {
  center: points2d::Point,
  arc_center: points2d::Point,
  dir: points2d::Point,
}

struct OutPawlInfo {
  pawl: PawlInfo,
}

struct PawlOutRingParams {
  z1: f32,
  z2: f32,
  r_in: f32,
  r_out: f32,
  pawl_length: f32,
  pawl_count: usize,
  reversed: bool,
  index: PartIndex,
}

impl PawlOutRingParams {
  fn create(self) -> PawlOutRing {
    let mut pawl_infos = Vec::new();
    for i in 0..self.pawl_count {
      let dir_from_c =
        points2d::Point::from_angle((i as f32 + 0.5) / self.pawl_count as f32 * 2.0 * PI);

      let center = dir_from_c.scale(self.r_in - 1.0);
      let end = points2d::find_pp2(
        points2d::Point::ZERO,
        self.r_in + 1.0,
        center,
        self.pawl_length,
        !self.reversed,
      );
      let dir = (end - center).scale(1.0 / self.pawl_length);
      let arc_center = points2d::find_pp2(end, self.pawl_length, center, 2.0, !self.reversed);

      let pawl = PawlInfo { center, arc_center, dir };
      pawl_infos.push(OutPawlInfo { pawl });
    }

    PawlOutRing {
      z1: self.z1,
      z2: self.z2,
      r_in: self.r_in,
      r_out: self.r_out,
      pawl_length: self.pawl_length,
      index: self.index,
      pawl_infos,
    }
  }
}

struct PawlOutRing {
  z1: f32,
  z2: f32,
  r_in: f32,
  r_out: f32,
  pawl_length: f32,
  index: PartIndex,

  pawl_infos: Vec<OutPawlInfo>,
}

impl PawlOutRing {
  fn contains(&self, pos: Point, err: f32) -> bool {
    let z = pos.z;
    if z < self.z1 || z > self.z2 {
      return false;
    }
    let pos = xy(pos);
    let r = pos.len();

    if r > self.r_in + err + 0.1 && r < self.r_out - err {
      if r < self.r_in + err + 1.5 {
        for pawl_info in &self.pawl_infos {
          let p = &pawl_info.pawl;
          if (pos - p.arc_center).sqr_len() < sqr(self.pawl_length + err) {
            if points2d::dot(p.center - pos, p.dir) < -err
              && points2d::cross(p.center - pos, p.dir).abs() < 1.0 + err
            {
              return false;
            }
          }
        }
      }

      return true;
    }

    false
  }
}

struct PawlInRingParams {
  z1: f32,
  z2: f32,
  r_in: f32,
  r_out: f32,
  pawl_length: f32,
  pawl_count: usize,
  pawl_dist1: f32,
  pawl_dist2: f32,
  reversed: bool,
  index: PartIndex,
  pawl_index_delta: PartIndex,
}

impl PawlInRingParams {
  fn create(self) -> (PawlInRing, Vec<Pawl>) {
    let mut pawl_infos = Vec::new();
    let mut result_pawls = Vec::new();
    let mut pawl_index = self.index + self.pawl_index_delta;
    for i in 0..self.pawl_count {
      let center =
        points2d::Point::from_angle((i as f32 + 0.5) / self.pawl_count as f32 * 2.0 * PI)
          .scale(self.r_out - 1.0);
      let end = points2d::find_pp2(
        points2d::Point::ZERO,
        self.r_out - 1.0,
        center,
        self.pawl_length,
        !self.reversed,
      );
      let dir = (end - center).norm();
      let arc_center = points2d::find_pp2(end, self.pawl_length, center, 2.0, !self.reversed);
      pawl_infos.push(PawlInfo { center, arc_center, dir });

      let spring_center =
        points2d::find_pp2(end, self.r_out + 1.5, center, self.r_out - 1.0, self.reversed);
      let cut_center =
        points2d::find_pp2(end, self.r_out + 1.0, center, self.r_out - 1.0, self.reversed);

      result_pawls.push(Pawl {
        info: PawlInfo { center, arc_center, dir },
        pawl_length: self.pawl_length,
        spring_center,
        cut_center,
        r_out: self.r_out,
        z1: self.z1 + self.pawl_dist1,
        z2: self.z2 - self.pawl_dist2,
        reversed: self.reversed,
        index: pawl_index,
      });
      pawl_index += 1;
    }

    (
      PawlInRing {
        z1: self.z1,
        z2: self.z2,
        r_in: self.r_in,
        r_out: self.r_out,
        spring_shift: (self.z2 - self.z1 - self.pawl_dist2 + self.pawl_dist1) * 0.5,
        pawl_length: self.pawl_length,
        reversed: self.reversed,
        index: self.index,
        sat_holes_info: None,
        pawl_infos,
      },
      result_pawls,
    )
  }
}

struct Pawl {
  info: PawlInfo,
  spring_center: points2d::Point,
  cut_center: points2d::Point,
  pawl_length: f32,
  r_out: f32,
  z1: f32,
  z2: f32,
  reversed: bool,
  index: PartIndex,
}

impl Pawl {
  fn contains(&self, pos: Point, err: f32) -> bool {
    let z = pos.z;
    if z < self.z1 || z > self.z2 {
      return false;
    }
    let pos = xy(pos);
    let r = pos.len();

    if r > self.r_out - err {
      return false;
    }

    let ds = (pos - self.spring_center).len() - (self.r_out - 1.5 - err);
    if ds > -0.6 {
      if sqr(z - (self.z1 + self.z2) * 0.5) + sqr(f32::min(0.0, ds)) < sqr(0.6 + err) {
        return false;
      }
    }
    if (pos - self.cut_center).sqr_len() > sqr(self.r_out - err + 1.5) {
      return false;
    }

    let pcd = (pos - self.info.center).sqr_len();
    if pcd < sqr(self.pawl_length + 3.0 - err) {
      if (pos - self.info.arc_center).sqr_len() < sqr(self.pawl_length - err) {
        let d = points2d::dot(self.info.center - pos, self.info.dir);
        let c = points2d::cross(self.info.center - pos, self.info.dir);
        let c = if self.reversed { -c } else { c };
        if c > 0.9 - err {
        } else {
          if pcd < sqr(2.9 - err) {
            return true;
          }
          if c > -2.0 + err && d < 0.0 {
            return true;
          }
        }
      }
    }

    return false;
  }
}

struct PawlInRing {
  z1: f32,
  z2: f32,
  r_in: f32,
  r_out: f32,
  pawl_length: f32,
  spring_shift: f32,
  index: PartIndex,
  reversed: bool,
  sat_holes_info: Option<SatHolesInfo>,
  pawl_infos: Vec<PawlInfo>,
}

impl PawlInRing {
  fn sat_holes_info(mut self, sat_holes_info: SatHolesInfo) -> Self {
    self.sat_holes_info = Some(sat_holes_info);
    self
  }

  fn contains(&self, pos: Point, err: f32) -> bool {
    let z = pos.z;
    if z < self.z1 || z > self.z2 {
      return false;
    }
    let pos = xy(pos);
    let r = pos.len();
    if r > self.r_in + err && r < self.r_out - err {
      let dr = r - (self.r_out - 1.5 - err);
      if dr > -0.6 {
        if sqr(z - (self.z1 + self.spring_shift)) + sqr(f32::min(dr, 0.0)) < sqr(0.6 + err)
          || pos.y.abs() < 0.6 + err
        {
          return false;
        }
      }

      if let Some(holes) = self.sat_holes_info {
        for i in 0..3 {
          if (pos - RAYS3[i].scale(holes.sat_r)).sqr_len() < sqr(holes.axle_r + err) {
            return false;
          }
        }
      }

      for pawl_info in &self.pawl_infos {
        let pcd = (pos - pawl_info.center).sqr_len();
        if pcd < sqr(3.0 + err) {
          return false;
        }
        if pcd < sqr(self.pawl_length + 3.1 + err) {
          if (pos - pawl_info.arc_center).sqr_len() < sqr(self.pawl_length + 0.2 + err) {
            let d = points2d::dot(pawl_info.center - pos, pawl_info.dir);
            let c = points2d::cross(pawl_info.center - pos, pawl_info.dir);
            let c = if self.reversed { -c } else { c };
            if c > 0.0 {
              if d < 3.0 + err {
                return false;
              }
            } else if c > -2.0 - err {
              if d < 0.0 {
                return false;
              }
            }
          }
        }
      }

      return true;
    }

    false
  }
}

struct HorzClutch {
  z1: f32,
  z2: f32,
  r_in: f32,
  r_out: f32,
  clutch1: f32,
  clutch2: f32,
  index: PartIndex,
}

impl HorzClutch {
  fn contains(&self, pos: Point, err: f32) -> bool {
    let z = pos.z;
    if z < self.z1 || z > self.z2 {
      return false;
    }

    let pos = xy(pos);
    let r = pos.len();
    if r > self.r_in + err && r < self.r_out - err {
      let ok1 = z < self.z1 + self.clutch1;
      let ok2 = z > self.z2 - self.clutch2;
      if ok1 || ok2 {
        for i in 0..6 {
          let a0 = if ok1 { i as f32 } else { i as f32 + 0.5 };
          let d1 = points2d::Point::from_angle((a0 + 0.0) / 6.0 * 2.0 * PI);
          let d2 = points2d::Point::from_angle((a0 + 0.5) / 6.0 * 2.0 * PI);
          if points2d::dot(pos, d1) > -err && points2d::dot(pos, d2) < err {
            return false;
          }
        }
      }
      return true;
    }

    return false;
  }
}

struct Crank {
  z1: f32,
  z2: f32,
  r_in: f32,
  r_out: f32,
  l: f32,
  index: PartIndex,
}

impl Crank {
  fn contains(&self, pos: Point, err: f32) -> bool {
    let z = pos.z;
    if z < self.z1 || z > self.z2 {
      return false;
    }

    let pos = xy(pos);
    if pos.len() < self.r_in + err {
      return false;
    }

    let zp = points2d::Point::ZERO;
    let fp = points2d::Point { x: self.l, y: 0.0 };

    if points2d::dist_pl(pos, zp, fp) < self.r_out - err {
      return true;
    }

    return false;
  }
}

struct Axle {
  z1: f32,
  z2: f32,
  r_in: f32,
  r1: f32,
  r2: f32,
  z_step_1: f32,
  z_step_2: f32,
  dropout: f32,

  zw11: f32,
  zw12: f32,
  zw21: f32,
  zw22: f32,

  index: PartIndex,
}

impl Axle {
  fn contains(&self, pos: Point, err: f32) -> bool {
    let z = pos.z;
    if z < self.z1 || z > self.z2 {
      return false;
    }

    let pos = xy(pos);
    let r = pos.len();
    if r < self.r_in + err {
      return false;
    }

    if z > self.z_step_1 && z < self.z_step_2 {
      if r > self.r2 - err {
        return false;
      }
    } else {
      if r > self.r1 - err {
        return false;
      }
    }

    if z > self.zw11 && z < self.zw12 || z > self.zw21 && z < self.zw22 {
      if pos.x.abs() < 2.1 + err {
        return false;
      }
    }

    if z.abs() > self.dropout - 5.0 {
      if pos.x.abs() > 4.0 - err {
        return false;
      }

      let a = (f32::atan2(pos.y, pos.x) / (2.0 * PI)).rem_euclid(1.0);
      let az = (z / 1.5).rem_euclid(1.0);
      let da = (a - az).rem_euclid(1.0);
      let da = (da - 0.5).abs() - 0.5;
      if r > f32::max(4.1, 3.8 + 2.0 * da.abs()) {
        return false;
      }
    }

    return true;
  }
}

pub struct SwampTodCreator {
  err: f32,
  g_err: f32,
  axle_r: f32,
  gears: Vec<Gear>,
  cylinders: Vec<Cylinder>,
  springs: Vec<Spring>,
  cams: Vec<Cam>,
  pawl_in_rings: Vec<PawlInRing>,
  pawl_out_rings: Vec<PawlOutRing>,
  pawls: Vec<Pawl>,
  horz_clutches: Vec<HorzClutch>,
  carrier_unit_connectors: Vec<CarrierUnitConnector>,
  crank: Option<Crank>,
  axle: Option<Axle>,
}

struct SingleBlockParams {
  ring: i32,
  sat: i32,
  sun: i32,
  z: f32,
  w: f32,
  sun_extra_w: f32,
  sat_ir: f32,
  sat_axle_r: f32,
  inner_r: f32,
  index_start: PartIndex,
}

struct DoubleBlockParams {
  ring: i32,
  sat1: i32,
  sat2: i32,
  sun1: i32,
  sun2: i32,
  z: f32,
  w1: f32,
  w2: f32,
  sat_ir: f32,
  sat_axle_r: f32,
  inner_r: f32,
  index_start: PartIndex,
}

struct SingleBlockResult {
  ring: PartIndex,
  sun: PartIndex,
  carrier: PartIndex,
  carrier_cup: PartIndex,
  sat_holes_info: SatHolesInfo,
  ring_size: f32,
  out_size: f32,
  carrier_size: f32,
  sun_clutch_profile: GearProfile,
}

struct DoubleBlockResult {
  ring: PartIndex,
  sun1: PartIndex,
  sun2: PartIndex,
  carrier: PartIndex,
  carrier_cup: PartIndex,
  sat_holes_info: SatHolesInfo,
  ring_size: f32,
  out_size: f32,
  carrier_size: f32,
}

impl SwampTodCreator {
  fn make_single_block(&mut self, block: SingleBlockParams) -> SingleBlockResult {
    let iz = block.z + 2.2;
    let iw = block.w;

    let sat_corr;
    let sun_corr;
    sat_corr = if block.sat < 18 { 0.8 } else { 1.0 };
    sun_corr = 1.2;

    let carrier_r = (block.sat + block.sun) as f32 * 0.5 + sat_corr + sun_corr;
    let corrr = sat_corr * 2.0
      + sun_corr
      + ((block.sat + block.sun) as f32 - (block.ring - block.sat) as f32) * 0.5;

    println!("{}+{}", block.ring, corrr);
    println!("{}+{}", block.sat, sat_corr);
    println!("{}+{}", block.sun, sun_corr);

    for i in 0..3 {
      let sc = RAYS3[i as usize].scale(carrier_r);

      let sat_profile = GearProfileParams {
        teeth_count: block.sat,
        phase: -(block.sat + block.sun) as f32 * i as f32 / 3.0,
        module: 1.0,
        correction: sat_corr,
        inner: false,
      }
      .create();
      let sat_gear =
        GearParams { profile: sat_profile, z1: iz, z2: iz + iw, index: block.index_start + i }
          .create()
          .shift(sc)
          .r_in(block.sat_ir);
      self.gears.push(sat_gear);

      let axle = CylinderParams {
        z1: block.z + 0.0,
        z2: block.z + block.w + 4.0 + 0.4,
        r_in: 0.0,
        r_out: block.sat_axle_r,
        index: block.index_start + 301 + i as PartIndex,
      }
      .create()
      .shift(sc);

      let bear = CylinderParams {
        z1: block.z + 2.2,
        z2: block.z + block.w + 2.0 + 0.2,
        r_in: block.sat_axle_r + 0.1,
        r_out: block.sat_ir,
        index: block.index_start + 311 + i as PartIndex,
      }
      .create()
      .shift(sc);

      self.cylinders.push(axle);
      self.cylinders.push(bear);
    }

    let sun_phase = (block.sat + 1) as f32 * 0.5;
    let sun_profile = GearProfileParams {
      teeth_count: block.sun,
      phase: sun_phase,
      module: 1.0,
      correction: sun_corr,
      inner: false,
    }
    .create();

    let sun_clutch_profile = GearProfileParams {
      teeth_count: block.sun,
      phase: sun_phase,
      module: 1.0,
      correction: sun_corr,
      inner: true,
    }
    .create();

    let sun_gear = GearParams {
      profile: sun_profile,
      z1: iz,
      z2: iz + iw + block.sun_extra_w,
      index: block.index_start + 3,
    }
    .create()
    .r_in(self.axle_r + 3.1);

    let ring_profile = GearProfileParams {
      teeth_count: block.ring,
      phase: 0.0,
      module: 1.0,
      correction: corrr,
      inner: true,
    }
    .create();

    let ring_r_max = ring_profile.r_max;

    let ring_gear =
      GearParams { profile: ring_profile, z1: iz - 4.0, z2: iz + iw, index: block.index_start + 5 }
        .create()
        .r_out(ring_r_max + 3.0);

    let sat_holes_info = SatHolesInfo { sat_r: carrier_r, axle_r: block.sat_axle_r };

    let (cu1, cu2, conn) = CarrierUnitParams {
      wall1: CylinderParams {
        z1: block.z + 0.0,
        z2: block.z + 2.0,
        r_in: block.inner_r,
        r_out: carrier_r + 6.0,
        index: block.index_start + 200,
      }
      .create(),
      wall2: CylinderParams {
        z1: block.z + block.w + 2.4,
        z2: block.z + block.w + 4.4,
        r_in: carrier_r - 6.0,
        r_out: carrier_r + 6.0,
        index: block.index_start + 201,
      }
      .create(),

      body_r_in: carrier_r - 6.0,
      body_r_out: carrier_r + 6.0,
      body_width: 16.0,
      sat_holes_info,
    }
    .create();

    let result = SingleBlockResult {
      ring: ring_gear.index,
      sun: sun_gear.index,
      sat_holes_info,
      carrier: cu1.index,
      carrier_cup: cu2.index,
      ring_size: ring_gear.r_out,
      out_size: ring_gear.r_out,
      carrier_size: carrier_r + 5.0,
      sun_clutch_profile,
    };

    self.gears.push(sun_gear);
    self.gears.push(ring_gear);
    self.cylinders.push(cu1);
    self.cylinders.push(cu2);
    self.carrier_unit_connectors.push(conn);
    return result;
  }

  fn make_double_block(&mut self, block: DoubleBlockParams) -> DoubleBlockResult {
    let iw1;
    let ow1;
    let iw2;
    let ow2;
    let iz1;
    let iz2;
    let oz1;
    let oz2;

    if block.sat1 > block.sat2 {
      iw1 = block.w1;
      iw2 = block.w2 + 1.0;
      ow1 = block.w1 + 1.0;
      ow2 = block.w2;
      iz1 = block.z + 2.2;
      iz2 = block.z + 2.2 + block.w1;
      oz1 = block.z + 2.2;
      oz2 = block.z + 2.2 + block.w1 + 1.0;
    } else {
      iw1 = block.w1 + 1.0;
      iw2 = block.w2;
      ow1 = block.w1;
      ow2 = block.w2 + 1.0;
      iz1 = block.z + 2.2;
      iz2 = block.z + 2.2 + block.w1 + 1.0;
      oz1 = block.z + 2.2;
      oz2 = block.z + 2.2 + block.w1;
    }

    let sat_corr1;
    let sun_corr1;
    let sat_corr2;
    let sun_corr2;

    let dcorr = (block.sun1 + block.sat1) - (block.sat2 + block.sun2);

    assert!(
      dcorr.abs() <= 1,
      "Cant determine correction to require all conditions for dcorr={dcorr}!"
    );

    if block.sat1 < 15 {
      if dcorr < 0 {
        sat_corr1 = 0.5;
        sun_corr1 = 1.7;
      } else {
        sat_corr1 = 0.6;
        sun_corr1 = 1.1;
      }
    } else if dcorr >= 0 {
      sat_corr1 = 0.8;
      sun_corr1 = 0.9;
    } else {
      sat_corr1 = 1.1;
      sun_corr1 = 1.1;
    }

    if block.sat2 < 15 {
      if dcorr > 0 {
        sat_corr2 = 0.5;
        sun_corr2 = 1.7;
      } else {
        sat_corr2 = 0.6;
        sun_corr2 = 1.1;
      }
    } else if dcorr <= 0 {
      sat_corr2 = 0.8;
      sun_corr2 = 0.9;
    } else {
      sat_corr2 = 1.1;
      sun_corr2 = 1.1;
    }

    let carrier_r = (block.sat1 + block.sun1) as f32 * 0.5 + sat_corr1 + sun_corr1;

    let corrr = sat_corr1 * 2.0
      + sun_corr1
      + ((block.sat1 + block.sun1) as f32 - (block.ring - block.sat1) as f32) * 0.5;

    println!("{}+{}", block.ring, corrr);
    println!("{}+{}\t{}+{}", block.sat1, sat_corr1, block.sat2, sat_corr2);
    println!("{}+{}\t{}+{}", block.sun1, sun_corr1, block.sun2, sun_corr2);

    for i in 0..3 {
      let sc = RAYS3[i].scale(carrier_r);

      let sat1_profile = GearProfileParams {
        teeth_count: block.sat1,
        phase: -(block.sat1 + block.sun1) as f32 * i as f32 / 3.0,
        module: 1.0,
        correction: sat_corr1,
        inner: false,
      }
      .create();
      let sat1_gear = GearParams {
        profile: sat1_profile,
        z1: iz1,
        z2: iz1 + iw1,
        index: block.index_start + i as PartIndex,
      }
      .create()
      .labeled_bottom(block.sat1 > block.sat2, i)
      .shift(sc)
      .r_in(block.sat_ir);

      let sat2_profile = GearProfileParams {
        teeth_count: block.sat2,
        phase: -(block.sat2 + block.sun2) as f32 * i as f32 / 3.0,
        module: 1.0,
        correction: sat_corr2,
        inner: false,
      }
      .create();
      let sat2_gear = GearParams {
        profile: sat2_profile,
        z1: iz2,
        z2: iz2 + iw2,
        index: block.index_start + i as PartIndex,
      }
      .create()
      .labeled_top(block.sat1 < block.sat2, i)
      .shift(sc)
      .r_in(block.sat_ir);

      self.gears.push(sat1_gear);
      self.gears.push(sat2_gear);

      let axle = CylinderParams {
        z1: block.z + 0.0,
        z2: block.z + block.w1 + block.w2 + 1.0 + 4.0 + 0.4,
        r_in: 0.0,
        r_out: block.sat_axle_r,
        index: block.index_start + 301 + i as PartIndex,
      }
      .create()
      .shift(sc);

      let bear = CylinderParams {
        z1: block.z + 2.2,
        z2: block.z + block.w1 + block.w2 + 1.0 + 2.0 + 0.2,
        r_in: block.sat_axle_r + 0.1,
        r_out: block.sat_ir,
        index: block.index_start + 311 + i as PartIndex,
      }
      .create()
      .shift(sc);

      self.cylinders.push(axle);
      self.cylinders.push(bear);
    }

    let sun1_phase = (block.sat1 + 1) as f32 * 0.5;
    let sun1_profile = GearProfileParams {
      teeth_count: block.sun1,
      phase: sun1_phase,
      module: 1.0,
      correction: sun_corr1,
      inner: false,
    }
    .create();

    fn sun_size_to_clutch_size(sun: i32) -> f32 {
      if sun < 24 {
        2.0
      } else {
        10.0
      }
    }

    let sun1_gear = GearParams {
      profile: sun1_profile,
      z1: oz1 - 0.0,
      z2: oz1 + ow1,
      index: block.index_start + 3,
    }
    .create()
    .r_in(self.axle_r + 0.1)
    .axle_clutch(GearAxleClutchParams {
      groove1: 0.0,
      clutch1: 0.0,
      groove2: ow1 - if block.sun1 > block.sun2 { 3.5 } else { 2.5 },
      clutch2: 10.0,
      clutch_count: if block.sun1 < 24 { 2 } else { 4 },
    });

    let sun2_phase = (block.sat2 + 1) as f32 * 0.5;
    let sun2_profile = GearProfileParams {
      teeth_count: block.sun2,
      phase: sun2_phase,
      module: 1.0,
      correction: sun_corr2,
      inner: false,
    }
    .create();
    let sun2_gear = GearParams {
      profile: sun2_profile,
      z1: oz2,
      z2: oz2 + ow2 + 2.0,
      index: block.index_start + 4,
    }
    .create()
    .r_in(self.axle_r + 0.1)
    .axle_clutch(GearAxleClutchParams {
      groove1: ow2 + 2.0 - if block.sun1 > block.sun2 { 2.5 } else { 3.5 },
      clutch1: 10.0,
      groove2: 0.0,
      clutch2: 0.0,
      clutch_count: if block.sun2 < 24 { 2 } else { 4 },
    });

    let ring_profile = GearProfileParams {
      teeth_count: block.ring,
      phase: 0.0,
      module: 1.0,
      correction: corrr,
      inner: true,
    }
    .create();

    let ring_r_max = ring_profile.r_max;

    let ring_gear = GearParams {
      profile: ring_profile,
      z1: oz1 - 3.0,
      z2: oz1 + ow1,
      index: block.index_start + 5,
    }
    .create()
    .r_out(ring_r_max + 3.0);

    let sat_holes_info = SatHolesInfo { sat_r: carrier_r, axle_r: block.sat_axle_r };

    let (cu1, cu2, conn) = CarrierUnitParams {
      wall1: CylinderParams {
        z1: block.z + 0.0,
        z2: block.z + 2.0,
        r_in: block.inner_r,
        r_out: carrier_r + 5.0,
        index: block.index_start + 200,
      }
      .create(),
      wall2: CylinderParams {
        z1: block.z + block.w1 + block.w2 + 3.4,
        z2: block.z + block.w1 + block.w2 + 5.4,
        r_in: carrier_r - 5.0,
        r_out: carrier_r + 5.0,
        index: block.index_start + 201,
      }
      .create(),

      body_r_in: carrier_r - 5.0,
      body_r_out: carrier_r + 5.0,
      body_width: 12.0,
      sat_holes_info,
    }
    .create();

    let result = DoubleBlockResult {
      ring: ring_gear.index,
      sun1: sun1_gear.index,
      sun2: sun2_gear.index,
      sat_holes_info,
      carrier: cu1.index,
      carrier_cup: cu2.index,
      ring_size: ring_gear.r_out,
      out_size: f32::max(
        ring_gear.r_out,
        carrier_r + block.sat2 as f32 * 0.5 + sat_corr2 + 1.0 + 2.0,
      ),
      carrier_size: carrier_r + 5.0,
    };

    self.gears.push(sun1_gear);
    self.gears.push(sun2_gear);
    self.gears.push(ring_gear);

    self.cylinders.push(cu1);
    self.cylinders.push(cu2);
    self.carrier_unit_connectors.push(conn);

    return result;
  }

  pub fn make_driver(&mut self, z1: f32, z2: f32, index: PartIndex) {
    let driver = CylinderParams { z1, z2, r_in: 13.0, r_out: 17.5, index }
      .create()
      .holes_neg(SatHolesInfo { sat_r: 19.0, axle_r: 3.0 })
      .spring_info(SpringInfo { shift: 11.0, spring_r: 1.0, axle_r: 17.0 });
    self.cylinders.push(driver);
  }

  pub fn create_left_wall(&mut self, dropout: f32, z: f32, index: PartIndex) -> f32 {
    let reaction_pawl_ring = PawlOutRingParams {
      z1: z - 13.4,
      z2: z - 2.4,
      r_in: 24.0,
      r_out: 30.0,
      pawl_length: 6.0,
      pawl_count: 18,
      reversed: true,
      index,
    }
    .create();
    self.pawl_out_rings.push(reaction_pawl_ring);

    let left_bear_border =
      CylinderParams { z1: z - 13.4, z2: z - 12.4, r_in: 32.0, r_out: 33.5, index }.create();
    self.cylinders.push(left_bear_border);

    let left_bear =
      CylinderParams { z1: z - 12.4, z2: z - 2.4, r_in: 32.5, r_out: 42.5, index: 400300 }.create();
    self.cylinders.push(left_bear);

    let left_wall =
      CylinderParams { z1: z - 13.4, z2: z - 11.4, r_in: 5.0, r_out: 32.5, index }.create();
    self.cylinders.push(left_wall);

    let result = z - 11.4;

    let left_wall_plate =
      CylinderParams { z1: -dropout + 5.0, z2: z - 13.4, r_in: 5.0, r_out: 9.0, index }.create();
    self.cylinders.push(left_wall_plate);

    let left_wall_clutch_fill =
      CylinderParams { z1: z - 16.4, z2: z - 13.4, r_in: 8.0, r_out: 15.0, index }.create();
    self.cylinders.push(left_wall_clutch_fill);

    let left_wall_clutch = HorzClutch {
      z1: z - 16.4,
      z2: z - 13.4,
      r_in: 14.0,
      r_out: 17.0,
      index,
      clutch1: 3.0,
      clutch2: 0.0,
    };
    self.horz_clutches.push(left_wall_clutch);

    let reaction_clutch = HorzClutch {
      z1: z - 16.4,
      z2: z - 13.4,
      r_in: 14.0,
      r_out: 18.0,
      index: index + 1,
      clutch1: 0.0,
      clutch2: 3.0,
    };
    self.horz_clutches.push(reaction_clutch);

    let crank_part =
      CylinderParams { z1: z - 16.4, z2: z - 13.4, r_in: 17.0, r_out: 27.0, index: index + 1 }
        .create();
    self.cylinders.push(crank_part);

    self.crank = Some(Crank {
      z1: z - 16.4,
      z2: z - 13.4,
      r_in: 17.0,
      r_out: 17.0,
      l: 100.0,
      index: index + 1,
    });

    result
  }

  pub fn new() -> Self {
    let mut result = Self {
      err: 0.0,
      g_err: 0.05,
      axle_r: 6.25,

      gears: Vec::new(),
      cylinders: Vec::new(),
      springs: Vec::new(),
      cams: Vec::new(),
      pawl_in_rings: Vec::new(),
      pawl_out_rings: Vec::new(),
      pawls: Vec::new(),
      horz_clutches: Vec::new(),
      carrier_unit_connectors: Vec::new(),
      crank: None,
      axle: None,
    };

    let dropout = 60.0;

    let chainline = 40.0;
    let driver_z1 = chainline + 2.5 - 6.0;
    result.make_driver(driver_z1, dropout - 5.0, 100);

    let first_block_w = 10.0;
    let first_block_z = driver_z1 - first_block_w - 11.4;

    let first_block = result.make_double_block(DoubleBlockParams {
      index_start: 100100,
      ring: 63,
      sat1: 20,
      sat2: 12,
      sun1: 21,
      sun2: 30,
      z: first_block_z,
      w1: 4.0,
      w2: 5.0,
      sat_ir: 3.5, // K4x7x10TN; 1900N
      sat_axle_r: 2.0,
      inner_r: result.axle_r + 0.5,
    });

    let input_pawl_ring = PawlOutRingParams {
      z1: 4.6 + first_block_z + first_block_w,
      z2: 12.4 + first_block_z + first_block_w,
      r_in: 25.0,
      r_out: 30.0,
      pawl_length: 6.0,
      pawl_count: 18,
      reversed: true,
      index: 100,
    }
    .create();

    println!("input pawln contact at {}", 26.0);
    println!("ring radius {}", first_block.ring_size);
    println!("carrier radius {}", first_block.carrier_size);

    let (input_in_pawl_ring, input_pawls) = PawlInRingParams {
      z1: 4.1 + first_block_z + first_block_w,
      z2: 10.2 + first_block_z + first_block_w,
      r_in: first_block.carrier_size - 10.0,
      r_out: 25.0,
      pawl_length: 6.0,
      pawl_dist1: 0.0,
      pawl_dist2: 0.0,
      pawl_count: 2,
      reversed: true,
      index: first_block.carrier_cup,
      pawl_index_delta: 40,
    }
    .create();
    let input_in_pawl_ring = input_in_pawl_ring.sat_holes_info(first_block.sat_holes_info);
    result.pawl_in_rings.push(input_in_pawl_ring);
    result.pawls.extend(input_pawls);
    result.pawl_out_rings.push(input_pawl_ring);

    let intermediate_ring = CylinderParams {
      z1: first_block_z,
      z2: first_block_z + 2.0,
      r_in: first_block.carrier_size + 1.0,
      r_out: first_block.carrier_size + 5.0,
      index: 100106,
    }
    .create();
    result.cylinders.push(intermediate_ring);

    let small_bearing_border = CylinderParams {
      z1: 11.4 + first_block_z + first_block_w,
      z2: dropout - 21.0,
      r_in: 12.0,
      r_out: 14.0,
      index: 100,
    }
    .create();
    result.cylinders.push(small_bearing_border);
    let small_bearing_1 = CylinderParams {
      z1: dropout - 21.0,
      z2: dropout - 13.0,
      r_in: 5.0,
      r_out: 12.9,
      index: 400100,
    }
    .create();
    result.cylinders.push(small_bearing_1);
    let small_bearing_2 = CylinderParams {
      z1: dropout - 13.0,
      z2: dropout - 5.0,
      r_in: 5.0,
      r_out: 12.9,
      index: 400101,
    }
    .create();
    result.cylinders.push(small_bearing_2);
    let small_spring_border = CylinderParams {
      z1: 11.4 + first_block_z + first_block_w,
      z2: dropout - 21.2,
      r_in: result.axle_r + 0.2,
      r_out: 14.0,
      index: 100,
    }
    .create();
    result.cylinders.push(small_spring_border);
    let right_bearing_in = CylinderParams {
      z1: 2.4 + first_block_z + first_block_w,
      z2: 12.4 + first_block_z + first_block_w,
      r_in: 30.0,
      r_out: 32.5,
      index: 100,
    }
    .create();
    result.cylinders.push(right_bearing_in);
    let right_bearing = CylinderParams {
      z1: 2.4 + first_block_z + first_block_w,
      z2: 12.4 + first_block_z + first_block_w,
      r_in: 32.5,
      r_out: 42.5,
      index: 400200,
    }
    .create();
    result.cylinders.push(right_bearing);
    let right_bearing_border = CylinderParams {
      z1: 12.4 + first_block_z + first_block_w,
      z2: 13.4 + first_block_z + first_block_w,
      r_in: 30.0,
      r_out: 33.5,
      index: 100,
    }
    .create();
    result.cylinders.push(right_bearing_border);
    let right_bearing_wall = CylinderParams {
      z1: 11.4 + first_block_z + first_block_w,
      z2: 13.4 + first_block_z + first_block_w,
      r_in: result.axle_r + 0.5,
      r_out: 32.5,
      index: 100,
    }
    .create();
    result.cylinders.push(right_bearing_wall);

    let right_spring = Spring {
      z1: 4.2 + first_block_z + first_block_w,
      z2: 11.4 + first_block_z + first_block_w,
      r_in: result.axle_r + 0.5,
      r_out: result.axle_r + 2.5,
      index: 100107,
    };
    result.springs.push(right_spring);

    let right_cam = Cam {
      z1: first_block_z + 4.0,
      z2: first_block_z + 8.0,
      r_in: 1.0,
      r_out: result.axle_r + 1.9,
      index: 100200,
    };
    result.cams.push(right_cam);

    let second_block_w = 10.0;
    let second_block_z = first_block_z - 9.8 - second_block_w;
    let second_block = result.make_double_block(DoubleBlockParams {
      index_start: 200100,
      ring: 54,
      sat1: 12,
      sat2: 20,
      sun1: 30,
      sun2: 21,
      z: second_block_z,
      w1: 5.0,
      w2: 4.0,
      sat_ir: 4.0, // K5x8x10; 2500N
      sat_axle_r: 2.5,
      inner_r: result.axle_r + 2.5,
    });

    let middle_spring = Spring {
      z1: 4.2 + second_block_z + second_block_w,
      z2: first_block_z,
      r_in: result.axle_r + 0.5,
      r_out: result.axle_r + 2.5,
      index: 200107,
    };
    result.springs.push(middle_spring);

    let middle_cam = Cam {
      z1: second_block_z + 4.0,
      z2: second_block_z + 8.0,
      r_in: 1.0,
      r_out: result.axle_r + 1.9,
      index: 200200,
    };
    result.cams.push(middle_cam);

    let ring_pawl_ring = PawlOutRingParams {
      z1: first_block_z - 5.2,
      z2: first_block_z - 0.2,
      r_in: first_block.carrier_size,
      r_out: second_block.out_size,
      pawl_length: 6.0,
      pawl_count: 26,
      reversed: false,
      index: first_block.ring,
    }
    .create();
    result.pawl_out_rings.push(ring_pawl_ring);

    println!("step1 pawln contact at {}", first_block.carrier_size);

    let (carrier_pawl_ring, carrier_pawls) = PawlInRingParams {
      z1: first_block_z - 5.2,
      z2: first_block_z + 0.3,
      r_in: first_block.carrier_size - 6.0,
      r_out: first_block.carrier_size,
      pawl_length: 6.0,
      pawl_dist1: 0.1,
      pawl_dist2: 0.6,
      pawl_count: 2,
      reversed: false,
      index: first_block.carrier,
      pawl_index_delta: 50,
    }
    .create();
    result.pawls.extend(carrier_pawls);

    let carrier_pawl_ring = carrier_pawl_ring.sat_holes_info(first_block.sat_holes_info);
    result.pawl_in_rings.push(carrier_pawl_ring);

    let ring_clutch = HorzClutch {
      z1: first_block_z - 7.2,
      z2: first_block_z - 3.2,
      r_in: second_block.out_size - 1.5,
      r_out: second_block.out_size,
      clutch1: 0.0,
      clutch2: 0.0,
      index: first_block.ring,
    };
    result.horz_clutches.push(ring_clutch);

    let ring2_clutch = HorzClutch {
      z1: second_block_z + 4.2,
      z2: first_block_z - 4.2,
      r_in: second_block.out_size - 1.5,
      r_out: second_block.out_size,
      clutch1: 3.0,
      clutch2: 0.0,
      index: first_block.ring,
    };
    let ring2_clutch_w = HorzClutch {
      z1: second_block_z + 4.2,
      z2: second_block_z + 7.2,
      r_in: second_block.out_size - 2.0,
      r_out: second_block.out_size,
      clutch1: 0.0,
      clutch2: 3.0,
      index: second_block.ring,
    };
    let ring2_fill = CylinderParams {
      z1: second_block_z + 4.2,
      z2: second_block_z + 7.2,
      r_in: second_block.ring_size - 0.5,
      r_out: second_block.out_size - 1.5,
      index: second_block.ring,
    }
    .create();
    let ring2_wall = CylinderParams {
      z1: second_block_z - 2.2,
      z2: second_block_z - 0.2,
      r_in: 20.0,
      r_out: second_block.ring_size,
      index: second_block.ring,
    }
    .create();
    result.horz_clutches.push(ring2_clutch);
    result.horz_clutches.push(ring2_clutch_w);
    result.cylinders.push(ring2_wall);
    result.cylinders.push(ring2_fill);

    let third_block_z = second_block_z - 24.4 - 2.4;
    let third_block = result.make_single_block(SingleBlockParams {
      index_start: 300100,
      ring: 62,
      sat: 16,
      sun: 28,
      z: third_block_z,
      w: 13.0,
      sun_extra_w: 11.4,
      sat_ir: 5.5, // K8x11x13; 5000N
      sat_axle_r: 4.0,
      inner_r: result.axle_r + 2.5,
    });

    println!("ring3size={}", third_block.out_size);
    println!("third_block_z={third_block_z}");

    let sun_clutch = GearParams {
      profile: third_block.sun_clutch_profile,
      z1: second_block_z - 2.2,
      z2: second_block_z - 0.2,
      index: second_block.ring,
    }
    .create()
    .r_out(21.0);

    result.gears.push(sun_clutch);

    let (output_pawl_ring, output_pawls) = PawlInRingParams {
      z1: third_block_z + 15.4,
      z2: third_block_z + 24.4,
      r_in: third_block.carrier_size,
      r_out: 36.0,
      pawl_length: 6.0,
      pawl_dist1: 0.0,
      pawl_dist2: 0.0,
      pawl_count: 2,
      reversed: false,
      index: third_block.carrier_cup,
      pawl_index_delta: 50,
    }
    .create();
    result.pawls.extend(output_pawls);
    result.pawl_in_rings.push(output_pawl_ring);

    println!("output pawl contact at {}", 39.5);

    let second_block_clutch_to_third_block = HorzClutch {
      z1: second_block_z,
      z2: second_block_z + 2.0,
      r_in: result.axle_r + 0.5,
      r_out: result.axle_r + 5.5,
      clutch1: 2.0,
      clutch2: 0.0,
      index: second_block.carrier,
    };

    let third_block_clutch_to_second_block = HorzClutch {
      z1: third_block_z,
      z2: third_block_z + 2.0,
      r_in: result.axle_r + 0.5,
      r_out: result.axle_r + 5.5,
      clutch1: 0.0,
      clutch2: 2.0,
      index: third_block.carrier,
    };

    let clutch_connector = HorzClutch {
      z1: third_block_z,
      z2: second_block_z + 2.0,
      r_in: result.axle_r + 0.5,
      r_out: result.axle_r + 2.5,
      clutch1: 2.0,
      clutch2: 2.0,
      index: third_block.carrier + 10,
    };

    result.horz_clutches.push(second_block_clutch_to_third_block);
    result.horz_clutches.push(third_block_clutch_to_second_block);
    result.horz_clutches.push(clutch_connector);

    let third_ring_wall = CylinderParams {
      z1: third_block_z - 2.2,
      z2: third_block_z - 0.2,
      r_in: 20.0,
      r_out: third_block.ring_size,
      index: third_block.ring,
    }
    .create();
    result.cylinders.push(third_ring_wall);

    let (reaction_in_pawl_ring, reaction_pawls) = PawlInRingParams {
      z1: third_block_z - 9.4,
      z2: third_block_z - 1.9,
      r_in: 18.0,
      r_out: 24.0,
      pawl_length: 6.0,
      pawl_dist1: 0.1,
      pawl_dist2: 0.6,
      pawl_count: 2,
      reversed: true,
      index: third_block.ring,
      pawl_index_delta: 50,
    }
    .create();
    result.pawl_in_rings.push(reaction_in_pawl_ring);
    result.pawls.extend(reaction_pawls);

    println!("reaction pawl contact at {}", 24.0);

    let wall_step = result.create_left_wall(dropout, third_block_z, third_block.ring + 20);

    result.axle = Some(Axle {
      z1: -70.0,
      z2: 70.0,
      r_in: 2.1,
      r1: 5.0,
      r2: result.axle_r,
      dropout: 60.0,
      z_step_1: wall_step,
      z_step_2: dropout - 21.0,
      zw11: first_block_z + 2.0,
      zw12: first_block_z + 14.4,
      zw21: second_block_z + 2.0,
      zw22: second_block_z + 14.4,
      index: 500100,
    });

    result
  }

  pub fn faces(&self) -> usize {
    0
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex { 
    if pos.x.abs() > 79.0 || pos.y.abs() > 79.0 || pos.z.abs() > 79.0 {
      return 0;
    }

    if pos.y < 0.0 {
   //   return 0;
    }

    let result = self.get_component_index(pos);

   // if result != 100300 && result != 100301 { return 0; }

    if result / 100000 == 4 {
      return 0;
    }

    if result > 500000 {
      if pos.y < -0.11 {
        return result;
      }

      if pos.y > 0.11 {
        return result + 1;
      }

      return 0;
    }

    return result;
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    0.6
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    1
  }

  pub fn get_name(&self, current_normal: usize) -> Option<String> {
    None
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn get_quality() -> usize {
    250
  }

  pub fn get_size() -> f32 {
    160.0
  }

  pub fn get_component_index(&self, pos: Point) -> PartIndex {
    for g in &self.gears {
      if g.contains(pos, self.err, self.g_err) {
        return g.index;
      }
    }

    for c in &self.cylinders {
      if c.contains(pos, self.err) {
        return c.index;
      }
    }

    for s in &self.springs {
      if s.contains(pos, self.err) {
        return s.index;
      }
    }

    for c in &self.cams {
      if c.contains(pos, self.err) {
        return c.index;
      }
    }

    for p in &self.pawl_in_rings {
      if p.contains(pos, self.err) {
        return p.index;
      }
    }

    for p in &self.pawl_out_rings {
      if p.contains(pos, self.err) {
        return p.index;
      }
    }

    for p in &self.pawls {
      if p.contains(pos, self.err) {
        return p.index;
      }
    }

    for h in &self.horz_clutches {
      if h.contains(pos, self.err) {
        return h.index;
      }
    }

    for c in &self.carrier_unit_connectors {
      if c.contains(pos, self.err) {
        return c.body.index;
      }
    }

    if let Some(c) = &self.crank {
      if c.contains(pos, self.err) {
        return c.index;
      }
    }

    if let Some(a) = &self.axle {
      if a.contains(pos, self.err) {
        return a.index;
      }
    }

    return 0;
  }
}
