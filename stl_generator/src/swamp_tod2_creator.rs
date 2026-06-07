use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points2d;
use common::points3d::*;
use common::solid::*;
use lazy_static::*;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

fn sqr(x: f32) -> f32 {
  x * x
}

fn round01(f: f32) -> f32 {
  (f * 10.0).ceil() * 0.1
}

fn xy(p: Point) -> points2d::Point {
  points2d::Point { x: p.x, y: p.y }
}

const SIN0: f32 = 0.34202012;
const COS0: f32 = 0.9396926;
const TAN0: f32 = 0.36397022;
const T0: f32 = 0.01490438;

const GEAR_ERR: f32 = 0.06;
const ERR: f32 = 0.1;
//const GEAR_ERR: f32 = 0.00;
//const ERR: f32 = 0.0;

const GAP: f32 = 0.1;

const BLOCK1CR: f32 = 25.429527;
const SAT11: Profile = Profile { z: 22.0, x: 0.7, r_in: 10.374041, r_out: 12.497245 };
const SUN11: Profile = Profile { z: 26.0, x: 1.0, r_in: 12.625689, r_out: 14.925213 };
const SAT12: Profile = Profile { z: 13.0, x: 0.7, r_in: 5.86133, r_out: 7.916644 };
const SUN12: Profile = Profile { z: 35.0, x: 1.0, r_in: 17.139479, r_out: 19.434082 };
const RING1: Profile = Profile { z: 70.0, x: 2.4, r_in: 35.935215, r_out: 38.189995 };

const BLOCK2CR: f32 = 25.429527;
const SAT21: Profile = Profile { z: 13.0, x: 0.7, r_in: 5.86133, r_out: 7.916644 };
const SUN21: Profile = Profile { z: 35.0, x: 1.0, r_in: 17.139479, r_out: 19.434082 };
const SAT22: Profile = Profile { z: 22.0, x: 0.7, r_in: 10.374041, r_out: 12.497245 };
const SUN22: Profile = Profile { z: 26.0, x: 1.0, r_in: 12.625689, r_out: 14.925213 };
const RING2: Profile = Profile { z: 61.0, x: 2.4, r_in: 31.424973, r_out: 33.62956 };

const BLOCK3CR: f32 = 24.925913;
const SAT3: Profile = Profile { z: 19.0, x: 0.7, r_in: 8.870237, r_out: 10.8643055 };
const SUN3: Profile = Profile { z: 28.0, x: 1.0, r_in: 13.629049, r_out: 15.929901 };
const RING3: Profile = Profile { z: 65.0, x: 3.1102476, r_in: 33.921925, r_out: 36.12102 };

const INPUT_MOMENT: f32 = 100.0;
const MTB_MOMENT: f32 = 200.0 * 36.0 / 22.0;
const WIDTH_FACTOR: f32 = 2.5;

const BALL: f32 = 4.76;
const CRING: f32 = 4.8;
const AXLE_R: f32 = 10.5;
const OLD: f32 = 135.0;
const NUTW: f32 = 6.0;
const CHAINLINE: f32 = 50.0;
const CHAINGEAR: f32 = 35.0;

const SPRING_W: f32 = 2.0;

#[derive(Debug)]
struct Bear {
  width: f32,
  d_in: f32,
  d_out: f32,
  max_load: f32,
}

const K3X5X9TN: Bear = Bear { width: 9.0, d_in: 3.0, d_out: 5.0, max_load: 1400.0 };
const K4X7X10TN: Bear = Bear { width: 10.0, d_in: 4.0, d_out: 7.0, max_load: 1920.0 };
const K5X8X10TN: Bear = Bear { width: 10.0, d_in: 5.0, d_out: 8.0, max_load: 2500.0 };
const K6X9X10TN: Bear = Bear { width: 10.0, d_in: 6.0, d_out: 9.0, max_load: 3100.0 };
const K7X10X10TN: Bear = Bear { width: 10.0, d_in: 7.0, d_out: 10.0, max_load: 3220.0 };
const K8X11X13TN: Bear = Bear { width: 13.0, d_in: 8.0, d_out: 11.0, max_load: 5000.0 };
const K9X12X13TN: Bear = Bear { width: 13.0, d_in: 9.0, d_out: 12.0, max_load: 5610.0 };

const SMALL_BEAR: Bear = Bear { width: 8.0, d_in: 12.0, d_out: 28.0, max_load: 2360.0 };
const BIG_BEAR: Bear = Bear { width: 7.0, d_in: 35.0, d_out: 47.0, max_load: 4050.0 };

const SUN11_PROGRAM: [bool; 9] = [false, true, true, false, false, true, true, true, false];
const SUN12_PROGRAM: [bool; 9] = [false, false, true, false, false, false, false, true, true];
const SUN21_PROGRAM: [bool; 9] = [false, false, false, true, true, true, true, true, true];
const SUN22_PROGRAM: [bool; 9] = [false, false, false, false, true, false, true, false, true];

mod config {
  use super::*;

  #[derive(Debug)]
  pub struct Sun {
    pub profile: Profile,
    pub axle_moment: f32,
    pub width: f32,
    pub z1: f32,
    pub z2: f32,
    pub pawl_z1: f32,
    pub pawl_z2: f32,
    pub spacer_z: f32,
    pub pawl_axle_z: f32,
    pub part: PartIndex,
    pub part_pawl: PartIndex,
    pub part_fake_ring: PartIndex,
    pub part_control_ring: PartIndex,
    pub part_spacer_ring: PartIndex,
    pub part_ball: PartIndex,
  }

  #[derive(Debug)]
  pub struct DrivingSun {
    pub profile: Profile,
    pub r_in: f32,
    pub part: PartIndex,
    pub width: f32,
    pub z1: f32,
    pub z2: f32,
  }

  #[derive(Debug)]
  pub struct SingleSatellite {
    pub profile: Profile,
    pub part: PartIndex,
    pub z1: f32,
    pub z2: f32,
  }

  #[derive(Debug)]
  pub struct DoubleSatellite {
    pub profile1: Profile,
    pub profile2: Profile,
    pub part: PartIndex,
    pub z1: f32,
    pub z2: f32,
  }

  #[derive(Debug)]
  pub struct Ring {
    pub profile: Profile,
    pub part: PartIndex,
    pub r_out: f32,
  }

  #[derive(Debug)]
  pub struct FixedRing {
    pub profile: Profile,
    pub part: PartIndex,
    pub r_out: f32,
    pub moment: f32,
  }

  #[derive(Debug)]
  pub struct Carrier {
    pub r: f32,
    pub z1: f32,
    pub z2: f32,
    pub part: PartIndex,
    pub bear: Bear,
    pub bear_loading: f32,
    pub part_sat_axle: PartIndex,
    pub part_bear: PartIndex,
  }

  #[derive(Debug)]
  pub struct PawlSystem {
    pub contact_r: f32,
    pub z1: f32,
    pub z2: f32,
    pub part_in: PartIndex,
    pub part_pawl: PartIndex,
    pub part_out: PartIndex,
  }

  #[derive(Debug)]
  pub struct DoubleBlock1 {
    pub sat: DoubleSatellite,
    pub sun1: Sun,
    pub sun2: Sun,
    pub ring: Ring,
    pub carrier: Carrier,
    pub input_ps: PawlSystem,
    pub output_ps: PawlSystem,
    pub output_z1: f32,
    pub output_z2: f32,
  }

  #[derive(Debug)]
  pub struct DoubleBlock2 {
    pub sat: DoubleSatellite,
    pub sun1: Sun,
    pub sun2: Sun,
    pub ring: Ring,
    pub carrier: Carrier,
    pub ring_ps: PawlSystem,
  }

  #[derive(Debug)]
  pub struct SingleBlock {
    pub sat: SingleSatellite,
    pub sun: DrivingSun,
    pub ring: FixedRing,
    pub carrier: Carrier,
    pub output_ps: PawlSystem,
  }

  #[derive(Debug)]
  pub struct Input {
    pub gear_r_in: f32,
    pub part_driver: PartIndex,
    pub part_small_bear_1: PartIndex,
    pub part_small_bear_2: PartIndex,
    pub part_big_bear: PartIndex,
    pub part_cup: PartIndex,
    pub clutch_z1: f32,
    pub clutch_z2: f32,
    pub clutch_r_in: f32,
    pub clutch_r_out: f32,
    pub bear_end: f32,
  }

  #[derive(Debug)]
  pub struct RingTraction {
    pub part_traction: PartIndex,
    pub part_shaft: PartIndex,
    pub part_cup: PartIndex,
    pub part_big_bear: PartIndex,
    pub bear_end: f32,
    pub clutch_z1: f32,
    pub clutch_z2: f32,
    pub clutch_r_in: f32,
    pub clutch_r_out: f32,
    pub tr_clutch_r_in: f32,
    pub tr_clutch_r_out: f32,
  }

  #[derive(Debug)]
  pub struct Stand {
    pub part_right: PartIndex,
    pub part_left: PartIndex,
    pub part_down: PartIndex,
  }

  #[derive(Debug)]
  pub struct Config {
    pub block1: DoubleBlock1,
    pub block2: DoubleBlock2,
    pub block3: SingleBlock,
    pub input: Input,
    pub ring_traction: RingTraction,
    pub input_monent: f32,
    pub output_moment: f32,
    pub part_axle: PartIndex,
    pub part_control_shaft: PartIndex,
    pub slide_ball: Point,
    pub stand: Stand,
  }

