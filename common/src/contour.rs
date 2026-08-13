use crate::bit_buffer::*;
use crate::points2d::*;
use dxf::Drawing;
use dxf::entities::*;
use dxf::objects::*;
use fxhash::FxHashMap;
use std::collections::HashMap;
use std::default;

pub type PartIndex = u32;
pub const BAD_INDEX: PartIndex = PartIndex::MAX;
pub const BAD_VERTEX: usize = usize::MAX;
pub const BAD_ORD: usize = usize::MAX;
pub const BAD_EDGE: usize = usize::MAX;

#[derive(Debug, Clone)]
pub struct Contour {
  pub points: Vec<Point>,
}

pub type Triangle = [usize; 3];

#[derive(Debug)]
pub struct FlatFigure {
  contours: Vec<Contour>,
  triangles: Vec<Triangle>,
}

impl Contour {
  pub fn new() -> Self {
    Self { points: Vec::new() }
  }

  pub fn get_square(&self) -> f32 {
    if self.points.len() < 3 {
      return 0.0;
    }

    let mut result = 0.0;
    let first = self.points[0];
    let mut prev = self.points[1] - first;
    for &p in &self.points[2..] {
      let cur = p - first;
      result += cross(prev, cur);
      prev = cur;
    }
    result * 0.5
  }

  pub fn get_length(&self) -> f32 {
    if self.points.len() < 2 {
      return 0.0;
    }

    let mut result = 0.0;
    let mut prev = *self.points.last().unwrap();

    for &cur in &self.points {
      result += (prev - cur).len();
      prev = cur;
    }

    result
  }
}

impl FlatFigure {
  pub fn new() -> Self {
    Self { contours: Vec::new(), triangles: Vec::new() }
  }

  pub fn get_square(&self) -> f32 {
    self.contours.iter().map(|c| c.get_square()).sum()
  }

  pub fn get_length(&self) -> f32 {
    self.contours.iter().map(|c| c.get_length()).sum()
  }

  pub fn points_count(&self) -> usize {
    self.contours.iter().map(|c| c.points.len()).sum()
  }

  pub fn save_to_dxf(&self, path: &std::path::Path) -> Result<(), String> {
    self.save_to_dxf_with_grid(path, false)
  }

  pub fn save_to_dxf_with_grid(
    &self,
    path: &std::path::Path,
    with_grid: bool,
  ) -> Result<(), String> {
    let mut drawing = Drawing::new();
    drawing.header.drawing_units = dxf::enums::DrawingUnits::Metric;
    {
      // CYPCUT access violation fix
      let dc = drawing.dim_styles().count();
      for i in 0..dc {
        drawing.remove_dim_style(dc - 1 - i);
      }
    }

    let mut aabb = AABB::empty();
    for contour in &self.contours {
      for &p in &contour.points {
        aabb = aabb.with(p);
      }
    }

    if with_grid {
      fn point2d_to_dxf(pt: Point) -> dxf::Point {
        dxf::Point { x: pt.x as f64, y: pt.y as f64, z: 0.0 }
      }
      let cx1: &dyn Fn(_) -> _ = &|i: isize| Point { x: aabb.x1, y: i as f32 };
      let cx2: &dyn Fn(_) -> _ = &|i: isize| Point { x: aabb.x2, y: i as f32 };
      let cy1: &dyn Fn(_) -> _ = &|i: isize| Point { x: i as f32, y: aabb.y1 };
      let cy2: &dyn Fn(_) -> _ = &|i: isize| Point { x: i as f32, y: aabb.y2 };
      for (d, lc, skip10, color) in [
        ((aabb.y1, aabb.y2), (&cx1, &cx2), true, 254),
        ((aabb.x1, aabb.x2), (&cy1, &cy2), true, 254),
        ((aabb.y1, aabb.y2), (&cx1, &cx2), false, 253),
        ((aabb.x1, aabb.x2), (&cy1, &cy2), false, 253),
      ] {
        for i in d.0.floor() as isize..d.1.ceil() as isize + 1 {
          if (i % 10 == 0) == skip10 {
            continue;
          }
          let p1 = lc.0(i);
          let p2 = lc.1(i);
          let l = Line::new(point2d_to_dxf(p1), point2d_to_dxf(p2));
          let mut e = Entity::new(EntityType::Line(l));
          e.common.color = dxf::Color::from_index(color);
          drawing.add_entity(e);
        }
      }
    }
    for contour in &self.contours {
      let mut pl = Polyline::default();
      for i in 0..contour.points.len() {
        fn point2d_to_dxf(pt: Point) -> dxf::entities::Vertex {
          dxf::entities::Vertex::new(dxf::Point { x: pt.x as f64, y: pt.y as f64, z: 0.0 })
        }
        let v = point2d_to_dxf(contour.points[i]);
        pl.add_vertex(&mut drawing, v);
      }
      pl.set_is_closed(true);
      let mut e = Entity::new(EntityType::Polyline(pl));
      if with_grid {
        e.common.color = dxf::Color::from_index(250);
      }
      drawing.add_entity(e);
    }

    drawing
      .save_file(path)
      .map_err(|e| format!("Unable to open file {} for writing: {}", path.to_string_lossy(), e))
  }

