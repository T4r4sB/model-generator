use crate::bit_buffer::*;
use crate::points2d::*;
use dxf::entities::*;
use dxf::objects::*;
use dxf::Drawing;
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
  pub points: Vec<usize>,
}

pub type Triangle = [usize; 3];

pub struct Triangulation {
  triangles: Vec<Triangle>,
}

#[derive(Debug, Clone, Default)]
pub struct Contours {
  contours: Vec<Contour>,
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
}

impl Contours {
  pub fn save_to_dxf(&self, path: &std::path::Path, points: &[Point]) -> Result<(), String> {
    self.save_to_dxf_with_grid(path, points, false)
  }

  pub fn save_to_dxf_with_grid(
    &self,
    path: &std::path::Path,
    points: &[Point],
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
        aabb = aabb.with(points[p]);
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
        let v = point2d_to_dxf(points[contour.points[i] as usize]);
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
}

impl Triangulation {
  pub fn to_contours(&self) -> Contours {
    Contours { contours: self.triangles.iter().map(|t| Contour { points: t.to_vec() }).collect() }
  }

  pub fn extrude(&self, topology: &ContourTopology, width: f32) -> crate::model::Model {
    let mut vertices = Vec::with_capacity(topology.vertices.len() * 2);
    for &p in &topology.vertices {
      vertices.push(crate::points3d::Point { x: p.x, y: p.y, z: 0.0 });
      vertices.push(crate::points3d::Point { x: p.x, y: p.y, z: width });
    }

    let mut triangles = Vec::with_capacity(topology.vertices.len() * 4);
    /*
    for c in &topology.contours {
      let mut prev = c.points[c.points.len() - 1];
      for &p in &c.points {
        triangles.push([prev * 2, p * 2, p * 2 + 1]);
        triangles.push([prev * 2, p * 2 + 1, prev * 2 + 1]);
        prev = p;
      }
    }*/

    for &t in &self.triangles {
      triangles.push([t[2] as u32 * 2, t[1] as u32 * 2, t[0] as u32 * 2]);
      triangles.push([t[0] as u32 * 2 + 1, t[1] as u32 * 2 + 1, t[2] as u32 * 2 + 1]);
    }

    crate::model::Model { vertices, triangles }
  }
}

#[derive(Debug, Clone, Copy)]
struct TopologyEdge {
  begin: usize,
  end: usize,
}

#[derive(Debug)]
pub struct ContourTopology {
  vertices: Vec<Point>, // sorted from low to top by merge-sort
  edges: FxHashMap<PartIndex, Vec<TopologyEdge>>, // sorted from left to right by construction
}

impl ContourTopology {
  fn new() -> Self {
    Self { vertices: Vec::new(), edges: FxHashMap::default() }
  }

