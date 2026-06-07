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

  fn inside_profile_or_in(&self, r: f32, a: f32, err: f32) -> bool {
    if r < self.r_in - err {
      return true;
    }
    let t = 2.0 * PI / self.z;
    let m = self.w_angle(r);
    let m = f32::max(0.0, m - err / (self.z * 0.5 * COS0));
    let a = a.rem_euclid(t);
    let a = f32::min(a, t - a);
    a < m
  }

  fn inside_out_gear(&self, r: f32, a: f32, z: f32, w: f32, err: f32) -> bool {
    if z < 0.0 || z > w {
      return false;
    }
    let dz = f32::max(0.0, 0.5 - (f32::min(z, w - z)));
    if r > self.r_out - err - dz * 4.0 {
      return false;
    }
    if r < self.r_in - err {
      return true;
    }
    self.inside_profile(r, a, err)
  }

  fn inside_in_gear(&self, r: f32, a: f32, z: f32, w: f32, err: f32) -> bool {
    if z < 0.0 || z > w {
      return false;
    }
    let dz = f32::max(0.0, 0.5 - (f32::min(z, w - z)));
    if r > self.r_out + err {
      return true;
    }
    if r < self.r_in + (err + dz * 4.0) {
      return false;
    }
    !self.inside_profile(r, a, -err)
  }
}

fn sat_center(i: usize, cd: f32) -> points2d::Point {
  points2d::Point::from_angle(i as f32 * PI * 2.0 / 3.0).scale(cd)
}

pub struct Task {
  z0: f32,
  index0: PartIndex,
  car_r: f32,
  scale: f32,
  sun1: Profile,
  sat1: Profile,
  sat2: Profile,
  ring2: Profile,
}

const TASK_EASY: Task = Task {
  z0: -65.0,
  index0: 1000,
  car_r: 6.2339616,
  scale: 4.0,
  sun1: Profile { z: 6.0, x: 0.4, r_in: 2.038175, r_out: 4.4082665 },
  sat1: Profile { z: 5.0, x: 0.6, r_in: 1.575695, r_out: 3.9457865 },
  sat2: Profile { z: 4.0, x: 1.4, r_in: 1.4286623, r_out: 3.7098887 },
  ring2: Profile { z: 15.0, x: 2.4, r_in: 7.9126244, r_out: 10.1938505 },
};

const TASK_MEDIUM: Task = Task {
  z0: -40.0,
  index0: 2000,
  car_r: 6.8667192,
  scale: 4.0,
  sun1: Profile { z: 7.0, x: 0.6, r_in: 2.6709328, r_out: 5.04159 },
  sat1: Profile { z: 5.0, x: 0.6, r_in: 1.575129, r_out: 3.9457865 },
  sat2: Profile { z: 4.0, x: 1.4, r_in: 1.4112787, r_out: 3.7098887 },
  ring2: Profile { z: 16.0, x: 2.6, r_in: 8.527998, r_out: 10.826608 },
};

const TASK_HARD: Task = Task {
  z0: -15.0,
  index0: 3000,
  car_r: 5.661671,
  scale: 4.0,
  sun1: Profile { z: 5.0, x: 0.45, r_in: 1.5300295, r_out: 3.8816416 },
  sat1: Profile { z: 5.0, x: 0.45, r_in: 1.5300295, r_out: 3.8816416 },
  sat2: Profile { z: 4.0, x: 1.3, r_in: 1.4353871, r_out: 3.670145 },
  ring2: Profile { z: 14.0, x: 2.2, r_in: 7.3470583, r_out: 9.581816 },
};

const TASK_NIHTMARE: Task = Task {
  z0: 10.0,
  index0: 4000,
  car_r: 10.940303,
  scale: 2.0,
  sun1: Profile { z: 5.0, x: 0.6, r_in: 1.7370806, r_out: 3.9457865 },
  sat1: Profile { z: 15.0, x: 0.6, r_in: 6.7445164, r_out: 8.953222 },
  sat2: Profile { z: 12.0, x: 1.0, r_in: 4.904685, r_out: 7.7908163 },
  ring2: Profile { z: 32.0, x: 2.2, r_in: 16.094988, r_out: 18.98112 },
};

const TASK_BRUTAL: Task = Task {
  z0: 35.0,
  index0: 5000,
  car_r: 9.03408,
  scale: 2.5,
  sun1: Profile { z: 4.0, x: 1.2, r_in: 1.2915349, r_out: 3.6302495 },
  sat1: Profile { z: 12.0, x: 0.2, r_in: 5.15383, r_out: 7.4925447 },
  sat2: Profile { z: 9.0, x: 0.6, r_in: 3.3793974, r_out: 5.9004116 },
  ring2: Profile { z: 24.0, x: 2.8897452, r_in: 12.663478, r_out: 15.184492 },
};