  pub fn generate_triangle_contours(&self) -> Self {
    let mut contours = Vec::new();
    let vc = self.points_count();
    let mut vertices = Vec::with_capacity(vc);
    for c in &self.contours {
      vertices.extend(c.points.clone());
    }
    for t in &self.triangles {
      contours.push(Contour { points: vec![vertices[t[0]], vertices[t[1]], vertices[t[2]]] });
    }

    Self { contours, triangles: Vec::new() }
  }

  pub fn contours(&self) -> &[Contour] {
    &self.contours
  }

  pub fn extend(&mut self, other: Self) {
    let mut offset = self.points_count();
    self.contours.extend(other.contours.clone());
    self
      .triangles
      .extend(other.triangles.iter().map(|t| [t[0] + offset, t[1] + offset, t[2] + offset]));
  }

  pub fn extrude(&self, width: f32) -> crate::model::Model {
    let vc = self.points_count();
    let mut vertices = Vec::with_capacity(vc * 2);
    for c in &self.contours {
      for p in &c.points {
        vertices.push(crate::points3d::Point { x: p.x, y: p.y, z: 0.0 });
        vertices.push(crate::points3d::Point { x: p.x, y: p.y, z: width });
      }
    }

    let mut triangles = Vec::with_capacity(vc * 4);

    let mut offset = 0;
    for c in &self.contours {
      let mut prev = c.points.len() + offset - 1;
      for p in offset..c.points.len() + offset {
        triangles.push([prev as u32 * 2, p as u32 * 2, p as u32 * 2 + 1]);
        triangles.push([prev as u32 * 2, p as u32 * 2 + 1, prev as u32 * 2 + 1]);
        prev = p;
      }
      offset += c.points.len();
    }

    for &t in &self.triangles {
      triangles.push([t[2] as u32 * 2, t[1] as u32 * 2, t[0] as u32 * 2]);
      triangles.push([t[0] as u32 * 2 + 1, t[1] as u32 * 2 + 1, t[2] as u32 * 2 + 1]);
    }

    crate::model::Model { vertices, triangles }
  }
}

#[derive(Debug, Clone, Copy)]
struct Edge {
  begin: usize,
  end: usize,
}

#[derive(Debug, Default)]
pub struct ContourTopology {
  vertices: Vec<Point>, // sorted from low to top by merge-sort
  edges: Vec<Edge>,     // sorted from left to right by construction
}

impl ContourTopology {
  fn new() -> Self {
    Self { vertices: Vec::new(), edges: Vec::new() }
  }

