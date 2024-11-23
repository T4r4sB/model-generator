use crate::points2d::*;
use crate::solid::*;

use crate::slots_and_holes::*;

pub struct AxleToolCreator {
  rolls: Vec<Point>,
}

impl AxleToolCreator {
  pub fn new() -> Self {
    let mut rolls = Vec::new();

    check_cb2();
    panic!();

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

    Self { rolls }
  }

  pub fn get_quality() -> usize {
    1
  }

  pub fn get_size() -> f32 {
    1.0
  }

  pub fn faces(&self) -> usize {
    7
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    2.0
  }

  pub fn get_name(&self, current_normal: usize) -> Option<&str> {
    None
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    if current_normal == 0 {
      2
    } else {
      1
    }
  }

  pub fn get_sticker_index(&self, pos: Point, current_normal: usize) -> PartIndex {
    match current_normal {
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
    _ => {
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
      }
    } 
  }

  pub fn get_part_index(&self, pos: crate::points3d::Point) -> PartIndex {
    return 0;
  }
}
