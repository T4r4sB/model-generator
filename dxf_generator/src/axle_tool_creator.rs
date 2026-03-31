use common::points2d::*;
use common::solid::*;

use common::slots_and_holes::*;

use num::*;

#[derive(Debug)]
struct Gear {
  tc: f32,
  phase: f32,
  module: f32,
  err: f32,
  correction: f32,
  inner: bool,
  pos: Point,

  ir: f32,
  r_min: f32,
  r_max: f32,
  tan0: f32,
  a1: f32,
}

enum GearInstrument {
  Rail,
  Round { z: f32, d: f32, r_out: f32, tga_gi: f32 },
}

impl GearInstrument {
  fn cut_in_gear(&self, basic_profile: BasicGearProfile) -> CuttenInGear {
    unimplemented!();
  }

  fn cut_out_gear(&self, basic_profile: BasicGearProfile) -> CuttenOutGear {
    unimplemented!();
  }
}

struct BasicGearProfile {
  z: f32,
  d: f32,
}

struct CuttenOutGear {
  profile: BasicGearProfile,
  evo_in: f32,
  r_in: f32,
}

struct CuttenInGear {
  profile: BasicGearProfile,
  evo_out: f32,
  r_out: f32,
}

struct CompleteOutGear {
  profile: BasicGearProfile,
  evo_in: f32,
  r_in: f32,
  r_out: f32,
}

struct CompleteInGear {
  profile: BasicGearProfile,
  evo_out: f32,
  r_in: f32,
  r_out: f32,
}

impl Gear {
  fn new(tc: usize, phase: f32, module: f32, correction: f32, err: f32, inner: bool) -> Self {
    let tc = tc as f32;

    let ga = 2.0 * PI / tc;
    let (sin20, cos20) = Self::a0().sin_cos();
    let ir = cos20 * tc * module * 0.5;

    let br = (tc * 0.5) * module;
    let mr = br + correction * module;

    let a0 = (ir / br).acos();
    let tan0 = br * a0.sin();
    let a1 = a0; // - ga * 0.25;

    let r_min;
    let r_max;
    if inner {
      r_min = mr - module + err;
      r_max = mr + module * 1.25 + err;
    } else {
      r_min = mr - module * 1.25 - err;
      r_max = mr + module - err;
    }

    let tan0 = tan0 + PI * 0.25 * cos20 + correction * sin20;
    let pos = Point::ZERO;

    Self { tc, phase, module, err, correction, inner, pos, ir, r_min, r_max, tan0, a1 }
  }

  fn a0() -> f32 {
    20.0.to_radians()
  }

  fn couple(g0: &mut Self, g1: &mut Self) -> f32 {
    let teta0 = Self::a2t(Self::a0());
    if !g0.inner && !g1.inner {
      let corr_sum = g0.correction + g1.correction;
      let tc_sum = g0.tc + g1.tc;

      let teta = teta0 + corr_sum / tc_sum * 2.0 * Gear::a0().tan();
      let a = Gear::t2a(teta);
      let dist = tc_sum * 0.5 * Gear::a0().cos() / a.cos();
      g0.r_max = dist - g1.r_min - (g1.module * 0.05 + g0.err + g1.err);
      g1.r_max = dist - g0.r_min - (g0.module * 0.05 + g0.err + g1.err);
      dist
    } else if g0.inner && !g1.inner {
      let corr_sum = g0.correction - g1.correction;
      let tc_sum = g0.tc - g1.tc;

      let teta = teta0 + corr_sum / tc_sum * 2.0 * Gear::a0().tan();
      let a = Gear::t2a(teta);
      let dist = tc_sum * 0.5 * Gear::a0().cos() / a.cos();
      g0.r_min = dist + g1.r_min + (g1.module * 0.25 + g0.err + g1.err);
      g1.r_max = g0.r_max - dist - (g0.module * 0.25 + g0.err + g1.err);
      dist
    } else if !g0.inner && g1.inner {
      let corr_sum = g1.correction - g0.correction;
      let tc_sum = g1.tc - g0.tc;

      let teta = teta0 + corr_sum / tc_sum * 2.0 * Gear::a0().tan();
      let a = Gear::t2a(teta);
      let dist = tc_sum * 0.5 * Gear::a0().cos() / a.cos();
      g1.r_min = dist + g0.r_min + (g1.module * 0.25 + g0.err + g1.err);
      g0.r_max = g1.r_max - dist - (g0.module * 0.25 + g0.err + g1.err);
      dist
    } else {
      panic!("can't couple two inner gears")
    }
  }