  fn regroup_chains(&mut self, fixed_edges: Vec<Edge>, mut e2c: Vec<usize>) {
    #[derive(Copy, Clone, Debug)]
    struct VInfo {
      edge_from: usize,
      edge_to: usize,
    }
    let mut adj_e = Vec::new();
    adj_e.resize(self.vertices.len(), VInfo { edge_from: BAD_EDGE, edge_to: BAD_EDGE });
    for (i, e) in self.edges.iter().enumerate() {
      adj_e[e.begin].edge_from = i;
      adj_e[e.end].edge_to = i;
    }

    let mut chains = Vec::new();
    const FINISH: usize = BAD_EDGE - 1;
    chains.resize(self.edges.len() + 1, BAD_EDGE);
    chains[self.edges.len()] = FINISH;

    fn get_e2c(e: usize, e2c: &mut [usize]) -> usize {
      let mut r = e2c[e];
      if r == BAD_EDGE {
        return e;
      }
      r = get_e2c(r, e2c);
      e2c[e] = r;
      return r;
    };

    let mut sorted_e = BitBuffer::new(self.edges.len());

    fn insert(c_after: usize, e_new: usize, chains: &mut [usize], e2c: &mut [usize]) {
      let c_new = get_e2c(e_new, e2c);
      if chains[c_new] == BAD_EDGE {
        chains[c_new] = chains[c_after];
        chains[c_after] = c_new;
      }
    }

    fn get_prev_chain(
      edge: usize,
      sorted_e: &BitBuffer,
      if_fail: usize,
      e2c: &mut [usize],
    ) -> usize {
      let prev_e = sorted_e.upper_bound(edge, BAD_EDGE);
      if prev_e == BAD_EDGE { if_fail } else { get_e2c(prev_e, e2c) }
    }

    for v in 0..self.vertices.len() {
      let adj = adj_e[v];
      let e_from = self.edges[adj.edge_from];
      let e_to = self.edges[adj.edge_to];
      if e_from.end > v {
        if e_to.begin > v {
          // new chains
          let (l, r) = (adj.edge_from, adj.edge_to);
          let (l, r) = if l < r { (l, r) } else { (r, l) };
          let prev_c = get_prev_chain(l, &sorted_e, self.edges.len(), &mut e2c);
          insert(prev_c, r, &mut chains, &mut e2c);
          insert(prev_c, l, &mut chains, &mut e2c);
          sorted_e.put_number(l);
          sorted_e.put_number(r);
        } else {
          sorted_e.remove_number(adj.edge_to);
          let prev_c = get_prev_chain(adj.edge_to, &sorted_e, self.edges.len(), &mut e2c);
          insert(prev_c, adj.edge_from, &mut chains, &mut e2c);
          sorted_e.put_number(adj.edge_from);
        }
      } else {
        if e_to.begin > v {
          sorted_e.remove_number(adj.edge_from);
          let prev_c = get_prev_chain(adj.edge_from, &sorted_e, self.edges.len(), &mut e2c);
          insert(prev_c, adj.edge_to, &mut chains, &mut e2c);
          sorted_e.put_number(adj.edge_to);
        } else {
          // end chains
          sorted_e.remove_number(adj.edge_from);
          sorted_e.remove_number(adj.edge_to);
        }
      }
    }
    let mut ch = chains[self.edges.len()];
    self.edges.clear();
    while ch != FINISH {
      self.edges.push(fixed_edges[ch]);
      ch = chains[ch];
    }

    adj_e.clear();
    adj_e.resize(self.vertices.len(), VInfo { edge_from: BAD_EDGE, edge_to: BAD_EDGE });
    for (i, e) in self.edges.iter().enumerate() {
      assert!(adj_e[e.begin].edge_from == BAD_EDGE);
      assert!(adj_e[e.end].edge_to == BAD_EDGE);
      adj_e[e.begin].edge_from = i;
      adj_e[e.end].edge_to = i;
    }
    for a in adj_e {
      assert!((a.edge_from == BAD_EDGE) == (a.edge_to == BAD_EDGE));
    }
  }

  fn retain_edges_by(&mut self, condition: impl Fn(usize) -> bool) {
    let mut j = 0;
    for i in 0..self.edges.len() {
      if condition(i) {
        self.edges[j] = self.edges[i];
        j += 1;
      }
    }
    self.edges.truncate(j);
  }

  fn retain_vertices_by(&mut self, condition: impl Fn(usize) -> bool) {
    let mut fix = Vec::new();
    fix.resize(self.vertices.len(), 0);

    let mut j = 0;
    for (i, fix) in fix.iter_mut().enumerate() {
      if condition(i) {
        *fix = j;
        self.vertices[j] = self.vertices[i];
        j += 1;
      }
    }
    self.vertices.truncate(j);

    for e in &mut self.edges {
      e.begin = fix[e.begin];
      e.end = fix[e.end];
    }
  }

