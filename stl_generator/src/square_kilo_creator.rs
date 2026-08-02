use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use fxhash::*;
use num::Float;

use std::cell::RefCell;
use std::ops::DerefMut;

const PI: f32 = std::f32::consts::PI;

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

#[derive(Debug, Default, Clone)]
struct SpecialEdgePartition {
  axis: Vec<Point>,
  ignore: Vec<usize>,
}

pub struct SquareKiloCreator {
  axis: Vec<Point>,
  special_edges: FxHashMap<PartIndex, SpecialEdgePartition>,
  add_a: FxHashMap<PartIndex, Vec<Point>>,
  add_b: FxHashMap<PartIndex, Vec<Point>>,
  normals: Vec<Point>,
  groove: Vec<f32>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
  n_dists: RefCell<Vec<(f32, usize)>>,
  edge: f32,
  sz: f32,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

impl SquareKiloCreator {
  pub fn new() -> Self {
    let rot_a = PI * 2.0 / 5.0;
    let c_rot = rot_a.cos();
    let edge = c_rot / (1.0 - c_rot);
    let tr_a = (rot_a * 2.0 / 3.0).cos();
    let e_vis = -sqr(tr_a) + (1.0 - sqr(tr_a)) * edge;
    let tr = ((edge - e_vis) / (1.0 - e_vis)).sqrt();
    let min_angle = tr.acos();
    let max_angle = edge.acos();
    let max_edge_angle = (edge / ((edge + 1.0) / 2.0).sqrt()).acos();

    let phi = (5.0.sqrt() - 1.0) / 2.0;

    let mut axis: Vec<_> = [
      Point { x: 0.0, y: -phi, z: -1.0 },
      Point { x: 0.0, y: phi, z: -1.0 },
      Point { x: 0.0, y: -phi, z: 1.0 },
      Point { x: 0.0, y: phi, z: 1.0 },
      Point { x: -phi, y: -1.0, z: 0.0 },
      Point { x: phi, y: -1.0, z: 0.0 },
      Point { x: -phi, y: 1.0, z: 0.0 },
      Point { x: phi, y: 1.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: -phi },
      Point { x: -1.0, y: 0.0, z: phi },
      Point { x: 1.0, y: 0.0, z: -phi },
      Point { x: 1.0, y: 0.0, z: phi },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();

    use rand::seq::SliceRandom;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    axis.shuffle(&mut rng);

    let mut special_edges = FxHashMap::default();
    let mut unavailable = FxHashSet::default();

    for (ia1, &a1) in axis.iter().enumerate() {
      for (ia2, &a2) in axis.iter().enumerate().skip(ia1) {
        if (dot(a1, a2) - edge).abs() > 0.01 {
          continue;
        }
        let mut ignore = Vec::new();
        for (ia3, &a3) in axis.iter().enumerate().skip(ia2) {
          if (dot(a1, a3) - edge).abs() > 0.01 {
            continue;
          }
          if (dot(a2, a3) - edge).abs() > 0.01 {
            continue;
          }
          unavailable.insert((ia1, ia3));
          unavailable.insert((ia2, ia3));
          ignore.push(ia3);
        }
        special_edges.insert((ia1, ia2), ignore);
      }
    }
    special_edges.retain(|i, _| !unavailable.contains(i));

    let mut add_a = FxHashMap::default();
    let mut add_b = FxHashMap::default();

    let special_edges: FxHashMap<_, _> = special_edges
      .into_iter()
      .take(3)
      .map(|(e, ignore)| {
        let index = 1 << e.0 | 1 << e.1;
        use rand::Rng;
        let (ia1, ia2) = if rng.gen_bool(0.5) { (e.0, e.1) } else { (e.1, e.0) };
        let a1 = axis[ia1];
        let a2 = axis[ia2];
        let al = axis.len();
        let axis = vec![
          a1.rotate(a2, -rot_a * 4.0 / 3.0),
          a1.rotate(a2, -rot_a * 2.0 / 3.0),
          a1.rotate(a2, rot_a * 2.0 / 3.0),
          a1.rotate(a2, rot_a * 4.0 / 3.0),
        ];

        let e1 = a1.rotate(a2, rot_a * -1.0 / 3.0);
        let e2 = a1.rotate(a2, rot_a * 1.0 / 3.0);

        add_a.insert(
          1 << ia1 | 1 << ia2 | 1 << (al + 1),
          vec![a2.rotate(e1, -rot_a / 3.0), a2.rotate(e1, rot_a / 3.0)],
        );
        add_a.insert(
          1 << ia1 | 1 << ia2 | 1 << (al + 2),
          vec![a2.rotate(e2, -rot_a / 3.0), a2.rotate(e2, rot_a / 3.0)],
        );

        let oppo = a2.rotate(a1, rot_a * 2.0);
        let oppo = oppo.rotate(a2, -rot_a / 3.0);
        add_b.insert(1 << ia1 | 1 << ia2 | 1 << al | 1 << (al + 1), vec![oppo]);

        let oppo = oppo.rotate(a2, rot_a * 2.0 / 3.0);
        let oppo2 = oppo.rotate(a2, rot_a / 3.0);
        add_b.insert(1 << ia1 | 1 << ia2 | 1 << (al + 1) | 1 << (al + 2), vec![oppo, oppo2]);

        let oppo = a2.rotate(a1, -rot_a * 2.0);
        let oppo = oppo.rotate(a2, rot_a / 3.0);
        add_b.insert(1 << ia1 | 1 << ia2 | 1 << (al + 2) | 1 << (al + 3), vec![oppo]);

        (index, SpecialEdgePartition { axis, ignore })
      })
      .collect();

    for (ia1, &a1) in axis.iter().enumerate() {
      for (ia2, &a2) in axis.iter().enumerate().skip(ia1) {
        if (dot(a1, a2) - edge).abs() > 0.01 || special_edges.contains_key(&(1 << ia1 | 1 << ia2)) {
          continue;
        }
        let v = vec![
          a1.rotate(a2, -rot_a / 3.0),
          a1.rotate(a2, rot_a / 3.0),
          a2.rotate(a1, -rot_a / 3.0),
          a2.rotate(a1, rot_a / 3.0),
        ];
        add_a.insert((1 << ia1) + (1 << ia2), v);
      }
    }

    let normals = axis.clone();

    let sz = 35.0;
    let sphere_r = (4.0 + 3.0 + 2.0) / (max_angle - min_angle);

    println!("angles from {min_angle} to {max_angle} r {sphere_r}");
    println!("edge={edge}, max_egde_angle={max_edge_angle}");

    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());
    let n_dists = RefCell::new(Vec::new());

    let groove = vec![
      (max_angle - 4.0 / (sz - 5.6)).cos(),
      sz - 14.2,
      (max_angle - 8.0 / (sz - 5.6)).cos(),
      sz - 7.8,
      (max_angle - 5.0 / (sz - 5.6)).cos(),
      sz - 5.4,
      (max_angle - 8.0 / (sz - 5.6)).cos(),
      sz - 3.0,
      (max_edge_angle + 2.0 / (sz - 3.2)).cos(),
    ];

    Self {
      axis,
      special_edges,
      normals,
      groove,
      axis_pos,
      axis_neg,
      n_dists,
      add_a,
      add_b,
      edge,
      sz,
    }
  }