  fn pos(mut self, pos: Point) -> Self {
    self.pos = pos;
    self
  }

  fn contains(&self, pos: Point) -> bool {
    let pos = pos - self.pos;
    let r = pos.len();
    if r <= self.r_min {
      return !self.inner;
    }
    if r >= self.r_max {
      return self.inner;
    }
    let a = f32::atan2(pos.y, pos.x);
    let ga = 2.0 * PI / self.tc;
    let a = a + self.phase * ga;
    let a = a.rem_euclid(ga);
    let a = if a > ga * 0.5 { ga - a } else { a };
    let da = f32::min(1.0, self.ir / r).acos();
    let tanl = r * da.sin();

    if self.inner {
      tanl - self.tan0 > (da - a - self.a1) * self.ir + self.err
    } else {
      tanl - self.tan0 + self.err < (da - a - self.a1) * self.ir
    }
  }

  fn a2t(a: f32) -> f32 {
    a.tan() - a
  }

  fn t2a(t: f32) -> f32 {
    println!("t={t}");
    let mut a0 = 0.0.to_radians();
    let mut a1 = 85.0.to_radians();
    loop {
      let am = (a0 + a1) * 0.5;
      if am == a0 || am == a1 {
        return am;
      }
      if Self::a2t(am) > t {
        a1 = am
      } else {
        a0 = am
      }
    }
  }
}

pub struct AxleToolCreator {
  rolls: Vec<Point>,
  g1: Gear,
  g2: Gear,
}

impl AxleToolCreator {
  pub fn new() -> Self {
    let mut rolls = Vec::new();

    fn abp(p1: Point, p2: Point) -> f32 {
      f32::atan2(cross(p1, p2), dot(p1, p2))
    }

    let drum_pos = Point { x: 38.0, y: 22.0 };
    let crank1_pos = Point { x: 65.0, y: 29.0 };
    let crank2_pos = Point { x: 48.307, y: 45.0 };
    let roll1 = Point { x: -9.1920, y: -16.1036 };
    let roll2 = Point { x: 6.46, y: -12.0977 };

    let dcp1 = crank1_pos - drum_pos;
    let dcp2 = crank2_pos - drum_pos;

    let get_rp = |a1: f32, a2: f32| -> Point {
      let dcp2 = complex_mul(dcp2, Point::from_angle(a1));
      let rp2 = complex_mul(roll2, Point::from_angle(a1 + a2));
      dcp2 + rp2
    };

    let max_ra = 2.0 / roll2.len();
    let over_a = 0.3 / roll2.len();
    let mut cur_a = -1.056;

    let ca1 = 0.0;
    let ca2 = 0.48;
    let ca3 = 0.94;
    let ca4 = 3.16 - 0.94;
    let ca5 = 3.16 - 0.48;
    let ca6 = 3.16;

    let dt = 0.55;

    println!("max_ra={max_ra}");

    while cur_a < ca6 {
      let ra = if cur_a < ca1 {
        -max_ra
      } else if cur_a < ca2 {
        f32::min(f32::min((cur_a - ca1) * dt - max_ra, (ca2 - cur_a) * dt), over_a)
      } else if cur_a < ca3 {
        f32::min(f32::min((cur_a - ca2) * dt, (ca3 - cur_a) * dt + max_ra), max_ra + over_a)
      } else if cur_a < ca4 {
        max_ra
      } else if cur_a < ca5 {
        f32::min(f32::min((ca5 - cur_a) * dt, (cur_a - ca4) * dt + max_ra), max_ra + over_a)
      } else {
        f32::min(f32::min((ca6 - cur_a) * dt - max_ra, (cur_a - ca5) * dt), over_a)
      };

      rolls.push(get_rp(cur_a, ra));
      cur_a += 0.001;
    }

    let mut g1 = Gear::new(10, 0.0, 1.0, 0.8, 0.00, false);
    let mut g2 = Gear::new(20, 0.5, 1.0, 2.2, 0.00, true);
    let dist = Gear::couple(&mut g1, &mut g2);
    g1.r_min =  1.414614;
    g1.r_max = 3.5499637;
    g2.r_min = 1.9996414;
    g2.r_max = 4.134991;


    let g2 = g2.pos(Point::X.scale(dist));

    println!("dist={}, g1={:?}, g2={:?}", dist, g1, g2);

    Self { rolls, g1, g2 }
  }