  impl Config {
    pub fn new() -> Self {
      let high_gear1 = 1.0 + SUN11.z / RING1.z;
      let high_gear2 = 1.0 + SUN12.z / SAT12.z * SAT11.z / RING1.z;
      let axle_moment11 = (1.0 - 1.0 / high_gear1) * INPUT_MOMENT;
      let axle_moment12 = (1.0 - 1.0 / high_gear2) * INPUT_MOMENT;

      let width11 = axle_moment11 / SUN11.z * WIDTH_FACTOR + 1.0;
      let width12 = axle_moment12 / SUN12.z * WIDTH_FACTOR + 1.0;
      let width11 = f32::max(width11, width12 * SAT12.z / SAT11.z);

      let bear1_loading = INPUT_MOMENT * 1000.0 / BLOCK1CR / 3.0;
      let bear1 = K4X7X10TN;
      assert!(
        bear1.max_load > bear1_loading,
        "Bear1 is too weak, need {}, has {}!",
        bear1_loading,
        bear1.max_load
      );

      let low_gear1 = 1.0 / (1.0 + SUN21.z / RING2.z);
      let low_gear2 = 1.0 / (1.0 + SUN22.z / SAT22.z * SAT21.z / RING2.z);

      let low_gear1 = (low_gear1 * RING3.z + SUN3.z) / (RING3.z + SUN3.z);
      let low_gear2 = (low_gear2 * RING3.z + SUN3.z) / (RING3.z + SUN3.z);
      let ulow_gear = SUN3.z / (RING3.z + SUN3.z);

      let axle_moment21 = (1.0 / low_gear1 - 1.0) * INPUT_MOMENT;
      let axle_moment22 = (1.0 / low_gear2 - 1.0) * INPUT_MOMENT;

      let width21 = axle_moment21 / SUN21.z * WIDTH_FACTOR + 1.0;
      let width22 = axle_moment22 / SUN22.z * WIDTH_FACTOR + 1.0;
      let width21 = f32::max(width21, width22 * SAT22.z / SAT21.z);
      let block2_output_moment =
        (INPUT_MOMENT / f32::min(low_gear1, low_gear2)) * RING3.z / (RING3.z + SUN3.z);
      let bear2_loading = block2_output_moment * 1000.0 / BLOCK2CR / 3.0;
      let bear2 = K4X7X10TN;
      assert!(
        bear2.max_load > bear2_loading,
        "Bear2 is too weak, need {}, has {}!",
        bear2_loading,
        bear2.max_load
      );

      let output_moment = INPUT_MOMENT / ulow_gear;
      assert!(output_moment >= MTB_MOMENT, "Hub is weak and cant troll MTB-ers!");
      let ring_moment3 = output_moment - INPUT_MOMENT;

      let width3 = INPUT_MOMENT / SUN3.z * WIDTH_FACTOR + 1.0;
      let bear3_loading = output_moment * 1000.0 / BLOCK3CR / 3.0;
      let bear3 = K9X12X13TN;
      assert!(
        bear3.max_load > bear3_loading,
        "Bear3 is too weak, need {}, has {}!",
        bear3_loading,
        bear3.max_load
      );

      let stage1_gears = [1.0, high_gear1, high_gear2];
      let stage2_gears = [low_gear1, low_gear2, ulow_gear];
      let mut gears = Vec::new();
      for &h in &stage1_gears {
        for &l in &stage2_gears {
          gears.push(h * l);
        }
      }
      gears.sort_by(|a, b| a.partial_cmp(b).unwrap());
      println!("GEARS:");
      println!("  {}", gears[0]);
      for i in 1..gears.len() {
        println!("    +{}%", (gears[i] / gears[i - 1] - 1.0) * 100.0);
        println!("  {}", gears[i]);
      }
      println!();
      println!("TOTAL: {}", gears.last().unwrap() / gears.first().unwrap());
      println!();

      let width11 = round01(width11);
      let width12 = round01(width12);
      let width21 = round01(width21);
      let width22 = round01(width22);
      let width3 = round01(width3);

      // for symmerty, only when gears are totally equals
      let width21 = width12;
      let width22 = width11;

      let part_sat = 1000;
      let part_ring = 2000;
      let part_sun1 = 3000;
      let part_sun2 = 4000;
      let part_carrier = 5000;
      let part_sat_axle = 6000;
      let part_bear = 7000;

      let part_gear = 100;
      let part_pawl = 200;
      let part_spacer_ring = 300;
      let part_control_ring = 400;
      let part_spacer_ring = 500;
      let part_ball = 600;
      let part_input = 40000;
      let part_axle = 50000;
      let part_traction = 60000;
      let part_control_shaft = 50100;

      let part_stand = 70000;

      assert!(SUN11.r_in > AXLE_R + 2.0 + GAP, "Sun11 is so weak!");
      assert!(SUN12.r_in > AXLE_R + 2.0 + GAP, "Sun12 is so weak!");
      assert!(SUN21.r_in > AXLE_R + 2.0 + GAP, "Sun21 is so weak!");
      assert!(SUN22.r_in > AXLE_R + 2.0 + GAP, "Sun22 is so weak!");
      assert!(SUN3.r_in > AXLE_R + 2.0 + GAP, "Sun3 is so weak!");

      let sun11_pw = 3.0;
      let sun12_pw = 5.0;
      let sun21_pw = 4.0;
      let sun22_pw = 2.0;

      assert!(sun11_pw * AXLE_R > axle_moment11, "Sun11 pawl is so weak!");
      assert!(sun12_pw * AXLE_R > axle_moment12, "Sun12 pawl is so weak!");
      assert!(sun21_pw * AXLE_R > axle_moment21, "Sun21 pawl is so weak!");
      assert!(sun22_pw * AXLE_R > axle_moment22, "Sun22 pawl is so weak!");

      // BLOCK1 STUFF
      let part_stage = 10000;

      let block1_gb = -26.5;
      let block1_w = bear1.width + GAP * 2.0;
      let block1_ge = block1_gb + block1_w;
      let block1_gs = width11 + width12 + GAP;
      let block1_gg = round01((block1_w - block1_gs) * 0.5);
      let c1z1 = block1_gb - 2.0;
      let c1z2 = block1_ge + 2.0;
      let p1_w = 6.0;
      let p1_cr = 27.0;
      assert!((p1_w - SPRING_W * 0.5 - 2.0) * p1_cr * 1.5 > INPUT_MOMENT, "Input pawl is so weak!");
      let sun_mid = block1_gb + block1_gg + width12;

      let sat = DoubleSatellite {
        profile1: SAT11,
        profile2: SAT12,
        part: part_stage + part_sat + part_gear,
        z1: block1_gb + GAP,
        z2: block1_ge - GAP,
      };
      let ring_profile = RING1;
      let ring_r_out = round01(ring_profile.r_out) + 2.0;
      let ring =
        Ring { profile: ring_profile, part: part_stage + part_ring + part_gear, r_out: ring_r_out };

      let sun1_z1 = sun_mid + GAP;
      let sun1_z2 = sun1_z1 + width11;
      let sun1_pz1 = sun1_z2 + 2.5;
      let sun1_pz2 = sun1_pz1 + sun11_pw;
      let sun1 = Sun {
        profile: SUN11,
        axle_moment: axle_moment11,
        width: width11,
        z1: sun1_z1,
        z2: sun1_z2,
        pawl_z1: sun1_pz1,
        pawl_z2: sun1_pz2,
        spacer_z: sun1_pz2 + 1.5 + CRING,
        pawl_axle_z: sun1_pz1 - 3.0,
        part: part_stage + part_sun1 + part_gear,
        part_pawl: part_stage + part_sun1 + part_pawl,
        part_fake_ring: part_stage + part_sun1 + part_spacer_ring,
        part_control_ring: part_stage + part_sun1 + part_control_ring,
        part_spacer_ring: part_stage + part_sun1 + part_spacer_ring,
        part_ball: part_stage + part_sun1 + part_ball,
      };
      let sun2_z2 = sun_mid;
      let sun2_z1 = sun2_z2 - width12;
      let sun2_pz2 = sun2_z2 - 0.5;
      let sun2_pz1 = sun2_pz2 - sun12_pw;
      let sun2 = Sun {
        profile: SUN12,
        axle_moment: axle_moment12,
        width: width12,
        z1: sun2_z1,
        z2: sun2_z2,
        pawl_z1: sun2_pz1,
        pawl_z2: sun2_pz2,
        spacer_z: sun2_pz1 - 1.5 - CRING,
        pawl_axle_z: sun2_pz2 + 3.0,
        part: part_stage + part_sun2 + part_gear,
        part_pawl: part_stage + part_sun2 + part_pawl,
        part_fake_ring: part_stage + part_sun2 + part_spacer_ring,
        part_control_ring: part_stage + part_sun2 + part_control_ring,
        part_spacer_ring: part_stage + part_sun2 + part_spacer_ring,
        part_ball: part_stage + part_sun2 + part_ball,
      };
      let carrier = Carrier {
        r: BLOCK1CR,
        z1: c1z1,
        z2: c1z2,
        part: part_stage + part_carrier,
        bear: bear1,
        bear_loading: bear1_loading,
        part_sat_axle: part_stage + part_sat_axle,
        part_bear: part_stage + part_bear,
      };

      let input_ps = PawlSystem {
        contact_r: p1_cr,
        z1: c1z1 - p1_w - GAP,
        z2: c1z1 - GAP,
        part_in: part_input,
        part_out: carrier.part,
        part_pawl: part_input + 10,
      };

      let output_ps_z1 = c1z2 + GAP;
      let output_ps_z2 = output_ps_z1 + p1_w;

      let output_ps = PawlSystem {
        contact_r: p1_cr,
        z1: output_ps_z1,
        z2: output_ps_z2,
        part_in: carrier.part + 1,
        part_out: ring.part + 1,
        part_pawl: carrier.part + 20,
      };

      let output_z1 = f32::max(sun1.spacer_z + GAP, output_ps_z2 + GAP);
      let output_z2 = output_z1 + 2.0;

      let block1 =
        DoubleBlock1 { sat, sun1, sun2, ring, carrier, input_ps, output_ps, output_z1, output_z2 };

      // BLOCK3 STUFF
      let output_cr = 39.0;
      assert!(output_cr + 2.0 > block1.ring.r_out + 0.5, "Ring1 cant be placed to hub!");
      let output_w = 9.0;
      assert!(
        output_cr * (output_w - SPRING_W * 0.5 - 2.0) * 1.5 >= output_moment,
        "Output pawl is so weak!"
      );
      let block3_w = bear3.width + GAP * 2.0;

      let part_stage = 30000;
      let ring_profile = RING3;
      let ring_r_out = round01(ring_profile.r_out) + 2.0;
      assert!(ring_r_out < output_cr - 0.5, "Unable to disassemble ring3!");

      let ps_z1 =
        f32::max(block1.carrier.z2 + 2.0 + GAP * 2.0, block1.output_z2 - output_w + 2.0 + GAP);
      let ps_z2 = ps_z1 + output_w;
      let c3_z1 = ps_z2 - 2.0; // target: block1.output_ps.z2 + 2.0 + GAP * 2.0,
      let block3_gb = c3_z1 + 2.0;
      let block3_ge = block3_gb + block3_w;
      let c3_z2 = block3_ge + 2.0;
      let sat = SingleSatellite {
        profile: SAT3,
        part: part_stage + part_sat + part_gear,
        z1: block3_gb + GAP,
        z2: block3_ge - GAP,
      };
      let block3_gg = round01((block3_w - width3) * 0.5);
      let sun = DrivingSun {
        profile: SUN3,
        part: part_stage + part_sun1 + part_gear,
        r_in: round01(SUN3.r_in - 2.1),
        width: width3,
        z1: block3_gb + block3_gg,
        z2: block3_gb + block3_gg + width3,
      };
      let ring = FixedRing {
        profile: RING3,
        part: part_stage + part_ring + part_gear,
        r_out: ring_r_out,
        moment: ring_moment3,
      };
      let carrier = Carrier {
        r: BLOCK3CR,
        z1: c3_z1,
        z2: c3_z2,
        part: part_stage + part_carrier,
        bear: bear3,
        bear_loading: bear3_loading,
        part_sat_axle: part_stage + part_sat_axle,
        part_bear: part_stage + part_bear,
      };
      let output_ps = PawlSystem {
        contact_r: output_cr,
        z1: ps_z1,
        z2: ps_z2,
        part_in: carrier.part,
        part_pawl: carrier.part + 20,
        part_out: 0, // corpse?
      };
      let block3 = SingleBlock { sat, sun, ring, carrier, output_ps };

      // BLOCK2 STUFF
      let part_stage = 20000;
      let block2_w = bear2.width + GAP * 2.0;
      let c2z1 = block3.carrier.z2 + 5.0 + GAP * 2.0 + 0.4; // magic for align sun1.carrier_z
      let block2_gb = c2z1 + 2.0;
      let block2_ge = block2_gb + block2_w;
      let c2z2 = block2_ge + 2.0;
      let ap_w = 8.2;
      let ap_cr = 30.0;
      assert!((ap_w - SPRING_W * 0.5 - 2.0) * ap_cr * 1.5 > ring_moment3, "Axle pawl is so weak!");
      let block2_gs = width21 + width22 + GAP;
      let block2_gg = round01((block2_w - block2_gs) * 0.5);

      let sat = DoubleSatellite {
        profile1: SAT21,
        profile2: SAT22,
        part: part_stage + part_sat + part_gear,
        z1: block2_gb + GAP,
        z2: block2_ge - GAP,
      };
      let ring_profile = RING2;
      let ring_r_out = round01(ring_profile.r_out) + 2.0;
      let ring =
        Ring { profile: ring_profile, part: part_stage + part_ring + part_gear, r_out: ring_r_out };

      let sun_mid = block2_gb + block2_gg + width21;

      let sun1_z2 = sun_mid;
      let sun1_z1 = sun1_z2 - width12;
      let sun1_pz2 = sun1_z2 - 0.5;
      let sun1_pz1 = sun1_pz2 - sun21_pw;
      let sun1 = Sun {
        profile: SUN21,
        axle_moment: axle_moment21,
        width: width21,
        z1: sun1_z1,
        z2: sun1_z2,
        pawl_z1: sun1_pz1,
        pawl_z2: sun1_pz2,
        spacer_z: sun1_pz1 - 1.5 - CRING,
        pawl_axle_z: sun1_pz2 + 3.0,
        part: part_stage + part_sun1 + part_gear,
        part_pawl: part_stage + part_sun1 + part_pawl,
        part_fake_ring: part_stage + part_sun1 + part_spacer_ring,
        part_control_ring: part_stage + part_sun1 + part_control_ring,
        part_spacer_ring: part_stage + part_sun1 + part_spacer_ring,
        part_ball: part_stage + part_sun1 + part_ball,
      };

      let sun2_z1 = sun_mid + GAP;
      let sun2_z2 = sun2_z1 + width22;
      let sun2_pz1 = sun2_z2 + 2.5;
      let sun2_pz2 = sun2_pz1 + sun22_pw;
      let sun2 = Sun {
        profile: SUN22,
        axle_moment: axle_moment22,
        width: width22,
        z1: sun2_z1,
        z2: sun2_z2,
        pawl_z1: sun2_pz1,
        pawl_z2: sun2_pz2,
        spacer_z: sun2_pz2 + 1.5 + CRING,
        pawl_axle_z: sun2_pz1 - 3.0,
        part: part_stage + part_sun2 + part_gear,
        part_pawl: part_stage + part_sun2 + part_pawl,
        part_fake_ring: part_stage + part_sun2 + part_spacer_ring,
        part_control_ring: part_stage + part_sun2 + part_control_ring,
        part_spacer_ring: part_stage + part_sun2 + part_spacer_ring,
        part_ball: part_stage + part_sun2 + part_ball,
      };

      assert!(
        (block3.carrier.z2 + 2.0 + GAP * 2.0 - sun1.spacer_z).abs() < 0.001,
        "Should be aligned for {} and {}!",
        block3.carrier.z2 + 2.0 + GAP * 2.0,
        sun1.spacer_z
      );

      let carrier = Carrier {
        r: BLOCK2CR,
        z1: c2z1,
        z2: c2z2,
        part: part_stage + part_carrier,
        bear: bear2,
        bear_loading: bear2_loading,
        part_sat_axle: part_stage + part_sat_axle,
        part_bear: part_stage + part_bear,
      };

      assert!(carrier.r + sat.profile2.r_out < output_cr - 0.5, "Unable to put block2 satellites!");
      assert!(ring.r_out < output_cr - 0.5, "Unable to put block2!");

      let az2 = f32::min(sun2.spacer_z + GAP + 2.0, c2z2 + ap_w + GAP);
      let az1 = az2 - ap_w;

      let ring_ps = PawlSystem {
        contact_r: ap_cr,
        z1: az1,
        z2: az2,
        part_in: carrier.part + 30,
        part_out: carrier.part + 1,
        part_pawl: carrier.part + 20,
      };

      let block2 = DoubleBlock2 { sat, sun1, sun2, ring, carrier, ring_ps };
      assert!(block3.sun.z1 > block1.sun1.spacer_z + 2.0 + GAP, "No place for sun3!");
      assert!(block3.sun.z2 < block2.sun1.spacer_z - 2.0 - GAP, "No place for sun3!");

      let gear_r_in = CHAINGEAR * 0.5; // Shimano etc
      let driver = part_input + 100;
      let clutch_z2 = block1.sun2.spacer_z - GAP;
      let clutch_z1 = f32::min(clutch_z2 - 2.0, block1.input_ps.z1);
      let clutch_r_in = 15.0;
      let clutch_r_out = 17.0;

      let small_bear_1 = driver + 1000;
      let small_bear_2 = driver + 1100;
      let big_bear = driver + 1200;
      let cup = driver + 10;
      let bear_end = -OLD * 0.5 + NUTW + 2.0 * SMALL_BEAR.width;

      let input = Input {
        gear_r_in,
        part_driver: driver,
        clutch_z1,
        clutch_z2,
        clutch_r_in,
        clutch_r_out,
        part_small_bear_1: small_bear_1,
        part_small_bear_2: small_bear_2,
        part_big_bear: big_bear,
        part_cup: cup,
        bear_end,
      };

      let clutch_z2 = f32::max(block2.ring_ps.z2, block2.sun2.spacer_z + 2.0 + GAP);
      let clutch_z1 = f32::min(clutch_z2 - 2.0, block2.sun2.spacer_z + GAP);
      let tr_clutch_r_in = 18.5;
      let tr_clutch_r_out = 20.5;

      let ring_traction = RingTraction {
        part_traction,
        clutch_z1,
        clutch_z2,
        clutch_r_in,
        clutch_r_out,
        part_shaft: part_traction + 1,
        part_cup: part_traction + 2,
        part_big_bear: part_traction + 1000,
        bear_end: OLD * 0.5 - NUTW - 5.0,
        tr_clutch_r_in,
        tr_clutch_r_out,
      };

      let slide_ball = Point { x: 0.0, y: -(1.2 + BALL * 0.5), z: OLD * 0.5 - NUTW - 2.0 - 2.5 };

      let stand =
        Stand { part_right: part_stand + 1, part_left: part_stand + 2, part_down: part_stand + 3 };

      let result = Self {
        block1,
        block2,
        block3,
        input,
        input_monent: INPUT_MOMENT,
        output_moment,
        part_axle,
        part_control_shaft,
        ring_traction,
        slide_ball,
        stand,
      };

      println!("config={:#?}", result);
      result
    }
  }
}

