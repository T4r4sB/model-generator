use crate::points2d::*;
use dxf::entities::*;
use dxf::objects::*;
use dxf::Drawing;
use fxhash::FxHashMap;
use std::collections::HashMap;

pub type PartIndex = u32;
const BAD_INDEX: PartIndex = 0xFFFFFFFF;

#[derive(Debug, Clone)]
pub struct Contour {
  pub points: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct ConnectedPart {
  pub contours: Vec<Contour>,
}

#[derive(Debug, Clone)]
pub struct FragmentedParts {
  pub contours: Vec<Contour>,
}

#[derive(Debug, Clone)]
pub struct ContourSet {
  pub points: Vec<Point>,
  pub parts: Vec<ConnectedPart>,
}

impl Contour {
  pub fn new() -> Self {
    Self { points: Vec::new() }
  }

  pub fn get(&self, points: &[Point], i: usize) -> Point {
    points[self.points[i] as usize]
  }

  pub fn get_square(&self, points: &[Point]) -> f32 {
    if self.points.len() < 3 {
      return 0.0;
    }

    let mut result = 0.0;
    let last = self.get(points, self.points.len() - 1);
    let mut prev = self.get(points, 0) - last;

    for i in 1..self.points.len() - 1 {
      let cur = self.get(points, i) - last;
      result += cross(prev, cur);
      prev = cur;
    }
    result * 0.5
  }

  pub fn get_length(&self, points: &[Point]) -> f32 {
    if self.points.len() < 2 {
      return 0.0;
    }

    let mut result = 0.0;
    let mut prev = self.get(points, self.points.len() - 1);

    for i in 0..self.points.len() - 1 {
      let cur = self.get(points, i);
      result += (prev - cur).len();
      prev = cur;
    }

    result
  }

  pub fn contains(&self, points: &[Point], pi: u32) -> bool {
    if self.points.is_empty() {
      return false;
    }
    let p = points[pi as usize];
    let mut c_in = 0;
    let mut c_out = 0;
    let mut prev = self.get(points, self.points.len() - 1) - p;
    for i in 0..self.points.len() {
      let cur = self.get(points, i) - p;
      if prev.y >= 0.0 && cur.y < 0.0 && cross(prev, cur) <= 0.0 {
        c_in += 1;
      } else if prev.y < 0.0 && cur.y >= 0.0 && cross(prev, cur) > 0.0 {
        c_out += 1;
      }
      prev = cur;
    }
    c_in < c_out
  }

  pub fn split_to_triangles_if_convex(self, points: &[Point]) -> Vec<ConnectedPart> {
    if self.points.len() < 3 {
      return vec![];
    }

    if self.points.len() == 3 {
      return vec![ConnectedPart { contours: vec![self] }];
    }

    // same logic with bad-angle and splic, but lighter

    let pprev = self.get(points, self.points.len() - 2);
    let mut prev = self.get(points, self.points.len() - 1);
    let mut prev_prev_i = self.points.len() - 1;
    let mut prev_i = self.points.len() - 1;
    let mut prev_delta = prev - pprev;

    let mut worst_angle = (0, 0, 0, Point::ZERO, f32::NEG_INFINITY);
    for i in 0..self.points.len() {
      let cur = self.get(points, i);
      let delta = cur - prev;
      let cr = cross(delta, prev_delta);
      if cr > worst_angle.4 {
        worst_angle = (prev_prev_i, prev_i, i, -delta.perp() - prev_delta.perp(), cr);
      }

      prev_delta = delta;
      prev = cur;
      prev_prev_i = prev_i;
      prev_i = i;
    }

    let mut farest_angle = (0, f32::NEG_INFINITY);
    for i in 0..self.points.len() {
      if i == worst_angle.0 || i == worst_angle.1 || i == worst_angle.2 {
        continue;
      }
      let cur = self.get(points, i);
      let d = dot(cur, worst_angle.3);
      if d > farest_angle.1 {
        farest_angle = (i, d);
      }
    }

    let i1 = worst_angle.1;
    let i2 = farest_angle.0;
    let (i1, i2) = if i1 < i2 { (i1, i2) } else { (i2, i1) };
    let c1 = Contour { points: self.points[i1..=i2].to_owned() };
    let c2 = Contour { points: [&self.points[i2..], &self.points[..=i1]].concat() };
    let mut lhs = c1.split_to_triangles_if_convex(points);
    let rhs = c2.split_to_triangles_if_convex(points);
    lhs.extend(rhs);
    lhs
  }