  pub fn get_quality() -> usize {
    1
  }

  pub fn get_size() -> f32 {
    1.0
  }

  pub fn faces(&self) -> usize {
    10
  }

  pub fn get_height(&self, part_index: usize) -> f32 {
    3.0
  }

  pub fn get_name(&self, part_index: usize) -> Option<&str> {
    match part_index {
      8 => Some("gear1"),
      9 => Some("gear2"),
      _ => None,
    }
  }

  pub fn get_count(&self, part_index: usize) -> usize {
    if part_index == 0 {
      2
    } else {
      1
    }
  }

  pub fn aabb(&self, part_index: usize) -> Option<AABB> {
    if part_index < 8 {
      return Some(AABB::empty());
    }
    None
  }

  pub fn get_sticker_index(&self, pos: Point, part_index: usize) -> PartIndex {
    match part_index {
      /*
      0 => {
        if pos.len() > 10.0 && (pos.y.abs() > 7.0 || pos.x > 17.0 || pos.x < 0.0) {
          return 0;
        }
        if pos.len() < 5.1 && pos.y.abs() < 4.0 {
          return 0;
        }
        if pos.x > 7.9 && pos.x < 13.1 && pos.y.abs() < 4.0 {
          return 0;
        }
        return 1;
      }
      1 => {
        if (pos.x.abs() < 4.0 && pos.y.abs() < 3.9) || (pos.x.abs() < 1.9 && pos.y.abs() < 7.0) {
          if pos.len() > 2.0 {
            return 1;
          }
        }
        return 0;
      }
      2 => {
        if pos.x.abs() < 8.0 && pos.y.abs() < 15.0 {
          if pos.len() > 2.0 && (pos - Point::X).len() > 2.0 && (pos + Point::X).len() > 2.0 {
            return 1;
          }
        }
        return 0;
      }
      3 => {
        if pos.len() > 24.0 && (pos.y.abs() > 10.0 || pos.x > 120.0 || pos.x < 0.0) {
          return 0;
        }

        let a0 = Point::from_angle(PI / 12.0);
        let pos0 = complex_mul(pos, a0);
        let a = Point::from_angle(PI / 3.0 * 2.0);
        let pos1 = complex_mul(pos0, a);
        let pos2 = complex_mul(pos1, a);
        if pos0.y.abs() < 8.0 && pos1.y.abs() < 8.0 && pos2.y.abs() < 8.0 {
          return 0;
        }

        if pos0.y.abs() < 8.0 && pos0.x < 0.0 {
          return 0;
        }
        if pos0.x < -8.0 {
          return 0;
        }

        return 1;
      }
      4 => {
        let l1 = pos.len() - 39.0;
        let l2 = pos.y.abs() - 7.0;

        if l1 > 0.0
          && (l2 > 0.0 || pos.x > 120.0 || pos.x < 0.0)
          && (l1 > 3.0 || l2 > 3.0 || sqr(l1 - 3.0) + sqr(l2 - 3.0) < sqr(3.0))
        {
          return 0;
        }
        if pos.len() < 29.0 {
          let r = 4.4;
          let d = 29.0 + r - 2.0;
          if (pos - Point::X.scale(d)).len() < r
            || (pos - Point::Y.scale(d)).len() < r
            || (pos + Point::X.scale(d)).len() < r
            || (pos + Point::Y.scale(d)).len() < r
          {
            return 1;
          }
          return 0;
        }
        return 1;
      }
      5 => {
        let l1 = pos.len() - 49.0;
        let l2 = pos.y.abs() - 7.0;
        let error = 0.0;

        if l1 > 0.0
          && (l2 > 0.0 || pos.x > 140.0 || pos.x < 0.0)
          && (l1 > 3.0 || l2 > 3.0 || sqr(l1 - 3.0) + sqr(l2 - 3.0) < sqr(3.0))
        {
          return 0;
        }
        if pos.len() < 39.0 + error {
          let r = 3.05;
          let d = 39.0 + r - 2.7;
          for i in 0..3 {
            let a = i as f32 * PI / 3.0;
            let a = Point::from_angle(a);
            if (pos - a.scale(d)).len() < r - error {
              return 1;
            }
            if (pos + a.scale(d)).len() < r - error {
              return 1;
            }
          }
          return 0;
        }
        return 1;
      }
      6 => {
        let max_r = if pos.y > 0.0 { 17.0 } else { 16.0 };
        if pos.len() > max_r || pos.len() < 6.0 {
          return 0;
        }
        let bc = Point::from_angle(105.0 * PI / 180.0).scale(11.5);
        if (pos - bc).len() < 1.5 {
          return 0;
        }

        let bc = Point::from_angle(60.0 * PI / 180.0).scale(9.25);
        if (pos - bc).len() < 1.25 || (pos + bc).len() < 1.25 {
          return 0;
        }

        for i in 0..self.rolls.len() - 1 {
          if dist_pl(pos, self.rolls[i], self.rolls[i + 1]) < 6.0 {
            return 0;
          }
        }

        return 1;
      }
      7 => {
        if pos.len() < 6.0 || pos.len() > 10.5 {
          return 0;
        }
        let bc = Point::from_angle(105.0 * PI / 180.0).scale(11.5);
        if (pos - bc).len() < 3.5 {
          return 0;
        }
        let bc = Point::from_angle(105.0 * PI / 180.0).scale(10.0);
        let bc2 = bc + Point::from_angle((105.0 - 130.0) * PI / 180.0).scale(4.0);
        if dist_pl(pos, bc, bc2) < 0.8 {
          return 0;
        }
        let bc2 = bc + Point::from_angle((105.0 + 130.0) * PI / 180.0).scale(4.0);
        if dist_pl(pos, bc, bc2) < 0.8 {
          return 0;
        }

        let bc = Point::from_angle(60.0 * PI / 180.0).scale(9.25);
        if (pos - bc).len() < 1.25 || (pos + bc).len() < 1.25 {
          return 0;
        }
        let bc = Point::from_angle(60.0 * PI / 180.0).scale(10.5);
        if (pos - bc).len() < 1.0 || (pos + bc).len() < 1.0 {
          return 0;
        }

        return 1;
      }*/
      8 => {
        if self.g1.contains(pos) && (pos - self.g1.pos).len() < self.g1.r_max + 1.0 {
          return 1;
        }
        return 0;
      }
      9 => {
        if self.g2.contains(pos) && (pos - self.g2.pos).len() < self.g2.r_max + 1.0 {
          return 1;
        }
        return 0;
      }
      _ => 0,
    }
  }
}