#[derive(Debug)]
struct Profile {
  z: f32,
  x: f32,
  r_in: f32,
  r_out: f32,
}

impl Profile {
  fn a2t(a: f32) -> f32 {
    a.tan() - a
  }

  pub fn basic_r(&self) -> f32 {
    self.z * 0.5 * COS0
  }

  fn basic_w_angle(&self) -> f32 {
    T0 + (PI * 0.25 * COS0 + self.x * SIN0) / self.basic_r()
  }

  fn w_angle(&self, r: f32) -> f32 {
    let a = f32::min(1.0, self.basic_r() / r).acos();
    self.basic_w_angle() - Self::a2t(a)
  }

  fn inside_profile(&self, r: f32, a: f32, err: f32) -> bool {
    let t = 2.0 * PI / self.z;
    let m = self.w_angle(r);
    let m = f32::max(0.0, m - err / (self.z * 0.5 * COS0));
    let a = a.rem_euclid(t);
    let a = f32::min(a, t - a);
    a < m
  }

  fn inside_out_gear(&self, r: f32, a: f32, z: f32, w: f32) -> bool {
    if z < ERR || z > w - ERR {
      return false;
    }
    let dz = f32::max(0.0, 0.5 - (f32::min(z, w - z) - ERR));
    if r > self.r_out - GEAR_ERR - dz * 4.0 {
      return false;
    }
    if r < self.r_in - GEAR_ERR {
      return true;
    }
    self.inside_profile(r, a, GEAR_ERR)
  }

  fn inside_in_gear(&self, r: f32, a: f32, z: f32, w: f32) -> bool {
    if z < ERR || z > w - ERR {
      return false;
    }
    let dz = f32::max(0.0, 0.5 - (f32::min(z, w - z) - ERR));
    if r > self.r_out + GEAR_ERR {
      return true;
    }
    if r < self.r_in + (GEAR_ERR + dz * 4.0) {
      return false;
    }
    !self.inside_profile(r, a, -GEAR_ERR)
  }
}

struct PosStage {
  carrier_bb: f32,
  carrier_be: f32,
}

const fn make_pos_stage() -> PosStage {
  const CARRIER_BB: f32 = -30.0;
  const CARRIER_BE: f32 = CARRIER_BB + 2.0;
  PosStage { carrier_bb: CARRIER_BB, carrier_be: CARRIER_BE }
}

const POS_STAGE: PosStage = make_pos_stage();

fn sat_center(i: usize, cd: f32) -> points2d::Point {
  points2d::Point::from_angle(i as f32 * PI * 2.0 / 3.0).scale(cd)
}

pub struct Carrier {
  z1: f32,
  z2: f32,
  r1_in: f32,
  r1_out: f32,
  r2_in: f32,
  r2_out: f32,
  sat_r: f32,
  axle_r: f32,
}

pub enum CarrierResult {
  Empty,
  Gap,
  SideT,
  SideD,
  Connector,
}

impl Carrier {
  fn inside(&self, proj: points2d::Point, z: f32, r: f32, sr: f32) -> CarrierResult {
    if z < self.z1 - ERR || z > self.z2 + ERR {
      return CarrierResult::Empty;
    }
    let wall1 =
      z > self.z1 + ERR && z < self.z1 + 2.0 - ERR && r > self.r1_in + ERR && r < self.r1_out - ERR;
    let wall2 =
      z < self.z2 - ERR && z > self.z2 - 2.0 + ERR && r > self.r2_in + ERR && r < self.r2_out - ERR;
    let wall = wall1 || wall2;
    let min_r = f32::min(self.r1_out, self.r2_out);

    if sr < self.axle_r + ERR {
      return CarrierResult::Empty;
    }

    let d1 = r - (min_r - 5.0);
    let d2 = (min_r - 3.0) - r;
    let d3 = sr - self.sat_r;

    if z < self.z1 + 2.0 + ERR || z > self.z2 - 2.0 - ERR {
      if d1 > -ERR && d2 > -ERR && d3 > -ERR + 2.0 {
        if d1 > ERR
          && d2 > ERR
          && d3 > ERR + 2.0
          && z > self.z1 + 0.5 + ERR
          && z < self.z2 - 0.5 - ERR
        {
          return CarrierResult::Connector;
        }
        return CarrierResult::Gap;
      }
      if wall1 {
        return CarrierResult::SideD;
      } else if wall2 {
        return CarrierResult::SideT;
      }
    } else {
      if d1 > ERR && d2 > ERR && d3 > -ERR {
        return CarrierResult::Connector;
      }
    }

    CarrierResult::Empty
  }
}

mod clutch {
  use super::ERR;
  use common::points2d;

  pub fn in6_1(p: points2d::Point, w: f32) -> bool {
    if p.x.abs() < w - ERR {
      return true;
    }
    let r = points2d::Point::from_angle(crate::PI * 2.0 / 3.0);
    let p = points2d::complex_mul(p, r);
    if p.x.abs() < w - ERR {
      return true;
    }
    let p = points2d::complex_mul(p, r);
    if p.x.abs() < w - ERR {
      return true;
    }
    false
  }

  pub fn in6_2(p: points2d::Point, w: f32) -> bool {
    if p.x.abs() < w + ERR {
      return false;
    }
    let r = points2d::Point::from_angle(crate::PI * 2.0 / 3.0);
    let p = points2d::complex_mul(p, r);
    if p.x.abs() < w + ERR {
      return false;
    }
    let p = points2d::complex_mul(p, r);
    if p.x.abs() < w + ERR {
      return false;
    }
    true
  }

  pub fn in3_1(p: points2d::Point, w: f32) -> bool {
    if p.y.abs() < w + ERR && p.x > 0.0 {
      return false;
    }
    let r = points2d::Point::from_angle(crate::PI * 2.0 / 3.0);
    let p = points2d::complex_mul(p, r);
    if p.y.abs() < w + ERR && p.x > 0.0 {
      return false;
    }
    let r = points2d::Point::from_angle(crate::PI * 2.0 / 3.0);
    let p = points2d::complex_mul(p, r);
    if p.y.abs() < w + ERR && p.x > 0.0 {
      return false;
    }
    true
  }

  pub fn in3_2(p: points2d::Point, w: f32) -> bool {
    if p.y.abs() < w - ERR && p.x > 0.0 {
      return true;
    }
    let r = points2d::Point::from_angle(crate::PI * 2.0 / 3.0);
    let p = points2d::complex_mul(p, r);
    if p.y.abs() < w - ERR && p.x > 0.0 {
      return true;
    }
    let r = points2d::Point::from_angle(crate::PI * 2.0 / 3.0);
    let p = points2d::complex_mul(p, r);
    if p.y.abs() < w - ERR && p.x > 0.0 {
      return true;
    }
    false
  }
}

pub struct Pawl {
  start: points2d::Point,
  dir: points2d::Point,
  width: f32,
  r: f32,
  angle: f32,
  bound1: (points2d::Point, f32),
  bound2: (points2d::Point, f32),
  bound3: (points2d::Point, f32),
  spring_center: points2d::Point,
}

impl Pawl {
  fn new(contact_r: f32, l: f32) -> Self {
    use points2d::*;
    let rot_r = 2.8;
    let rot_rr = contact_r - rot_r;
    let rot_r = rot_r - GAP;
    let end_rr_lo = contact_r - GAP;
    let end_rr_hi = contact_r + 2.5;

    fn oppo_angle(a: f32, b: f32, oppo: f32) -> f32 {
      ((sqr(a) + sqr(b) - sqr(oppo)) / (2.0 * a * b)).acos()
    }

    let diag_a = (rot_r / l).asin();
    let a_lo = oppo_angle(l, rot_rr, end_rr_lo);
    let a_hi = oppo_angle(l, rot_rr, end_rr_hi);

    let start = Point { x: 0.0, y: rot_rr };
    let dir = Point::from_angle(a_lo - PI * 0.5 - diag_a);
    let dir_lo = Point::from_angle(a_lo - PI * 0.5);
    let end_lo = start + dir_lo.scale(l);
    let bound_center = start + Point { x: 0.0, y: rot_r };

    let width = 4.5 - GAP * 2.0;
    let bound1 = (bound_center, (bound_center - end_lo).len());
    let angle = a_hi - a_lo;
    let angle_v = Point::from_angle(angle);
    let bound2 = (start - complex_mul(start, angle_v.conj()), contact_r + 2.0);
    let spring_center = start - complex_mul(start, Point::from_angle(-angle - 0.5 / 3.0));
    let bound3 = (Point::ZERO, contact_r - GAP);
    Self { start, dir, width, r: rot_r, angle, bound1, bound2, bound3, spring_center }
  }

  fn inside_axle(&self, proj: points2d::Point) -> bool {
    (proj - self.start).len() < self.r - ERR
  }

  fn outside_axle(&self, proj: points2d::Point) -> bool {
    use points2d::*;
    if dot(proj - self.start, self.start) > 0.0 {
      return cross(proj, self.start).abs() > (self.r + ERR + GAP) * self.start.len();
    }
    (proj - self.start).len() > self.r + ERR + GAP || proj.len() < self.start.len() - self.r - ERR
  }

  fn inside(&self, proj: points2d::Point) -> bool {
    use points2d::*;
    if (proj - self.start).len() < self.r - ERR {
      return true;
    }
    let c = cross(self.dir, proj - self.start);
    if c > self.r - ERR || c < self.r - self.width + ERR {
      return false;
    }
    let d = dot(self.dir, proj - self.start);
    if d < 0.0 {
      return false;
    }
    if (proj - self.bound1.0).len() > self.bound1.1 - ERR {
      return false;
    }
    if (proj - self.bound2.0).len() > self.bound2.1 - ERR {
      return false;
    }
    if (proj - self.bound3.0).len() > self.bound3.1 - ERR {
      return false;
    }

    true
  }

  fn inside_no_b(&self, proj: points2d::Point) -> bool {
    use points2d::*;
    if (proj - self.start).len() < self.r - ERR {
      return true;
    }
    let c = cross(self.dir, proj - self.start);
    if c < self.r - self.width + ERR {
      return false;
    }
    let d = dot(self.dir, proj - self.start);
    if d < 0.0 {
      return false;
    }
    if (proj - self.bound1.0).len() > self.bound1.1 - ERR {
      return false;
    }
    if (proj - self.bound2.0).len() > self.bound2.1 - ERR {
      return false;
    }

    true
  }

  fn outside(&self, proj: points2d::Point) -> bool {
    use points2d::*;

    if !self.outside_axle(proj) {
      return false;
    }

    let c = cross(self.dir, proj - self.start);
    let d = dot(self.dir, proj - self.start);

    if d < 0.0 {
      if c > 0.0 && d > -self.r - ERR {
        return false;
      }
    } else {
      if c > self.r - self.width - ERR
        && (proj - self.bound1.0).len() < self.bound1.1 + ERR + GAP * 2.0
      {
        return false;
      }
    }

    if proj.x < 0.0 && proj.y > 0.0 {
      let db1 = -(d + self.r + ERR);
      let db2 = self.start.y + self.r - ERR - proj.len();
      if db1 > 0.0 && db1 * 0.3 + db2 < 0.5 {
        return false;
      }
    }

    true
  }

