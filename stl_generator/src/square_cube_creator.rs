use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::*;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct SquareCubeCreator {
  axis: Vec<Point>,
  axis1: Vec<Point>,
  axis2: Vec<Point>,
  add_a: FxHashMap<PartIndex, Vec<Point>>,
  add_b: FxHashMap<PartIndex, (Vec<Point>, Point)>,
  normals: Vec<Point>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
  trenchs: Vec<(f32, f32)>,
  pin_factor: f32,
  sz: f32,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl SquareCubeCreator {
  pub fn new() -> Self {
    let axis: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
    ]
    .into_iter()
    .map(|a| a.rotate(Point::Z, PI / 12.0))
    .collect();

    let mut add_a = FxHashMap::default();
    for ia1 in 0..axis.len() {
      let a1 = axis[ia1];
      for ia2 in ia1 + 1..axis.len() {
        let a2 = axis[ia2];
        if dot(a1, a2).abs() > 0.01 {
          continue;
        }
        let v = vec![
          a1.rotate(a2, -PI * 2.0 / 6.0),
          a1.rotate(a2, PI * 2.0 / 6.0),
          a2.rotate(a1, -PI * 2.0 / 6.0),
          a2.rotate(a1, PI * 2.0 / 6.0),
        ];
        add_a.insert((1 << ia1) + (1 << ia2), v);
      }
    }

    let mut add_b = FxHashMap::default();
    for ia1 in 0..axis.len() {
      let a1 = axis[ia1];
      if a1.z.abs() < 0.99 {
        continue;
      }
      for ia2 in 0..axis.len() {
        let a2 = axis[ia2];
        if a2.z.abs() > 0.99 {
          continue;
        }
        for ia3 in ia2 + 1..axis.len() {
          let a3 = axis[ia3];
          if a3.z.abs() > 0.99 {
            continue;
          }
          if dot(a2, a3).abs() > 0.01 {
            continue;
          }
          let vol = dot(a1, cross(a2, a3));
          let (v, mid) = if vol > 0.0 {
            if a1.z > 0.0 {
              (
                vec![a2.rotate(a1, -PI * 1.0 / 6.0), a2.rotate(a1, PI * 5.0 / 6.0)],
                a2.rotate(a1, PI * 2.0 / 6.0),
              )
            } else {
              (
                vec![a3.rotate(a1, PI * 1.0 / 6.0), a3.rotate(a1, -PI * 5.0 / 6.0)],
                a3.rotate(a1, -PI * 2.0 / 6.0),
              )
            }
          } else {
            if a1.z > 0.0 {
              (
                vec![a3.rotate(a1, -PI * 1.0 / 6.0), a3.rotate(a1, PI * 5.0 / 6.0)],
                a3.rotate(a1, PI * 2.0 / 6.0),
              )
            } else {
              (
                vec![a2.rotate(a1, PI * 1.0 / 6.0), a2.rotate(a1, -PI * 5.0 / 6.0)],
                a2.rotate(a1, PI * -2.0 / 6.0),
              )
            }
          };
          let av = vec![
            mid.rotate(a1, -PI * 2.0 / 6.0),
            mid.rotate(a1, PI * 2.0 / 6.0),
            a1.rotate(mid, -PI * 2.0 / 6.0),
            a1.rotate(mid, PI * 2.0 / 6.0),
          ];
          add_b.insert((1 << ia1) + (1 << ia2) + (1 << ia3), (v, mid));
          add_a.insert((1 << ia1) + (1 << ia2) + (1 << ia3), av);
        }
      }
    }

    let normals: Vec<_> = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    let cac = -sqr((PI * 5.0 / 12.0).cos());
    let min_angle = (-cac / (1.0 - cac)).sqrt().acos();
    let min_angle_core = ((PI / 4.0).cos() * (PI / 3.0).cos()).acos();
    let max_angle = PI * 0.5;

    let sz = 73.0 * 0.5;
    let sphere_r = (1.25 + 1.5 + 5.1) / (max_angle - min_angle);

    println!("angles from {min_angle} to {max_angle} r {sphere_r}");

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let groove = vec![
      (max_angle - 6.0 / (sz - 9.2)).cos(),
      sz - 13.0,
      (max_angle - 2.25 / (sz - 9.2)).cos(),
      sz - 9.0,
      (min_angle_core + 2.0 / (sz - 9.2)).cos(),
      sz - 7.0,
      (max_angle - 2.25 / (sz - 7.2)).cos(),
      sz - 5.0,
      (max_angle - 4.25 / (sz - 7.2)).cos(),
      sz - 3.0,
      (max_angle - 2.25 / (sz - 7.2)).cos(),
    ];

    let mut axis1 = Vec::new();
    let mut axis2 = Vec::new();
    for &a in &axis {
      let a1 = a.any_perp();
      let a2 = cross(a, a1);
      axis1.push(a1);
      axis2.push(a2);
    }

    let a0 = axis[0];
    let a1 = axis[2];
    let a2 = a0.rotate(a1, PI / 6.0);
    let a3 = a0.rotate(a1, PI / 3.0);

    let mut edges = Vec::new();
    edges.push((a0, a1));
    edges.push((a2, a1));
    edges.push((a3, a1));
    edges.push((a1.rotate(a2, PI / 6.0), a2));
    edges.push((a1.rotate(a2, PI / 3.0), a2));
    edges.push((a1.rotate(a3, PI / 6.0), a3));
    edges.push((a1.rotate(a3, PI / 3.0), a3));
    let mut trenchs = Vec::new();

    let pin_factor = 0.35;
    for e in edges {
      let da = 1.45 / (sz - 11.2);

      let p = (e.0 + (e.1 - e.0).scale(pin_factor)).norm();
      let a = dot(p, a0).acos();
      trenchs.push(((a + da).cos(), (a - da).cos()));

      let p = (e.1 + (e.0 - e.1).scale(pin_factor)).norm();
      let a = dot(p, a0).acos();
      trenchs.push(((a + da).cos(), (a - da).cos()));
    }

    Self {
      axis,
      axis1,
      axis2,
      normals,
      groove,
      axis_pos,
      axis_neg,
      add_a,
      add_b,
      pin_factor,
      sz,
      trenchs,
    }
  }

  pub fn faces(&self) -> usize {
    //self.normals.len()
    2
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.faces())
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
    if current_normal == 0 {
      return self.get_part_index(Point { x: pos.x, y: pos.y, z: 0.0 });
    }
    if current_normal == 1 {
      return self.get_part_index(Point { x: pos.x, y: 0.0, z: pos.y });
    }
    0
  }

  pub fn get_quality() -> usize {
    512
  }

  pub fn get_size() -> f32 {
    150.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 49.999 || pos.y.abs() > 49.999 || pos.z.abs() > 49.999 {
      return 0;
    }

    let sz = self.sz;

    let sphere_or = self.groove[3] - 2.2;
    let mut sphere_r = sphere_or;
    let sphere_ir = self.groove[3] - 8.4;

    if r < sphere_r {
      if r > sphere_r - 4.0 {
        for &a in &self.axis {
          let c = dot(pos, a);
          for trench in &self.trenchs {
            if c > trench.0 * r && c < trench.1 * r {
              sphere_r = self.groove[3] - 4.2;
            }
          }
        }

        for &a in &self.axis {
          let c = dot(pos, a);
          let s = cross(pos, a).len();
          if c > 0.0 && s < 6.4 {
            sphere_r = self.groove[3] - 6.2;
          }
        }
      }

      if r < sphere_r {
        if r < sphere_ir {
          return 0;
        }
        let mut frame = false;
        for &a in &self.axis {
          let c = dot(pos, a);
          let s = cross(pos, a).len();
          if c > 0.0 && s < 1.5 {
            return 0;
          }
          if c.abs() < 5.0 || s < 5.0 {
            frame = true;
          }
        }
        if !frame {
          return 0;
        }
        return 31;
      }
    }

    //return 0;

    let mut dists = [(f32::INFINITY, 0); 6];
    for (i, n) in self.normals.iter().enumerate() {
      let d = sz - dot(pos, *n);
      if d < 0.0 {
        return 0;
      }
      dists[i] = (d, i);
    }
    dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let out_r = 4.0;
    if sqr(out_r - f32::min(dists[0].0, out_r))
      + sqr(out_r - f32::min(dists[1].0, out_r))
      + sqr(out_r - f32::min(dists[2].0, out_r))
      > sqr(out_r)
    {
      return 0;
    }

    let mut index: PartIndex = 0;

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);

    let factor = (self.groove[3] - 0.2) / r;
    if factor > 1.0 {
      shift_out *= factor;
      shift_in *= factor;
    }
    let factor = (self.groove[5] - 0.2) / r;
    if factor < 1.0 {
      shift_out *= factor;
      shift_in *= factor;
    }

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();

    axis_pos.clear();
    axis_neg.clear();

    let mut spiral = false;

    enum PositionRelAxis {
      Inside(f32),
      Between,
      Outside(f32),
    };

    let get_pos_rel_axis = |pos: Point, a: Point, i: usize| {
      let c = dot(pos, a) / r;
      let s = cross(pos, a).len();
      let check_in = c - shift_in;
      if check_in > 0.0 {
        return PositionRelAxis::Inside(check_in);
      } else {
        let check_out = shift_out - c;
        if check_out > 0.0 {
          return PositionRelAxis::Outside(check_out);
        } else {
          return PositionRelAxis::Between;
        }
      }
    };

    let mut match_axis =
      |index: &mut PartIndex, pos: Point, a: Point, i: usize| match get_pos_rel_axis(pos, a, i) {
        PositionRelAxis::Inside(check_in) => {
          *index |= (1 << i);
          axis_pos.push((check_in, a));
          1
        }
        PositionRelAxis::Outside(check_out) => {
          axis_neg.push((check_out, a));
          -1
        }
        PositionRelAxis::Between => 0,
      };

    for (i, &a) in self.axis.iter().enumerate() {
      if match_axis(&mut index, pos, self.axis[i], i) == 0 {
        return 0;
      }
    }

    let hole_r = if r < sphere_r { 1.5 } else { 1.2 };

    if index.count_ones() == 1 {
      let a = self.axis[index.ilog2() as usize];
      let c = cross(pos, a).len();
      let d = dot(pos, a);
      if c > shift_out * r {
        return 0;
      }

      if d > sz || r < self.groove[1] - 1.8 {
        return 0;
      }
      if d > sz - 1.0 {
        if d < sz - 0.8 {
          return 0;
        }
        index += (dists[0].1 as PartIndex + 1) * 10000;
      } else if d > 0.0 && c < hole_r {
        return 0;
      }
      return index;
    }

    if let Some((add_b, _)) = self.add_b.get(&index) {
      for i in 0..add_b.len() {
        if match_axis(&mut index, pos, add_b[i], i + 6) == 0 {
          return 0;
        }
      }
    }

    if let Some(add_a) = self.add_a.get(&index) {
      let mut dev_null = 0;
      for i in 0..add_a.len() {
        if match_axis(&mut dev_null, pos, add_a[i], i + 6) != 1 {
          return 0;
        }
      }
    }

    let check_pins = |p1: Point, p2: Point| {
      let mc = 1.0 * r / sphere_or;
      cross(pos, (p1 + (p2 - p1).scale(self.pin_factor)).norm()).len() < mc
        || cross(pos, (p2 + (p1 - p2).scale(self.pin_factor)).norm()).len() < mc
    };

    if r < self.groove[3] - 0.8 && r > self.groove[3] - 3.8 {
      if let Some((_, mid)) = self.add_b.get(&index) {
        if check_pins(axis_pos[0].1, *mid) {
          return index;
        }
      }
      if index.count_ones() == 2 {
        if check_pins(axis_pos[0].1, axis_pos[1].1) {
          return index;
        }
      }
    }

    if r < self.groove[3] - 1.8 || index.count_ones() > 3 && r < self.groove[5] + 0.2 {
      return 0;
    }

    axis_pos.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    axis_neg.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut in_sr = |a, b, d| {
      let r = 0.096 * d;
      if a < r && b < r {
        return r - (sqr(r - a) + sqr(r - b)).sqrt();
      }
      return f32::INFINITY;
    };

    let mut minimal = axis_pos
      .iter()
      .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
      .map(|a| a.0)
      .unwrap_or(f32::INFINITY);
    minimal = f32::min(
      minimal,
      axis_neg
        .iter()
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .map(|a| a.0)
        .unwrap_or(f32::INFINITY),
    );

    let mut rr = 1.0f32;

    if index.count_ones() == 2 || index.count_ones() == 3 {
      if r > self.groove[9] + 0.2 || r < self.groove[7] - 0.2 {
        rr = 0.1f32;
      } else {
        rr = 0.2f32;
      }
    }

    if axis_pos.len() >= 2 {
      minimal = f32::min(minimal, in_sr(axis_pos[0].0, axis_pos[1].0, rr));
    }
    if axis_neg.len() >= 2 {
      minimal = f32::min(minimal, in_sr(axis_neg[0].0, axis_neg[1].0, rr));
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
      minimal = f32::min(minimal, in_sr(axis_pos[0].0, axis_neg[0].0, rr));
    }

    if minimal < 0.0 {
      return 0;
    }

    let mut minn = f32::INFINITY;
    for &n in &self.normals {
      minn = f32::min(minn, cross(pos, n).len());
    }

    if dists[0].0 < 1.0 || dists[0].0 + dists[1].0 < 6.0 {
      if dists[0].0 + 0.15 > dists[1].0 {
        return 0;
      }
      if dists[0].0 < 0.8 || dists[0].0 + dists[1].0 < 5.7 {
        index += (dists[0].1 as PartIndex + 1) * 10000;
      } else {
        return 0;
      }
    }

    return index;
  }
}