  pub fn optimize(&mut self, points: &[Point], eps: f32) {
    let ok = |i1: usize, i2: usize| {
      if i1 == i2 {
        return false;
      }
      let mut i = i1;
      loop {
        i += 1;
        if i == self.points.len() {
          i = 0;
        }
        if i == i2 {
          return true;
        }
        if dist_pl(self.get(points, i), self.get(points, i1), self.get(points, i2)) > eps {
          return false;
        }
      }
    };

    let mut v = Vec::<(usize, usize)>::with_capacity(self.points.len()); // point index, next

    for i in 0..self.points.len() - 1 {
      v.push((i, i + 1));
    }
    v.push((self.points.len() - 1, 0));
    let mut i = 0;
    let mut ni = v[i].1;
    let mut nni = v[ni].1;
    let mut lv: Option<usize> = None;
    loop {
      let next = v[nni].1;
      if i == next {
        break;
      }
      if ok(i, nni) {
        v[i].1 = nni;
        lv = None;
      } else {
        i = ni;
        if lv == Some(i) {
          break;
        }
        lv = lv.or(Some(i));
      }
      ni = nni;
      nni = next;
    }

    let si = i;
    let mut fixed_points = Vec::new();
    loop {
      fixed_points.push(self.points[v[i].0]);
      i = v[i].1;
      if i == si {
        break;
      }
    }
    self.points = fixed_points;
  }

  fn find_bad_angle(&self, points: &[Point]) -> Option<usize> {
    if self.points.len() < 3 {
      return None;
    }

    let mut prev_prev_i = self.points.len() - 2;
    let pprev = self.get(points, prev_prev_i);
    let mut prev_i = self.points.len() - 1;
    let mut prev = self.get(points, prev_i);

    let mut prev_delta = prev - pprev;

    let mut result = None;
    let mut change = true;

    for i in 0..self.points.len() {
      let cur = self.get(points, i);
      let delta = cur - prev;

      let bad;

      if self.points[prev_prev_i] == self.points[i] {
        // Hateful "hair" case
        let mut c2 = self.points[prev_prev_i];
        let mut c1 = self.points[prev_i];
        let mut j1 = prev_prev_i;
        let mut j2 = i;
        loop {
          j1 = if j1 == 0 { self.points.len() - 1 } else { j1 - 1 };
          j2 = if j2 == self.points.len() - 1 { 0 } else { j2 + 1 };
          let pt1 = self.points[j1];
          let pt2 = self.points[j2];
          if pt1 == pt2 {
            c2 = c1;
            c1 = pt1;
            continue;
          }

          let coo0 = points[c2 as usize];
          let coo1 = points[c1 as usize] - coo0;
          let coo2 = points[pt1 as usize] - coo0;
          let coo3 = points[pt2 as usize] - coo0;

          bad = (cross(coo1, coo2) > 0.0) as i32
            + (cross(coo2, coo3) > 0.0) as i32
            + (cross(coo3, coo1) > 0.0) as i32
            >= 2;
          break;
        }
      } else {
        bad = cross(delta, prev_delta) > 0.0;
      }

      if bad {
        if change {
          // try to get "middle" bad angle
          result = Some(prev_i);
        }
        change = !change;
      }

      prev_delta = delta;
      prev = cur;
      prev_prev_i = prev_i;
      prev_i = i;
    }

    result
  }
}

impl ConnectedPart {
  pub fn new() -> Self {
    Self { contours: Vec::new() }
  }

  pub fn points_count(&self) -> usize {
    let mut result = 0;
    for c in &self.contours {
      result += c.points.len();
    }
    result
  }

  pub fn optimize(&mut self, points: &[Point], eps: f32) {
    for c in &mut self.contours {
      c.optimize(points, eps)
    }
  }

  pub fn merge(&mut self, other: Self) {
    self.contours.extend(other.contours)
  }

  pub fn remove_trash(&mut self, points: &[Point]) {
    self.contours.retain(|c| {
      let sq = c.get_square(points);
      sq < -1.0 || sq > 10.0
    })
  }

  fn split_by(mut self, points: &[Point], c1: usize, p1: usize, c2: usize, p2: usize) -> Vec<Self> {
    if c1 == c2 {
      let src = &self.contours[c1].points;
      let (p1, p2) = if p1 < p2 { (p1, p2) } else { (p2, p1) };
      let new_c1 = Contour { points: src[p1..=p2].to_vec() };
      let new_c2 = Contour { points: [&src[p2..], &src[..=p1]].concat() };
      assert!(new_c1.points.len() > 2);
      assert!(new_c2.points.len() > 2);

      let fragmented_parts = FragmentedParts {
        contours: [&self.contours[..c1], &[new_c1, new_c2], &self.contours[c2 + 1..]].concat(),
      };

      fragmented_parts.split_to_connected_areas(points)
    } else {
      let src1 = &self.contours[c1].points;
      let src2 = &self.contours[c2].points;
      let new_c =
        Contour { points: [&src1[..=p1], &src2[p2..], &src2[..=p2], &src1[p1..]].concat() };
      let (c1, c2) = if c1 < c2 { (c1, c2) } else { (c2, c1) };
      self.contours =
        [&self.contours[..c1], &self.contours[c1 + 1..c2], &self.contours[c2 + 1..], &[new_c]]
          .concat();
      // here we dont create new connected parts
      vec![self]
    }
  }