  fn outside_ring(&self, proj: points2d::Point) -> bool {
    use points2d::*;
    if (proj - self.bound1.0).len() > self.bound1.1 + ERR {
      return true;
    }
    if (proj - self.bound2.0).len() > self.bound2.1 + ERR {
      return true;
    }
    if (proj - self.bound3.0).len() > self.bound3.1 + ERR {
      return true;
    }
    let c = cross(self.dir, proj - self.start);
    if c > self.r + ERR {
      return true;
    }

    false
  }

  fn rotate(&self, center: points2d::Point, angle: f32) -> Self {
    use points2d::*;
    let angle_v = Point::from_angle(angle);
    Self {
      start: complex_mul(self.start - center, angle_v) + center,
      dir: complex_mul(self.dir, angle_v),
      width: self.width,
      r: self.r,
      angle: self.angle,
      bound1: (complex_mul(self.bound1.0 - center, angle_v) + center, self.bound1.1),
      bound2: (complex_mul(self.bound2.0 - center, angle_v) + center, self.bound2.1),
      bound3: (complex_mul(self.bound3.0 - center, angle_v) + center, self.bound3.1),
      spring_center: complex_mul(self.spring_center - center, angle_v) + center,
    }
  }
}

pub struct PawlSystem {
  contact_r: f32,
  l: f32,
  z1: f32,
  z2: f32,
  oz1: f32,
  oz2: f32,
  cnt: usize,
  p_lo: Pawl,
  p_hi: Pawl,
  p_ulo: Pawl,
  hi_pawls_for_ring: Vec<Pawl>,
  spring_pos: i32,
  revert: bool,
  orientation: points2d::Point,
}

impl PawlSystem {
  fn new(contact_r: f32, l: f32, z1: f32, z2: f32, cnt: usize) -> Self {
    use points2d::*;
    let mut hi_pawls_for_ring = Vec::new();

    let p_lo = Pawl::new(contact_r, l).rotate(Point::ZERO, l * 0.15 / contact_r);
    let p_hi = p_lo.rotate(p_lo.start, p_lo.angle);
    let p_ulo = p_lo.rotate(p_lo.start, -p_lo.angle * 0.1);
    let p_mid = p_lo.rotate(p_lo.start, p_lo.angle * 0.5).rotate(Point::ZERO, l / contact_r * 0.3);

    for i in 0..cnt {
      let a = i as f32 / cnt as f32 * PI;
      hi_pawls_for_ring.push(p_hi.rotate(Point::ZERO, a));
      hi_pawls_for_ring.push(p_mid.rotate(Point::ZERO, a));
    }

    let oz1 = z1;
    let oz2 = z2;

    Self {
      contact_r,
      l,
      z1,
      z2,
      oz1,
      oz2,
      cnt,
      p_lo,
      p_hi,
      p_ulo,
      hi_pawls_for_ring,
      spring_pos: 0,
      revert: false,
      orientation: points2d::Point::X,
    }
  }

  fn oz1(mut self, oz1: f32) -> Self {
    self.oz1 = f32::min(self.oz1, oz1);
    self
  }

  fn oz2(mut self, oz2: f32) -> Self {
    self.oz2 = f32::max(self.oz2, oz2);
    self
  }

  fn spring_pos(mut self, spring_pos: i32) -> Self {
    self.spring_pos = spring_pos;
    self
  }

  fn revert(mut self) -> Self {
    self.revert = true;
    self
  }

  fn orientation(mut self, orientation: points2d::Point) -> Self {
    self.orientation = orientation;
    self
  }

  fn in_spring(proj: points2d::Point, r: f32, z: f32, contact_r: f32, sz: f32) -> bool {
    let zd = (z - sz).abs();
    let rd = r - (contact_r - 3.3);
    if zd < 1.0 + ERR && rd > zd - ERR {
      return true;
    }
    // if proj.y.abs() < 1.0 + ERR && rd > 0.0 || proj.y.abs() < r - (contact_r - GAP) + 1.7 {
    //   return true;
    // }
    if proj.y.abs() < 1.0 + ERR && rd > proj.y.abs() - ERR {
      return true;
    }

    false
  }

  fn in_ring(&self, proj: points2d::Point, r: f32) -> bool {
    self.p_lo.outside(proj)
      && self.p_lo.outside(-proj)
      && r < self.contact_r - 6.0
      && proj.y.abs() < self.contact_r - 7.5
  }

  fn inside(&self, proj: points2d::Point, z: f32, r: f32) -> PartIndex {
    let proj = if self.revert { -proj.conj() } else { proj };
    let proj = points2d::complex_mul(proj, self.orientation.conj());

    if z < self.oz1 + ERR || z > self.oz2 - ERR {
      return 0;
    }
    if r < self.contact_r - 8.5 {
      return 0;
    }

    let spring_z = match self.spring_pos {
      0 => (self.z1 + self.z2) * 0.5,
      -1 => self.z1 + 2.0,
      1 => self.z2 - 2.0,
      _ => panic!("Spring_pos should me -1, 0, 1!"),
    };

    if Self::in_spring(proj, r, z, self.contact_r, spring_z) {
      return 0;
    }

    if self.in_ring(proj, r) {
      return 0;
    }
    {
      let (p_lo, p_hi) = (&self.p_lo, &self.p_hi);
      if (p_hi.inside(proj) || p_hi.inside(-proj)) && (z < self.z2 - ERR && z > self.z1 + ERR) {
        if self.spring_pos == -1 && z < spring_z || self.spring_pos == 1 && z > spring_z {
          if !p_hi.inside_axle(proj) && !p_hi.inside_axle(-proj) {
            return 0;
          }
        }

        return 2;
      }
      if !p_lo.outside(proj) || !p_lo.outside(-proj) || !p_hi.outside(proj) || !p_hi.outside(-proj)
      {
        return 0;
      }
    }
    if r < self.contact_r - 2.0 - ERR
      || r < self.contact_r - ERR - GAP && (proj.x.abs() < 12.0 || proj.y.abs() < 6.0)
    {
      /*  if self.spring_pos == -1 && z > spring_z + 2.0 || self.spring_pos == 1 && z < spring_z - 2.0 {
        if proj.x.abs() > 13.0 && proj.y.abs() > 13.0 {
          return 0;
        }
      }*/

      return 1;
    }

    return 0;
  }

  fn outside_clutch(&self, proj: points2d::Point) -> bool {
    let proj = if self.revert { -proj.conj() } else { proj };
    let proj = points2d::complex_mul(proj, self.orientation.conj());

    for p_hi in &self.hi_pawls_for_ring {
      if !p_hi.outside_ring(proj) || !p_hi.outside_ring(-proj) {
        return false;
      }
    }
    true
  }
}

pub struct ControlPawl {
  start: points2d::Point,
  r: f32,
  angle: f32,
  step: f32,
  ball_dir: points2d::Point,
  bp_lo: points2d::Point,
  bp_hi: points2d::Point,
  bound: (points2d::Point, f32),
}

impl ControlPawl {
  fn new(start: points2d::Point, r: f32, angle: f32, step: f32) -> Self {
    use points2d::*;
    let bpr_lo = 1.0 + BALL * 0.5;
    let bpr_hi = 2.3 + BALL * 0.5;
    let sr = start.len();

    let lever = 4.0;
    let a = (lever / sr).asin();

    let ap = Point::from_angle(a);
    let rap = Point::from_angle(angle);
    let ball_dir = complex_mul(start, ap).norm();
    let bp_lo = ball_dir.scale(bpr_lo);
    let bp_hi = complex_mul(ball_dir.scale(bpr_hi) - start, rap) + start;

    let bound = (start - complex_mul(start, rap), start.len() + r);
    Self { start, r, angle, step, ball_dir, bp_lo, bp_hi, bound }
  }

  fn inside(&self, proj: points2d::Point, r: f32) -> bool {
    use points2d::*;

    let br = BALL * 0.5 + ERR;
    if (proj - self.bp_lo).len() < br || (proj - self.bp_hi).len() < br {
      return false;
    }
    if (proj - self.bound.0).len() > self.bound.1 - ERR {
      return false;
    }
    let psl = (proj - self.start).len();
    if psl < self.r - ERR {
      return true;
    }
    if cross(proj - self.start, self.bound.0 - self.start) < ERR {
      return false;
    }

    let sl = self.start.len();
    let sn = self.start.scale(1.0 / sl);
    let tan = cross(proj, sn);

    r > sl - self.r + ERR && tan < 0.0 && psl < 6.0 - ERR - GAP
  }

  fn outside_axle(&self, proj: points2d::Point) -> bool {
    use points2d::*;
    if dot(proj - self.start, self.start) > 0.0 {
      return cross(proj, self.start).abs() > (self.r + ERR + GAP) * self.start.len();
    }
    (proj - self.start).len() > self.r + ERR + GAP || proj.len() < self.start.len() - self.r - ERR
  }

  fn outside(&self, proj: points2d::Point, r: f32) -> bool {
    use points2d::*;
    let sl = self.start.len();
    let psl = (proj - self.start).len();

    if !self.outside_axle(proj) {
      return false;
    }

    let sn = self.start.scale(1.0 / sl);
    let tan = cross(proj, sn);

    r < sl - self.r - ERR || tan > 0.0 || psl > 6.0 + ERR
  }
}

pub struct ControlPawlSystem {
  ps: PawlSystem,
  cp: ControlPawl,
}

enum ControlPawlResult {
  Empty,
  Hole,
  Axis(PartIndex),
  Ball(PartIndex),
  Pawl(PartIndex),
  FakeRing,
  ControlRing,
  ControlShaft,
  SpacerRing,
}

impl ControlPawlSystem {
  fn new(contact_r: f32, l: f32, z1: f32, z2: f32, cnt: usize) -> Self {
    let ps = PawlSystem::new(contact_r, l, z1, z2, cnt);
    let cp = ControlPawl::new(ps.p_hi.start, ps.p_hi.r, ps.p_hi.angle, 1.0);
    Self { ps, cp }
  }

  fn cp_w() -> f32 {
    5.0
  }

  fn ball_z(&self) -> f32 {
    self.ps.z1 - Self::cp_w() * 0.5
  }

  fn inside(&self, proj: points2d::Point, z: f32, r: f32, pos: Point) -> ControlPawlResult {
    let bp_lo = self.cp.bp_lo;
    let bp_lo = bp_lo.scale(1.0 - GAP / bp_lo.len());
    let bp1 = Point { x: bp_lo.x, y: bp_lo.y, z: self.ball_z() };
    let bp2 = Point { x: -bp_lo.x, y: -bp_lo.y, z: self.ball_z() };
    if (pos - bp1).len() < BALL * 0.5 - ERR {
      return ControlPawlResult::Ball(0);
    }

    if z > self.ps.z2 + 2.0 + ERR && z < self.ps.z2 + 4.0 - ERR && r < self.ps.contact_r + 1.0 - ERR
    {
      return ControlPawlResult::Axis(0);
    }

    let z1 = self.ps.z1 - Self::cp_w() - 1.0;

    if r < 2.5 - ERR && z > z1 - 10.0 && z < self.ps.z2 + 2.0 - ERR
      || r < 12.5 - ERR && proj.x.abs() < 3.0 && z > z1 - 10.0 && z < z1 - 5.0
    {
      if (pos - bp1).len() < BALL * 0.5 + ERR {
        return ControlPawlResult::Empty;
      }
      if (pos - bp2).len() < BALL * 0.5 + ERR {
        return ControlPawlResult::Empty;
      }
      return ControlPawlResult::ControlRing;
    }

    /*
    if z > self.ps.z1 - 7.0 + ERR
      && z < self.ps.z1 - ERR - GAP
      && r > self.ps.contact_r + ERR
      && r < self.ps.contact_r + 3.0 - ERR
    {
      if z < self.ps.z1 - 3.0 - ERR - GAP || r > self.ps.contact_r + 2.0 + ERR {
        return ControlPawlResult::Ring;
      }
    }*/

    if z > self.ps.z2 + ERR + GAP
      && z < self.ps.z2 + 2.0 - ERR
      && r > self.ps.contact_r + ERR
      && r < self.ps.contact_r + 3.0 - ERR
    {
      return ControlPawlResult::FakeRing;
    }

    let spring_z = self.ps.z1 - Self::cp_w() - 1.0;

    if z > self.ps.z1 - Self::cp_w() - 3.0 + ERR + GAP && z < self.ps.z2 + 2.0 - ERR - GAP {
      let z_s = f32::max(z, spring_z);
      let proj_s = proj - self.ps.p_hi.spring_center;
      let r_s = proj_s.len();
      if !PawlSystem::in_spring(proj_s, r_s, z_s, self.ps.contact_r + 0.5, spring_z) {
        if self.ps.p_hi.inside_axle(proj) {
          return ControlPawlResult::Pawl(0);
        }
      }

      let proj_s = proj + self.ps.p_hi.spring_center;
      let r_s = proj_s.len();
      if !PawlSystem::in_spring(-proj_s, r_s, z_s, self.ps.contact_r + 0.5, spring_z) {
        if self.ps.p_hi.inside_axle(-proj) {
          return ControlPawlResult::Pawl(1);
        }
      }
    }

    if z > self.ps.z1 - 1.0 + ERR + GAP && z < self.ps.z2 - 1.0 - ERR - GAP {
      if self.ps.p_hi.inside(proj) {
        return ControlPawlResult::Pawl(0);
      }
      if self.ps.p_hi.inside(-proj) {
        return ControlPawlResult::Pawl(1);
      }
    }
    if z > self.ps.z1 - Self::cp_w() + ERR + GAP && z < self.ps.z1 - ERR - GAP {
      if self.cp.inside(proj, r) {
        return ControlPawlResult::Pawl(0);
      }
      if self.cp.inside(-proj, r) {
        return ControlPawlResult::Pawl(1);
      }
    }

    if z > self.ps.z1 - 1.0 - ERR && z < self.ps.z2 - 1.0 + ERR {
      if !self.ps.p_lo.outside(proj)
        || !self.ps.p_lo.outside(-proj)
        || !self.ps.p_ulo.outside(proj)
        || !self.ps.p_ulo.outside(-proj)
        || !self.ps.p_hi.outside(proj)
        || !self.ps.p_hi.outside(-proj)
      {
        return ControlPawlResult::Empty;
      }
    }

    if z > self.ps.z1 - Self::cp_w() - 3.0 - ERR && z < self.ps.z2 + 2.0 + ERR {
      if !self.ps.p_lo.outside_axle(proj) || !self.ps.p_lo.outside_axle(-proj) {
        return ControlPawlResult::Empty;
      }
    }

    if z > self.ps.z1 - Self::cp_w() - ERR && z < self.ps.z1 + ERR {
      if !self.cp.outside(proj, r) || !self.cp.outside(-proj, r) {
        return ControlPawlResult::Empty;
      }
    }

    let ball_dir = self.cp.bp_lo.norm();
    let b_dot1 = points2d::dot(ball_dir, proj - self.cp.bp_lo);
    let b_dot2 = points2d::dot(ball_dir, proj + self.cp.bp_lo);
    let b_cross = points2d::cross(ball_dir, proj);
    let outbr = BALL * 0.5 + ERR + GAP;

    if b_dot1 > 0.0 && sqr(b_cross) + sqr(z - bp1.z) < sqr(outbr) {
      return ControlPawlResult::Empty;
    }
    if b_dot2 < 0.0 && sqr(b_cross) + sqr(z - bp2.z) < sqr(outbr) {
      return ControlPawlResult::Empty;
    }
    if (pos - bp1).len() < outbr || (pos - bp2).len() < outbr {
      return ControlPawlResult::Empty;
    }

    if z > self.ps.z1 - Self::cp_w() - 5.0 + ERR && z < self.ps.z2 + 4.0 - ERR {
      if r < self.ps.contact_r - ERR - GAP && r > 2.5 + ERR + GAP {
        let check_d = f32::max(r - 7.0, proj.x.abs() - 5.0);
        if !PawlSystem::in_spring(proj, r, z, self.ps.contact_r, spring_z) {
          return ControlPawlResult::Axis(0);
        }
      }
    }

    return ControlPawlResult::Empty;
  }
}