  pub fn faces(&self) -> usize {
    //self.normals.len()
    0
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
    320
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();
    if pos.x.abs() > 49.999 || pos.y.abs() > 49.999 || pos.z.abs() > 49.999 {
      return 0;
    }

    if pos.x < 2.999 || pos.y < -5.999 || pos.z < 4.999 {
      //   return 0;
    }

    let sz = self.sz;

    let sphere_r = self.groove[1] - 2.4;

    if r < sphere_r {
      if r > sphere_r - 0.2 {
        return 0;
      }
      for &a in &self.axis {
        if dot(pos, a) > 0.0 && cross(pos, a).len() > 1.2 {
          return 0;
        }
      }

      return 31;
    }

    let mut n_dists = self.n_dists.borrow_mut();
    let n_dists = n_dists.deref_mut();
    n_dists.clear();
    for (i, n) in self.normals.iter().enumerate() {
      let d = sz - dot(pos, *n);
      if d < 0.0 {
        return 0;
      }
      n_dists.push((d, i));
    }
    n_dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let out_r = 2.0;
    if sqr(out_r - f32::min(n_dists[0].0, out_r))
      + sqr(out_r - f32::min(n_dists[1].0, out_r))
      + sqr(out_r - f32::min(n_dists[2].0, out_r))
      > sqr(out_r)
    {
      return 0;
    }

    let mut index: PartIndex = 0;

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);

    let factor = r / (self.groove[self.groove.len() - 2] - 0.2);
    if factor > 1.0 {
      shift_out *= (1.0 + factor) * 0.5;
      shift_in *= (1.0 + factor) * 0.5;
    }

