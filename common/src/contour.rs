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
  pub points: Vec<usize>,
}

pub type Triangle = [usize; 3];

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

#[derive(Debug, Clone)]
struct TopologyEdge {
  part: PartIndex,
  begin: usize,
  end: usize,
  p_right: Vec<usize>,
  p_left: Vec<usize>,
  ascending: bool,
}

#[derive(Debug, Clone, Copy)]
struct TopologyVertex {
  part: PartIndex,
  edge_in: usize,
  edge_out: usize,
}

#[derive(Debug)]
pub struct ContourTopology {
  vertices: Vec<TopologyVertex>,
  points: Vec<Point>,
  edges: Vec<TopologyEdge>,
}

impl ContourTopology {
  fn new() -> Self {
    Self { vertices: Vec::new(), points: Vec::new(), edges: Vec::new() }
  }

  fn validate(&self) {
    let mut l_checked = fxhash::FxHashSet::default();
    let mut r_checked = fxhash::FxHashSet::default();
    for e in &self.edges {
      if e.ascending {
        for &l in &e.p_left {
          assert!(self.vertices[l].part == e.part);
          assert!(l_checked.insert(l));
        }
      } else {
        for &r in &e.p_right {
          assert!(self.vertices[r].part == e.part);
          assert!(r_checked.insert(r));
        }
      }
    }
  }

  pub fn optimize(&mut self, treshhold: f32) {
    enum SkippedEdge {
      Skipped,
      Points(Vec<usize>),
    }
    let mut skips = Vec::<SkippedEdge>::new();
    skips.resize_with(self.edges.len(), || SkippedEdge::Points(Vec::new()));
    'each_edge: for i in 0..self.edges.len() {
      match &skips[i] {
        SkippedEdge::Skipped => {}
        SkippedEdge::Points(p1) => {
          let e1 = &self.edges[i];
          let mid = e1.end;
          let e2i = self.vertices[mid].edge_out;
          let e2 = &self.edges[e2i];
          let new_begin = e1.begin;
          let new_end = e1.end;
          let p_begin = self.points[new_begin];
          let p_end = self.points[new_end];
          match &skips[e2i] {
            SkippedEdge::Skipped => panic!("Valid edge cant be continued by invalid"),
            SkippedEdge::Points(p2) => {
              for p in p1.iter().copied().chain([mid]).chain(p2.iter().copied()) {
                if dist_pl(self.points[p], p_begin, p_end) > treshhold {
                  continue 'each_edge;
                }
              }
              if e1.ascending && e2.ascending {
                for l in e1.p_left.iter().copied().chain(e2.p_left.iter().copied()) {
                  let p = self.points[l];
                  if cross(p_begin - p, p_end - p) < 0.0 {
                    continue 'each_edge;
                  }
                }
                for r in e1.p_right.iter().copied().chain(e2.p_right.iter().copied()) {
                  let p = self.points[r];
                  if cross(p_begin - p, p_end - p) > 0.0 {
                    continue 'each_edge;
                  }
                }

              //  todo!()
              } else if !e1.ascending && !e2.ascending {
                for l in e1.p_left.iter().copied().chain(e2.p_left.iter().copied()) {
                  let p = self.points[l];
                  if cross(p_begin - p, p_end - p) > 0.0 {
                    continue 'each_edge;
                  }
                }
                for r in e1.p_right.iter().copied().chain(e2.p_right.iter().copied()) {
                  let p = self.points[r];
                  if cross(p_begin - p, p_end - p) < 0.0 {
                    continue 'each_edge;
                  }
                }

               // todo!()
              } else if e1.ascending && !e2.ascending {
                if i > e2i {
                  // this case need deep rearrangement of topology, so we skip
                  continue 'each_edge;
                } else {
                  if e2.p_right.is_empty() {
                    assert!(*e1.p_left.last().unwrap() == new_end);
                    for &l in &e1.p_left[..e1.p_left.len() - 1] {
                      let p = self.points[l];
                      if cross(p_begin - p, p_end - p) > 0.0 {
                        continue 'each_edge;
                      }
                    }

                 //   todo!()
                  } else if e1.p_left.is_empty() {
                    assert!(*e1.p_right.last().unwrap() == new_end);
                    for &r in &e1.p_right[..e1.p_right.len() - 1] {
                      let p = self.points[r];
                      if cross(p_begin - p, p_end - p) > 0.0 {
                        continue 'each_edge;
                      }
                    }
                //    todo!()
                  } else {
                      continue 'each_edge;
                  }
                }
              } else if !e1.ascending && e2.ascending {
                if i < e2i {
                  // this case need deep rearrangement of topology, so we skip
                  continue 'each_edge;
                } else {
                  if e1.p_right.is_empty() {
                    assert!(e2.p_left[0] == new_end);
                    for &l in &e2.p_left[1..] {
                      let p = self.points[l];
                      if cross(p_begin - p, p_end - p) > 0.0 {
                        continue 'each_edge;
                      }
                    }

                //    todo!()
                  } else if e2.p_left.is_empty() {
                    assert!(e1.p_right[0] == new_begin);
                    for &r in &e1.p_right[1..] {
                      let p = self.points[r];
                      if cross(p_begin - p, p_end - p) > 0.0 {
                        continue 'each_edge;
                      }
                    }

                //    todo!()
                  } else {
                        continue 'each_edge;
                  }
                }
              }
            }
          }
        }
      }
    }
  }

  pub fn get_contours(&self) -> FxHashMap<PartIndex, Contours> {
    let mut visited = Vec::new();
    let mut result = FxHashMap::<PartIndex, Contours>::default();
    visited.resize(self.edges.len(), false);
    for i in 0..self.edges.len() {
      if visited[i] {
        continue;
      }
      let mut contour = Contour::new();
      let part = self.edges[i].part;
      let mut cur = i;
      loop {
        visited[cur] = true;
        let e = &self.edges[cur];
        contour.points.push(e.end);
        cur = self.vertices[e.end].edge_out;
        if visited[cur] {
          break;
        }
      }
      result.entry(part).or_default().contours.push(contour);
    }
    result
  }

  pub fn get_points(&self) -> &[Point] {
    &self.points
  }

  pub fn get_triangles(&self) -> FxHashMap<PartIndex, Vec<Triangle>> {
    todo!()
  }
}