  fn regroup_chains(&mut self) {
    let mut adj_e = Vec::new();
    adj_e.resize(self.vertices.len(), (0, 0));
    let mut sorted_v = Vec::new();
    let mut e2chain = Vec::new();
    #[derive(Debug, Copy, Clone)]
    struct ChainInfo {
      first_e: usize,
      last_e: usize,
      next_chain: usize,
    };
    let mut chains = Vec::<ChainInfo>::new();
    for (_, edges) in &mut self.edges {
      let mut sorted_e = BitBuffer::new(edges.len());
      for (i, e) in edges.iter().enumerate() {
        adj_e[e.begin].1 = i;
        adj_e[e.end].0 = i;
        sorted_v.push(e.begin);
      }
      sorted_v.sort();
      chains.push(ChainInfo { first_e: BAD_EDGE, last_e: BAD_EDGE, next_chain: BAD_EDGE });
      e2chain.resize(edges.len(), usize::MAX);
      for &v in &sorted_v {
        let adj = adj_e[v];
        let e_from = edges[adj.0];
        let e_to = edges[adj.1];
        if e_from.begin > v {
          if e_to.end > v {
            // new chains
            let (l, r) = if adj.0 < adj.1 { (adj.0, adj.1) } else { (adj.1, adj.0) };
            let prev_e = sorted_e.upper_bound_included(l, BAD_EDGE);
            let prev_chain = if prev_e == BAD_EDGE { 0 } else { e2chain[prev_e] };
            let cl = chains.len();
            e2chain[l] = cl;
            e2chain[r] = cl + 1;
            chains.push(ChainInfo { first_e: l, last_e: BAD_EDGE, next_chain: cl + 1 });
            chains.push(ChainInfo {
              first_e: r,
              last_e: BAD_EDGE,
              next_chain: chains[prev_chain].next_chain,
            });
            chains[prev_chain].next_chain = cl;
            sorted_e.put_number(l);
            sorted_e.put_number(r);
          } else {
            sorted_e.remove_number(adj.1);
            sorted_e.put_number(adj.0);
            e2chain[adj.0] = e2chain[adj.1];
          }
        } else {
          if e_to.end > v {
            sorted_e.remove_number(adj.0);
            sorted_e.put_number(adj.1);
            e2chain[adj.1] = e2chain[adj.0];
          } else {
            // end chains
            sorted_e.remove_number(adj.0);
            sorted_e.remove_number(adj.1);
            chains[e2chain[adj.0]].last_e = adj.0;
            chains[e2chain[adj.1]].last_e = adj.1;
          }
        }
      }
      let mut cn = chains[0].next_chain;
      let mut fixed = Vec::new();
      while cn != BAD_EDGE {
        let chain = chains[cn];
        let mut edge_index = chain.first_e;
        loop {
          let e = edges[edge_index];
          fixed.push(e);
          if edge_index == chain.last_e {
            break;
          }
          edge_index = if e.begin < e.end { adj_e[e.end].1 } else { adj_e[e.begin].0 };
        }
        cn = chain.next_chain;
      }
      *edges = fixed;

      e2chain.clear();
      chains.clear();
      sorted_v.clear();
      sorted_e.clear();
    }
  }

  fn fix_by_ord(&mut self, ord2v: &[usize]) {
    let mut sorted_vertices = Vec::with_capacity(self.vertices.len());
    for &i in ord2v {
      sorted_vertices.push(self.vertices[i]);
    }
    self.vertices = sorted_vertices;

    let mut v2ord = Vec::new();
    v2ord.resize(ord2v.len(), 0);
    for (i, &v) in ord2v.iter().enumerate() {
      v2ord[v] = i;
    }
    for (_, e) in &mut self.edges {
      for e in e {
        e.begin = v2ord[e.begin];
        e.end = v2ord[e.end];
      }
    }
  }

  fn retain_edges_by(edges: &mut Vec<TopologyEdge>, condition: impl Fn(usize) -> bool) {
    let mut j = 0;
    for i in 0..edges.len() {
      if condition(i) {
        edges[j] = edges[i];
        j += 1;
      }
    }
    edges.truncate(j);
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

    for (_, edges) in &mut self.edges {
      for e in edges {
        e.begin = fix[e.begin];
        e.end = fix[e.end];
      }
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

    for (&part, edges) in &mut self.edges {
      for (i, e) in edges.iter().enumerate() {
        next[e.begin] = i;
      }
      visited.resize(edges.len(), EdgeResult::UNKNOWN);
      for i in 0..edges.len() {
        if visited[i] != EdgeResult::UNKNOWN {
          continue;
        }
        let mut sq = 0.0;
        let mut cur = i;
        loop {
          let e = edges[cur];
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
          skipped_v[edges[e].begin] = bad;
        }

        in_process.clear();
      }

      Self::retain_edges_by(edges, |i| visited[i] == EdgeResult::GOOD);
      visited.clear();
    }
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
    let mut skipped_edges = Vec::new();

    for (_, edges) in &mut self.edges {
      skipped_edges.resize(edges.len(), false);

      for (i, e) in edges.iter().enumerate() {
        next[e.begin] = VInfo { edge_from: i, next_v: e.end, skipped: false };
      }

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
            assert!(!skipped_edges[j]);
            let new_end_index = edges[j].end;
            let new_end = self.vertices[new_end_index];
            let mut skip = next[e.begin].next_v;
            while skip != new_end_index {
              if dist_pl(self.vertices[skip], new_begin, new_end) > treshhold {
                continue 'each_edge;
              }
              skip = next[skip].next_v;
            }

            if e.begin < e.end && edges[j].end < e.begin
              || e.begin > e.end && edges[j].end > e.begin
            {
              edges[j].begin = e.begin;
              skipped_edges[i] = true;
              next[e.begin].edge_from = j;
            } else {
              edges[i].end = new_end_index;
              skipped_edges[j] = true;
            }
            next[e.end].skipped = true;
            finished = false;
          }
        }
      }