pub struct ControlRingPawlSystem {
  ps: PawlSystem,
  ball_dir: points2d::Point,
  ring_r: f32,
  ring_center_lo: points2d::Point,
  ring_center_lo_rel: points2d::Point,
  ring_center_hi: points2d::Point,

  ring_center_lo_fa: points2d::Point,
  ring_center_hi_fa: points2d::Point,

  swap_z: bool,
}

impl ControlRingPawlSystem {
  fn new(contact_r: f32, l: f32, z1: f32, z2: f32, cnt: usize) -> Self {
    use points2d::*;
    let ps = PawlSystem::new(contact_r, l, z1, z2, cnt);
    let lever = -2.0;
    let ball_dir = complex_mul(-ps.p_hi.start.norm(), Point::from_angle(lever / contact_r));
    let ring_r = contact_r;
    // 0.8, 1.7 : works but tight
    let ring_center_hi = ball_dir.scale((BALL * 2.0 + 0.8) - ring_r);
    let ring_center_lo = ball_dir.scale((BALL * 2.0 + 2.3) - ring_r);

    let ring_center_hi_fa = ball_dir.scale((BALL * 2.0 + 0.5) - ring_r);
    let ring_center_lo_fa = ball_dir.scale((BALL * 2.0 + 2.5) - ring_r);

    let ring_center_lo_rel =
      complex_mul(ring_center_lo - ps.p_hi.start, Point::from_angle(ps.p_hi.angle)) + ps.p_hi.start;
    Self {
      ps,
      ball_dir,
      ring_r,
      ring_center_hi,
      ring_center_lo,
      ring_center_hi_fa,
      ring_center_lo_fa,
      ring_center_lo_rel,
      swap_z: false,
    }
  }

  fn swap_z(mut self) -> Self {
    self.swap_z = true;
    self
  }

  fn revert(mut self) -> Self {
    self.ps = self.ps.revert();
    self
  }

  fn orientation(mut self, orientation: points2d::Point) -> Self {
    self.ps = self.ps.orientation(orientation);
    self
  }

  fn ball_z(&self) -> f32 {
    self.ps.z1 - CRING * 0.5 - 0.5
  }

  fn inner_ball(&self) -> Point {
    let bp = self.ball_dir.scale(BALL * 0.5 + 1.0);
    let bp = points2d::complex_mul(bp, self.ps.orientation);
    let bp = if self.ps.revert { -bp.conj() } else { bp };
    let bz = self.ball_z();
    let bz = if self.swap_z { self.ps.z1 + self.ps.z2 - bz } else { bz };
    Point { x: bp.x, y: bp.y, z: bz }
  }

  fn inside(&self, proj: points2d::Point, z: f32, r: f32) -> ControlPawlResult {
    let z = if self.swap_z { self.ps.z1 + self.ps.z2 - z } else { z };
    let proj = if self.ps.revert { -proj.conj() } else { proj };
    let proj = points2d::complex_mul(proj, self.ps.orientation.conj());

    let pos = Point { x: proj.x, y: proj.y, z };
    let depth = self.ps.contact_r - r;
    let facet = f32::max(0.5 - depth, 0.0);
    let rz1 = self.ps.z1 - CRING - 0.5;
    let rz2 = self.ps.z1 - 0.5;

    let big_ring_hole = z > rz1 - facet - ERR && z < rz2 + facet + ERR;

    if z > rz1 + GAP + ERR && z < rz2 - GAP - ERR {
      let rr = (proj - self.ring_center_hi_fa).len();
      if rr > self.ring_r + ERR && rr < self.ring_r + 3.0 - ERR {
        return ControlPawlResult::ControlRing;
      }
    }
    if z > rz1 - 1.0 + ERR && z < self.ps.z1 - 0.5 - ERR {
      let sr = self.ring_center_lo_fa.len() + self.ring_r + 3.0;
      if r > sr + ERR && r < sr + 1.0 - ERR
        || z < rz1 - ERR && r > self.ps.contact_r + ERR && r < sr + 1.0 - ERR
      {
        return ControlPawlResult::SpacerRing;
      }
    }

    if z < self.ball_z() {
      //return ControlRingResult::Empty;
    }

    let bp = self.ball_dir.scale(BALL * 0.5 + 1.0);
    let bp = Point { x: bp.x, y: bp.y, z: self.ball_z() };
    if r > 2.5 + ERR + GAP {
      let bop = Point { x: -bp.x, y: -bp.y, z: self.ball_z() };
      let bp2 = self.ball_dir.scale(BALL * 1.5 + 1.0);
      let bp2 = Point { x: bp2.x, y: bp2.y, z: self.ball_z() };
      let bpf = self.ball_dir.scale(100.0);
      let bpf = Point { x: bpf.x, y: bpf.y, z: self.ball_z() };
      let d = dist_pl(pos, bp, bpf);
      if d < BALL * 0.5 + ERR + GAP {
        if (pos - bp).len() < BALL * 0.5 - ERR {
          return ControlPawlResult::Ball(0);
        }
        if (pos - bp2).len() < BALL * 0.5 - ERR {
          return ControlPawlResult::Ball(1);
        }
        return ControlPawlResult::Hole;
      }
    }

    /*
    // For demo models only
    if r < 2.5 - ERR && (z > self.ps.z1 - CRING - 2.0 + ERR && z < self.ps.z2 + 5.0 - GAP - ERR)
    || r < 12.5 - ERR && proj.x.abs() < 3.0 && z < self.ps.z1 - 8.0 && z > self.ps.z1 - 13.0
    {
      if (pos - bp).len() < BALL * 0.5 + ERR {
        return ControlPawlResult::Hole;
      }
      return ControlPawlResult::ControlShaft;
    }
    */

    if z > rz1 - 0.5 + ERR && z < self.ps.z2 - ERR && self.ps.p_hi.inside_no_b(proj) {
      let cond_pawl = self.ps.p_hi.inside(proj);
      let cond_ring = (proj - self.ring_center_hi).len() < self.ring_r - ERR
        && (proj - self.ring_center_lo_rel).len() < self.ring_r - ERR;

      if z > rz1 + 1.0 + ERR && z < self.ps.z2 - ERR {
        if z > self.ps.z1 + ERR {
          if cond_pawl {
            return ControlPawlResult::Pawl(0);
          }
        }
        if z < rz2 - 1.0 - ERR {
          if cond_ring {
            return ControlPawlResult::Pawl(0);
          }
        }
        if cond_pawl && cond_ring {
          return ControlPawlResult::Pawl(0);
        }
      }

      if !cond_ring {
        return ControlPawlResult::Hole;
      }
    }

    let spring_z = self.ps.z2 + 0.5;

    if z > rz1 - 2.0 + ERR && z < self.ps.z2 + 3.0 - ERR && self.ps.p_hi.inside_axle(proj) {
      let proj_s = proj - self.ps.p_hi.spring_center;
      let r_s = proj_s.len();
      let sz = f32::min(z, spring_z);
      if PawlSystem::in_spring(proj_s, r_s, sz, self.ps.contact_r, spring_z) {
        return ControlPawlResult::Hole;
      }
      return ControlPawlResult::Pawl(0);
    }

    let zend = rz1 - 4.0;

    if z > zend + ERR && z < self.ps.z2 + 3.5 - ERR && r > 2.5 + ERR + GAP && r < self.ps.contact_r
    {
      if z < zend + 0.5 + ERR && r > (z - zend + 7.0) - ERR * 2.0 {
        return ControlPawlResult::Hole;
      }

      if z < zend + 1.0 + ERR && r > (z - zend - 1.0) + self.ps.contact_r - GAP - ERR * 2.0 {
        return ControlPawlResult::Hole;
      }

      if r > self.ps.contact_r - ERR - GAP {
        return ControlPawlResult::Hole;
      }

      if z > rz1 - 2.0 - ERR - GAP
        && z < self.ps.z2 + 3.0 + ERR + GAP
        && !self.ps.p_hi.outside_axle(proj)
      {
        return ControlPawlResult::Hole;
      }

      if z > rz1 - ERR - GAP
        && z < self.ps.z2 + ERR + GAP
        && (!self.ps.p_hi.outside(proj)
          || !self.ps.p_lo.outside(proj)
          || !self.ps.p_ulo.outside(proj))
      {
        return ControlPawlResult::Hole;
      }
      if z > spring_z - 4.0 - ERR
        && z < spring_z + 1.0 + ERR
        && PawlSystem::in_spring(proj, r, z, self.ps.contact_r, self.ps.z2 + 1.0)
      {
        return ControlPawlResult::Hole;
      }

      if big_ring_hole {
        let rr_hi = (proj - self.ring_center_hi_fa).len();
        let rr_lo = (proj - self.ring_center_lo_fa).len();
        if rr_hi > self.ring_r - ERR || rr_lo > self.ring_r - ERR {
          return ControlPawlResult::Hole;
        }
      }
      return ControlPawlResult::Axis(0);
    }

    return ControlPawlResult::Empty;
  }
}

enum SectionKind {
  Sweep(f32),
  X,
  Y,
  Z(f32),
}

struct Section {
  kind: SectionKind,
  name: String,
}

pub struct SwampTodCreator {
  config: config::Config,
  carrier1: Carrier,
  carrier2: Carrier,
  carrier3: Carrier,
  input_ps: PawlSystem,
  output_ps: PawlSystem,
  stage1_output_ps: PawlSystem,
  ring_ps: PawlSystem,
  gear11_ps: ControlRingPawlSystem,
  gear12_ps: ControlRingPawlSystem,
  gear21_ps: ControlRingPawlSystem,
  gear22_ps: ControlRingPawlSystem,
  sections: Vec<Section>,
}