    let mut axis_pos = self.axis_pos.borrow_mut();
    let axis_pos = axis_pos.deref_mut();
    axis_pos.clear();
    let mut axis_neg = self.axis_neg.borrow_mut();
    let axis_neg = axis_neg.deref_mut();
    axis_neg.clear();

    let mut spiral = false;

    enum PositionRelAxis {
      Inside(f32),
      Between,
      Outside(f32),
    };

    let get_pos_rel_axis = |pos: Point, a: Point| {
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

    let mut match_axis = |a, index: &mut PartIndex, i| -> bool {
      match get_pos_rel_axis(pos, a) {
        PositionRelAxis::Inside(check_in) => {
          *index += (1 << i) as PartIndex;
          axis_pos.push((check_in, a));
          return true;
        }
        PositionRelAxis::Outside(check_out) => {
          axis_neg.push((check_out, a));
          return true;
        }
        PositionRelAxis::Between => return false,
      };
    };

    for (i, &a) in self.axis.iter().enumerate() {
      if let Some(special) = self.special_edges.get(&index) {
        if special.ignore.contains(&i) {
          continue;
        }
      }
      if !match_axis(a, &mut index, i) {
        return 0;
      }
    }

    if let Some(special) = self.special_edges.get(&index) {
      for (i, &a) in special.axis.iter().enumerate() {
        if !match_axis(a, &mut index, i + self.axis.len()) {
          return 0;
        }
      }
    }

    let mask = (1 << self.axis.len()) - 1;
    let triangle = (index & mask).count_ones() == 3 || (index & !mask).count_ones() == 2;

    if !triangle {
      if r > self.groove[self.groove.len() - 2] - 0.2 {
        return 0;
      }
    }

    if index.count_ones() == 1 {
      let a = self.axis[index.ilog2() as usize];
      let c = cross(pos, a).len();
      let d = dot(pos, a);
      if d < (self.edge.acos() - shift_out.acos()).cos() * r {
        return 0;
      }

      let hole_r = if r < sphere_r + 4.5 { 1.5 } else { 3.2 };

      if d > sz - 1.0 {
        if d < sz - 0.8 {
          return 0;
        }
        index += (n_dists[0].1 as PartIndex + 1) * 10000;
      } else if d > 0.0 && c < hole_r {
        return 0;
      }
      return index;
    }

    if let Some(add_a) = self.add_a.get(&index) {
      for (i, add_a) in add_a.iter().enumerate() {
        match get_pos_rel_axis(pos, *add_a) {
          PositionRelAxis::Inside(check_in) => {
            axis_pos.push((check_in, *add_a));
          }
          _ => return 0,
        }
      }
    }

    if let Some(add_b) = self.add_b.get(&index) {
      for (i, add_b) in add_b.iter().enumerate() {
        match get_pos_rel_axis(pos, *add_b) {
          PositionRelAxis::Outside(check_out) => {
            axis_pos.push((check_out, *add_b));
          }
          _ => return 0,
        }
      }
    }

    if !triangle {
      return index;
    }

    axis_pos.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    axis_neg.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut in_sr = |a, b, r| {
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

    let mut rr = 0.05f32;

    if axis_pos.len() >= 2 {
      minimal = f32::min(minimal, in_sr(axis_pos[0].0, axis_pos[1].0, rr));
    }
    if axis_neg.len() >= 2 {
      if dot(axis_neg[0].1, axis_neg[1].1) < 0.5 {
        minimal = f32::min(minimal, in_sr(axis_neg[0].0, axis_neg[1].0, rr));
      }
    }
    if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
      if dot(axis_pos[0].1, axis_neg[0].1) > -0.4 {
        minimal = f32::min(minimal, in_sr(axis_pos[0].0, axis_neg[0].0, rr));
      }
    }

    if minimal < 0.0 {
      return 0;
    }

    let mut minn = f32::INFINITY;
    for &n in &self.normals {
      minn = f32::min(minn, cross(pos, n).len());
    }

    if n_dists[0].0 < 1.0 || n_dists[0].0 + n_dists[1].0 < 6.0 {
      if n_dists[0].0 + 0.15 > n_dists[1].0 {
        return 0;
      }
      if n_dists[0].0 < 0.8 || n_dists[0].0 + n_dists[1].0 < 5.7 {
        index += (n_dists[0].1 as PartIndex + 1) * 100000;
      } else {
        return 0;
      }
    }

    return index;
  }
}
