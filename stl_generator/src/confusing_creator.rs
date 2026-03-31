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

pub struct ConfusingCreator {
  axis: Vec<Point>,
  axis1: Vec<Point>,
  axis2: Vec<Point>,
  mid_r: f32,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<f32>>,
  axis_neg: RefCell<Vec<f32>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}
impl ConfusingCreator {
  pub fn new() -> Self {
    let axis: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 1.0, y: 1.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let main_cos = 0.0;
    let main_angle = main_cos.acos();

    let corner_cos = ((main_cos * 2.0 + 1.0) / 3.0).sqrt();
    let corner_angle = corner_cos.acos();

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let in_r = 16.0;

    let mid_r = 6.8;

    let groove = vec![
      (main_angle * 0.5 + 4.0 / (in_r - 4.8)).cos(),
      in_r - 4.6,
      (main_angle * 0.5 + 1.5 / (in_r - 4.8)).cos(),
      in_r - 2.2,
      (corner_angle + 5.0 / in_r).cos(),
      in_r + 0.2,
      (corner_angle + 2.0 / in_r).cos(),
      in_r + 2.6,
      mid_r / (in_r + 2.4),
    ];

    let mut axis1 = Vec::new();
    let mut axis2 = Vec::new();
    for &a in &axis {
      let a1 = a.any_perp();
      let a2 = cross(a, a1);
      axis1.push(a1);
      axis2.push(a2);
    }

    Self { axis, axis1, axis2, mid_r, groove, axis_pos, axis_neg }
  }

