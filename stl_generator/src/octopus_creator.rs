use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct OctopusCreator {
  axis: Vec<Point>,
  bounds: Vec<Option<Point>>,
  normals: Vec<Point>,
  n_basis: Vec<(Point, Point)>,

  cone_angle: f32,
  cone_angle_core: f32,
  screw_diam: f32,
  head_diam: f32,
  thread_diam: f32,

  mirr1_24: Point,
  mirr3_46: Point,
  mirr5_60: Point,
  mirr7_02: Point,

  axis_pos: RefCell<Vec<NearAxis>>,
  axis_neg: RefCell<Vec<NearAxis>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

pub fn find_face(a1: Point, a2: Point, a3: Point) -> Point {
  let det = dot(a1, cross(a2, a3)).recip();
  let x;
  let y;
  let z;

  {
    let mut a1 = a1;
    a1.x = 1.0;
    let mut a2 = a2;
    a2.x = 1.0;
    let mut a3 = a3;
    a3.x = 1.0;
    x = dot(a1, cross(a2, a3)) * det;
  }

  {
    let mut a1 = a1;
    a1.y = 1.0;
    let mut a2 = a2;
    a2.y = 1.0;
    let mut a3 = a3;
    a3.y = 1.0;
    y = dot(a1, cross(a2, a3)) * det;
  }

  {
    let mut a1 = a1;
    a1.z = 1.0;
    let mut a2 = a2;
    a2.z = 1.0;
    let mut a3 = a3;
    a3.z = 1.0;
    z = dot(a1, cross(a2, a3)) * det;
  }

  Point { x, y, z }
}

impl OctopusCreator {
  pub fn new() -> Self {
    let sqrt3 = 3.0f32.sqrt();
    let sqrt5 = 5.0f32.sqrt();
    let sqrt2 = 2.0f32.sqrt();

    let u = ((7.0 - 4.0 * sqrt2) / 17.0).sqrt(); // 0.28108463771482025
    let v = ((5.0 + 2.0 * sqrt2) / 17.0).sqrt(); // 0.67859834454584703

    println!("u={u}, v={v}");

    let axis = vec![
      Point { x: v, y: v, z: -u },
      Point { x: v, y: -u, z: -v },
      Point { x: v, y: -v, z: u },
      Point { x: -u, y: -v, z: v },
      Point { x: -v, y: -v, z: -u },
      Point { x: -v, y: u, z: -v },
      Point { x: -v, y: v, z: u },
      Point { x: u, y: v, z: v },
    ];

    fn refl(what: Point, c: Point) -> Point {
      c.scale(dot(c, what) * 2.0) - what
    }

    let bounds = vec![
      Some(refl((axis[1] + axis[7]).norm(), axis[0])),
      None,
      Some(refl((axis[3] + axis[1]).norm(), axis[2])),
      None,
      Some(refl((axis[5] + axis[3]).norm(), axis[4])),
      None,
      Some(refl((axis[7] + axis[5]).norm(), axis[6])),
      None,
    ];

    let mirr1_24 = Point { x: -v, y: -u, z: v };
    let mirr3_46 = Point { x: -u, y: v, z: -v };
    let mirr5_60 = Point { x: v, y: u, z: v };
    let mirr7_02 = Point { x: u, y: -v, z: -v };

    let normals = vec![
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
    ];

    let n_basis = normals
      .iter()
      .map(|&n| {
        let n1 = n.any_perp().norm();
        let n2 = cross(n, n1).norm();
        (n1, n2)
      })
      .collect();

    let cone_angle = 0.855;
    let cone_angle_core = 0.735;
    let screw_diam = 3.0;
    let head_diam = 6.4;
    let thread_diam = 2.5;

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    Self {
      axis,
      bounds,
      normals,
      n_basis,
      cone_angle,
      cone_angle_core,
      screw_diam,
      head_diam,
      thread_diam,
      mirr1_24,
      mirr3_46,
      mirr5_60,
      mirr7_02,
      axis_pos,
      axis_neg,
    }
  }

  pub fn faces(&self) -> usize {
    self.normals.len() + 4 - 4
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.normals.len())
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    let n = self.normals[current_normal];
    let (n1, n2) = self.n_basis[current_normal];

    let pos = n.scale(35.0 / n.sqr_len()) + n1.scale(pos.x) + n2.scale(pos.y);
    let result = self.get_part_index_impl(pos, current_normal);

    (result > 0) as PartIndex
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 64.999 || pos.y.abs() > 64.999 || pos.z.abs() > 64.999 {
      return 0;
    }

    let sticker = current_normal < self.normals.len();

    let mut min_wall = f32::MAX;

    let mut wall = false;
    let mut cup = false;
    let mut core = true;
    for i in 0..self.normals.len() {
      let n = self.normals[i];
      let d = dot(n, pos);
      let center_dist = if sticker { 33.999 } else { 34.999 };
      if i != current_normal && d > center_dist {
        return 0;
      }

      if i != current_normal {
        min_wall = f32::min(min_wall, center_dist - d);
      }

      if d > 33.5 {
        wall = true;
        cup = true;
      } else if d > 33.3 {
        core = false;
      }
    }

    if r <= 15.3 {
      for a in &self.axis {
        if dot(pos, *a) > 0.0 && cross(pos, *a).sqr_len() < sqr(self.thread_diam * 0.5) {
          return 0;
        }
      }

      let mut index: PartIndex = 1024;

      let s = -0.2f32;
      let c = (1.0 - sqr(s)).sqrt();

      let pos = Point { x: pos.x * c - pos.y * s, y: pos.x * s + pos.y * c, z: pos.z };

      for v in [(8.0, 0.0, 2), (-11.0, 0.0, 4)] {
        if (pos.z - v.0).abs() < 2.0 && (pos.y - v.1).abs() < 2.0 {
          return index + v.2;
        } else if (pos.z - v.0).abs() < 2.2 && (pos.y - v.1).abs() < 2.2 {
          return 0;
        }
      }

      if pos.x.abs() < 0.1 {
        return 0;
      }
      if pos.x > 0.0 {
        index += 1;
      }

      return index;
    }

    let mut index: PartIndex = 0;
    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();
    axis_pos.clear();
    axis_neg.clear();

    let mut sharp_in = false;
    let mut sharp_out = false;

    let mut min_cut = f32::MAX;
    macro_rules! match_axis {
      ($pos: expr, $a: expr, $cone_angle: expr, $cone_angle_core: expr, $id: expr) => {
        let pos = $pos;
        let a = $a;
        let mut cone_angle = $cone_angle;
        let cone_angle_core = $cone_angle_core;
        let id = $id;

        let mut cone_angle_in = cone_angle;
        if r > 33.70 {
        } else if r > 33.30 {
          cone_angle_in -= (33.70 - r) * 0.01;
        } else if r > 32.90 {
          cone_angle_in -= 0.050;
          sharp_in = true;
        } else if r > 30.90 {
          cone_angle_in -= 0.050;
          cone_angle -= 0.050;
          sharp_in = true;
          sharp_out = true;
        } else if r > 30.50 {
          cone_angle_in -= 0.050;
          sharp_in = true;
        } else if r > 30.10 {
          cone_angle_in -= (r - 30.10) * 0.011;
        } else if r > 28.10 {
        } else if r > 27.70 {
          cone_angle_in -= (28.10 - r) * 0.02;
        } else if r > 27.30 {
          cone_angle_in = cone_angle_core;
          sharp_out = true;
        } else if r > 25.30 {
          cone_angle_in = cone_angle_core;
          cone_angle = cone_angle_core;
        } else if r > 24.90 {
          cone_angle_in = cone_angle_core - 0.110;
          cone_angle = cone_angle_core;
          sharp_out = true;
        } else if r > 24.50 {
          cone_angle_in = cone_angle_core - 0.110;
          cone_angle = cone_angle_core - 0.110 + (r - 24.50) * 0.025;
          sharp_in = true;
          sharp_out = true;
        } else if r > 22.50 {
          cone_angle_in = cone_angle_core - 0.110;
          cone_angle = cone_angle_core - 0.110;
          sharp_in = true;
          sharp_out = true;
        } else if r > 22.10 {
          cone_angle_in = cone_angle_core - 0.110;
          cone_angle = cone_angle_core - 0.110 + (22.50 - r) * 0.025;
          sharp_in = true;
          sharp_out = true;
        } else if r > 21.70 {
          cone_angle_in = cone_angle_core - 0.110;
          cone_angle = cone_angle_core;
          sharp_out = true;
        } else {
          cone_angle_in = cone_angle_core;
          cone_angle = cone_angle_core;
        }

        let mut curvyness = if r > 27.7 { 32.0 } else { 16.0 };
        if sticker {
          cone_angle += 0.01;
          cone_angle_in -= 0.01;
          curvyness = 36.0;
        }

        let p1 = a.any_perp().norm();
        let p2 = cross(a, p1);
        let spiral_a = f32::atan2(dot(pos, p1), dot(pos, p2)) / std::f32::consts::PI;

        if dot(pos, a) > 0.0 {
          let sin = cross(pos, a).len() / r;
          if sin < cone_angle_in {
            let d = 1.0 - (cone_angle_in - sin) * curvyness;
            if d > 0.0 {
              axis_pos.push(NearAxis { dist: d, pos: a });
            }

            min_cut = f32::min(min_cut, cone_angle_in - sin);
            let in_spiral = r * 0.2 - spiral_a * 15.0;
            let in_spiral = in_spiral - in_spiral.floor();
            let in_spiral = f32::max(in_spiral * (0.2 - in_spiral) * 0.6, 0.0);
            if sharp_in || wall || sin < cone_angle_in - in_spiral {
              index += 1 << id;
            } else {
              return 0;
            }
          } else if sin > cone_angle {
            let d = 1.0 - (sin - cone_angle) * curvyness;
            if d > 0.0 {
              axis_neg.push(NearAxis { dist: d, pos: a });
            }

            min_cut = f32::min(min_cut, sin - cone_angle);
            let in_spiral = r * 0.2 + spiral_a * 15.0;
            let in_spiral = in_spiral - in_spiral.floor();
            let in_spiral = f32::max(in_spiral * (0.2 - in_spiral) * 0.6, 0.0);

            if sharp_out || wall || sin > cone_angle_in + in_spiral {
              // nothing
            } else {
              return 0;
            }
          } else {
            return 0;
          }
        }
      };
    }

    let ei = self.axis.len();
    let cac = self.cone_angle_core;
    let ca = self.cone_angle;

    for i in 0..self.axis.len() {
      match_axis!(pos, self.axis[i], ca, cac, i);
    }

    if index == (1 << 2 | 1 << 3 | 1 << 4) {
      match_axis!(pos, self.mirr1_24, ca, cac, ei);
    }
    if index == (1 << 4 | 1 << 5 | 1 << 6) {
      match_axis!(pos, self.mirr3_46, ca, cac, ei);
    }
    if index == (1 << 6 | 1 << 7 | 1 << 0) {
      match_axis!(pos, self.mirr5_60, ca, cac, ei);
    }
    if index == (1 << 0 | 1 << 1 | 1 << 2) {
      match_axis!(pos, self.mirr7_02, ca, cac, ei);
    }

    if sticker && min_cut > 0.03 && min_wall > 3.0 {
      return 0;
    }

    if index.count_ones() == 1 {
      let aindex = index.ilog2() as usize;
      if r > 27.5 && !sharp_out {
        if let Some(bound) = self.bounds[aindex] {
          if cross(pos, bound).len() / r > 0.7 {
            return 0;
          }
        }
      }

      if r > 27.50 {
        index += 1 << self.axis.len();
      } else if r > 27.30 {
        return 0;
      } else {
        let a = self.axis[aindex];
        let diam = if r > 17.3 {
          self.head_diam
        } else {
          self.screw_diam
        };
        if dot(pos, a) > 0.0 && cross(pos, a).sqr_len() < sqr(diam * 0.5) {
          return 0;
        }
      }
    }

    let validate = |ap: &[f32], k: f32| -> bool {
      let d1 = f32::max(0.0, 1.0 - k * (1.0 - ap[0]));
      let d2 = f32::max(0.0, 1.0 - k * (1.0 - ap[1]));
      return sqr(d1) + sqr(d2) < 1.0;
    };

    if axis_pos.len() == 2
      && (r < 27.7 || !sharp_in)
      && !validate(&[axis_pos[0].dist, axis_pos[1].dist], 1.0)
    {
      return 0;
    }

    if axis_neg.len() == 2 && !validate(&[axis_neg[0].dist, axis_neg[1].dist], 1.0) {
      return 0;
    }

    if axis_pos.len() == 1 && axis_neg.len() == 1 && sharp_in == sharp_out {
      let dot_barrier = if r > 27.5 { -0.2 } else { 0.0 };
      if dot(axis_pos[0].pos, axis_neg[0].pos) > dot_barrier
        && !validate(&[axis_pos[0].dist, axis_neg[0].dist], 1.0)
      {
        return 0;
      }
    }

    return index;
  }
}