impl Task {
  fn get_part_index(&self, pos: Point) -> PartIndex {
    if pos.z < self.z0 || pos.z > self.z0 + 16.0 {
      return 0;
    }

    let wall1 = pos.z < self.z0 + 3.0;
    let wall2 = pos.z > self.z0 + 13.0;
    let wall = wall1 || wall2;
    let proj = xy(pos);
    let ge = GEAR_ERR / self.scale;
    let r = proj.len() / self.scale;
    let a = f32::atan2(proj.y, proj.x);

    for i in 0..3 {
      let sc = sat_center(i, self.car_r * self.scale);
      let sd = (proj - sc).len();
      if sd < if wall { 2.5 - 0.2 } else { 3.0 - 0.3 } {
        return self.index0 + 101 + i as PartIndex;
      }
      if sd < if wall { 2.5 } else { 3.0 } {
        return 0;
      }
      if wall1 && sd < 4.5 {
        return self.index0 + 201;
      }
      if wall2 && sd < 4.5 {
        return self.index0 + 202;
      }

      if wall1 && points2d::dist_pl(proj, -sc, -sc.scale(1.5)) < 3.0 {
        return self.index0 + 201;
      }
      if wall2 && points2d::dist_pl(proj, -sc, -sc.scale(0.0)) < 3.0 {
        return self.index0 + 202;
      }
    }

    if wall {
      let r = xy(pos).len();
      if r > self.car_r * self.scale - 3.0 && r < self.car_r * self.scale + 3.0 {
        return if wall1 { self.index0 + 201 } else { self.index0 + 202 };
      }
    }
    
    if r < self.sun1.r_in - 3.0 / self.scale && pos.x.abs() < 1.0 {
      return 0;
    }

    let z_scale = 2.0;

    let sun_a = a + ((self.sat1.z + 1.0) % 2.0) * PI / self.sun1.z;
    if self.sun1.inside_out_gear(r, sun_a, (pos.z - (self.z0 + 2.3)) / z_scale, 5.7 / z_scale, ge) {
      return self.index0 + 301;
    }

    if pos.z > self.z0 + 2.3 && pos.z < self.z0 + 3.3 && r < self.sun1.r_in + 2.0 {
      return self.index0 + 301;
    }

    if r < self.ring2.r_out + 3.0 / self.scale {
      let ring_z = (pos.z - (self.z0 + 8.0)) / z_scale;
      if self.ring2.inside_in_gear(r, a, ring_z, 5.7 / z_scale, ge) {
        return self.index0 + 305;
      }

      if pos.z > self.z0 + 12.7 && pos.z < self.z0 + 13.7 && r > self.ring2.r_out - 2.0 {
        return self.index0 + 305;
      }
    }

    for i in 0..1 {
      let sc = sat_center(i as usize, self.car_r * self.scale);
      let proj = proj - sc;
      let r = proj.len() / self.scale;
      let a = f32::atan2(proj.y, proj.x);

      let label_pos = f32::max(self.sat1.r_out, self.sat2.r_out) * self.scale - 10.0;
      if (proj - points2d::Point { x: label_pos, y: 0.0 }).len() < 1.0 {
        return 0;
      }

      if pos.z > self.z0 + 3.3 && pos.z < self.z0 - 12.7 && r < self.sat1.r_in && r < self.sat2.r_in
      {
        return self.index0 + 302 + i;
      }
      if self.sat1.inside_out_gear(r, a, (pos.z - (self.z0 + 3.3)) / z_scale, 4.7 / z_scale, ge) {
        return self.index0 + 302 + i;
      }
      if self.sat2.inside_out_gear(r, a, (pos.z - (self.z0 + 8.0)) / z_scale, 4.7 / z_scale, ge) {
        return self.index0 + 302 + i;
      }
      if pos.z > self.z0 + 6.0 && pos.z < self.z0 + 10.0 {
        if self.sat1.inside_profile_or_in(r, a, ge) && self.sat2.inside_profile_or_in(r, a, ge) {
          return self.index0 + 302 + i;
        }
      }
    }

    0
  }
}

pub struct AssembleGearCreator {}

impl AssembleGearCreator {
  pub fn new() -> Self {
    Self {}
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    if pos.x.abs() > 69.0 || pos.y.abs() > 69.0 || pos.z.abs() > 69.0 {
      return 0;
    }

    let index = TASK_EASY.get_part_index(pos);
    if index != 0 {
      return index;
    }

/*
    let index = TASK_MEDIUM.get_part_index(pos);
    if index != 0 {
      return index;
    }

    let index = TASK_HARD.get_part_index(pos);
    if index != 0 {
      return index;
    }

    let index = TASK_NIHTMARE.get_part_index(pos);
    if index != 0 {
      return index;
    }

    let index = TASK_BRUTAL.get_part_index(pos);
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
    1
  }

  pub fn get_name(&self, current_normal: usize) -> Option<String> {
    None
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    let pos = Point { x: pos.x, y: 0.0, z: pos.y };
    self.get_part_index(pos)
  }

  pub fn get_quality() -> usize {
    384
  }

  pub fn get_size() -> f32 {
    140.0
  }
}
