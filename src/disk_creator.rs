use crate::points3d::*;
use crate::solid::*;

pub struct DiskCreator {
  disk_centers: Vec<crate::points2d::Point>,
  min_p: crate::points2d::Point,
  max_p: crate::points2d::Point,
  mag_deltas: Vec<crate::points2d::Point>,
  deltas: Vec<crate::points2d::Point>,
  disk_states_b: Vec<u8>,
  disk_states_e: Vec<u8>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl DiskCreator {
  pub fn new() -> Self {
    let dp1 = crate::points2d::Point::from_angle(0.0);
    let dp2 = crate::points2d::Point::from_angle(std::f32::consts::PI / 3.0);

    let dc1 = dp1.scale(40.0);
    let dc2 = dp2.scale(40.0);

    let disk_centers = vec![
      dc2 - dc1.scale(1.5),
      dc2 - dc1.scale(0.5),
      dc2 + dc1.scale(0.5),
      -dc1.scale(1.5),
      -dc1.scale(0.5),
      dc1.scale(0.5),
      dc1.scale(1.5),
      -dc2 - dc1.scale(0.5),
      -dc2 + dc1.scale(0.5),
      -dc2 + dc1.scale(1.5),
    ];

    let mut min_p = crate::points2d::Point { x: f32::MAX, y: f32::MAX };
    let mut max_p = crate::points2d::Point { x: f32::MIN, y: f32::MIN };
    for &d in &disk_centers {
      min_p.x = f32::min(min_p.x, d.x);
      min_p.y = f32::min(min_p.y, d.y);
      max_p.x = f32::max(max_p.x, d.x);
      max_p.y = f32::max(max_p.y, d.y);
    }

    let dm1 = dp1.scale(16.5);
    let dm2 = dp2.scale(16.5);

    let mag_deltas = vec![dm1, dm1 - dm2, -dm2, -dm1, dm2 - dm1, dm2];
    let deltas = vec![dc1, dc1 - dc2, -dc2, -dc1, dc2 - dc1, dc2];
    let disk_states_b = vec![32, 21, 22, 32, 8, 4, 36, 50, 10, 18];
    let disk_states_e = vec![8, 21, 50, 1, 16, 2, 9, 25, 20, 9];

    Self { disk_centers, min_p, max_p, mag_deltas, deltas, disk_states_b, disk_states_e }
  }

  pub fn faces(&self) -> usize {
    2
  }

  pub fn get_sticker_index(
    &self,
    mut pos: crate::points2d::Point,
    current_normal: usize,
  ) -> PartIndex {
    let disk_states;
    if current_normal == 0 {
      disk_states = &self.disk_states_b;
    } else if current_normal == 1 {
      disk_states = &self.disk_states_e;
      pos.x = -pos.x;
    } else {
      return 0;
    }

    let mut ok = false;
    for (i, &d) in self.disk_centers.iter().enumerate() {
      let d = pos - d;
      let state = disk_states[i];

      if d.len() < 21.5 {
        let mut ok = true;
        for (bit, &dt) in self.deltas.iter().enumerate() {
          if ((1 << bit) & state) == 0 {
            let r = (d - dt).len();
            if r < 24.0 {
              ok = false;
              break;
            }
          }
        }
        if ok {
          return 1;
        }
      }
    }

    return 0;
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    if pos.z < 0.0 {
      if pos.z > -4.0 {
        let sz = f32::max(0.0, (pos.z + 2.0).abs() - 1.5) * 0.5;
        let proj = crate::points2d::Point { x: pos.x, y: pos.y };

        if (proj - crate::points2d::Point { x: 0.0, y: 0.0 }).len() < 2.0 - sz {
          return 7;
        }
      }
    } else if pos.z < 2.0 {
      return 0;
      let proj = crate::points2d::Point { x: pos.x, y: pos.y };
      let mut ok = false;
      for (i, &d) in self.disk_centers.iter().enumerate() {
        let d = proj - d;

        let r = d.len();
        if (i == 4 || i == 5) && r < 46.0 {
          ok = true;
          break;
        }

        if r < 21.0 {
          let mut local_ok = true;
          for &dt in &self.deltas {
            let r = (d - dt).len();
            if r < 22.5 {
              local_ok = false;
              break;
            }
          }

          if local_ok {
            ok = true;
            break;
          }
        }
      }

      if !ok {
        return 0;
      }

      let dc = crate::points2d::Point { x: 12.0, y: 0.0 };
      let mid = |i1: usize, i2: usize| (self.disk_centers[i1] + self.disk_centers[i2]).scale(0.5);

      for d in [
        self.disk_centers[0] - dc,
        self.disk_centers[3] - dc,
        self.disk_centers[7] - dc,
        self.disk_centers[2] + dc,
        self.disk_centers[6] + dc,
        self.disk_centers[9] + dc,
        mid(0, 1),
        mid(1, 2),
        mid(3, 4),
        mid(4, 5),
        mid(5, 6),
        mid(7, 8),
        mid(8, 9),
      ] {
        let d = proj - d;
        let r = d.len();
        if r < 2.1 {
          return 0;
        }
      }

      for (i, &d) in self.disk_centers.iter().enumerate() {
        let d = proj - d;
        let r = d.len();

        if r < 7.2 {
          return 0;
        }

        for &dm in &self.mag_deltas {
          let d = d - dm.perp();
          let r = d.len();
          if (r < 1.6 || r < 2.5 && d.x.abs() < 0.5) && pos.z > 1.0 {
            return 0;
          }
        }
      }
      return 1;
    } else if pos.z < 7.5 {
      return 0;
      let z = pos.z - 2.5;
      let proj = crate::points2d::Point { x: pos.x, y: pos.y };
      let mut ok = false;
      for (i, &d) in self.disk_centers.iter().enumerate() {
        if i != 1 { continue; }
        let d = proj - d;
        let state = self.disk_states_b[i];

        let mut nearest_border = f32::MAX;

        let r = d.len();
        if r < 22.5 {
          nearest_border = f32::min(nearest_border, 22.5 - r);
          let mut ok = true;
          for (bit, &dt) in self.deltas.iter().enumerate() {
            if ((1 << bit) & state) == 0 {
              let r = (d - dt).len();
              if r < 23.5 {
                ok = false;
                break;
              }

              nearest_border = f32::min(nearest_border, r - 23.5);
            }
          }
          if ok {
            let mut cup;
            if r < 13.0 {
              cup = z > 4.0;
            } else if nearest_border > 1.6 {
              cup = z > 1.4;
            } else {
              cup = z > 3.5;
            }

            if cup {
              return i as PartIndex + 22;
            }

            for &dm in &self.mag_deltas {
              let r = (d - dm.perp()).len();
              if r < 1.6 && z > 0.3 || r < 2.55 && z > 1.3 {
                return 0;
              }
            }

            let mut body;
            if nearest_border < 1.5 {
              body = z < 3.4;
            } else if r > 9.0 {
              body = z < 1.3;
            } else {
              body = z < 3.4;
            }

            if body {
              if r > 4.9 && r < 7.1 && (d.x > -0.1 && d.y > -0.1 || d.x < 0.1 && d.y < 0.1) {
                return 0;
              }

              if (r < 2.9 && z > 1.3) || r < 1.35 {
                return 0;
              }

              return i as PartIndex + 2;
            } else {
              return 0;
            }
          }
        }
        break;
      }
    } else if pos.z < 16.5 {
      return 0;    
      let z = pos.z - 7.5;
      let r = (sqr(pos.x) + sqr(pos.y)).sqrt();
      if r < 1.3 || r > 6.9 {
        return 0;
      }
      if z > 2.1 && z < 6.9 {
        return 40;
      }
      if r > 5.1 && (pos.x > 0.1 && pos.y > 0.1 || pos.x < -0.1 && pos.y < -0.1) {
        return 40;
      }

      return 0;
    }

    return 0;
  }
}