  fn find_bad_angle(&self, points: &[Point]) -> Option<(usize, usize)> {
    for ci in 0..self.contours.len() {
      if let Some(i) = self.contours[ci].find_bad_angle(points) {
        return Some((ci, i));
      }
    }
    None
  }

  fn get(&self, points: &[Point], c: usize, i: usize) -> Point {
    self.contours[c].get(points, i)
  }

  fn find_pair_for_bad_angle(&self, points: &[Point], c: usize, p: usize) -> (usize, usize) {
    self.find_pair_for_bad_angle_impl(points, c, p, false)
  }

  fn find_pair_for_bad_angle_impl(
    &self,
    points: &[Point],
    c: usize,
    p: usize,
    diagnostic: bool,
  ) -> (usize, usize) {
    const EPS: f32 = 1.0e-6;
    let ps0 = &self.contours[c];
    let p_base = ps0.get(points, p);
    let p1 = ps0.get(points, if p == 0 { ps0.points.len() - 1 } else { p - 1 }) - p_base;
    let p2 = ps0.get(points, if p == ps0.points.len() - 1 { 0 } else { p + 1 }) - p_base;

    let bisect = p1.norm().perp() - p2.norm().perp() - p1.norm() - p2.norm();

    // Stage1: find possible points
    let mut candidates = Vec::new();

    for ci in 0..self.contours.len() {
      let cp = &self.contours[ci];
      let prev_prev_i = cp.points.len() - 2;
      let mut prev_i = cp.points.len() - 1;
      let mut prev_prev = cp.get(points, prev_prev_i) - p_base;
      let mut prev = cp.get(points, prev_i) - p_base;
      for i in 0..cp.points.len() {
        let pi = cp.get(points, i) - p_base;

        if cross(prev, p1) > EPS || cross(p2, prev) > EPS {
          let cr1 = cross(prev_prev, prev);
          let cr2 = cross(prev, pi);
          let sq = cross(prev_prev - prev, pi - prev);

          if (cr1 > EPS) as i32 + (cr2 > EPS) as i32 + (sq > EPS) as i32 >= 2 {
            candidates.push((ci, prev_i));
          }
        }

        prev_prev = prev;
        prev = pi;
        prev_i = i;
      }
    }
    if candidates.is_empty() {
      let fail = ContourSet { points: points.to_owned(), parts: vec![self.clone()] };
      let _ = fail.save_to_dxf(std::path::Path::new("fail.dxf"));
    }
    assert!(!candidates.is_empty());
    let cache: FxHashMap<_, f32> = candidates
      .iter()
      .map(|&(c, p)| {
        let pt = self.contours[c].get(points, p) - p_base;
        ((c, p), (cross(bisect, pt) / dot(bisect, pt)).abs())
      })
      .collect();

    candidates
      .sort_by(|k1, k2| cache.get(k1).unwrap().partial_cmp(cache.get(k2).unwrap()).unwrap());

    if diagnostic {
      println!("candidates={:?}", candidates);
      println!("cache={:?}", cache);
    }

    let mut intersectors = FxHashMap::default();

    // Stage2: check candidates
    'check_candidates: for (c, i) in candidates {
      let control = self.contours[c].get(points, i);

      fn intersect(p1: Point, p2: Point, p3: Point, p4: Point) -> bool {
        let cr1 = cross(p1 - p4, p3 - p4);
        let cr2 = cross(p2 - p4, p3 - p4);
        if cr1 < 0.0 && cr2 > 0.0 || cr1 > 0.0 && cr2 < 0.0 {
          let cr3 = cross(p3 - p2, p1 - p2);
          let cr4 = cross(p4 - p2, p1 - p2);
          if cr3 <= 0.0 && cr4 >= 0.0 || cr3 >= 0.0 && cr4 <= 0.0 {
            return true;
          }
        }
        return false;
      }

      for (_, &(prev, cur)) in &intersectors {
        if intersect(p_base, control, prev, cur) {
          continue 'check_candidates;
        }
      }

      for ci in 0..self.contours.len() {
        let cp = &self.contours[ci];
        let mut prev = cp.get(points, cp.points.len() - 1);
        for i in 0..cp.points.len() {
          let cur = cp.get(points, i);
          if intersectors.contains_key(&(ci, i)) {
            continue;
          }

          if intersect(p_base, control, prev, cur) {
            // bad
            if diagnostic {
              println!("failed with {}:{}", ci, i);
            }
            intersectors.insert((ci, i), (prev, cur));
            continue 'check_candidates;
          }

          prev = cur;
        }
      }

      return (c, i);
    }

    panic!("All candidates are wrong!");
  }

  pub fn split_to_triangles_if_convex(self, points: &[Point]) -> Vec<ConnectedPart> {
    let mut result = Vec::new();
    for c in self.contours {
      result.extend(c.split_to_triangles_if_convex(points));
    }
    result
  }