  pub fn remove_trash(&mut self) {
    let mut next = Vec::new();
    next.resize(self.vertices.len(), BAD_VERTEX);
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    enum EdgeResult {
      UNKNOWN,
      BAD,
      GOOD,
    };
    let mut visited = Vec::new();
    let mut skipped_v = Vec::new();
    let mut in_process = Vec::new();

    skipped_v.resize(self.vertices.len(), false);

    for (i, e) in self.edges.iter().enumerate() {
      next[e.begin] = i;
    }
    visited.resize(self.edges.len(), EdgeResult::UNKNOWN);
    for i in 0..self.edges.len() {
      if visited[i] != EdgeResult::UNKNOWN {
        continue;
      }
      let mut sq = 0.0;
      let mut cur = i;
      loop {
        let e = self.edges[cur];
        in_process.push(cur);
        sq += cross(self.vertices[e.begin], self.vertices[e.end]);
        cur = next[e.end];
        if cur == i {
          break;
        }
      }
      let bad = sq.abs() < 5.0;
      for &e in &in_process {
        visited[e] = if bad { EdgeResult::BAD } else { EdgeResult::GOOD };
        skipped_v[self.edges[e].begin] = bad;
      }

      in_process.clear();
    }

    self.retain_edges_by(|i| visited[i] == EdgeResult::GOOD);
    self.retain_vertices_by(|i| !skipped_v[i]);
  }

  pub fn optimize(&mut self, treshhold: f32) {
    #[derive(Copy, Clone, Debug)]
    struct VInfo {
      edge_from: usize,
      next_v: usize,
      skipped: bool,
    }
    let mut next = Vec::new();
    next.resize(
      self.vertices.len(),
      VInfo { edge_from: BAD_VERTEX, next_v: BAD_VERTEX, skipped: false },
    );
    for (i, e) in self.edges.iter().enumerate() {
      next[e.begin] = VInfo { edge_from: i, next_v: e.end, skipped: false };
    }

    let mut skipped_edges = Vec::new();
    skipped_edges.resize(self.edges.len(), false);

    let mut e2c = Vec::new();
    e2c.resize(self.edges.len(), BAD_EDGE);

    let mut edges = self.edges.clone();
    let mut finished = false;
    while !finished {
      finished = true;
      'each_edge: for i in 0..edges.len() {
        loop {
          if skipped_edges[i] {
            continue 'each_edge;
          }
          let e = edges[i];
          let new_begin = self.vertices[e.begin];
          let j = next[e.end].edge_from;
          let ne = edges[j];
          //  assert!(e.end == ne.begin);
          let new_end = self.vertices[ne.end];
          let mut skip = next[e.begin].next_v;
          while skip != ne.end {
            if dist_pl(self.vertices[skip], new_begin, new_end) > treshhold {
              continue 'each_edge;
            }
            skip = next[skip].next_v;
          }

          skipped_edges[j] = true;
          e2c[j] = i;
          edges[i].end = ne.end;
          next[e.end].skipped = true;
          finished = false;
        }
      }
    }