  pub fn faces(&self) -> usize {
    0
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, 6)
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
    let n = self.axis[current_normal];
    let n1 = n.any_perp();
    let n2 = cross(n, n1);
    let pos = n.scale(29.9) + n1.scale(pos.x) + n2.scale(pos.y);
    let result = self.get_part_index_impl(pos, current_normal);
    (result > 0) as PartIndex
  }

  pub fn get_quality() -> usize {
    // 384 generates edge used twice
    120
  }

  pub fn get_size() -> f32 {
    90.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if pos.x.abs() > 44.999 || pos.y.abs() > 44.999 || pos.z.abs() > 44.999 {
      return 0;
    }
    let r = pos.len();

    let sphere_r = self.groove[1] - 2.2;

    //if pos.y < 0.0 { return 0; }

    if r < sphere_r {
      //return 0; // tmp
      if r > sphere_r - 0.3 {
        return 0;
      }
      for &a in &self.axis {
        let s = cross(pos, a).len();
        let d = dot(pos, a);
        if s < 1.2 && d > 0.0 {
          return 0;
        }
      }
      return 63;
    }

    let mut depth_to_face = f32::INFINITY;
    let sticker_gap = 0.5;

    let mr = self.mid_r;

    let nx = f32::min(mr * 3.0 - pos.x, mr * 5.0 + pos.x);
    let ny = f32::min(mr * 3.0 - pos.y, mr * 5.0 + pos.y);
    let nz = f32::min(mr * 3.0 - pos.z, mr * 5.0 + pos.z);

    let orr = 2.0;
    let fd = orr
      - (sqr(f32::max(orr - nx, 0.0))
        + sqr(f32::max(orr - ny, 0.0))
        + sqr(f32::max(orr - nz, 0.0)))
      .sqrt();
    if fd < 0.0 {
      return 0;
    }

    let mut index: PartIndex = 0;
    let shaft_r = f32::min(1.5, self.groove[1] - 0.9 - r);

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);
    let last_groove_r = self.groove[self.groove.len() - 2];
    /* if shaft_r > 0.0 {
      shift_out += shaft_r * 0.05;
      shift_in += shaft_r * 0.05;
    }*/

    let extra = r - (last_groove_r - 0.2);

    if extra >= 0.0 {
      shift_out = f32::max(shift_out - extra * 0.03, mr / r);
      shift_in = f32::max(shift_in - extra * 0.03, mr / r);
    }

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    axis_pos.clear();
    axis_neg.clear();

    for (i, &a) in self.axis.iter().enumerate().take(4) {
      let c = dot(pos, a);

      let check_in = c - shift_in * r;
      if check_in > 0.0 {
        index += (1 << i);
        axis_pos.push(check_in);
      } else {
        let check_out = shift_out * r - c;
        if check_out > 0.0 {
          axis_neg.push(check_out);
        } else {
          return 0;
        }
      }
    }

    if index == 0 {
      index = 256;
    }

    let rr;

    rr = 2.0 * f32::min(1.0, 1.0 * r / (mr * 3.0 - 2.0));

    if current_normal < 6 {
      for p in axis_pos.iter_mut() {
        if *p < sticker_gap {
          return 0;
        }
        *p -= 0.5;
      }
      for n in axis_neg.iter_mut() {
        if *n < sticker_gap {
          return 0;
        }
        *n -= 0.5;
      }
    }

    let rr = if current_normal < 6 { rr - sticker_gap } else { rr };

    let mut in_sr = |a, b| -> f32 {
      if a < 0.0 {
        return 0.0;
      }
      if b < 0.0 {
        return 0.0;
      }
      if a < rr && b < rr {
        return rr - (sqr(rr - a) + sqr(rr - b)).sqrt();
      }
      return f32::min(a, b);
    };

    axis_pos.sort_by(|a, b| a.partial_cmp(b).unwrap());
    axis_neg.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut cd = f32::INFINITY;
    if axis_pos.len() >= 1 {
      cd = f32::min(cd, axis_pos[0]);
    }
    if axis_neg.len() >= 1 {
      cd = f32::min(cd, axis_neg[0]);
    }

    if axis_pos.len() >= 2 {
      cd = f32::min(cd, in_sr(axis_pos[0], axis_pos[1]));
      if cd < 0.0 {
        return 0;
      }
    }
    if axis_neg.len() >= 2 {
      cd = f32::min(cd, in_sr(axis_neg[0], axis_neg[1]));
      if cd < 0.0 {
        return 0;
      }
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
      cd = f32::min(cd, in_sr(axis_pos[0], axis_neg[0]));
      if cd < 0.0 {
        return 0;
      }
    }

    if fd < orr && cd < orr && sqr(orr - fd) + sqr(orr - cd) > sqr(orr) {
      return 0;
    }

    let scenario = 0;
    let mut x_tile = 2000;
    let mut y_tile = 3000;
    let mut z_tile = 1000;
    let mut diag_tol = 0.08;
    if scenario == 1 {
      (x_tile, y_tile, z_tile) = (5000, 5000, 5000);
      diag_tol = -0.08;
    } else if scenario == 2 {
      (x_tile, y_tile, z_tile) = (6000, 6000, 6000);
      diag_tol = -0.08;
    }

    let border_full = 1.8;
    let border_thin = 1.3;
    let border_to_diag = border_full + 0.1;

    if nz < nx && nz < ny {
      if nz < 2.0 && (nz + diag_tol > nx || nz + diag_tol > ny) {
        return 0;
      }
      let t1 = f32::min(nx - nz, ny - nz);
      let t2 = nz;
      let border = if cd < 1.0 { border_full } else { border_thin };
      let t3 = f32::min(8.0 - (nx - 13.0).abs(), 8.0 - (ny - 13.0).abs());
      let mut t3s = t3;
      if scenario == 1 && t2 > border_full {
        t3s = 0.0
      } else if scenario == 2 {
        let bd = t2 - border_to_diag;
        if bd > 0.0 {
          t3s = f32::min(8.0 - (nx - bd - 13.0).abs(), 8.0 - (ny - bd - 13.0).abs());
        }
      }

      if t2 < border || t3 > 0.12 && t3s > 0.12 && t2 < 3.7 {
        index += z_tile;
      } else if t2 < 2.0 || t3 > 0.0 && t2 < 4.0 {
        return 0;
      }
    } else {
      let check_tile = |a: f32, b: f32| {
        if a < 2.0 && (a + diag_tol > b || a + diag_tol > nz) {
          return 0;
        }

        let t1 = f32::min(nz - a, b - a);
        let t2 = a;
        let get_t3 = |z: f32, b: f32| {
          f32::min(
            f32::max(
              f32::max(2.5 - (z - (mr * 3.0 - 7.5)).abs(), 3.0 - (z).abs()),
              8.0 - (z - (-mr * 5.0 + 13.0)).abs(),
            ),
            7.0 - (b - 16.0).abs(),
          )
        };

        let border = if cd < 1.0 { border_full } else { border_thin };

        let t3 = get_t3(pos.z, b);
        let mut t3s = t3;

        let bd = t2 - border_to_diag;
        if bd > 0.0 && scenario > 0 {
          let zs = if scenario == 1 {
            pos.z
          } else if pos.z < -mr {
            pos.z - bd
          } else if pos.z < mr {
            pos.z
          } else {
            pos.z + bd
          };
          t3s = get_t3(zs, b - bd);
        }

        if t2 < border || t3 > 0.12 && t3s > 0.12 && t2 < 3.7 {
          return 2;
        } else if t2 < 2.0 || t3 > 0.0 && t2 < 4.0 {
          return 0;
        }
        return 1;
      };

      let (ct, d) =
        if nx < ny { (check_tile(nx, ny), x_tile) } else { (check_tile(ny, nx), y_tile) };
      if ct == 0 {
        return 0;
      } else if ct == 2 {
        index += d;
      }
    }

    if index < 1000  {
   //   return 0; //tmp
    }

    let cupd = 1.0;
    if (index < 1000 && index.count_ones() == 1 || index % 1000 == 2)
      && nx > cupd
      && ny > cupd
      && nz > cupd
    {
      if depth_to_face > 0.5 {
        let hole_r = if r < sphere_r + 2.0 { 1.5 } else { 3.2 };
        for (i, &a) in self.axis.iter().enumerate() {
          let s = cross(pos, a).len();
          let d = dot(pos, a);
          if d > 0.0 && s < hole_r {
            return 0;
          }
        }
      }
    }

    return index;
  }
}