  pub fn split_to_triangles(self, points: &[Point]) -> Vec<ConnectedPart> {
    self.split_to_triangles_impl(points, false)
  }

  pub fn split_to_triangles_impl(self, points: &[Point], diagnostic: bool) -> Vec<ConnectedPart> {
    let mut result_before = vec![self];
    let mut result_after = Vec::new();

    let mut output = Vec::new();
    let mut iter = 0;
    loop {
      if diagnostic {
        let to_save = ContourSet { points: points.to_owned(), parts: result_before.clone() };
        println!("results_before={:?}", result_before);
        let _ = to_save.save_to_dxf(std::path::Path::new(&format!("split_{}.dxf", iter)));
        if iter == 202 {
          return vec![];
        };
        iter += 1;
        println!("iter={}", iter);
      }

      for r in result_before {
        if let Some((c, i)) = r.find_bad_angle(points) {
          let (c2, i2) = r.find_pair_for_bad_angle_impl(points, c, i, diagnostic);
          result_after.extend(r.split_by(points, c, i, c2, i2));
        } else {
          output.extend(r.split_to_triangles_if_convex(points));
        }
      }

      if result_after.is_empty() {
        break;
      }
      result_before = std::mem::take(&mut result_after);
    }

    output
  }

  pub fn extrude(&self, points: &[Point], width: f32) -> crate::model::Model {
    let mut points_3d = Vec::with_capacity(points.len() * 2);
    for &p in points {
      points_3d.push(crate::points3d::Point { x: p.x, y: p.y, z: 0.0 });
      points_3d.push(crate::points3d::Point { x: p.x, y: p.y, z: width });
    }

    let mut triangles = Vec::with_capacity(points.len() * 4);
    for c in &self.contours {
      let mut prev = c.points[c.points.len() - 1];
      for &p in &c.points {
        triangles.push(crate::model::Triangle(prev * 2, p * 2, p * 2 + 1));
        triangles.push(crate::model::Triangle(prev * 2, p * 2 + 1, prev * 2 + 1));
        prev = p;
      }
    }

    for t in self.clone().split_to_triangles(points) {
      for c in t.contours {
        assert!(c.points.len() == 3);
        triangles.push(crate::model::Triangle(c.points[2] * 2, c.points[1] * 2, c.points[0] * 2));
        triangles.push(crate::model::Triangle(
          c.points[0] * 2 + 1,
          c.points[1] * 2 + 1,
          c.points[2] * 2 + 1,
        ));
      }
    }

    crate::model::Model { vertices: points_3d, triangles, free_vertices: Vec::new() }
  }

  pub fn get_square(&self, points: &[Point]) -> f32 {
    let mut result = 0.0;
    for c in &self.contours {
      result += c.get_square(points)
    }
    result
  }

  pub fn get_length(&self, points: &[Point]) -> f32 {
    let mut result = 0.0;
    for c in &self.contours {
      result += c.get_length(points)
    }
    result
  }
}

impl ContourSet {
  pub fn save_to_dxf(&self, path: &std::path::Path) -> Result<(), String> {
    let mut drawing = Drawing::new();

    drawing.header.drawing_units = dxf::enums::DrawingUnits::Metric;

    {
      // CYPCUT access violation fix
      let dc = drawing.dim_styles().count();
      for i in 0..dc {
        drawing.remove_dim_style(dc - 1 - i);
      }
    }

    for part in &self.parts {
      for contour in &part.contours {
        let mut pl = Polyline::default();
        for i in 0..contour.points.len() {
          fn point2d_to_dxf(pt: Point) -> dxf::entities::Vertex {
            dxf::entities::Vertex::new(dxf::Point { x: pt.x as f64, y: pt.y as f64, z: 0.0 })
          }

          let v = point2d_to_dxf(self.points[contour.points[i] as usize]);
          pl.add_vertex(&mut drawing, v);
        }
        pl.set_is_closed(true);

        drawing.add_entity(Entity::new(EntityType::Polyline(pl)));
      }
    }

    drawing
      .save_file(path)
      .map_err(|e| format!("Unable to open file {} for writing: {}", path.to_string_lossy(), e))
  }

  pub fn points_count(&self) -> usize {
    self.points.len()
  }

  pub fn optimize(&mut self, eps: f32) {
    for p in &mut self.parts {
      p.optimize(&self.points, eps)
    }
  }

  pub fn get_square(&self) -> f32 {
    let mut result = 0.0;
    for p in &self.parts {
      result += p.get_square(&self.points)
    }
    result
  }

  pub fn get_length(&self) -> f32 {
    let mut result = 0.0;
    for p in &self.parts {
      result += p.get_length(&self.points)
    }
    result
  }

  pub fn remove_trash(&mut self) {
    for p in &mut self.parts {
      p.remove_trash(&self.points)
    }
  }