impl SwampTodCreator {
  pub fn new() -> Self {
    let config = config::Config::new();

    let carrier1 = Carrier {
      z1: config.block1.carrier.z1,
      z2: config.block1.carrier.z2,
      r1_in: 18.0,
      r1_out: config.block1.input_ps.contact_r + 4.0,
      r2_in: 18.0,
      r2_out: config.block1.input_ps.contact_r + 4.0,
      sat_r: f32::max(SAT11.r_out, SAT12.r_out) + 2.0,
      axle_r: config.block1.carrier.bear.d_in * 0.5,
    };

    let carrier2 = Carrier {
      z1: config.block2.carrier.z1,
      z2: config.block2.carrier.z2,
      r1_in: 18.0,
      r1_out: 31.0,
      r2_in: 18.0,
      r2_out: config.block3.ring.r_out,
      sat_r: f32::max(SAT11.r_out, SAT12.r_out) + 2.0,
      axle_r: config.block2.carrier.bear.d_in * 0.5,
    };

    let carrier3 = Carrier {
      z1: config.block3.carrier.z1,
      z2: config.block3.carrier.z2,
      r1_in: 17.0,
      r1_out: config.block3.output_ps.contact_r,
      r2_in: 17.0,
      r2_out: 33.0,
      sat_r: f32::max(SAT11.r_out, SAT12.r_out) + 2.0,
      axle_r: config.block3.carrier.bear.d_in * 0.5,
    };

    let pl = 9.0;
    let input_ps = PawlSystem::new(
      config.block1.input_ps.contact_r,
      pl,
      config.block1.input_ps.z1,
      config.block1.input_ps.z2,
      8,
    )
    .revert()
    .spring_pos(-1)
    .oz1(config.input.clutch_z1);

    let output_ps = PawlSystem::new(
      config.block3.output_ps.contact_r,
      pl,
      config.block3.output_ps.z1,
      config.block3.output_ps.z2,
      8,
    )
    .revert()
    .spring_pos(1)
    .oz2(config.block3.carrier.z1 + 2.0);

    let stage1_output_ps = PawlSystem::new(
      config.block1.output_ps.contact_r,
      pl,
      config.block1.output_ps.z1,
      config.block1.output_ps.z2,
      8,
    )
    .revert()
    .spring_pos(-1)
    .oz1(config.block1.carrier.z2 - 2.0);

    let ring_ps = PawlSystem::new(
      config.block2.ring_ps.contact_r,
      pl,
      config.block2.ring_ps.z1,
      config.block2.ring_ps.z2,
      8,
    )
    .spring_pos(1)
    .revert()
    .oz2(config.block2.sun2.spacer_z + 2.0 + GAP);

    let gear11_ps = ControlRingPawlSystem::new(
      AXLE_R,
      pl,
      config.block1.sun1.pawl_z1,
      config.block1.sun1.pawl_z2,
      3,
    )
    .swap_z()
    .orientation(points2d::Point::from_angle(PI / 2.0));
    let gear12_ps = ControlRingPawlSystem::new(
      AXLE_R,
      pl,
      config.block1.sun2.pawl_z1,
      config.block1.sun2.pawl_z2,
      3,
    );

    let gear21_ps = ControlRingPawlSystem::new(
      AXLE_R,
      pl,
      config.block2.sun1.pawl_z1,
      config.block2.sun1.pawl_z2,
      3,
    )
    .revert();

    let gear22_ps = ControlRingPawlSystem::new(
      AXLE_R,
      pl,
      config.block2.sun2.pawl_z1,
      config.block2.sun2.pawl_z2,
      3,
    )
    .swap_z()
    .revert()
    .orientation(points2d::Point::from_angle(-PI / 2.0));

    let sections = [
      Section { kind: SectionKind::X, name: "section-x".to_owned() },
      Section { kind: SectionKind::Y, name: "section-y".to_owned() },
    ]
    .into_iter()
    .collect();

    Self {
      config,
      carrier1,
      carrier2,
      carrier3,
      input_ps,
      output_ps,
      stage1_output_ps,
      ring_ps,
      gear11_ps,
      gear12_ps,
      gear21_ps,
      gear22_ps,
      sections,
    }
  }

  fn match_first_block(&self, proj: points2d::Point, z: f32, r: f32, a: f32) -> PartIndex {
    let config = &self.config;
    let block = &config.block1;
    let carrier = &block.carrier;
    let bear = &carrier.bear;

    if z < f32::min(block.input_ps.z1 + ERR, config.input.clutch_z1 + ERR)
      || z > config.block2.carrier.z1 - GAP - ERR
    {
      return 0;
    }

    if r < AXLE_R + ERR {
      return 0;
    }

    let mut sr = f32::INFINITY;
    for i in 0..3 {
      let proj = proj - sat_center(i, carrier.r);
      sr = f32::min(sr, proj.len());
    }

    let psi = &self.input_ps;
    let pso = &self.stage1_output_ps;

    if sr < bear.d_in * 0.5 + ERR && z > carrier.z1 - ERR && z < carrier.z2 + ERR {
      if sr < bear.d_in * 0.5 - ERR && z > carrier.z1 + ERR && z < carrier.z2 - ERR {
        return carrier.part_sat_axle;
      }
      return 0;
    }

    match self.carrier1.inside(proj, z, r, sr) {
      CarrierResult::Empty => {}
      CarrierResult::Gap => return 0,
      CarrierResult::SideT => {
        if !pso.in_ring(proj, r) {
          return carrier.part + 1;
        }
      }
      CarrierResult::SideD => {
        if proj.y.abs() > 18.0 + ERR || r > 20.0 + ERR {
          return carrier.part;
        }
      }
      CarrierResult::Connector => return carrier.part + 2,
    }

    // pawl system
    match psi.inside(proj, z, r) {
      1 => return block.input_ps.part_in,
      2 => return block.input_ps.part_pawl,
      _ => {}
    }
    match pso.inside(proj, z, r) {
      1 => {
        if sr > bear.d_in * 0.5 + ERR {
          return block.output_ps.part_in;
        }
      }
      2 => return block.output_ps.part_pawl,
      _ => {}
    }

    if z > block.input_ps.z1 + (2.0 + SPRING_W * 0.5 - 0.5) * 1.0 + ERR
      && z < carrier.z1 + 2.0 - ERR
      && r < psi.contact_r + 4.0 - ERR
      && r > psi.contact_r + ERR
      && psi.outside_clutch(proj)
    {
      return block.input_ps.part_out;
    }

    let ci = &config.input;

    if z > ci.clutch_z1 + ERR
      && z < ci.clutch_z1 + 2.0 - ERR
      && psi.in_ring(proj, r)
      && r > if clutch::in6_1(proj, 3.0) { ci.clutch_r_in } else { ci.clutch_r_out } + ERR
    {
      return block.input_ps.part_in;
    }

    let sat = &block.sat;
    if z > sat.z1 + ERR && z < sat.z2 - ERR {
      for i in 0..3 {
        if sr > bear.d_in * 0.5 + ERR + GAP && sr < bear.d_out * 0.5 - ERR {
          return carrier.part_bear;
        }
        let gear_index = block.sat.part + i as PartIndex;
        let proj = proj - sat_center(i, carrier.r);
        let r = proj.len();
        if r > bear.d_out * 0.5 + ERR {
          if r < bear.d_out * 0.5 + 2.0 - ERR {
            return gear_index;
          }
          let a = f32::atan2(proj.y, proj.x);
          if sat.profile2.inside_out_gear(r, a, z - block.sun2.z1, block.sun2.width + 1.0) {
            return gear_index;
          }
          if sat.profile1.inside_out_gear(r, a, z - block.sun1.z1, block.sun1.width) {
            if (proj - points2d::Point { x: SAT11.z * 0.5 - 3.0, y: 0.0 }).sqr_len() < sqr(1.0)
              && z > block.sun1.z2 - 1.0
            {
              return 0;
            }
            return gear_index;
          }
        }
      }
    }

    let sun = &block.sun2;
    if (sun.profile.inside_out_gear(r, a, z - sun.z1, sun.width)
      || (r < AXLE_R + 6.0 - ERR && z > sun.pawl_z1 - 0.5 + ERR && z < sun.pawl_z2 - ERR))
      && r > AXLE_R + ERR
      && self.gear12_ps.ps.outside_clutch(proj)
    {
      return sun.part;
    }

    let sun = &block.sun1;
    if r < sun.profile.r_out - ERR && r > AXLE_R + ERR {
      let z2 = sun.pawl_z1 - if clutch::in6_2(proj, 3.0) { 0.5 } else { 2.0 };
      if sun.profile.inside_out_gear(r, a + PI / sun.profile.z, z - sun.z1, z2 - sun.z1) {
        return sun.part;
      }
    }

    if r < AXLE_R + 6.0 - ERR
      && r > AXLE_R + ERR
      && z > sun.pawl_z1 - if clutch::in6_1(proj, 3.0) { 2.0 } else { 0.5 } + ERR
      && z < sun.pawl_z2 + 0.5 - ERR
      && self.gear11_ps.ps.outside_clutch(proj)
    {
      return sun.part + 1;
    }

    if r < block.ring.r_out - ERR && r > block.ring.profile.r_in {
      let rr = r + f32::max(0.0, z - block.sun1.z2 - 0.5);
      let z2 = carrier.z2 + 2.0 + GAP;
      if block.ring.profile.inside_in_gear(rr, a, z - block.sun1.z1, z2 - block.sun1.z1) {
        return block.ring.part;
      }
    }

    if z > carrier.z2 + GAP + ERR
      && z < carrier.z2 + 2.0 + GAP - ERR
      && r < block.ring.r_out - ERR
      && r > if clutch::in6_1(proj, 3.0) { 2.0 } else { 4.0 } + pso.contact_r + ERR
    {
      return block.ring.part;
    }

    if z > if clutch::in6_2(proj, 3.0) { 0.0 } else { 2.0 } + carrier.z2 + GAP + ERR
      && z < block.output_z2 - ERR
      && r > pso.contact_r + ERR
      && r < pso.contact_r + 4.0 - ERR
      && pso.outside_clutch(proj)
    {
      return block.output_ps.part_out;
    }

    let sun3r = config.block3.sun.r_in;
    if z > block.output_z1 + ERR
      && z < block.output_z2 - ERR
      && r < pso.contact_r + 4.0 - ERR
      && r > sun3r + ERR + if clutch::in6_1(proj, 3.0) { 0.0 } else { 2.0 }
    {
      return block.output_ps.part_out;
    }

    return 0;
  }

  fn match_second_block(&self, proj: points2d::Point, z: f32, r: f32, a: f32) -> PartIndex {
    let config = &self.config;
    let block = &config.block2;
    let carrier = &block.carrier;
    let bear = &carrier.bear;

    if z < config.block3.carrier.z2 + GAP + ERR || z > self.ring_ps.oz2 - ERR {
      return 0;
    }

    if r < AXLE_R + ERR {
      return 0;
    }

    let mut sr = f32::INFINITY;
    for i in 0..3 {
      let proj = proj - sat_center(i, carrier.r);
      sr = f32::min(sr, proj.len());
    }

    match self.carrier2.inside(proj, z, r, sr) {
      CarrierResult::Empty => {}
      CarrierResult::Gap => return 0,
      CarrierResult::SideT => {
        if r < config.block3.ring.r_out - ERR - if clutch::in6_2(proj, 5.0) { 0.0 } else { 2.0 } {
          return carrier.part + 1;
        }
      }
      CarrierResult::SideD => {
        if proj.y.abs() > 18.0 + ERR || r > 20.0 + ERR {
          return carrier.part;
        }
      }
      CarrierResult::Connector => return carrier.part + 2,
    }

    if sr < bear.d_in * 0.5 + ERR && z > carrier.z1 - ERR && z < carrier.z2 + ERR {
      if sr < bear.d_in * 0.5 - ERR && z > carrier.z1 + ERR && z < carrier.z2 - ERR {
        return carrier.part_sat_axle;
      }
      return 0;
    }

    let psr = &self.ring_ps;

    if z > carrier.z2 - 2.0 + ERR
      && z < block.ring_ps.z2 - (2.0 + SPRING_W * 0.5 - 0.5) * 1.0 - ERR
      && psr.outside_clutch(proj)
      && r > psr.contact_r + ERR
      && r < psr.contact_r + 4.0 - ERR
    {
      return carrier.part + 1;
    }

    match psr.inside(proj, z, r) {
      1 => return block.ring_ps.part_in,
      2 => return block.ring_ps.part_pawl,
      _ => {}
    }

    let rt = &config.ring_traction;

    if z > self.ring_ps.oz2 - 2.0 + ERR
      && z < self.ring_ps.oz2 - ERR
      && psr.in_ring(proj, r)
      && r > if clutch::in6_1(proj, 3.0) { rt.clutch_r_in } else { rt.clutch_r_out } + ERR
    {
      return block.ring_ps.part_in;
    }

    let sat = &block.sat;
    if z > sat.z1 + ERR && z < sat.z2 - ERR {
      for i in 0..3 {
        if sr > bear.d_in * 0.5 + ERR + GAP && sr < bear.d_out * 0.5 - ERR {
          return carrier.part_bear;
        }
        let gear_index = block.sat.part + i as PartIndex;
        let proj = proj - sat_center(i, carrier.r);
        let r = proj.len();
        if r > bear.d_out * 0.5 + ERR {
          if r < bear.d_out * 0.5 + 2.0 - ERR {
            return gear_index;
          }
          let a = f32::atan2(proj.y, proj.x);
          if sat.profile2.inside_out_gear(r, a, z - block.sun2.z1, block.sun2.width) {
            if (proj - points2d::Point { x: SAT11.z * 0.5 - 3.0, y: 0.0 }).sqr_len() < sqr(1.0)
              && z > block.sun2.z2 - 1.0
            {
              return 0;
            }
            return gear_index;
          }
          if sat.profile1.inside_out_gear(r, a, z - block.sun1.z1, block.sun1.width + 1.0) {
            return gear_index;
          }
        }
      }
    }

    let sun = &block.sun1;
    if (sun.profile.inside_out_gear(r, a, z - sun.z1, sun.width)
      || (r < AXLE_R + 6.0 - ERR && z > sun.pawl_z1 - 0.5 + ERR && z < sun.pawl_z2 - ERR))
      && r > AXLE_R + ERR
      && self.gear21_ps.ps.outside_clutch(proj)
    {
      return sun.part;
    }

    let sun = &block.sun2;
    if r < sun.profile.r_out - ERR && r > AXLE_R + ERR {
      let z2 = sun.pawl_z1 - if clutch::in6_2(proj, 3.0) { 0.5 } else { 2.0 };
      if sun.profile.inside_out_gear(r, a + PI / sun.profile.z, z - sun.z1, z2 - sun.z1) {
        return sun.part;
      }
    }

    if r < AXLE_R + 6.0 - ERR
      && r > AXLE_R + ERR
      && z > sun.pawl_z1 - if clutch::in6_1(proj, 3.0) { 2.0 } else { 0.5 } + ERR
      && z < sun.pawl_z2 + 0.5 - ERR
      && self.gear22_ps.ps.outside_clutch(proj)
    {
      return sun.part + 1;
    }

    if r < block.ring.r_out - ERR && r > block.ring.profile.r_in {
      let rr = r + f32::max(0.0, block.sun1.z1 + 0.5 - z);
      let z1 = carrier.z1 - 2.0 - GAP;
      if block.ring.profile.inside_in_gear(rr, a, z - z1, block.sun1.z2 - z1) {
        return block.ring.part;
      }
    }

    if z > carrier.z1 - 2.5 + ERR && z < carrier.z1 + 2.0 - ERR && r > 21.0 + ERR && r < 23.0 - ERR
    {
      return carrier.part;
    }

    if z > carrier.z1 - 2.0 - GAP + ERR
      && z < carrier.z1 - GAP - ERR
      && r < block.ring.r_out - ERR
      && r > 23.5 + ERR + if clutch::in6_1(proj, 3.0) { 0.0 } else { 2.0 }
    {
      return block.ring.part;
    }

    if r > 23.5 + ERR
      && r < 25.5 - ERR
      && z > config.block3.carrier.z2 + GAP + ERR
      && z < carrier.z1 - GAP - ERR - if clutch::in6_2(proj, 3.0) { 0.0 } else { 2.0 }
    {
      return block.ring.part + 1;
    }

    let sun3r = config.block3.sun.r_in;

    if z > config.block3.carrier.z2 + GAP + ERR
      && z < config.block3.carrier.z2 + 2.0 + GAP - ERR
      && r < 25.5 - ERR
      && r > sun3r + ERR + if clutch::in6_1(proj, 3.0) { 0.0 } else { 2.0 }
    {
      return block.ring.part + 1;
    }

    0
  }