    self.regroup_chains(edges, e2c);
    self.retain_vertices_by(|i| !next[i].skipped);
  }

  fn create_contour_numbers(&self) -> Vec<Vec<usize>> {
    let mut next = Vec::new();
    let mut contours = Vec::new();
    next.resize(self.vertices.len(), 0);
    for (i, e) in self.edges.iter().enumerate() {
      next[e.begin] = i;
    }
    for i in 0..self.edges.len() {
      let mut c = i;
      let mut contour = Vec::new();
      loop {
        let e = &self.edges[c];
        let n = &mut next[e.end];
        if *n == BAD_VERTEX {
          if !contour.is_empty() {
            contours.push(contour);
          }
          break;
        }
        contour.push(e.end);
        c = *n;
        *n = BAD_VERTEX;
      }
    }
    contours
  }

  fn validate_edge_order(&self) -> bool {
    let mut bit_buf = BitBuffer::new(self.vertices.len());
    let mut vis = Vec::new();

    for e in &self.edges {
      bit_buf.put_range(e.begin, e.end, true, &mut vis);
      let v_begin = self.vertices[e.begin];
      let v_end = self.vertices[e.end];
      let (v_begin, v_end) = if e.begin < e.end { (v_begin, v_end) } else { (v_end, v_begin) };
      for &v in &vis {
        if cross(self.vertices[v] - v_end, v_begin - v_end) < 0.0 {
          return false;
        }
      }
      vis.clear();
    }

    true
  }

  pub fn to_flat_figure(self) -> FlatFigure {
    let mut bit_buf = BitBuffer::new(self.vertices.len());
    let mut loc_buf = BitBuffer::new(self.vertices.len());
    let mut vis = Vec::new();
    let mut next = Vec::new();
    next.resize(self.vertices.len(), BAD_VERTEX);

    let contour_numbers = self.create_contour_numbers();
    let mut v2c = Vec::new();
    v2c.resize(self.vertices.len(), 0);
    let mut i = 0;
    for n in &contour_numbers {
      for &n in n {
        v2c[n] = i;
        i += 1;
      }
    }

    let mut triangles = Vec::new();

    let mut handle_range = |e: &Edge, rev: bool, vis: &mut Vec<usize>, bit_buf: &mut BitBuffer| {
      if e.begin < e.end {
        bit_buf.put_range(e.begin, e.end, !rev, vis);
      } else {
        bit_buf.put_range(e.end, e.begin, rev, vis);
      }

      let cr = self.vertices[e.end] - self.vertices[e.begin];
      if !vis.is_empty() {
        let l = vis.len();
        loc_buf.put_number(e.begin);
        for &l in &*vis {
          loc_buf.put_number(l);
        }
        loc_buf.put_number(e.end);
        vis.sort_by(|&a, &b| {
          cross(self.vertices[a], cr).partial_cmp(&cross(self.vertices[b], cr)).unwrap()
        });
        for &l in &*vis {
          let (prev, next) = loc_buf.remove_number_get_adj(l);
          if e.begin < e.end {
            triangles.push([v2c[prev], v2c[next], v2c[l]]);
          } else {
            triangles.push([v2c[prev], v2c[l], v2c[next]]);
          }
        }
        loc_buf.remove_number(e.begin);
        loc_buf.remove_number(e.end);
      }
      vis.clear();
    };

    for e in &self.edges {
      handle_range(e, false, &mut vis, &mut bit_buf);
    }
    bit_buf.clear();
    for e in self.edges.iter().rev() {
      handle_range(e, true, &mut vis, &mut bit_buf);
    }
    bit_buf.clear();

    let contours = contour_numbers
      .into_iter()
      .map(|n| Contour { points: n.into_iter().map(|n| self.vertices[n]).collect() })
      .collect();

    FlatFigure { triangles, contours }
  }
}

#[derive(Debug)]
struct OrdBuffer {
  v: Vec<usize>,
  l: Vec<usize>,
  c: usize,
}

impl OrdBuffer {
  fn new() -> Self {
    Self { v: Vec::new(), l: Vec::new(), c: 0 }
  }

  fn clear_l(&mut self) {
    self.l.clear();
  }

  fn save_vertex_cursor(&mut self) {
    self.c = self.v.len()
  }

  fn extend(&mut self, v: &[usize], use_l: bool) {
    let mut l = 0;
    for &v in v {
      if v != BAD_VERTEX {
        self.v.push(v);
        l += 1;
      }
    }
    if use_l && l > 0 {
      self.l.push(l);
    }
  }

