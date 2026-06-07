use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct GripshiftCreator {}

pub fn sqr(x: f32) -> f32 {
  x * x
}

pub fn sqr_if_neg(x: f32) -> f32 {
  if x > 0.0 {
    0.0
  } else {
    x * x
  }
}

impl GripshiftCreator {
  pub fn new() -> Self {
    Self {}
  }

  pub fn faces(&self) -> usize {
    0
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    let result = self.get_part_index_impl(pos, self.faces());
    // if result != 20 && result != 21 { return 0; }
    result
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
    let pos = Point { x: pos.x, y: 0.0, z: pos.y };
    let part = self.get_part_index_impl(pos, current_normal);
    (part != 0) as PartIndex
  }

  pub fn get_quality() -> usize {
    100
  }

  pub fn get_size() -> f32 {
    150.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = (sqr(pos.y) + sqr(pos.z)).sqrt();
    let x = pos.x;
    if r < 11.1 {
      return 0;
    }

    if pos.y < 0.0 {
      // return 0;
    }

    let drum_min_r = 12.6;
    let cable_traj_r = 18.2;
    let drum_max_r = 22.2;
    let cable1_x = 8.0;
    let cable2_x = 13.0;
    let corpse_max = 25.2;
    let min_x = -6.5;
    let max_x = 70.0;

    fn z_to_i(z: f32, cup: bool) -> PartIndex {
      let dcup = if cup { 2 } else { 0 };
      if z > 1.0 {
        return 10 + dcup;
      }

      if z < -1.0 {
        return 11 + dcup;
      }

      return 0;
    }

    if x > min_x - 19.0 && x < min_x + 1.0 && r < drum_min_r + 2.0 {
      if pos.y.abs() > 3.0 && pos.z.abs() > 3.0 {
        if r > drum_min_r && x > min_x - 3.0 && x < min_x + 1.0 && r < drum_min_r + 2.0
          || x > min_x - 3.0 && x < min_x - 1.0 && r < drum_min_r + 2.0
          || x > min_x - 19.0 && x < min_x - 1.0 && r < drum_min_r - 0.5
        {
          return z_to_i(pos.z, false);
        }
      }
    }

    if r > drum_max_r + 0.5 && x > -30.0 && x < cable1_x + cable2_x
      || r > drum_min_r && x > min_x && x < 3.0
    {
      if x > 3.0 && r < drum_max_r + 1.5 + 0.5 * (1.0 - pos.z.abs()) {
        return 0;
      }

      let mut cup = false;

      let mut ok = || {
        fn h(x: f32, y: f32) -> f32 {
          f32::max(
            x.abs(),
            f32::max((-x * 0.5 + y * 0.75.sqrt()).abs(), (-x * 0.5 - y * 0.75.sqrt()).abs()),
          )
        }

        for cy in [drum_min_r + 4.0, -drum_min_r - 4.0] {
          let shaft_d = sqr(pos.x + 2.0) + sqr(pos.y - cy);
          let shaft_dh = h(pos.x + 2.0, pos.y - cy);
          if shaft_d < sqr(1.5) {
            return false;
          }
          if pos.z > 5.0 && shaft_d < sqr(2.8) {
            return false;
          }
          if pos.z < -5.0 && shaft_dh < 2.7 {
            return false;
          }
          if pos.z.abs() < 5.0 && shaft_d < sqr(4.5) {
            return true;
          }
        }

        let ten_y = 19.0;
        // make 17.0 to start roundind
        let rr = 15.0;

        let corpse_y = r - f32::min(corpse_max, drum_min_r + 3.0 + (x + 6.0) * 1.0);

        for (cx, ct) in [(cable1_x, cable_traj_r), (cable2_x, -cable_traj_r)] {
          let mut axle = pos.y;
          let mut rot1 = pos.x - cx;
          let rot2 = pos.z - ct;

          let rcx = -rr;
          let rcy = -15.5;

          let last_axle = rot1 - rcx - ten_y - 0.0;
          let last_rot1 = rcy - rr - axle;

          if axle < rcy && rot1 > rcx {
            let n_rot1 = (sqr(axle - rcy) + sqr(rot1 - rcx)).sqrt() - rr;
            let n_axle = -ten_y;
            (rot1, axle) = (n_rot1, n_axle);
            if rot1 < -0.5 && rot1 > -3.0 && rot2.abs() < 3.0 {
              return true;
            }
          } else if rcy - axle > rot1 - rcx {
            (rot1, axle) = (last_rot1, last_axle);
          } else {
            // leave as is
          }

          let d1 = (sqr(rot1) + sqr(rot2)).sqrt();
          let mut angle = rot1 > rot2.abs();
          if axle < 0.0 {
            if d1 < 2.5 {
              return false;
            }
            if angle && (d1 < 6.1 || last_axle < -ten_y + 2.0 || corpse_y > 2.0) {
              if rot1 > rot2.abs() + 0.3 && (d1 < 5.9 || last_axle < -ten_y + 2.0 || corpse_y > 2.2)
              {
                cup = true;
              } else {
                return false;
              }
            }
          }
          let last_d = (sqr(last_rot1) + sqr(rot2)).sqrt();
          if last_axle < -ten_y && last_d < 5.2 {
            let a = f32::atan2(last_rot1, rot2);
            let phase = a / (2.0 * PI) - last_axle;
            let phase = phase - phase.floor();
            let phase = (phase - 0.5).abs();
            if last_d < 4.8 + phase {
              return false;
            }
          }

          if axle < -ten_y - 13.0 && d1 < 9.5 {
            return false;
          }
          let or = 6.0 + f32::clamp(-ten_y - last_axle + 2.0, 0.0, 1.0);
          if axle < 0.0 && d1 < or {
            return true;
          }

          let corpse_x = d1 - or;
          let sm = f32::clamp(5.0 - axle, 0.0, 5.0);
          if corpse_x < sm && corpse_y < sm {
            if sqr(sm - corpse_x) + sqr(sm - corpse_y) > sqr(sm) {
              return true;
            }
          }

          if axle < -ten_y - 6.0 && axle > -ten_y - 8.0 && d1 < 9.0 {
            return true;
          }
        }

        if x < min_x {
          return false;
        }

        if corpse_y > 0.0 {
          return false;
        }
        true
      };

      if !ok() {
        return 0;
      }

      return z_to_i(pos.z, cup);
    }

    for cy in [6.0, -6.0] {
      let keep_d = sqr(pos.x + 2.0) + sqr(pos.y - cy);
      if keep_d < sqr(3.0) {
        return z_to_i(pos.z, false);
      } else if keep_d < sqr(3.3) {
        return 0;
      }
    }
    return 0; // tmp


    if (r < drum_max_r && x > 3.0 || r < drum_max_r + 10.0 && x > cable1_x + cable2_x + 0.5)
      && r > drum_min_r + f32::clamp(x - (max_x - 5.0), 0.0, 2.0)
      && x < max_x
    {
      let dx = x - (cable2_x + cable1_x);

      let dr = r - f32::max(drum_max_r - 2.5 - dx * 0.05, drum_max_r + 3.0 - dx * 0.7);
      if dx > 0.0 && dr > 0.5 {
        return 0;
      }
      if dx > 0.0 && dr > 0.0 {
        let a = f32::atan2(pos.y, pos.z);
        fn cabs(a: f32) -> f32 {
          (PI - a.abs()).abs()
        }

        let s1 = (sqr(cabs(a - 0.0) * 20.0 - 5.0) + sqr(x - 40.0)).sqrt();
        let s2 = (sqr(cabs(a - 0.5) * 20.0 - 25.0) + sqr(x - 30.0)).sqrt();
        let s3 = (sqr(cabs(a + 2.0) * 20.0 - 35.0) + sqr(x - 10.0)).sqrt();
        if dr - 0.25 > (s1.sin() + s2.sin() + s3.sin()) * 0.5 {
          return 0;
        }
      }

      if sqr(x - cable1_x) + sqr_if_neg(r - cable_traj_r) < sqr(0.8) {
        return 0;
      }

      let (s1, c1) = 0.0.to_radians().sin_cos();
      let (y1, z1) = (pos.y * c1 - pos.z * s1, pos.y * s1 + pos.z * c1);

      if y1 > 4.0 && y1 < 10.0 && (x - cable1_x).abs() < 2.5 && z1 > cable_traj_r {
        return 0;
      }
      if y1 > 0.0 && y1 < 10.0 && sqr(x - cable1_x) + sqr(z1 - cable_traj_r) < sqr(2.5) {
        return 0;
      }

      if sqr(x - cable2_x) + sqr_if_neg(r - cable_traj_r) < sqr(0.8) {
        return 0;
      }

      let (s2, c2) = (-75.0).to_radians().sin_cos();
      let (y2, z2) = (-(pos.y * c2 - pos.z * s2), pos.y * s2 + pos.z * c2);

      if y2 > 4.0 && y2 < 10.0 && (x - cable2_x).abs() < 2.5 && z2 > cable_traj_r {
        return 0;
      }
      if y2 > 0.0 && y2 < 10.0 && sqr(x - cable2_x) + sqr(z2 - cable_traj_r) < sqr(2.5) {
        return 0;
      }

      let f = f32::max(dx - 5.5, (drum_min_r + 1.0) - r);
      let f = f32::max(f, f32::min(dx - 0.5, r - (drum_min_r + 6.0)));

      let mut m = -f32::INFINITY;
      for a in 0..3 {
        let a = a as f32 / 6.0 * 2.0 * PI;
        let (s, c) = a.sin_cos();
        let y = pos.y * c - pos.z * s;
        let z = pos.z * c + pos.y * s;
        m = f32::max(m, f32::min(2.0 - y.abs(), f32::min(dx + 2.5, (drum_min_r + 6.0) - r)));
      }

      let f = f32::max(f, m);

      if f > 0.0 {
        return 1;
      }
      if f > -0.5 {
        return 0;
      }

      if pos.y + pos.z > 0.4 {
        return 2;
      }
      if pos.y + pos.z < -0.4 {
        return 3;
      }
      return 0;
    }

    if r < drum_min_r + f32::clamp(x - (max_x - 5.0) - 0.2, 0.0, 2.0) - 0.5
      && x > min_x
      && x < max_x
    {
      if pos.y.abs() < 0.5 {
        return 0;
      }
      return 20 + (pos.y > 0.0) as PartIndex;
    }

    return 0;
  }
}