      Self::retain_edges_by(edges, |i| !skipped_edges[i]);
    }

    self.retain_vertices_by(|i| !next[i].skipped);
  }

  pub fn get(&self) -> FxHashMap<PartIndex, Contours> {
    let mut result = FxHashMap::<PartIndex, Contours>::default();

    let mut next = Vec::new();
    next.resize(self.vertices.len(), BAD_VERTEX);

    for (&part, edges) in &self.edges {
      let mut contours = Vec::new();
      for (i, e) in edges.iter().enumerate() {
        next[e.begin] = i;
      }
      for i in 0..edges.len() {
        let mut c = i;
        let mut contour = Contour::new();
        loop {
          let e = &edges[c];
          let n = &mut next[e.end];
          if *n == BAD_VERTEX {
            if !contour.points.is_empty() {
              contours.push(contour);
            }
            break;
          }
          contour.points.push(e.end);
          c = *n;
          *n = BAD_VERTEX;
        }
      }
      result.insert(part, Contours { contours });
    }

    result
  }

  pub fn get_points(&self) -> &[Point] {
    &self.vertices
  }

  pub fn get_triangles(&self) -> FxHashMap<PartIndex, Triangulation> {
    let mut result = FxHashMap::default();
    let mut bit_buf = BitBuffer::new(self.vertices.len());
    let mut loc_buf = BitBuffer::new(self.vertices.len());
    let mut vis = Vec::new();

    for (&part, e) in &self.edges {
      let mut triangles = Vec::new();

      let mut handle_range =
        |e: &TopologyEdge, rev: bool, vis: &mut Vec<usize>, bit_buf: &mut BitBuffer| {
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
                triangles.push([prev, next, l]);
              } else {
                triangles.push([prev, l, next]);
              }
            }
            loc_buf.remove_number(e.begin);
            loc_buf.remove_number(e.end);
          }
          vis.clear();
        };

      for e in e {
        handle_range(e, false, &mut vis, &mut bit_buf);
      }
      bit_buf.clear();
      for e in e.iter().rev() {
        handle_range(e, true, &mut vis, &mut bit_buf);
      }
      bit_buf.clear();

      result.insert(part, Triangulation { triangles });
    }

    result
  }
}
struct OrdBuffer {
  v: Vec<usize>,
  l: Vec<usize>,
}

impl OrdBuffer {
  fn new() -> Self {
    Self { v: Vec::new(), l: Vec::new() }
  }