  fn sort_mid(&mut self, vertices: &[Point]) {
    let mut tmp = Vec::new();
    let mut dst0 = &mut self.v[self.c..];
    tmp.resize(dst0.len(), 0);
    let mut dst1: &mut [_] = &mut tmp;
    let mut original = true;

    while self.l.len() > 1 {
      let mut i = 0;
      let mut cursor = 0;
      while i * 2 < self.l.len() {
        let mut l1 = self.l[i * 2];
        let mut l2 = self.l.get(i * 2 + 1).copied().unwrap_or(0);
        let mut i1 = 0;
        let mut i2 = 0;
        loop {
          if i1 == l1 {
            if i2 == l2 {
              break;
            } else {
              dst1[cursor + i1 + i2] = dst0[cursor + l1 + i2];
              i2 += 1;
            }
          } else {
            if i2 == l2 {
              dst1[cursor + i1 + i2] = dst0[cursor + i1];
              i1 += 1;
            } else {
              let m1 = dst0[cursor + i1];
              let m2 = dst0[cursor + l1 + i2];
              if vertices[m1].y <= vertices[m2].y {
                dst1[cursor + i1 + i2] = m1;
                i1 += 1;
              } else {
                dst1[cursor + i1 + i2] = m2;
                i2 += 1;
              }
            }
          }
        }
        cursor += l1 + l2;
        self.l[i] = l1 + l2;
        i += 1;
      }
      (dst0, dst1) = (dst1, dst0);
      original = !original;
      self.l.truncate((self.l.len() + 1) / 2);
    }
    if !original {
      for (dst, src) in self.v[self.c..].iter_mut().zip(tmp) {
        *dst = src;
      }
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ContourCell {
  corner_part: PartIndex,
  corner: Point,
  v_mz: usize,
  v_pz: usize,
  v_zm: usize,
  v_zp: usize,
}

impl ContourCell {
  fn new() -> Self {
    Self {
      corner_part: 0,
      corner: Point { x: 0.0, y: 0.0 },
      v_mz: BAD_VERTEX,
      v_pz: BAD_VERTEX,
      v_zm: BAD_VERTEX,
      v_zp: BAD_VERTEX,
    }
  }
}

type GeneratorResult = FxHashMap<PartIndex, ContourTopology>;

#[derive(Debug)]
struct TmpResult {
  vertices: Vec<Point>,
  edges: FxHashMap<PartIndex, Vec<Edge>>,
  ord_buffer: OrdBuffer,
}

impl TmpResult {
  fn new() -> Self {
    Self { vertices: Vec::new(), edges: FxHashMap::default(), ord_buffer: OrdBuffer::new() }
  }

  fn to_generator_result(self) -> GeneratorResult {
    let mut v2ord = Vec::new();
    v2ord.resize(self.vertices.len(), (0, 0));

    let mut result = GeneratorResult::default();
    for (&part, edges) in &self.edges {
      for e in edges {
        v2ord[e.begin].0 = part;
      }
    }
    for v in self.ord_buffer.v {
      let part = result.entry(v2ord[v].0).or_default();
      v2ord[v].1 = part.vertices.len();
      part.vertices.push(self.vertices[v]);
    }
    for (part, edges) in self.edges {
      let part = result.entry(part).or_default();
      for e in edges {
        part.edges.push(Edge { begin: v2ord[e.begin].1, end: v2ord[e.end].1 });
      }
    }
    result
  }

  fn index_of_new_point(&mut self, p: Point) -> usize {
    let result = self.vertices.len();
    self.vertices.push(p);
    result
  }

  fn fill_ti(&mut self, i1: PartIndex, i2: PartIndex, i3: PartIndex, p12: usize, p13: usize) {
    if i1 != 0 && i1 != i2 && i1 != i3 {
      assert!(p12 != BAD_VERTEX);
      assert!(p13 != BAD_VERTEX);
      self.edges.entry(i1).or_default().push(Edge { begin: p12, end: p13 });
    }
  }

  fn fill_to(&mut self, i1: PartIndex, i2: PartIndex, i3: PartIndex, p21: usize, p31: usize) {
    if i1 != i2 && i2 != 0 && i2 == i3 {
      assert!(p21 != BAD_VERTEX);
      assert!(p31 != BAD_VERTEX);
      self.edges.entry(i2).or_default().push(Edge { begin: p31, end: p21 });
    }
  }

  fn fill_t(
    &mut self,
    i1: PartIndex,
    i2: PartIndex,
    i3: PartIndex,
    p12: usize,
    p21: usize,
    p13: usize,
    p31: usize,
    p23: usize,
    p32: usize,
  ) {
    self.fill_ti(i1, i2, i3, p12, p13);
    self.fill_to(i1, i2, i3, p21, p31);
    self.fill_ti(i2, i3, i1, p23, p21);
    self.fill_to(i2, i3, i1, p32, p12);
    self.fill_ti(i3, i1, i2, p31, p32);
    self.fill_to(i3, i1, i2, p13, p23);
  }
}

#[derive(Debug)]
pub struct ContourCreator {
  aabb: AABB,
  scale: f32,
  size_x: usize,
  size_y: usize,
  tries: usize,
}

impl ContourCreator {
  pub fn new(aabb: AABB, scale: f32, tries: usize) -> Self {
    let size_x = ((aabb.x2 - aabb.x1) / scale).ceil() as usize + 1;
    let size_y = ((aabb.y2 - aabb.y1) / scale).ceil() as usize + 1;
    Self { aabb, scale, size_x, size_y, tries }
  }

  fn index_to_point(&self, x: usize, y: usize) -> Point {
    Point {
      x: x as f32 * self.scale * 0.5 + self.aabb.x1,
      y: y as f32 * self.scale * 0.5 + self.aabb.y1,
    }
  }

  fn center_of_cell(&self, x: usize, y: usize) -> Point {
    self.index_to_point(x * 2 - 1, y * 2 - 1)
  }

  fn corner_of_cell(&self, x: usize, y: usize) -> Point {
    self.index_to_point(x * 2, y * 2)
  }

  fn init_cell(
    &mut self,
    cell: &mut ContourCell,
    x: usize,
    y: usize,
    part_f: &dyn Fn(Point) -> PartIndex,
  ) {
    cell.corner = self.corner_of_cell(x, y);
    cell.corner_part = part_f(cell.corner);
  }

  pub fn make_topology(mut self, part_f: &dyn Fn(Point) -> PartIndex) -> GeneratorResult {
    if self.size_x == 0 || self.size_y == 0 {
      return GeneratorResult::default();
    }
    let mut result = TmpResult::new();

    let mut cells = vec![ContourCell::new(); self.size_x * 2];
    let szx = self.size_x;
    let szy = self.size_y;

    macro_rules! fill_mids {
      (
        $part_index1: expr, $point1: expr, $target1: expr,
        $part_index2: expr, $point2: expr, $target2: expr
      ) => {
        let part_index1 = $part_index1;
        let part_index2 = $part_index2;
        if part_index1 != part_index2 {
          if part_index1 != 0 {
            if part_index2 != 0 {
              let (pt1, pt2) =
                find_2roots(part_f, $point1, $point2, part_index1, part_index2, self.tries);
              $target1 = result.index_of_new_point(pt1);
              $target2 = result.index_of_new_point(pt2);
            } else {
              let pt = find_root(part_f, $point1, $point2, part_index1, self.tries);
              $target1 = result.index_of_new_point(pt);
            }
          } else {
            if part_index2 != 0 {
              let pt = find_root(part_f, $point2, $point1, part_index2, self.tries);
              $target2 = result.index_of_new_point(pt);
            }
          }
        }
      };
    }

    macro_rules! fill_side_mids {
      ($ci1: expr, $target_field1: ident, $ci2: expr, $target_field2: ident) => {
        let c1 = &cells[$ci1];
        let c2 = &cells[$ci2];
        fill_mids!(
          c1.corner_part,
          c1.corner,
          cells[$ci1].$target_field1,
          c2.corner_part,
          c2.corner,
          cells[$ci2].$target_field2
        );
      };
    }

    self.init_cell(&mut cells[0], 0, 0, part_f);
    if cells[0].corner_part != 0 {
      panic!("Fail aabb in position {:?}", cells[0].corner);
    }

    for x in 1..szx {
      self.init_cell(&mut cells[x], x, 0, part_f);

      if cells[x].corner_part != 0 {
        panic!("Fail aabb in position {:?}", cells[x].corner);
      }

      fill_side_mids!(x - 1, v_pz, x, v_mz);
    }

    for x in (1..szx).rev() {
      result.ord_buffer.extend(&[cells[x].v_mz, cells[x - 1].v_pz], false);
    }

    let mut ofs_prev = 0;
    let mut ofs_cur = szx;

    for y in 1..szy {
      let c10 = ofs_prev;
      let c11 = ofs_cur;
      result.ord_buffer.save_vertex_cursor();
      assert!(result.vertices.len() == result.ord_buffer.c);

      self.init_cell(&mut cells[c11], 0, y, part_f);
      fill_side_mids!(c10, v_zp, c11, v_zm);

      if cells[c11].corner_part != 0 {
        panic!("Fail aabb in position {:?}", cells[c11].corner);
      }

      for x in 1..szx {
        let c00 = ofs_prev + x - 1;
        let c10 = ofs_prev + x;
        let c01 = ofs_cur + x - 1;
        let c11 = ofs_cur + x;
        self.init_cell(&mut cells[c11], x, y, part_f);

        if x == szx - 1 || y == szy - 1 {
          if cells[c11].corner_part != 0 {
            panic!("Fail aabb in position {:?}", cells[c11].corner);
          }
        }

        fill_side_mids!(c01, v_pz, c11, v_mz);
        fill_side_mids!(c10, v_zp, c11, v_zm);

        // fill cell here
        let center = self.center_of_cell(x, y);
        let center_part = part_f(center);

        macro_rules! fill_center_mid {
          ($ci: expr, $dst1: ident, $dst2: ident) => {
            let c = &cells[$ci];
            fill_mids!(center_part, center, $dst1, c.corner_part, c.corner, $dst2);
          };
        }

        let mut v_mmi = BAD_VERTEX;
        let mut v_mmo = BAD_VERTEX;
        let mut v_mpi = BAD_VERTEX;
        let mut v_mpo = BAD_VERTEX;
        let mut v_pmi = BAD_VERTEX;
        let mut v_pmo = BAD_VERTEX;
        let mut v_ppi = BAD_VERTEX;
        let mut v_ppo = BAD_VERTEX;

        fill_center_mid!(c00, v_mmi, v_mmo);
        fill_center_mid!(c01, v_mpi, v_mpo);
        fill_center_mid!(c10, v_pmi, v_pmo);
        fill_center_mid!(c11, v_ppi, v_ppo);

        result.fill_t(
          center_part,
          cells[c01].corner_part,
          cells[c00].corner_part,
          v_mpi,
          v_mpo,
          v_mmi,
          v_mmo,
          cells[c01].v_zm,
          cells[c00].v_zp,
        );

        result.fill_t(
          center_part,
          cells[c00].corner_part,
          cells[c10].corner_part,
          v_mmi,
          v_mmo,
          v_pmi,
          v_pmo,
          cells[c00].v_pz,
          cells[c10].v_mz,
        );

        result.fill_t(
          center_part,
          cells[c11].corner_part,
          cells[c01].corner_part,
          v_ppi,
          v_ppo,
          v_mpi,
          v_mpo,
          cells[c11].v_mz,
          cells[c01].v_pz,
        );

        result.fill_t(
          center_part,
          cells[c10].corner_part,
          cells[c11].corner_part,
          v_pmi,
          v_pmo,
          v_ppi,
          v_ppo,
          cells[c10].v_zp,
          cells[c11].v_zm,
        );

        result.ord_buffer.extend(&[cells[c00].v_zp, cells[c01].v_zm], true);
        result.ord_buffer.extend(&[v_mmo, v_mmi, v_mpi, v_mpo], true);
        result.ord_buffer.extend(&[v_pmo, v_pmi, v_ppi, v_ppo], true);
        if x == szx - 1 {
          result.ord_buffer.extend(&[cells[c10].v_zp, cells[c11].v_zm], true);
        }
      }

      result.ord_buffer.sort_mid(&result.vertices);
      result.ord_buffer.clear_l();
      for x in (1..szx).rev() {
        result.ord_buffer.extend(&[cells[ofs_cur + x].v_mz, cells[ofs_cur + x - 1].v_pz], false);
      }
      for x in 0..szx {
        cells[ofs_prev + x] = ContourCell::new();
      }
      (ofs_prev, ofs_cur) = (ofs_cur, ofs_prev);
    }

    result.to_generator_result()
  }
}