  fn match_third_block(&self, proj: points2d::Point, z: f32, r: f32, a: f32) -> PartIndex {
    let config = &self.config;
    let block = &config.block3;
    let carrier = &block.carrier;
    let bear = &carrier.bear;

    if z < block.output_ps.z1 + ERR || z > config.block2.carrier.z2 - ERR {
      return 0;
    }

    if r < AXLE_R + ERR {
      return 0;
    }

    let mut sr = f32::INFINITY;
    for i in 0..3 {
      let proj = proj - sat_center(i, carrier.r);
      sr = f32::min(sr, proj.len());
    }

    match self.carrier3.inside(proj, z, r, sr) {
      CarrierResult::Empty => {}
      CarrierResult::Gap => return 0,
      CarrierResult::SideT => {
        return carrier.part + 1;
      }
      CarrierResult::SideD => {
        if self.output_ps.in_ring(proj, r) {
          return carrier.part;
        }
      }
      CarrierResult::Connector => return carrier.part + 2,
    }

    // pawl system
    match self.output_ps.inside(proj, z, r) {
      1 => return block.output_ps.part_in,
      2 => return block.output_ps.part_pawl,
      _ => {}
    }

    if sr < bear.d_in * 0.5 + ERR && z > carrier.z1 - ERR && z < carrier.z2 + ERR {
      if sr < bear.d_in * 0.5 - ERR
        && sr > bear.d_in * 0.5 - 2.0 + ERR
        && z > carrier.z1 + ERR
        && z < carrier.z2 - ERR
      {
        return carrier.part_sat_axle;
      }
      return 0;
    }

    if z > carrier.z2 - 2.0 + ERR && z < carrier.z2 + 3.0 - ERR && r > 26.0 + ERR && r < 28.0 - ERR
    {
      return carrier.part + 1;
    }

    let sat = &block.sat;
    if z > sat.z1 + ERR && z < sat.z2 - ERR {
      for i in 0..3 {
        if sr > bear.d_in * 0.5 + ERR + GAP && sr < bear.d_out * 0.5 - ERR {
          return carrier.part_bear;
        }
        let gear_index = block.sat.part + i as PartIndex;
        let proj = proj - sat_center(i, carrier.r);
        let r = proj.len();
        if r > bear.d_out * 0.5 + ERR {
          if r < bear.d_out * 0.5 + 2.0 - ERR {
            return gear_index;
          }
          let a = f32::atan2(proj.y, proj.x) - i as f32 * 2.0 / PI / 3.0;
          if sat.profile.inside_out_gear(r, a, z - block.sun.z1, block.sun.width) {
            return gear_index;
          }
        }
      }
    }

    let sun = &block.sun;
    if r > AXLE_R + GAP + ERR && sun.profile.inside_out_gear(r, a, z - sun.z1, sun.width) {
      return block.sun.part;
    }

    if r > sun.r_in + ERR
      && r < sun.r_in + 2.0 - ERR
      && z
        > if clutch::in6_2(proj, 3.0) {
          config.block1.sun1.spacer_z + GAP
        } else {
          config.block1.output_z2
        } + ERR
      && z < block.carrier.z2 + GAP + if clutch::in6_2(proj, 3.0) { 2.0 } else { 0.0 } - ERR
    {
      return block.sun.part;
    }

    let ring = &block.ring;
    let z2 = carrier.z2 + 2.0 + GAP;
    if r < ring.r_out - ERR && r > ring.profile.r_in {
      let rr = r + f32::max(0.0, z - block.sun.z2 - 0.5);
      if ring.profile.inside_in_gear(rr, a, z - block.sun.z1, z2 - block.sun.z1) {
        if r < ring.r_out - ERR - 2.0
          || z < z2 - ERR - if clutch::in6_2(proj, 5.0) { 0.0 } else { 2.0 }
        {
          return ring.part;
        }
      }
    }

    if r > 28.5 + ERR + GAP && r < ring.r_out - 2.0 - ERR && z > z2 - 2.0 + ERR && z < z2 - ERR {
      return ring.part;
    }

    if r < ring.r_out - ERR
      && r > ring.r_out - 2.0 + ERR
      && z
        < if clutch::in6_1(proj, 5.0) {
          config.block2.carrier.z2
        } else if clutch::in3_1(proj, 10.0) {
          config.block2.carrier.z2 - 2.0
        } else {
          config.block2.sun1.z2 - 0.5
        } - ERR
      && z > z2 + ERR - if clutch::in6_1(proj, 5.0) { 1.5 } else { 0.0 }
    {
      return ring.part + 1;
    }

    0
  }

  fn match_right_side(
    &self,
    proj: points2d::Point,
    z: f32,
    r: f32,
    a: f32,
    pos: Point,
  ) -> PartIndex {
    let config = &self.config;
    let input = &config.input;
    let bear_z3 = input.bear_end;
    let bear_z2 = bear_z3 - SMALL_BEAR.width;
    let bear_z1 = bear_z2 - SMALL_BEAR.width;
    let bear_r_in = SMALL_BEAR.d_in * 0.5;
    let bear_r_out = SMALL_BEAR.d_out * 0.5;

    if z > bear_z1 + ERR
      && z < bear_z2 - ERR
      && r > bear_r_in + ERR + GAP
      && r < bear_r_out - ERR - GAP
    {
      return input.part_small_bear_1;
    }
    if z > bear_z2 + ERR
      && z < bear_z3 - ERR
      && r > bear_r_in + ERR + GAP
      && r < bear_r_out - ERR - GAP
    {
      return input.part_small_bear_2;
    }

    let bbz1 = -CHAINLINE + 1.5 + 2.0;
    let bbz2 = bbz1 + BIG_BEAR.width;

    if z > bear_z3 + ERR
      && z < bear_z3 + 2.0 - ERR
      && r > bear_r_out - 2.0 + ERR
      && r < BIG_BEAR.d_in * 0.5 - ERR
    {
      return input.part_driver;
    }

    if z > bear_z1 + ERR
      && z < bear_z3 + 2.0 - ERR
      && r > bear_r_out + ERR
      && r < input.gear_r_in - ERR
    {
      if z < -CHAINLINE + 1.5 + ERR {
        let shimano_keep_r = 3.0;
        let shimano_keep = points2d::Point { x: 19.0, y: 0.0 };
        if (proj - shimano_keep).len() < shimano_keep_r + ERR {
          return 0;
        }
        let r = points2d::Point::from_angle(crate::PI * 2.0 / 3.0);
        let shimano_keep = points2d::complex_mul(shimano_keep, r);
        if (proj - shimano_keep).len() < shimano_keep_r + ERR {
          return 0;
        }
        let r = points2d::Point::from_angle(crate::PI * 2.0 / 3.0);
        let shimano_keep = points2d::complex_mul(shimano_keep, r);
        if (proj - shimano_keep).len() < shimano_keep_r + ERR {
          return 0;
        }
      }

      if r > bear_r_out + 2.0 + f32::max(0.0, z - (-CHAINLINE - 8.0)) - ERR {
        return 0;
      }

      if z > bbz1 - ERR && r > BIG_BEAR.d_in * 0.5 - ERR {
        return 0;
      }

      if sqr(r - 17.25) + sqr(z - (-CHAINLINE - 1.5 - 1.25)) < sqr(1.25 + ERR) {
        return 0;
      }

      return input.part_driver;
    }

    if z > bbz1 - 2.0 + ERR
      && z < bbz1 - ERR
      && r > input.clutch_r_in + ERR
      && r < BIG_BEAR.d_in * 0.5 + 2.0 - ERR
    {
      return input.part_driver;
    }

    if z > bbz1 + ERR
      && z < bbz2 - ERR
      && r > input.clutch_r_in + ERR
      && r < BIG_BEAR.d_in * 0.5 - ERR
    {
      return input.part_driver;
    }

    if z > bbz1 + ERR
      && z < if clutch::in6_2(proj, 3.0) { input.clutch_z2 } else { input.clutch_z1 } - ERR
      && r > input.clutch_r_in + ERR
      && r < input.clutch_r_out - ERR
    {
      return input.part_driver;
    }

    if z > bbz1 + ERR
      && z < bbz2 - ERR
      && r > BIG_BEAR.d_in * 0.5 + ERR + GAP
      && r < BIG_BEAR.d_out * 0.5 - ERR
    {
      return input.part_big_bear;
    }

    let ofc = 4.5;

    if z > bbz1 + ERR
      && z < bbz2 + 2.0 - ERR
      && r > BIG_BEAR.d_out * 0.5 + ERR
      && r < BIG_BEAR.d_out * 0.5 + 2.0 - ERR
    {
      return input.part_cup;
    }

    if z > bbz2 + ERR
      && z < bbz2 + 2.0 - ERR
      && r > BIG_BEAR.d_out * 0.5 - 2.0 + ERR
      && r < BIG_BEAR.d_out * 0.5 + 2.0 - ERR
    {
      return input.part_cup;
    }

    if z > bbz2 + ofc + ERR
      && z < bbz2 + 2.0 + ofc - ERR
      && r > config.block3.output_ps.contact_r + ERR
      && r < config.block3.output_ps.contact_r + 4.0 - ERR
    {
      return input.part_cup;
    }

    let rmin = BIG_BEAR.d_out * 0.5;
    let rmax = config.block3.output_ps.contact_r + 2.0;

    if r > rmin && r < rmax {
      let z1 = bbz2 + (r - rmin) / (rmax - rmin) * (ofc - 1.0);
      if z > z1 + 1.0 + ERR && z < z1 + 2.0 - ERR {
        return input.part_cup;
      }
    }

    if z > bbz2 + ofc + ERR
      && z < bbz2 + 4.0 + ofc - ERR
      && r > config.block3.output_ps.contact_r + ERR
      && r < config.block3.output_ps.contact_r + 2.0 - ERR
    {
      return input.part_cup;
    }

    0
  }