  fn clear_l(&mut self) {
    self.l.clear();
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

  fn sort_mid(&mut self, vertex_cursor: usize, vertices: &[Point]) {
    let mut tmp = Vec::new();
    let mut dst0 = &mut self.v[vertex_cursor..];
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
      for (dst, src) in self.v[vertex_cursor..].iter_mut().zip(tmp) {
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

#[derive(Debug)]
pub struct ContourCreator {
  aabb: AABB,
  scale: f32,
  size_x: usize,
  size_y: usize,
  tries: usize,
  topology: ContourTopology,
}

impl ContourCreator {
  pub fn new(aabb: AABB, scale: f32, tries: usize) -> Self {
    let size_x = ((aabb.x2 - aabb.x1) / scale).ceil() as usize + 1;
    let size_y = ((aabb.y2 - aabb.y1) / scale).ceil() as usize + 1;
    Self { aabb, scale, size_x, size_y, tries, topology: ContourTopology::new() }
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

  fn index_of_new_point(&mut self, pt: Point, part: PartIndex) -> usize {
    let result = self.topology.vertices.len();
    self.topology.vertices.push(pt);
    result
  }

  fn fill_ti(&mut self, i1: PartIndex, i2: PartIndex, i3: PartIndex, p12: usize, p13: usize) {
    if i1 != 0 && i1 != i2 && i1 != i3 {
      assert!(p12 != BAD_VERTEX);
      assert!(p13 != BAD_VERTEX);
      self.topology.edges.entry(i1).or_default().push(TopologyEdge { begin: p12, end: p13 });
    }
  }

  fn fill_to(&mut self, i1: PartIndex, i2: PartIndex, i3: PartIndex, p21: usize, p31: usize) {
    if i1 != i2 && i2 != 0 && i2 == i3 {
      assert!(p21 != BAD_VERTEX);
      assert!(p31 != BAD_VERTEX);
      self.topology.edges.entry(i2).or_default().push(TopologyEdge { begin: p31, end: p21 });
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

  pub fn make_contour(mut self, part_f: &dyn Fn(Point) -> PartIndex) -> ContourTopology {
    if self.size_x == 0 || self.size_y == 0 {
      return ContourTopology::new();
    }

    let mut cells = vec![ContourCell::new(); self.size_x * self.size_y];
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
              $target1 = self.index_of_new_point(pt1, part_index1);
              $target2 = self.index_of_new_point(pt2, part_index2);
            } else {
              let pt = find_root(part_f, $point1, $point2, part_index1, self.tries);
              $target1 = self.index_of_new_point(pt, part_index1);
            }
          } else {
            if part_index2 != 0 {
              let pt = find_root(part_f, $point2, $point1, part_index2, self.tries);
              $target2 = self.index_of_new_point(pt, part_index2);
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

    let mut ord2v = OrdBuffer::new();

    for x in 1..szx {
      self.init_cell(&mut cells[x], x, 0, part_f);

      if cells[x].corner_part != 0 {
        panic!("Fail aabb in position {:?}", cells[x].corner);
      }

      fill_side_mids!(x - 1, v_pz, x, v_mz);
    }

    for x in (1..szx).rev() {
      ord2v.extend(&[cells[x].v_mz, cells[x - 1].v_pz], false);
    }

    for y in 1..szy {
      let c = szx * y;
      let c10 = c - szx;
      let c11 = c;

      let edge_cursor = self.topology.edges.len();
      let vertex_cursor = self.topology.vertices.len();

      assert_eq!(vertex_cursor, ord2v.v.len());

      self.init_cell(&mut cells[c11], 0, y, part_f);
      fill_side_mids!(c10, v_zp, c11, v_zm);

      if cells[c11].corner_part != 0 {
        panic!("Fail aabb in position {:?}", cells[c11].corner);
      }

      for x in 1..szx {
        let c = c + x;
        let c00 = c - 1 - szx;
        let c10 = c - szx;
        let c01 = c - 1;
        let c11 = c;
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

        self.fill_t(
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

        self.fill_t(
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

        self.fill_t(
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

        self.fill_t(
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

        ord2v.extend(&[cells[c00].v_zp, cells[c01].v_zm], true);
        ord2v.extend(&[v_mmo, v_mmi, v_mpi, v_mpo], true);
        ord2v.extend(&[v_pmo, v_pmi, v_ppi, v_ppo], true);
        if x == szx - 1 {
          ord2v.extend(&[cells[c10].v_zp, cells[c11].v_zm], true);
        }
      }

      ord2v.sort_mid(vertex_cursor, &self.topology.vertices);
      ord2v.clear_l();
      for x in (1..szx).rev() {
        ord2v.extend(&[cells[c + x].v_mz, cells[c + x - 1].v_pz], false);
      }
    }

    self.topology.fix_by_ord(&ord2v.v);
    self.topology.regroup_chains();
    self.topology
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
    assert!(c.contains_inner(&POINTS, 8));
    assert!(!c.contains_inner(&POINTS, 4));
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