  pub fn split_to_triangles(&mut self) {
    let mut parts = Vec::new();
    let old_parts = std::mem::take(&mut self.parts);
    for p in old_parts {
      parts.extend(p.split_to_triangles(&self.points));
    }

    self.parts = parts;
  }

  pub fn extrude(&self, width: f32) -> Vec<crate::model::Model> {
    self.parts.iter().map(|p| p.extrude(&self.points, width)).collect()
  }
}

impl FragmentedParts {
  pub fn new() -> Self {
    Self { contours: Vec::new() }
  }

  pub fn split_to_connected_areas(mut self, points: &[Point]) -> Vec<ConnectedPart> {
    let squares: Vec<(f32, i32)> = self
      .contours
      .iter()
      .map(|c| {
        if c.points.is_empty() {
          (0.0, 0)
        } else {
          let sq = c.get_square(points);
          (sq, sq.signum() as i32)
        }
      })
      .collect();

    let mut insides = Vec::new();
    insides.resize(self.contours.len(), Vec::new());

    for i in 0..self.contours.len() {
      if squares[i].1 == 1 {
        insides[i].push(i);
        continue;
      }
      if squares[i].1 == 0 {
        continue;
      }
      let pt0 = self.contours[i].points[0];
      let mut inside = (self.contours.len(), f32::INFINITY, false);
      for j in 0..self.contours.len() {
        if i == j || squares[j].1 != 1 || !self.contours[j].contains(points, pt0) {
          continue;
        }

        if inside.1 > squares[j].0 {
          inside = (j, squares[j].0, true);
        }
      }

      if !inside.2 {
        let fail = ContourSet {
          points: points.to_owned(),
          parts: vec![ConnectedPart { contours: self.contours.clone() }],
        };
        let _ = fail.save_to_dxf(std::path::Path::new("fail.dxf"));
      }
      assert!(inside.2);
      insides[inside.0].push(i);
    }

    insides
      .into_iter()
      .filter(|i| !i.is_empty())
      .map(|i| ConnectedPart {
        contours: i
          .into_iter()
          .map(|i| std::mem::replace(&mut self.contours[i], Contour::new()))
          .collect(),
      })
      .collect()
  }