  fn match_left_side(
    &self,
    proj: points2d::Point,
    z: f32,
    r: f32,
    a: f32,
    pos: Point,
  ) -> PartIndex {
    let config = &self.config;
    let rt = &config.ring_traction;

    let bbz1 = rt.bear_end - BIG_BEAR.width;
    let bbz2 = rt.bear_end;

    if z > bbz1 + ERR
      && z < bbz2 - ERR
      && r > BIG_BEAR.d_in * 0.5 + GAP + ERR
      && r < BIG_BEAR.d_out * 0.5 - ERR
    {
      return rt.part_big_bear;
    }

    let ofs = 3.5;

    if z > bbz1 + ERR
      && z < bbz2 + 2.0 - ERR
      && r > BIG_BEAR.d_in * 0.5 - 2.0 + ERR
      && r < BIG_BEAR.d_in * 0.5 - ERR
    {
      return rt.part_shaft;
    }

    if z > if clutch::in6_2(proj, 3.0) { rt.clutch_z1 } else { rt.clutch_z2 } + ERR
      && z < bbz2 + 5.0 - ERR
      && r > rt.clutch_r_in + ERR
      && r < rt.clutch_r_out - ERR
    {
      return rt.part_shaft;
    }

    if z > bbz2 + ERR
      && z < bbz2 + 5.0 - ERR
      && r > if z > bbz2 + 3.0 + ERR { 6.0 } else { rt.clutch_r_in + 1.0 } + ERR
      && r < if clutch::in6_2(proj, 3.0) { rt.tr_clutch_r_out } else { rt.tr_clutch_r_in } - ERR
    {
      if z > bbz2 + 1.0 + ERR || r < BIG_BEAR.d_in * 0.5 + 2.0 - ERR {
        return rt.part_shaft;
      }
    }

    if z > bbz2 + ERR
      && z < bbz2 + 1.0 - ERR
      && r > BIG_BEAR.d_in * 0.5 - 2.0 + ERR
      && r < BIG_BEAR.d_in * 0.5 + 2.0 - ERR
    {
      return rt.part_shaft;
    }

    if z > bbz2 + 1.0 + ERR
      && z < bbz2 + 5.0 - ERR
      && r > if clutch::in6_1(proj, 3.0) { rt.tr_clutch_r_in } else { rt.tr_clutch_r_out } + ERR
      && r < rt.clutch_r_out + 12.0 - ERR
    {
      return rt.part_traction;
    }

    if z > bbz2 + 1.0 + ERR
      && z < bbz2 + 5.0 - ERR
      && r > rt.clutch_r_out + 10.0
      && proj.x > 0.0
      && proj.x < 80.0
      && proj.y.abs() < (rt.clutch_r_out + 12.0) / (1.0 + proj.x * 0.02) - ERR
    {
      return rt.part_traction;
    }

    if z > bbz1 - 2.0 + ERR
      && z < bbz2 - ERR
      && r > BIG_BEAR.d_out * 0.5 + ERR
      && r < BIG_BEAR.d_out * 0.5 + 2.0 - ERR
    {
      return rt.part_cup;
    }

    if z > bbz1 - 2.0 + ERR
      && z < bbz1 - ERR
      && r > BIG_BEAR.d_out * 0.5 - 2.0 + ERR
      && r < BIG_BEAR.d_out * 0.5 + 2.0 - ERR
    {
      return rt.part_cup;
    }

    if z > bbz1 - 2.0 - ofs + ERR
      && z < bbz1 - ofs - ERR
      && r > config.block3.output_ps.contact_r - 2.0 + ERR
      && r < config.block3.output_ps.contact_r + 2.0 - ERR
    {
      return rt.part_cup;
    }

    if z > bbz1 - 4.0 - ofs + ERR
      && z < bbz1 - ofs - ERR
      && r > config.block3.output_ps.contact_r - 2.0 + ERR
      && r < config.block3.output_ps.contact_r - ERR
    {
      return rt.part_cup;
    }

    let rmin = BIG_BEAR.d_out * 0.5;
    let rmax = config.block3.output_ps.contact_r;

    if r > rmin && r < rmax {
      let z1 = bbz1 - (r - rmin) / (rmax - rmin) * (ofs - 1.0);
      if z > z1 - 2.0 + ERR && z < z1 - 1.0 - ERR {
        return rt.part_cup;
      }
    }

    0
  }

  fn match_axis(&self, proj: points2d::Point, z: f32, r: f32, a: f32, pos: Point) -> PartIndex {
    for (sun, ps) in [
      (&self.config.block1.sun1, &self.gear11_ps),
      (&self.config.block1.sun2, &self.gear12_ps),
      (&self.config.block2.sun1, &self.gear21_ps),
      (&self.config.block2.sun2, &self.gear22_ps),
    ] {
      match ps.inside(proj, z, r) {
        ControlPawlResult::Hole => return 0,
        ControlPawlResult::Axis(sub_part) => return self.config.part_axle + sub_part,
        ControlPawlResult::Ball(sub_part) => return sun.part_ball + sub_part,
        ControlPawlResult::Pawl(pawl) => return sun.part_pawl + pawl,
        ControlPawlResult::FakeRing => return sun.part_fake_ring,
        ControlPawlResult::ControlRing => return sun.part_control_ring,
        ControlPawlResult::ControlShaft => {}
        ControlPawlResult::SpacerRing => return sun.part_spacer_ring,
        _ => {}
      }
    }

    const SHAFT_Z_STEP: f32 = 4.0;

    fn bp_for_gear(p: Point, g: f32) -> Point {
      let a = points2d::Point::from_angle(g * PI / 2.0 * 0.0);
      let pg = points2d::complex_mul(xy(p), a);
      //let pg = xy(p);
      Point { x: pg.x, y: pg.y, z: p.z + g * SHAFT_Z_STEP }
    }

    let slide_ball = self.config.slide_ball;
    if (pos - slide_ball).len() < BALL * 0.5 - ERR {
      return self.config.part_axle + 10;
    }

    if (pos - slide_ball).len() < BALL * 0.5 + ERR {
      return 0;
    }

    if z > -85.0 && z < -80.0 - ERR && proj.x.abs() < 3.0 - ERR && proj.y.abs() < 10.0 - ERR {
      return self.config.part_control_shaft;
    }

    if z > -85.0 && z < 95.0 && r < 2.5 - ERR {
      let last_gear_ball = bp_for_gear(slide_ball, (SUN11_PROGRAM.len() - 1) as f32);
      let pr = Point { x: pos.x, y: pos.y, z: f32::min(pos.z, last_gear_ball.z) };
      if (pr - last_gear_ball).len() < BALL * 0.5 + ERR {
        return 0;
      }

      for gi10 in 0..=(SUN11_PROGRAM.len() - 1) * 10 {
        let bp = bp_for_gear(slide_ball, gi10 as f32 * 0.1);
        if (pos - bp).len() < BALL * 0.5 + ERR {
          return 0;
        }
      }

      if proj.y > 1.5 - ERR {
        for i in 0..SUN11_PROGRAM.len() {
          if i % 2 == 1 {
            let i = i as f32;
            if z > -80.0 + i * SHAFT_Z_STEP - ERR && z < -80.0 + (i + 1.0) * SHAFT_Z_STEP + ERR {
              return 0;
            }
          }
        }
      }

      for (ps, prog) in [
        (&self.gear11_ps, &SUN11_PROGRAM),
        (&self.gear12_ps, &SUN12_PROGRAM),
        (&self.gear21_ps, &SUN21_PROGRAM),
        (&self.gear22_ps, &SUN22_PROGRAM),
      ] {
        let ib = ps.inner_ball();
        let ibr = points2d::Point::from_angle(PI * 9.5 / 180.0);
        let ib0proj = points2d::complex_mul(xy(ib), ibr);
        let ib1proj = points2d::complex_mul(xy(ib), ibr.conj());
        let ib0 = Point { x: ib0proj.x, y: ib0proj.y, z: ib.z };
        let ib1 = Point { x: ib1proj.x, y: ib1proj.y, z: ib.z };

        /*
        // Code for testing ball trajectories
        for gi10 in 0..=(SUN11_PROGRAM.len() - 1) * 10 {
          let bp = bp_for_gear(ib, gi10 as f32 * 0.1);
          if (pos - bp).len() < BALL * 0.5 + ERR {
            return 0;
          }
        }
        */

        let mut prev_use = false;
        for (gear_index, use_gear) in prog.iter().enumerate() {
          if *use_gear {
            for loc_ib in [ib, ib0, ib1] {
              let bp = bp_for_gear(loc_ib, gear_index as f32);
              if (pos - bp).len() < BALL * 0.5 + ERR {
                return 0;
              }
              if prev_use {
                for gi in 0..10 {
                  let bp = bp_for_gear(loc_ib, gear_index as f32 - gi as f32 * 0.1);
                  if (pos - bp).len() < BALL * 0.5 + ERR {
                    return 0;
                  }
                }
              }
            }
          }
          prev_use = *use_gear;
        }
      }
      return self.config.part_control_shaft;
    }

    if z > -80.0 + ERR && z < 80.0 - ERR && r < 7.0 - ERR && r > 2.5 + ERR + GAP {
      if z.abs() > OLD * 0.5 - 0.5 - ERR && (r > 5.0 - ERR || pos.x.abs() > 4.0 - ERR) {
        return 0;
      }

      if r > 6.0 - ERR
        && (z < -OLD * 0.5 + NUTW + 2.0 * SMALL_BEAR.width + ERR || z > slide_ball.z - 2.5 - ERR)
      {
        if z > slide_ball.z - 2.5 + ERR
          && z < OLD * 0.5 - NUTW - 2.0 - ERR
          && r < 7.0 - ERR
          && r > 6.0 + ERR
        {
          return self.config.part_axle + 20;
        }

        return 0;
      }

      let sb_far = Point { x: slide_ball.x * 10.0, y: slide_ball.y * 10.0, z: slide_ball.z };
      if dist_pl(pos, sb_far, slide_ball) < BALL * 0.5 + ERR + GAP {
        return 0;
      }

      if z.abs() > OLD * 0.5 - NUTW - 0.5 {
        let a = (f32::atan2(pos.y, pos.x) / (2.0 * PI)).rem_euclid(1.0);
        let az = (z / 1.75).rem_euclid(1.0);
        let da = (a - az).rem_euclid(1.0);
        let da = (da - 0.5).abs() - 0.5;
        if r > f32::max(5.2, 4.9 + 2.0 * da.abs()) - ERR {
          return 0;
        }
      }

      return self.config.part_axle;
    }

    0
  }

  fn match_stand(&self, proj: points2d::Point, z: f32, r: f32, a: f32, pos: Point) -> PartIndex {
    let h = 50.0;
    let stand = &self.config.stand;
    if z.abs() > OLD * 0.5 + 2.0 + ERR && z.abs() < 75.0 - ERR {
      if proj.x.abs() < 15.0 - ERR {
        if r < 15.0 - ERR {
          if proj.x.abs() < 4.0 + ERR && (r < 5.0 + ERR || proj.y > 0.0) {
            return 0;
          }

          return stand.part_right;
        }
        if proj.y < 0.0 && proj.y > -h - 15.0 + ERR {
          return stand.part_right;
        }
      }
      if proj.x.abs() < 35.0 - ERR {
        if proj.y < -h - 10.0 - ERR && proj.y > -h - 15.0 + ERR {
          return stand.part_right;
        }
      }
    }

    if z.abs() < 80.0 - ERR
      && proj.x.abs() < 25.0 - ERR
      && proj.y < -h - 5.0 - ERR
      && proj.y > -h - 10.0 + ERR
    {
      if z.abs() > OLD * 0.5 + 2.0 - ERR && z.abs() < 75.0 + ERR && proj.x.abs() < 15.0 + ERR {
        return 0;
      }

      if proj.x.abs() > 10.0 - ERR + sqr(z / 80.0) * 15.0 {
        return 0;
      }

      return stand.part_down;
    }

    if proj.x.abs() < 3.0 - ERR
      && proj.y > -h - 10.0 + ERR
      && z.abs() < OLD * 0.5 + 2.0 - ERR
      && OLD * 0.5 + 2.0 - z.abs() < 25.0 - (proj.y + 55.0) - ERR
    {
      return stand.part_down;
    }

    if z.abs() < OLD * 0.5 - ERR
      && proj.x.abs() < 3.0 - ERR
      && proj.y < -h - ERR
      && proj.y > -h - 10.0 + ERR
    {
      return stand.part_down;
    }

    0
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    if pos.x.abs() > 95.0 || pos.y.abs() > 95.0 || pos.z.abs() > 95.0 {
      return 0;
    }

    //let pos = Point { x : pos.z, y: pos.x, z: pos.y };

    if pos.x > 0.0 && pos.y < 0.0 {
      //return 0;
    }

    let proj = xy(pos);
    let r = proj.len();
    let a = f32::atan2(proj.y, proj.x);

    let index = self.match_first_block(proj, pos.z, r, a);
    if index != 0 {
      return index;
    }
    let index = self.match_second_block(proj, pos.z, r, a);
    if index != 0 {
      return index;
    }

    let index = self.match_third_block(proj, pos.z, r, a);
    if index != 0 {
      return index;
    }
    let index = self.match_axis(proj, pos.z, r, a, pos);
    if index != 0 {
      /*  if index == self.config.part_axle {
        if proj.y > proj.x + ERR * 2.0.sqrt() {
          return index;
        } else if proj.y < proj.x - ERR * 2.0.sqrt() {
          return index + 1;
        } else {
          return 0;
        }
      }*/
      return index;
    }

    let index = self.match_right_side(proj, pos.z, r, a, pos);
    if index != 0 {
      return index;
    }
    let index = self.match_left_side(proj, pos.z, r, a, pos);
    if index != 0 {
      return index;
    }

    /*
    let index = self.match_stand(proj, pos.z, r, a, pos);
    if index != 0 {
      return index;
    }*/

    return 0;
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    0.6
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    1
  }

  pub fn faces(&self) -> usize {
    self.sections.len()
  }

  pub fn get_name(&self, current_normal: usize) -> Option<String> {
    let s = &self.sections[current_normal];
    Some(s.name.clone())
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    let s = &self.sections[current_normal];
    match s.kind {
      SectionKind::Sweep(r) => {
        if pos.x.abs() > 80.0 {
          return 0;
        }
        let p = crate::points2d::Point::from_angle(pos.x / r).scale(r);
        let p = Point { x: p.x, y: p.y, z: pos.y };
        self.get_part_index(p)
      }
      SectionKind::X => {
        let p = Point { x: 0.0, y: pos.x, z: pos.y };
        self.get_part_index(p)
      }
      SectionKind::Y => {
        let p = Point { x: pos.x, y: 0.0, z: pos.y };
        self.get_part_index(p)
      }
      SectionKind::Z(z) => {
        let p = Point { x: pos.x, y: pos.y, z };
        self.get_part_index(p)
      }
    }
  }

  pub fn get_quality() -> usize {
    100
  }

  pub fn get_size() -> f32 {
    200.0
  }
}