#[derive(Debug, Clone)]
struct BitBuffer {
  elements: Vec<usize>,
}

impl BitBuffer {
  fn new() -> Self {
    Self { elements: Vec::new() }
  }

  fn clear(&mut self) {
    self.elements.clear();
  }

  fn resize(&mut self, new_len: usize) {
    let bsz = usize::BITS as usize;
    let new_len = new_len.div_ceil(bsz);
    self.elements.resize(new_len, 0);
  }

  fn put_range(&mut self, mut begin: usize, mut end: usize) -> Vec<usize> {
    if begin > end {
      (begin, end) = (end, begin);
    }

    let mask = usize::BITS as usize - 1;
    let bsz = usize::BITS as usize;
    let mut result = Vec::new();
    let b = begin / bsz;
    let bn = begin & mask;
    let e = end / bsz;
    let en = end & mask;

    let mut push_bits = |pat: &mut usize, mask: usize, new_bits: usize, offset: usize| {
      let mut p = *pat & mask;
      while p != 0 {
        let bit = p.trailing_zeros() as usize;
        result.push(offset + bit);
        p -= 1 << bit;
      }
      *pat = *pat & !mask | new_bits;
    };

    if b == e {
      let be_mask = (1 << en) - (1 << (bn + 1));
      push_bits(&mut self.elements[b], be_mask, 1 << bn | 1 << en, b * bsz);
      self.elements[b] &= !be_mask;
      self.elements[b] |= 1 << bn | 1 << en;
    } else {
      let b_mask = (usize::MAX << bn) << 1;
      push_bits(&mut self.elements[b], b_mask, 1 << bn, b * bsz);
      for cb in b + 1..e {
        push_bits(&mut self.elements[cb], usize::MAX, 0, cb * bsz);
      }
      let e_mask = (1 << en) - 1;
      push_bits(&mut self.elements[e], e_mask, 1 << en, e * bsz);
    }

    result
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ContourCell {
  corner_part_index: PartIndex,
  corner: Point,
  corner_point_index: usize,
  v_mz: usize,
  v_pz: usize,
  v_zm: usize,
  v_zp: usize,
}

impl ContourCell {
  fn new() -> Self {
    Self {
      corner_part_index: 0,
      corner: Point { x: 0.0, y: 0.0 },
      corner_point_index: BAD_VERTEX,
      v_mz: BAD_VERTEX,
      v_pz: BAD_VERTEX,
      v_zm: BAD_VERTEX,
      v_zp: BAD_VERTEX,
    }
  }
}

#[derive(Debug)]
struct SortedBuffer {
  vertices: Vec<usize>, // indices
  edges: Vec<usize>,
}

impl SortedBuffer {
  fn merge(&mut self, other_v: &[usize], other_e: &[(usize, usize)]) {
    todo!()
  }
}

#[derive(Debug)]
struct TmpBuf {
  edge_by_cells: Vec<usize>, // buffer of count egdes in cells for sorting
  bottom_ord_rev: Vec<usize>,
  mid_ord: Vec<usize>,  // by sorted fragments
  mid_ord2: Vec<usize>, // second buffer for merge-sort
  mid_len: Vec<usize>,  // buffer or slice lenghts for merge-sort
  top_ord_rev: Vec<usize>,
  ord2v: Vec<usize>, // ord -> vertex index
  v2ord: Vec<usize>, // vertex index -> ord
  ranges: BitBuffer,
}

impl TmpBuf {
  fn new() -> Self {
    Self {
      edge_by_cells: Vec::new(),
      bottom_ord_rev: Vec::new(),
      mid_ord: Vec::new(),
      mid_ord2: Vec::new(),
      mid_len: Vec::new(),
      top_ord_rev: Vec::new(),
      ord2v: Vec::new(),
      v2ord: Vec::new(),
      ranges: BitBuffer::new(),
    }
  }

  fn clear(&mut self) {
    self.edge_by_cells.clear();
    self.bottom_ord_rev.clear();
    self.mid_ord.clear();
    self.mid_ord2.clear();
    self.mid_len.clear();
    self.ord2v.clear();
    // v2ord keep between iters because numeration for all vertices
    self.top_ord_rev.clear();
    self.ranges.clear();
  }

  fn extend_bot_rev(&mut self, v: &[usize]) {
    for &v in v {
      if v != BAD_VERTEX {
        self.bottom_ord_rev.push(v);
      }
    }
  }

  fn extend_top_rev(&mut self, v: &[usize]) {
    for &v in v {
      if v != BAD_VERTEX {
        self.top_ord_rev.push(v);
      }
    }
  }

  fn extend_mid(&mut self, v: &[usize]) {
    let mut l = 0;
    for &v in v {
      if v != BAD_VERTEX {
        self.mid_ord.push(v);
        l += 1;
      }
    }
    if l > 0 {
      self.mid_len.push(l);
    }
  }

  fn sort_mid(&mut self, points: &[Point]) {
    while self.mid_len.len() > 1 {
      let mut i = 0;
      let mut cursor = 0;
      self.mid_ord2.clear();
      while i * 2 < self.mid_len.len() {
        let mut l1 = self.mid_len[i * 2];
        let mut l2 = self.mid_len.get(i * 2 + 1).copied().unwrap_or(0);
        let mut i1 = 0;
        let mut i2 = 0;
        loop {
          if i1 == l1 {
            if i2 == l2 {
              break;
            } else {
              self.mid_ord2.push(self.mid_ord[cursor + l1 + i2]);
              i2 += 1;
            }
          } else {
            if i2 == l2 {
              self.mid_ord2.push(self.mid_ord[cursor + i1]);
              i1 += 1;
            } else {
              let m1 = self.mid_ord[cursor + i1];
              let m2 = self.mid_ord[cursor + l1 + i2];
              if points[m1].y <= points[m2].y {
                self.mid_ord2.push(m1);
                i1 += 1;
              } else {
                self.mid_ord2.push(m2);
                i2 += 1;
              }
            }
          }
        }
        cursor += l1 + l2;
        self.mid_len[i] = l1 + l2;
        i += 1;
      }
      std::mem::swap(&mut self.mid_ord, &mut self.mid_ord2);
      self.mid_len.truncate((self.mid_len.len() + 1) / 2);
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
  tmp_buf: TmpBuf,
}

impl ContourCreator {
  pub fn new(aabb: AABB, scale: f32, tries: usize) -> Self {
    let size_x = ((aabb.x2 - aabb.x1) / scale).ceil() as usize + 1;
    let size_y = ((aabb.y2 - aabb.y1) / scale).ceil() as usize + 1;
    Self {
      aabb,
      scale,
      size_x,
      size_y,
      tries,
      topology: ContourTopology::new(),
      tmp_buf: TmpBuf::new(),
    }
  }

  fn order_last_layer_points(&mut self) -> usize {
    self.tmp_buf.sort_mid(&self.topology.points);
    self.tmp_buf.v2ord.resize(self.topology.points.len(), 0);
    let mut ord = 0;
    for &p in (self.tmp_buf.bottom_ord_rev.iter().rev())
      .chain(self.tmp_buf.mid_ord.iter())
      .chain(self.tmp_buf.top_ord_rev.iter().rev())
    {
      self.tmp_buf.v2ord[p] = ord;
      self.tmp_buf.ord2v.push(p);
      ord += 1;
    }
    return ord;
  }

  fn make_edge_ascending_flags(&mut self, edge_cursor: usize) {
    for e in &mut self.topology.edges[edge_cursor..] {
      e.ascending = self.tmp_buf.v2ord[e.begin] < self.tmp_buf.v2ord[e.end];
    }
  }

  fn sort_last_layer_edges(&mut self, mut edge_cursor: usize) {
    for &l in &self.tmp_buf.edge_by_cells {
      for t in 1..l {
        for i in edge_cursor..edge_cursor + l - t {
          if self.topology.edges[i].ascending && !self.topology.edges[i + 1].ascending {
            self.topology.edges.swap(i - 1, i);
          }
        }
      }

      edge_cursor += l;
    }
  }

  fn fill_edges_p_left(&mut self, edge_cursor: usize, ords: usize) {
    self.tmp_buf.ranges.clear();
    self.tmp_buf.ranges.resize(ords);
    //   let mut used_ords = fxhash::FxHashSet::default();
    //  let mut used_v = fxhash::FxHashSet::default();
    for e in &mut self.topology.edges[edge_cursor..] {
      let mut range =
        self.tmp_buf.ranges.put_range(self.tmp_buf.v2ord[e.begin], self.tmp_buf.v2ord[e.end]);
      for r in &mut range {
        //   assert!(used_ords.insert(*r));
        *r = self.tmp_buf.ord2v[*r];
        //    assert!(used_v.insert(*r));
      }
      e.p_left = range;
    }
  }

  fn fill_edges_p_right(&mut self, edge_cursor: usize, ords: usize) {
    self.tmp_buf.ranges.clear();
    self.tmp_buf.ranges.resize(ords);
    // let mut used_ords = fxhash::FxHashSet::default();
    //  let mut used_v = fxhash::FxHashSet::default();
    for e in self.topology.edges[edge_cursor..].iter_mut().rev() {
      let mut range =
        self.tmp_buf.ranges.put_range(self.tmp_buf.v2ord[e.begin], self.tmp_buf.v2ord[e.end]);

      for r in &mut range {
        // assert!(used_ords.insert(*r));
        *r = self.tmp_buf.ord2v[*r];
        // assert!(used_v.insert(*r));
      }
      e.p_right = range;
    }
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
    cell.corner_part_index = part_f(cell.corner);
    cell.corner_point_index = self.index_of_new_point(cell.corner, cell.corner_part_index);
  }

  fn index_of_new_point(&mut self, pt: Point, part: PartIndex) -> usize {
    let result = self.topology.points.len();
    self.topology.points.push(pt);
    self.topology.vertices.push(TopologyVertex { edge_in: BAD_EDGE, edge_out: BAD_EDGE, part });
    result
  }

  fn fill_ti(
    &mut self,
    i1: PartIndex,
    i2: PartIndex,
    i3: PartIndex,
    p12: usize,
    p13: usize,
  ) -> bool {
    if i1 != 0 && i1 != i2 && i1 != i3 {
      assert!(p12 != BAD_VERTEX);
      assert!(p13 != BAD_VERTEX);
      self.topology.edges.push(TopologyEdge {
        begin: p12,
        end: p13,
        part: i1,
        p_right: Vec::new(),
        p_left: Vec::new(),
        ascending: false,
      });
      self.topology.vertices[p12].edge_out = self.topology.edges.len() - 1;
      self.topology.vertices[p13].edge_in = self.topology.edges.len() - 1;
      return true;
    }
    false
  }

  fn fill_to(
    &mut self,
    i1: PartIndex,
    i2: PartIndex,
    i3: PartIndex,
    p21: usize,
    p31: usize,
  ) -> bool {
    if i1 != i2 && i2 != 0 && i2 == i3 {
      assert!(p21 != BAD_VERTEX);
      assert!(p31 != BAD_VERTEX);
      self.topology.edges.push(TopologyEdge {
        begin: p31,
        end: p21,
        part: i2,
        p_right: Vec::new(),
        p_left: Vec::new(),
        ascending: false,
      });
      self.topology.vertices[p31].edge_out = self.topology.edges.len() - 1;
      self.topology.vertices[p21].edge_in = self.topology.edges.len() - 1;
      return true;
    }
    false
  }

  fn fill_t(
    &mut self,
    i1: PartIndex,
    i2: PartIndex,
    i3: PartIndex,
    p1: usize,
    p2: usize,
    p3: usize,
    p12: usize,
    p21: usize,
    p13: usize,
    p31: usize,
    p23: usize,
    p32: usize,
  ) {
    let mut ec = 0;
    ec += self.fill_ti(i1, i2, i3, p12, p13) as usize;
    ec += self.fill_to(i1, i2, i3, p21, p31) as usize;
    ec += self.fill_ti(i2, i3, i1, p23, p21) as usize;
    ec += self.fill_to(i2, i3, i1, p32, p12) as usize;
    ec += self.fill_ti(i3, i1, i2, p31, p32) as usize;
    ec += self.fill_to(i3, i1, i2, p13, p23) as usize;
    if ec > 0 {
      self.tmp_buf.edge_by_cells.push(ec);
    }
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
          c1.corner_part_index,
          c1.corner,
          cells[$ci1].$target_field1,
          c2.corner_part_index,
          c2.corner,
          cells[$ci2].$target_field2
        );
      };
    }

    self.init_cell(&mut cells[0], 0, 0, part_f);
    if cells[0].corner_part_index != 0 {
      panic!("Fail aabb in position {:?}", cells[0].corner);
    }

    for x in 1..szx {
      self.init_cell(&mut cells[x], x, 0, part_f);

      if cells[x].corner_part_index != 0 {
        panic!("Fail aabb in position {:?}", cells[x].corner);
      }

      fill_side_mids!(x - 1, v_pz, x, v_mz);
    }

    for y in 1..szy {
      let c = szx * y;
      let c10 = c - szx;
      let c11 = c;

      self.tmp_buf.clear();
      let edge_cursor = self.topology.edges.len();

      self.init_cell(&mut cells[c11], 0, y, part_f);
      fill_side_mids!(c10, v_zp, c11, v_zm);

      if cells[c11].corner_part_index != 0 {
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
          if cells[c11].corner_part_index != 0 {
            panic!("Fail aabb in position {:?}", cells[c11].corner);
          }
        }

        fill_side_mids!(c01, v_pz, c11, v_mz);
        fill_side_mids!(c10, v_zp, c11, v_zm);

        // fill cell here
        let center = self.center_of_cell(x, y);
        let center_part_index = part_f(center);
        let center_point_index = self.index_of_new_point(center, center_part_index);

        macro_rules! fill_center_mid {
          ($ci: expr, $dst1: ident, $dst2: ident) => {
            let c = &cells[$ci];
            fill_mids!(center_part_index, center, $dst1, c.corner_part_index, c.corner, $dst2);
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
          center_part_index,
          cells[c01].corner_part_index,
          cells[c00].corner_part_index,
          center_point_index,
          cells[c01].corner_point_index,
          cells[c00].corner_point_index,
          v_mpi,
          v_mpo,
          v_mmi,
          v_mmo,
          cells[c01].v_zm,
          cells[c00].v_zp,
        );

        self.fill_t(
          center_part_index,
          cells[c00].corner_part_index,
          cells[c10].corner_part_index,
          center_point_index,
          cells[c00].corner_point_index,
          cells[c10].corner_point_index,
          v_mmi,
          v_mmo,
          v_pmi,
          v_pmo,
          cells[c00].v_pz,
          cells[c10].v_mz,
        );

        self.fill_t(
          center_part_index,
          cells[c11].corner_part_index,
          cells[c01].corner_part_index,
          center_point_index,
          cells[c11].corner_point_index,
          cells[c01].corner_point_index,
          v_ppi,
          v_ppo,
          v_mpi,
          v_mpo,
          cells[c11].v_mz,
          cells[c01].v_pz,
        );

        self.fill_t(
          center_part_index,
          cells[c10].corner_part_index,
          cells[c11].corner_part_index,
          center_point_index,
          cells[c10].corner_point_index,
          cells[c11].corner_point_index,
          v_pmi,
          v_pmo,
          v_ppi,
          v_ppo,
          cells[c10].v_zp,
          cells[c11].v_zm,
        );
        self.tmp_buf.extend_bot_rev(&[cells[c00].v_pz, cells[c10].v_mz]);
        self.tmp_buf.extend_top_rev(&[cells[c01].v_pz, cells[c11].v_mz]);
        self.tmp_buf.extend_mid(&[cells[c00].v_zp, cells[c01].v_zm]);
        self.tmp_buf.extend_mid(&[v_mmo, v_mmi, v_mpi, v_mpo]);
        self.tmp_buf.extend_mid(&[v_pmo, v_pmi, v_ppi, v_ppo]);
        self.tmp_buf.extend_mid(&[cells[c10].v_zp, cells[c11].v_zm]);
      }
      let ords = self.order_last_layer_points();
      self.make_edge_ascending_flags(edge_cursor);
      self.sort_last_layer_edges(edge_cursor);
      self.fill_edges_p_left(edge_cursor, ords);
      self.fill_edges_p_right(edge_cursor, ords);
    }

    self.topology.validate();
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