  pub fn into_contour_set(mut self, points: Vec<Point>) -> ContourSet {
    let parts = self.split_to_connected_areas(&points);
    ContourSet { points, parts }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ContourCell {
  index: PartIndex,
  pos: Point,
  v_mz: u32,
  v_pz: u32,
  v_zm: u32,
  v_zp: u32,
}

impl ContourCell {
  fn new() -> Self {
    Self {
      index: 0,
      pos: Point { x: 0.0, y: 0.0 },
      v_mz: BAD_INDEX,
      v_pz: BAD_INDEX,
      v_zm: BAD_INDEX,
      v_zp: BAD_INDEX,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ContourCreator {
  size: usize,
  contour_size: f32,
  tries: usize,
  points: Vec<Point>,
  cells: Vec<ContourCell>,
}

impl ContourCreator {
  pub fn new(size: usize, contour_size: f32, tries: usize) -> Self {
    Self {
      size,
      contour_size,
      tries,
      points: Vec::new(),
      cells: vec![ContourCell::new(); size * size],
    }
  }

  fn index_to_point(size: usize, contour_size: f32, x: usize, y: usize) -> Point {
    let scale = contour_size / (size as f32 - 1.0);
    Point {
      x: x as f32 * scale * 0.5 - contour_size * 0.5,
      y: y as f32 * scale * 0.5 - contour_size * 0.5,
    }
  }

  fn center_of_cell(size: usize, contour_size: f32, x: usize, y: usize) -> Point {
    Self::index_to_point(size, contour_size, x * 2 - 1, y * 2 - 1)
  }

  fn corner_of_cell(size: usize, contour_size: f32, x: usize, y: usize) -> Point {
    Self::index_to_point(size, contour_size, x * 2, y * 2)
  }

  fn fill_cell(
    size: usize,
    contour_size: f32,
    cell: &mut ContourCell,
    x: usize,
    y: usize,
    part_f: &dyn Fn(Point) -> PartIndex,
  ) {
    cell.pos = Self::corner_of_cell(size, contour_size, x, y);
    cell.index = part_f(cell.pos);
  }

  fn index_of_new_point(points: &mut Vec<Point>, pt: Point) -> u32 {
    let result = points.len() as u32;
    points.push(pt);
    result
  }

  fn fill_ti(
    i1: u32,
    i2: u32,
    i3: u32,
    p12: PartIndex,
    p13: PartIndex,
    result: &mut HashMap<PartIndex, HashMap<u32, u32>>,
  ) {
    if i1 != 0 && i1 != i2 && i1 != i3 {
      assert!(p12 != BAD_INDEX);
      assert!(p13 != BAD_INDEX);
      let prev = result.entry(i1).or_default().insert(p12, p13);
      assert!(prev.is_none());
    }
  }

  fn fill_to(
    i1: u32,
    i2: u32,
    i3: u32,
    p21: PartIndex,
    p31: PartIndex,
    result: &mut HashMap<PartIndex, HashMap<u32, u32>>,
  ) {
    if i1 != i2 && i2 != 0 && i2 == i3 {
      assert!(p21 != BAD_INDEX);
      assert!(p31 != BAD_INDEX);
      let prev = result.entry(i2).or_default().insert(p31, p21);
      assert!(prev.is_none());
    }
  }

  fn fill_t(
    i1: u32,
    i2: u32,
    i3: u32,
    p12: PartIndex,
    p21: PartIndex,
    p13: PartIndex,
    p31: PartIndex,
    p23: PartIndex,
    p32: PartIndex,
    result: &mut HashMap<PartIndex, HashMap<u32, u32>>,
  ) {
    Self::fill_ti(i1, i2, i3, p12, p13, result);
    Self::fill_to(i1, i2, i3, p21, p31, result);
    Self::fill_ti(i2, i3, i1, p23, p21, result);
    Self::fill_to(i2, i3, i1, p32, p12, result);
    Self::fill_ti(i3, i1, i2, p31, p32, result);
    Self::fill_to(i3, i1, i2, p13, p23, result);
  }

  pub fn make_contour(
    &mut self,
    part_f: &dyn Fn(Point) -> PartIndex,
  ) -> HashMap<PartIndex, ContourSet> {
    if self.size == 0 {
      return HashMap::new();
    }

    let cells = &mut self.cells[..];
    let sz = self.size;

    let mut result = HashMap::new();

    macro_rules! fill_mid {
      ($c: expr, $field: ident, $p1: expr, $p2: expr) => {
        if cells[$c].index != 0 {
          let pt = find_root(part_f, $p1, $p2, cells[$c].index, self.tries);
          cells[$c].$field = Self::index_of_new_point(&mut self.points, pt);
        }
      };
    }

    macro_rules! fill_mids {
      ($c1: expr, $field1: ident,
            $c2: expr, $field2: ident) => {
        if cells[$c1].index != cells[$c2].index {
          fill_mid!($c1, $field1, cells[$c1].pos, cells[$c2].pos);
          fill_mid!($c2, $field2, cells[$c2].pos, cells[$c1].pos);
        }
      };
    }

    Self::fill_cell(sz, self.contour_size, &mut cells[0], 0, 0, part_f);
    for x in 1..sz {
      Self::fill_cell(sz, self.contour_size, &mut cells[x], x, 0, part_f);
      fill_mids!(x - 1, v_pz, x, v_mz);
    }

    for y in 1..sz {
      let c = sz * y;
      let c10 = c - sz;
      let c11 = c;

      Self::fill_cell(sz, self.contour_size, &mut cells[c11], 0, y, part_f);
      fill_mids!(c10, v_zp, c11, v_zm);

      for x in 1..sz {
        let c = c + x;
        let c00 = c - 1 - sz;
        let c10 = c - sz;
        let c01 = c - 1;
        let c11 = c;
        Self::fill_cell(sz, self.contour_size, &mut cells[c11], x, y, part_f);
        fill_mids!(c01, v_pz, c11, v_mz);
        fill_mids!(c10, v_zp, c11, v_zm);

        // fill cell here
        let center = Self::center_of_cell(sz, self.contour_size, x, y);
        let center_index = part_f(center);

        let mut v_mmi = BAD_INDEX;
        let mut v_mmo = BAD_INDEX;
        let mut v_mpi = BAD_INDEX;
        let mut v_mpo = BAD_INDEX;
        let mut v_pmi = BAD_INDEX;
        let mut v_pmo = BAD_INDEX;
        let mut v_ppi = BAD_INDEX;
        let mut v_ppo = BAD_INDEX;

        macro_rules! fill_center_mid {
          ($c: expr, $dst1: ident, $dst2: ident) => {
            let c_index = cells[$c].index;
            if center_index != c_index {
              let pt1 = find_root(part_f, center, cells[$c].pos, center_index, self.tries);
              $dst1 = Self::index_of_new_point(&mut self.points, pt1);
              let pt2 = find_root(part_f, cells[$c].pos, center, c_index, self.tries);
              $dst2 = Self::index_of_new_point(&mut self.points, pt2);
            }
          };
        }

        fill_center_mid!(c00, v_mmi, v_mmo);
        fill_center_mid!(c01, v_mpi, v_mpo);
        fill_center_mid!(c10, v_pmi, v_pmo);
        fill_center_mid!(c11, v_ppi, v_ppo);

        Self::fill_t(
          center_index,
          cells[c00].index,
          cells[c10].index,
          v_mmi,
          v_mmo,
          v_pmi,
          v_pmo,
          cells[c00].v_pz,
          cells[c10].v_mz,
          &mut result,
        );

        Self::fill_t(
          center_index,
          cells[c10].index,
          cells[c11].index,
          v_pmi,
          v_pmo,
          v_ppi,
          v_ppo,
          cells[c10].v_zp,
          cells[c11].v_zm,
          &mut result,
        );

        Self::fill_t(
          center_index,
          cells[c11].index,
          cells[c01].index,
          v_ppi,
          v_ppo,
          v_mpi,
          v_mpo,
          cells[c11].v_mz,
          cells[c01].v_pz,
          &mut result,
        );

        Self::fill_t(
          center_index,
          cells[c01].index,
          cells[c00].index,
          v_mpi,
          v_mpo,
          v_mmi,
          v_mmo,
          cells[c01].v_zm,
          cells[c00].v_zp,
          &mut result,
        );
      }
    }

    result
      .into_iter()
      .map(|(model_index, mut edges)| {
        let mut parts = FragmentedParts::new();

        let mut points = Vec::new();

        while let Some(&key) = edges.keys().next() {
          let mut new_contour = Contour::new();
          let mut current = key;
          loop {
            let new_point_index = points.len();
            points.push(self.points[current as usize]);
            new_contour.points.push(new_point_index as u32);

            current = edges.remove(&current).unwrap();
            if current == key {
              break;
            }
          }

          parts.contours.push(new_contour);
        }

        let contour_set = parts.into_contour_set(points);
        (model_index, contour_set)
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  static POINTS: [Point; 13] = [
    // BASE
    Point { x: 1.0, y: 0.0 },
    Point { x: 0.0, y: 1.0 },
    Point { x: -1.0, y: 0.0 },
    Point { x: 0.0, y: -1.0 },
    // SHIFTED
    Point { x: 11.0, y: 0.0 },
    Point { x: 10.0, y: 1.0 },
    Point { x: 9.0, y: 0.0 },
    Point { x: 10.0, y: -1.0 },
    // ZERO
    Point { x: 0.0, y: 0.0 },
    // OUTER POINTS
    Point { x: 2.0, y: 0.0 },
    Point { x: 0.0, y: 2.0 },
    Point { x: -2.0, y: 0.0 },
    Point { x: 0.0, y: -2.0 },
  ];

  static BAD_ANGLE: [Point; 4] = [
    Point { x: 1.0, y: 0.0 },
    Point { x: -2.0, y: 1.0 },
    Point { x: -1.0, y: 0.0 },
    Point { x: -2.0, y: -1.0 },
  ];

  #[test]
  fn test_optimize_contour4() {
    let mut c = Contour { points: vec![0, 1, 2, 3] };

    c.optimize(&POINTS, 0.5);
    assert_eq!(c.points.len(), 4);
    c.optimize(&POINTS, 1.5);
    assert_eq!(c.points.len(), 3);
  }

  #[test]
  fn test_split_contour4() {
    let c = ConnectedPart { contours: vec![Contour { points: vec![0, 1, 2, 3] }] };

    let cc = c.split_by(&POINTS, 0, 1, 0, 3);
    assert_eq!(cc.len(), 2);
    let c = &cc[0];
    assert_eq!(c.contours.len(), 1);
    assert_eq!(c.contours[0].points.len(), 3);
    let c = &cc[1];
    assert_eq!(c.contours.len(), 1);
    assert_eq!(c.contours[0].points.len(), 3);
  }

  #[test]
  fn test_split_2contour4() {
    let c = ConnectedPart {
      contours: vec![Contour { points: vec![0, 1, 2, 3] }, Contour { points: vec![4, 5, 6, 7] }],
    };

    let cc = c.split_by(&POINTS, 0, 2, 1, 0);
    assert_eq!(cc.len(), 1);
    let c = &cc[0];
    assert_eq!(c.contours.len(), 1);
    assert_eq!(c.contours[0].points.len(), 10);
  }

  #[test]
  fn test_contains_contour4() {
    let c = Contour { points: vec![0, 1, 2, 3] };
    assert!(c.contains(&POINTS, 8));
    assert!(!c.contains(&POINTS, 4));
  }

  #[test]
  fn test_bad_angle() {
    let c = Contour { points: vec![0, 1, 2, 3] };
    assert_eq!(c.find_bad_angle(&POINTS), None);
    assert_eq!(c.find_bad_angle(&BAD_ANGLE), Some(2));
  }

  #[test]
  fn test_bad_angle_hair_inside() {
    static BAD_ANGLE_HAIR: [Point; 4] = [
      Point { x: 0.0, y: 0.0 },
      Point { x: 2.0, y: 0.0 },
      Point { x: 1.0, y: 2.0 },
      Point { x: 1.0, y: 1.0 },
    ];
    let c = Contour { points: vec![0, 1, 2, 3, 2] };
    assert_eq!(c.find_bad_angle(&BAD_ANGLE_HAIR), Some(3));
  }

  #[test]
  fn test_bad_angle_hair_outside() {
    static BAD_ANGLE_HAIR: [Point; 4] = [
      Point { x: 0.0, y: 0.0 },
      Point { x: 2.0, y: 0.0 },
      Point { x: 1.0, y: 2.0 },
      Point { x: 1.0, y: 3.0 },
    ];
    let c = Contour { points: vec![0, 1, 2, 3, 2] };
    assert_eq!(c.find_bad_angle(&BAD_ANGLE_HAIR), Some(4));
  }

  #[test]
  fn test_pair_for_bad_angle() {
    let c = ConnectedPart { contours: vec![Contour { points: vec![0, 1, 2, 3] }] };
    assert_eq!(c.find_pair_for_bad_angle(&BAD_ANGLE, 0, 2), (0, 0));
  }

  #[test]
  fn test_pair_for_bad_angle_bad_case() {
    static BAD_ANGLE_PAIR: [Point; 6] = [
      Point { x: 0.0, y: 0.0 },
      Point { x: 1.0, y: -1.0 },
      Point { x: 3.0, y: 2.0 },
      Point { x: 2.0, y: 0.0 },
      Point { x: 4.0, y: 3.0 },
      Point { x: -3.0, y: -1.0 },
    ];
    let c = ConnectedPart { contours: vec![Contour { points: vec![0, 1, 2, 3, 4, 5] }] };
    assert_eq!(c.find_pair_for_bad_angle(&BAD_ANGLE_PAIR, 0, 0), (0, 2));
  }

  #[test]
  fn test_pair_for_bad_angle_intermediate_point_case() {
    static BAD_ANGLE_PAIR: [Point; 6] = [
      Point { x: 0.0, y: 0.0 },
      Point { x: 1.0, y: -1.0 },
      Point { x: 1.0, y: 1.0 },
      Point { x: 2.0, y: 0.0 },
      Point { x: 2.0, y: 2.0 },
      Point { x: -3.0, y: -1.0 },
    ];
    let c = ConnectedPart { contours: vec![Contour { points: vec![0, 1, 2, 3, 4, 5] }] };
    assert_eq!(c.find_pair_for_bad_angle(&BAD_ANGLE_PAIR, 0, 0), (0, 2));
  }

  #[test]
  fn test_pair_for_bad_angle_wrong_side() {
    static BAD_ANGLE_PAIR: [Point; 5] = [
      Point { x: 0.0, y: 0.0 },
      Point { x: 2.0, y: -2.0 },
      Point { x: 2.0, y: 4.0 },
      Point { x: -2.0, y: -1.0 },
      Point { x: -1.0, y: 0.0 },
    ];
    let c = ConnectedPart { contours: vec![Contour { points: vec![0, 1, 2, 3, 4] }] };
    assert_eq!(c.find_pair_for_bad_angle(&BAD_ANGLE_PAIR, 0, 0), (0, 2));
  }

  #[test]
  fn test_split_to_triangles() {
    let c = ConnectedPart {
      contours: vec![Contour { points: vec![3, 2, 1, 0] }, Contour { points: vec![9, 10, 11, 12] }],
    };

    let cc = c.split_to_triangles(&POINTS);
    assert_eq!(cc.len(), 8);
    for ccc in cc {
      assert!(ccc.get_square(&POINTS) >= 0.0);
    }
  }

  #[test]
  fn test_split_to_triangles_if_convex() {
    static CONVEX_CONTOUR: [Point; 4] = [
      Point { x: 1.0, y: 0.0 },
      Point { x: 0.0, y: 1.0 },
      Point { x: 0.0, y: 0.0 },
      Point { x: 0.0, y: -1.0 },
    ];
    let c = Contour { points: vec![0, 1, 2, 3] };
    let cc = c.split_to_triangles_if_convex(&CONVEX_CONTOUR);
    assert_eq!(cc.len(), 2);
    for ccc in cc {
      assert!(ccc.get_square(&POINTS) >= 0.0);
    }
  }

  #[test]
  fn test_connection_2contour4() {
    let c = FragmentedParts {
      contours: vec![Contour { points: vec![0, 1, 2, 3] }, Contour { points: vec![4, 5, 6, 7] }],
    };

    let cc = c.split_to_connected_areas(&POINTS);
    assert_eq!(cc.len(), 2);
    assert_eq!(cc[0].contours.len(), 1);
    assert_eq!(cc[1].contours.len(), 1);
  }

  #[test]
  fn test_connection_2contour4_inside() {
    let c = FragmentedParts {
      contours: vec![Contour { points: vec![3, 2, 1, 0] }, Contour { points: vec![9, 10, 11, 12] }],
    };

    let cc = c.split_to_connected_areas(&POINTS);
    assert_eq!(cc.len(), 1);
    assert_eq!(cc[0].contours.len(), 2);
  }
}
