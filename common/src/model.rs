use rand::SeedableRng;
use rand::seq::SliceRandom;

use crate::contour::*;
use crate::points3d::*;
use fxhash::{FxHashMap, FxHashSet};

// use u32 because of memory optimization
pub type Triangle = [u32; 3];

#[derive(Debug, Clone, Default)]
pub struct Model {
  pub vertices: Vec<Point>,
  pub triangles: Vec<Triangle>,
}

type MeshTopology = Vec<[u32; 3]>;

struct NormalGroups {
  group_of_t: Vec<u32>,
  normal_of_g: Vec<Point>,
}

const RSQ3: f32 = 0.57735026918962576;

const REPER_VECTORS: [Point; 7] = [
  Point { x: 1.0, y: 0.0, z: 0.0 },
  Point { x: 0.0, y: 1.0, z: 0.0 },
  Point { x: 0.0, y: 0.0, z: 1.0 },
  Point { x: -RSQ3, y: -RSQ3, z: RSQ3 },
  Point { x: -RSQ3, y: RSQ3, z: RSQ3 },
  Point { x: RSQ3, y: -RSQ3, z: RSQ3 },
  Point { x: RSQ3, y: RSQ3, z: RSQ3 },
];

#[derive(Debug, Default)]
struct ChangeTriangleBuffer {
  v: Vec<u32>,
  t: Vec<(u32, Triangle)>,
}

struct MergeContext {
  top: MeshTopology,
  ng: NormalGroups,
  buf: ChangeTriangleBuffer,
  ec: Vec<u32>,             // count of edges on each vertex
  e: FxHashSet<(u32, u32)>, // list of all egdes
}

impl MergeContext {
  fn ins_edge(e: &mut FxHashSet<(u32, u32)>, v0: u32, v1: u32) {
    let inserted = if v0 < v1 { e.insert((v0, v1)) } else { e.insert((v1, v0)) };
    assert!(inserted, "Edge {v0}: {v1} still exists!");
  }

  fn check_edge(&self, v0: u32, v1: u32) -> bool {
    if v0 < v1 { self.e.contains(&(v0, v1)) } else { self.e.contains(&(v1, v0)) }
  }

  fn ins_edge_if_ok(e: &mut FxHashSet<(u32, u32)>, v0: u32, v1: u32) {
    if v0 < v1 {
      let inserted = e.insert((v0, v1));
      assert!(inserted, "Edge {v0}: {v1} still exists!");
    }
  }

  fn remove_edge(e: &mut FxHashSet<(u32, u32)>, v0: u32, v1: u32) {
    let removed = if v0 < v1 { e.remove(&(v0, v1)) } else { e.remove(&(v1, v0)) };
    assert!(removed, "Edge {v0}: {v1} not exists!");
  }
}

#[derive(Debug, Clone, Default)]
pub struct ArrayBuffer {
  pub v: Vec<f32>,
  pub i: Vec<u32>,
}

impl Model {
  pub fn new() -> Self {
    Self { vertices: Vec::new(), triangles: Vec::new() }
  }

  pub fn add_vertex(&mut self, p: Point) -> u32 {
    self.vertices.push(p);
    self.vertices.len() as u32 - 1
  }

  pub fn write_to_buffer(&self, buffer: &mut ArrayBuffer, color: u32) {
    buffer.v.reserve(buffer.v.len() + self.vertices.len() * 9);
    buffer.i.reserve(buffer.i.len() + self.triangles.len() * 3);

    let old_v_len = (buffer.v.len() / 9) as u32;
    let c = self.center();

    for v in &self.vertices {
      buffer.v.push(v.x);
      buffer.v.push(v.z);
      buffer.v.push(v.y);
      buffer.v.push(c.x);
      buffer.v.push(c.z);
      buffer.v.push(c.y);
      buffer.v.push((color >> 16 & 0xff) as f32 / 255.0);
      buffer.v.push((color >> 8 & 0xff) as f32 / 255.0);
      buffer.v.push((color & 0xff) as f32 / 255.0);
    }
    for t in &self.triangles {
      buffer.i.push(old_v_len + t[0]);
      buffer.i.push(old_v_len + t[1]);
      buffer.i.push(old_v_len + t[2]);
    }
  }

  pub fn append(&mut self, other: Self) {
    self.triangles.reserve(self.triangles.len() + other.triangles.len());
    self.vertices.reserve(self.vertices.len() + other.vertices.len());
    let shift = self.vertices.len() as u32;
    for t in other.triangles {
      self.triangles.push([t[0] + shift, t[1] + shift, t[2] + shift]);
    }
    for v in other.vertices {
      self.vertices.push(v);
    }
  }

  pub fn smooth(&mut self, delta: f32) {
    let mut positions = vec![(Point::ZERO, 0); self.vertices.len()];
    for t in &self.triangles {
      let t0 = t[0] as usize;
      let t1 = t[1] as usize;
      let t2 = t[2] as usize;
      positions[t0].0 += self.vertices[t1];
      positions[t0].0 += self.vertices[t2];
      positions[t0].1 += 2;
      positions[t1].0 += self.vertices[t2];
      positions[t1].0 += self.vertices[t0];
      positions[t1].1 += 2;
      positions[t2].0 += self.vertices[t0];
      positions[t2].0 += self.vertices[t1];
      positions[t2].1 += 2;
    }

    for i in 0..self.vertices.len() {
      let dp = positions[i].0.scale((positions[i].1 as f32).recip()) - self.vertices[i];
      self.vertices[i] += dp.scale(delta);
    }
  }

  fn get_topology(&self) -> MeshTopology {
    let mut edge_to_face = FxHashMap::<(u32, u32), (u32, u32)>::default();
    const BAD_INDEX: u32 = u32::MAX;
    let mut make_edge = |t, v1, v2| {
      if v1 < v2 {
        let e = edge_to_face.entry((v1, v2)).or_insert((BAD_INDEX, BAD_INDEX));
        if e.0 != BAD_INDEX {
          panic!("fail with edge {v1} to {v2} while add {t}: {:?}, first elem still used", e);
        }
        e.0 = t;
      } else {
        let e = edge_to_face.entry((v2, v1)).or_insert((BAD_INDEX, BAD_INDEX));
        if e.1 != BAD_INDEX {
          panic!("fail with edge {v1} to {v2} while add {t}: {:?}, second elem still used", e);
        }
        e.1 = t;
      }
    };

    for i in 0..self.triangles.len() {
      let t = self.triangles[i];
      make_edge(i as u32, t[0], t[1]);
      make_edge(i as u32, t[1], t[2]);
      make_edge(i as u32, t[2], t[0]);
    }

    let mut face_adj = Vec::<[u32; 3]>::new();
    face_adj.resize(self.triangles.len(), [BAD_INDEX; 3]);

    let use_edge = |dst: &mut u32, v1, v2| {
      if v1 < v2 {
        *dst = edge_to_face.get(&(v1, v2)).unwrap().1;
      } else {
        *dst = edge_to_face.get(&(v2, v1)).unwrap().0;
      }
    };

    for i in 0..self.triangles.len() {
      let t = self.triangles[i];
      use_edge(&mut face_adj[i][0], t[0], t[1]);
      use_edge(&mut face_adj[i][1], t[1], t[2]);
      use_edge(&mut face_adj[i][2], t[2], t[0]);
    }

    face_adj
  }

  fn get_unchecked_vertex(&self, top: &MeshTopology, old_t: u32, new_t: u32) -> u32 {
    let top = top[new_t as usize];
    let t = self.triangles[new_t as usize];
    if old_t == top[0] {
      return t[2];
    } else if old_t == top[1] {
      return t[0];
    } else if old_t == top[2] {
      return t[1];
    } else {
      panic!("top {top:?} of {new_t} does not have {old_t}");
    }
  }

  fn get_normal_groups(
    &self,
    top: &MeshTopology,
    max_tol: f32,
    min_group_size: f32,
  ) -> NormalGroups {
    struct GroupInfo {
      extremal: [(f32, f32); REPER_VECTORS.len()],
      strength: f32,
    }

    impl GroupInfo {
      fn new() -> Self {
        Self { extremal: [(f32::INFINITY, -f32::INFINITY); REPER_VECTORS.len()], strength: 0.0 }
      }

      fn add_v(&mut self, p: Point) {
        for i in 0..REPER_VECTORS.len() {
          let dst = &mut self.extremal[i];
          let d = dot(p, REPER_VECTORS[i]);
          dst.0 = f32::min(dst.0, d);
          dst.1 = f32::max(dst.1, d);
        }
      }

      fn add_t(&mut self, model: &Model, ti: u32) {
        let t = model.triangles[ti as usize];
        self.add_v(model.vertices[t[0] as usize]);
        self.add_v(model.vertices[t[1] as usize]);
        self.add_v(model.vertices[t[2] as usize]);
      }

      fn strength(&self) -> f32 {
        let mut result = -f32::INFINITY;
        for e in &self.extremal {
          result = f32::max(result, e.1 - e.0);
        }
        result

        //self.strength
      }
    }

    let mut group_of_t = Vec::<u32>::new();
    let mut valid_group_mapping = Vec::<u32>::new();
    let mut normal_of_g = Vec::<Point>::new();
    let mut stack = std::collections::VecDeque::<u32>::new();
    let mut visited = Vec::<u32>::new();
    let mut visited_ng = Vec::<u32>::new();
    let mut info_of_g = Vec::<GroupInfo>::new();

    group_of_t.resize(self.triangles.len(), u32::MAX);

    let mut tol = 1.0e-6;

    let mut ti = Vec::with_capacity(self.triangles.len());
    for i in 0..self.triangles.len() {
      ti.push(i as u32);
    }
    ti.shuffle(&mut rand::rngs::StdRng::seed_from_u64(10938109));

    let mut cnt_grouped = 0;
    let max_step = max_tol * 0.2;

    let start = std::time::Instant::now();
    loop {
      println!(
        "get_normal_groups with tol {tol}, {} grouped...",
        cnt_grouped as f32 * 100.0 / self.triangles.len() as f32
      );
      let next_tol = tol + f32::min(tol * 9.0, max_step);
      let last_iter = next_tol > max_tol;
      let mut complete = true;

      for &ti in &ti {
        if group_of_t[ti as usize] != u32::MAX {
          continue;
        }
        let mut g = info_of_g.len();
        let t0 = self.triangles[ti as usize];
        let cn = self.get_normal(t0);
        let cl = cn.len();
        if cl == 0.0 {
          continue;
        }
        let cn = cn.scale(cl.recip());
        let mut ig = GroupInfo::new();
        ig.add_t(self, ti);
        stack.push_back(ti);
        group_of_t[ti as usize] = g as u32;
        visited.push(ti);
        while let Some(cur_ti) = stack.pop_front() {
          for new_ti in top[cur_ti as usize] {
            let g_ti = group_of_t[new_ti as usize];
            if g_ti != u32::MAX {
              continue;
            }

            let nn = self.get_perp(self.triangles[new_ti as usize]);
            let ln = nn.len();
            if dot(nn, cn) < (1.0 - tol) * ln {
              continue;
            }

            let new_v = self.vertices[self.get_unchecked_vertex(top, cur_ti, new_ti) as usize];
            ig.add_v(new_v);
            group_of_t[new_ti as usize] = g as u32;
            visited.push(new_ti);
            stack.push_back(new_ti);
          }
        }
        stack.clear();

        if ig.strength() >= min_group_size || last_iter {
          info_of_g.push(ig);
          normal_of_g.push(cn);
          cnt_grouped += visited.len();
          visited.clear();
        } else {
          complete = false;
          visited_ng.append(&mut visited);
        }
      }

      if last_iter {
        break;
      }

      tol = next_tol;
      for &ti in &visited_ng {
        group_of_t[ti as usize] = u32::MAX;
      }
      visited_ng.clear();

      if complete {
        break;
      }
    }

    println!("generating normals elapsed {:?}", std::time::Instant::now() - start);
    println!(
      "{} grouped in {} groups, {} ungrouped",
      cnt_grouped as f32 * 100.0 / self.triangles.len() as f32,
      normal_of_g.len(),
      self.triangles.len() - cnt_grouped
    );

    NormalGroups { group_of_t, normal_of_g }
  }

  pub fn v_near_t(&self, v_index: u32, t: Triangle, eps: f32) -> bool {
    let v = self.vertices[v_index as usize];
    let v0 = v - self.vertices[t[0] as usize];
    let n = self.get_normal(t);

    if dot(v0, n).abs() > eps {
      return false;
    }

    let v1 = v - self.vertices[t[1] as usize];
    let v2 = v - self.vertices[t[2] as usize];

    let cr01 = cross(v0, v1);
    let cr12 = cross(v1, v2);
    let cr20 = cross(v2, v0);

    if dot(n, cr01) > 0.0 && dot(n, cr12) > 0.0 && dot(n, cr20) > 0.0 {
      return true;
    }

    if cr01.sqr_len() < eps * eps * (v0 - v1).sqr_len()
      && dot(v1 - v0, v1) > 0.0
      && dot(v0 - v1, v0) > 0.0
    {
      return true;
    }

    if cr12.sqr_len() < eps * eps * (v1 - v2).sqr_len()
      && dot(v2 - v1, v2) > 0.0
      && dot(v1 - v2, v1) > 0.0
    {
      return true;
    }

    if cr20.sqr_len() < eps * eps * (v2 - v0).sqr_len()
      && dot(v0 - v2, v0) > 0.0
      && dot(v2 - v0, v2) > 0.0
    {
      return true;
    }

    v0.sqr_len() < eps * eps || v1.sqr_len() < eps * eps || v2.sqr_len() < eps * eps
  }

  fn create_merge_context(
    &self,
    top: MeshTopology,
    ng: NormalGroups,
    buf: ChangeTriangleBuffer,
  ) -> MergeContext {
    let mut ec = Vec::new();
    ec.resize(self.vertices.len(), 0);
    let mut e = FxHashSet::default();
    let mut mc = MergeContext { top, ng, buf, ec, e };
    for &t in &self.triangles {
      mc.ec[t[0] as usize] += 1;
      mc.ec[t[1] as usize] += 1;
      mc.ec[t[2] as usize] += 1;
      MergeContext::ins_edge_if_ok(&mut mc.e, t[0], t[1]);
      MergeContext::ins_edge_if_ok(&mut mc.e, t[1], t[2]);
      MergeContext::ins_edge_if_ok(&mut mc.e, t[2], t[0]);
    }
    mc
  }

  fn check_merge_context(&self, ctx: &MergeContext) {
    let mut ec = Vec::new();
    ec.resize(self.vertices.len(), 0);
    let mut e = FxHashSet::default();
    for (ti, t) in self.triangles.iter().enumerate() {
      if ctx.top[ti][0] == u32::MAX {
        continue;
      }
      ec[t[0] as usize] += 1;
      ec[t[1] as usize] += 1;
      ec[t[2] as usize] += 1;
      let ins01 = if t[0] < t[1] { e.insert((t[0], t[1])) } else { true };
      let ins12 = if t[1] < t[2] { e.insert((t[1], t[2])) } else { true };
      let ins20 = if t[2] < t[0] { e.insert((t[2], t[0])) } else { true };
      assert!(ins01 && ins12 && ins20, "Topology has duplicated edge!");
    }
    for i in 0..self.vertices.len() {
      assert!(
        ctx.ec[i] == ec[i],
        "Falied to update merge context on {i}, act={}, exp={}",
        ctx.ec[i],
        ec[i]
      );
    }
    for edge in &e {
      assert!(ctx.e.contains(edge), "Expected edge {edge:?} not existing in ctx!");
    }
    for edge in &ctx.e {
      assert!(e.contains(edge), "Actual edge {edge:?} not existing in expected set!");
    }
  }

  fn find_t(adj: [u32; 3], ti: u32) -> (usize, usize, usize) {
    if adj[0] == ti {
      return (0, 1, 2);
    } else if adj[1] == ti {
      return (1, 2, 0);
    } else if adj[2] == ti {
      return (2, 0, 1);
    } else {
      panic!("Cant find ti {ti} in adj {adj:?}!");
    }
  }

  fn exchange_top(top: &mut MeshTopology, t1: u32, t2: u32, what: u32) {
    for (t1, t2) in [(t1, t2), (t2, t1)] {
      let dst: &mut [u32; 3] = &mut top[t1 as usize];
      let (d0, d1, d2) = Self::find_t(*dst, what);
      dst[d0] = t2;
      assert!(dst[d1] != t2 && dst[d2] != t2);
    }
  }

  fn merge(
    &mut self,
    ctx: &mut MergeContext,
    tleft: u32,
    tright: u32,
    vfrom: u32,
    vto: u32,
  ) -> bool {
    if ctx.ec[vfrom as usize] - 2 + ctx.ec[vto as usize] - 2 < 3 {
      return false;
    }
    let (lt0, lt1, lt2) = Self::find_t(ctx.top[tleft as usize], tright);
    let vleft = self.triangles[tleft as usize][lt2];
    if ctx.ec[vleft as usize] - 1 < 3 {
      return false;
    }
    let (rt0, rt1, rt2) = Self::find_t(ctx.top[tright as usize], tleft);
    let vright = self.triangles[tright as usize][rt2];
    if ctx.ec[vright as usize] - 1 < 3 {
      return false;
    }

    ctx.buf.t.clear();
    ctx.buf.v.clear();
    let mut cur = tleft;
    let mut prev = tright;
    let mut pprev = u32::MAX;
    let mut nl = true;
    let gleft = ctx.ng.group_of_t[tleft as usize];
    let gright = ctx.ng.group_of_t[tright as usize];

    let both_valid = gleft != u32::MAX && gright != u32::MAX;

    while cur != tright {
      let t = ctx.top[cur as usize];
      let (_, nv1, nv2) = Self::find_t(t, prev);

      let next = t[nv1];
      let mut nt = self.triangles[cur as usize];
      assert!(nt[nv2] != vto, "Duplicate edge!");

      if cur != tleft {
        if next != tright {
          if ctx.check_edge(nt[nv2], vto) {
            return false;
          }
          ctx.buf.v.push(nt[nv2]);
        }

        if nl && ctx.ng.group_of_t[cur as usize] != gleft {
          nl = false;
        }
        if !nl && ctx.ng.group_of_t[cur as usize] != gright {
          return false;
        }

        assert!(nt[nv1] == vfrom, "Wrong vertex {vfrom} to merge triangle {nt:?} by index {nv1}!");
        nt[nv1] = vto;
        let nn = self.get_perp(nt);
        let nnl = nn.len();

        let gi = if nl { gleft } else { gright };
        if gi != u32::MAX && dot(ctx.ng.normal_of_g[gi as usize], nn) < 0.0 {
          return false;
        }

        ctx.buf.t.push((cur, nt));
      }

      pprev = prev;
      prev = cur;
      cur = next;
    }

    // Here we know we should merge
    for (tleft, tright) in [(tleft, tright), (tright, tleft)] {
      let fa = ctx.top[tleft as usize];
      let (_, fa1, fa2) = Self::find_t(fa, tright);
      Self::exchange_top(&mut ctx.top, fa[fa1], fa[fa2], tleft);
    }

    for (index, nt) in &ctx.buf.t {
      self.triangles[*index as usize] = *nt;
    }

    for v in [vleft, vright, vto] {
      MergeContext::remove_edge(&mut ctx.e, v, vfrom);
    }

    for &v in &ctx.buf.v {
      MergeContext::remove_edge(&mut ctx.e, v, vfrom);
      MergeContext::ins_edge(&mut ctx.e, v, vto);
    }

    ctx.ec[vto as usize] = ctx.ec[vfrom as usize] - 2 + ctx.ec[vto as usize] - 2;
    ctx.ec[vfrom as usize] = 0;
    ctx.ec[vleft as usize] -= 1;
    ctx.ec[vright as usize] -= 1;

    ctx.top[tleft as usize][0] = u32::MAX;
    ctx.top[tright as usize][0] = u32::MAX;

    true
  }

  pub fn optimize(&mut self, max_tol: f32, min_group_size: f32) {
    println!("get topology...");
    let top = self.get_topology();
    println!("get normal groups...");
    let mut ng = self.get_normal_groups(&top, max_tol, min_group_size);

    let buf = ChangeTriangleBuffer::default();
    let mut ctx = self.create_merge_context(top, ng, buf);

    let mut rng = rand::rngs::StdRng::seed_from_u64(1500);
    let mut ti = Vec::with_capacity(self.triangles.len());
    for t in 0..self.triangles.len() {
      ti.push(t);
    }
    let mut ttc = self.triangles.len();

    let start = std::time::Instant::now();
    loop {
      println!("start loop iter with {ttc} valid triangles");
      let mut merged = false;
      ti.shuffle(&mut rng);
      'enumerate_triangles: for &t in &ti {
        if ctx.top[t][0] == u32::MAX {
          continue;
        }
        let tr = self.triangles[t];
        for i in 0..3 {
          let ta = ctx.top[t][i];
          let vfrom = tr[(i + 1) % 3];
          let vto = tr[i];
          if self.merge(&mut ctx, t as u32, ta, vfrom, vto) {
            ttc -= 2;
            merged = true;
            continue 'enumerate_triangles;
          }
        }
      }
      if !merged {
        break;
      }
    }
    println!(
      "end loop with {ttc} valid triangles and {:?} time",
      std::time::Instant::now() - start
    );

    let mut j = 0;
    for i in 0..self.triangles.len() {
      if ctx.top[i][0] != u32::MAX {
        self.triangles[j] = self.triangles[i];
        j += 1;
      }
    }
    println!("check: {j} valid triangles");

    self.triangles.truncate(j);
  }

  pub fn delete_unused_v(&mut self) {
    const BAD_INDEX: u32 = u32::MAX;
    let mut mapping = Vec::<u32>::new();
    mapping.resize(self.vertices.len(), u32::MAX);

    let mut result = Vec::<Point>::new();

    let mut use_v = |v: &mut u32| {
      if mapping[*v as usize] == BAD_INDEX {
        mapping[*v as usize] = result.len() as u32;
        result.push(self.vertices[*v as usize]);
      }

      *v = mapping[*v as usize];
    };

    for t in &mut self.triangles {
      use_v(&mut t[0]);
      use_v(&mut t[1]);
      use_v(&mut t[2]);
    }

    self.vertices = result;
  }

  pub fn get_normal(&self, t: Triangle) -> Point {
    cross(
      self.vertices[t[1] as usize] - self.vertices[t[0] as usize],
      self.vertices[t[2] as usize] - self.vertices[t[0] as usize],
    )
    .norm()
  }

  pub fn get_perp(&self, t: Triangle) -> Point {
    cross(
      self.vertices[t[1] as usize] - self.vertices[t[0] as usize],
      self.vertices[t[2] as usize] - self.vertices[t[0] as usize],
    )
  }

  /// panics if mesh is not closed
  pub fn validate_and_delete_small_groups(&mut self) {
    let topology = self.get_topology();

    let mut face_group = Vec::<u32>::new();
    face_group.resize(self.triangles.len(), 0);

    let mut for_visit = Vec::<u32>::new();
    let mut last_group = 0;

    for i in 0..face_group.len() {
      if face_group[i] == 0 {
        last_group += 1;
        for_visit.push(i as u32);
        while let Some(f) = for_visit.pop() {
          face_group[f as usize] = last_group;
          for &f in &topology[f as usize] {
            if face_group[f as usize] == 0 {
              for_visit.push(f);
            }
          }
        }
      }
    }

    let mut counts = Vec::<u32>::new();
    counts.resize(last_group as usize, 0);
    for f in &face_group {
      counts[*f as usize - 1] += 1;
    }

    let mut max_c = 0;
    let mut max_index = 0;
    for i in 0..counts.len() {
      if counts[i] > max_c {
        max_c = counts[i];
        max_index = i + 1;
      }
    }

    let mut new_t = Vec::new();
    for i in 0..self.triangles.len() {
      if face_group[i] as usize == max_index {
        new_t.push(self.triangles[i]);
      }
    }
    self.triangles = new_t;
  }

  fn split_by(self, t_to_i: &dyn Fn(u32) -> u32) -> Vec<Model> {
    #[derive(Default)]
    struct Mapping {
      v: FxHashMap<u32, u32>, // vertex_id -> new_vertex_id
      m: Model,
    }

    let mut mappings = FxHashMap::<u32, Mapping>::default(); // model_id -> mapping

    for i in 0..self.triangles.len() {
      let ti = t_to_i(i as u32);
      if ti == 0 {
        continue;
      }
      let mapping = mappings.entry(ti).or_default();
      let t = self.triangles[i];

      let mut use_mapping = |v| {
        *mapping.v.entry(v).or_insert_with(|| {
          let result = mapping.m.vertices.len();
          mapping.m.vertices.push(self.vertices[v as usize]);
          result as u32
        })
      };

      let t0 = use_mapping(t[0]);
      let t1 = use_mapping(t[1]);
      let t2 = use_mapping(t[2]);
      mapping.m.triangles.push([t0, t1, t2]);
    }

    mappings.into_iter().map(|(_, m)| m.m).collect()
  }

  pub fn split_by_normal(mut self, max_tol: f32, min_group_size: f32) -> Vec<Model> {
    let top = self.get_topology();
    let mut ng = self.get_normal_groups(&top, max_tol, min_group_size);
    self.split_by(&|i| ng.group_of_t[i as usize].wrapping_add(2))
  }

  pub fn center(&self) -> Point {
    let mut sum = Point::ZERO;
    for v in &self.vertices {
      sum += *v;
    }
    sum.scale(1.0 / self.vertices.len() as f32)
  }

  pub fn out_of_center(&mut self, factor: f32) {
    let c = self.center();
    for v in &mut self.vertices {
      *v += c.scale(factor);
    }
  }

  fn add_to_convex(triangles: &[Triangle], vertices: &[Point], vi: u32) -> Vec<Triangle> {
    // add_to_convex stuff
    fn flip2(edges: &mut FxHashSet<(u32, u32)>, t0: u32, t1: u32) {
      if !edges.remove(&(t1, t0)) {
        edges.insert((t0, t1));
      }
    }

    fn flip3(edges: &mut FxHashSet<(u32, u32)>, t0: u32, t1: u32, t2: u32) {
      flip2(edges, t0, t1);
      flip2(edges, t1, t2);
      flip2(edges, t2, t0);
    }

    fn validate(edges: &FxHashSet<(u32, u32)>, buffer: &mut FxHashMap<u32, u32>) -> bool {
      buffer.clear();

      for &(v0, v1) in edges {
        if buffer.insert(v0, v1).is_some() {
          return false;
        }
      }

      if let Some(&first_v) = buffer.keys().next() {
        // Checking connectivity
        let mut cur_v = first_v;
        loop {
          cur_v = buffer.remove(&cur_v).unwrap();
          if cur_v == first_v {
            break;
          }
        }

        return buffer.is_empty();
      }

      return true;
    }

    let mut vols = Vec::<(f32, u32)>::new();

    for i in 0..triangles.len() {
      let t = triangles[i];
      let vol = dot(
        vertices[vi as usize] - vertices[t[0] as usize],
        cross(
          vertices[t[1] as usize] - vertices[t[0] as usize],
          vertices[t[2] as usize] - vertices[t[0] as usize],
        ),
      );
      vols.push((vol, i as u32));
    }

    vols.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut split = 0;
    while split < vols.len() && vols[split].0 < 0.0 {
      split += 1;
    }

    let mut edges = FxHashSet::<(u32, u32)>::default();
    for i in 0..split {
      let t = triangles[vols[i].1 as usize];
      flip3(&mut edges, t[2], t[1], t[0]);
    }

    let mut buffer = FxHashMap::<u32, u32>::default();

    if !validate(&edges, &mut buffer) {
      assert!(split > 0);
      assert!(split < vols.len() - 1);

      let mut s_down = split - 1;
      let mut s_up = split;

      let mut edges_up = edges.clone();

      loop {
        let t = triangles[vols[s_down].1 as usize];
        // reversed
        flip3(&mut edges, t[0], t[1], t[2]);
        if validate(&edges, &mut buffer) {
          split = s_down;
          break;
        }
        assert!(s_down > 0);
        s_down -= 1;

        let t = triangles[vols[s_up].1 as usize];
        flip3(&mut edges_up, t[2], t[1], t[0]);
        if validate(&edges_up, &mut buffer) {
          split = s_up + 1;
          edges = edges_up;
          break;
        }
        assert!(s_up < vols.len() - 1);
        s_up += 1;
      }
    }
    let mut new_t = Vec::new();
    for i in 0..split {
      new_t.push(triangles[vols[i].1 as usize]);
    }

    for &(v0, v1) in &edges {
      new_t.push([v0, v1, vi]);
    }
    new_t
  }

  pub fn get_volume(&self) -> f32 {
    let mut result = 0.0;
    for t in &self.triangles {
      let v0 = self.vertices[t[0] as usize];
      let v1 = self.vertices[t[1] as usize];
      let v2 = self.vertices[t[2] as usize];
      result += dot(v0, cross(v1, v2));
    }

    result / 6.0
  }

  pub fn convex_triangles(vertices: &[Point], eps: f32) -> Option<Vec<Triangle>> {
    let mut i1 = 1;
    let mut i2 = 2;
    let mut i3 = 3;

    let mut ok = false;
    for i in 1..vertices.len() {
      if (vertices[i] - vertices[0]).sqr_len() > eps * eps {
        i1 = i;
        ok = true;
        break;
      }
    }
    if !ok {
      return None;
    }

    ok = false;
    for i in 1..vertices.len() {
      if i == i1 {
        continue;
      }
      if cross(vertices[i] - vertices[0], vertices[i1] - vertices[0]).sqr_len() > eps * eps {
        i2 = i;
        ok = true;
        break;
      }
    }
    if !ok {
      return None;
    }

    let mut ok = 0.0;
    for i in 1..vertices.len() {
      if i == i2 || i == i2 {
        continue;
      }
      let t = dot(
        vertices[i] - vertices[0],
        cross(vertices[i2] - vertices[0], vertices[i1] - vertices[0]),
      );

      if t.abs() > eps * eps * eps {
        i3 = i;
        ok = t.signum();
        break;
      }
    }
    if ok == 0.0 {
      return None;
    }

    let mut result = Vec::new();

    if ok < 0.0 {
      std::mem::swap(&mut i2, &mut i3);
    }

    result.push([0, i1 as u32, i2 as u32]);
    result.push([0, i2 as u32, i3 as u32]);
    result.push([0, i3 as u32, i1 as u32]);
    result.push([i3 as u32, i2 as u32, i1 as u32]);

    for i in 1..vertices.len() {
      if i == i1 || i == i2 || i == i3 {
        continue;
      }
      result = Self::add_to_convex(&result, vertices, i as u32);
    }

    Some(result)
  }

  pub fn convex(vertices: &[Point], eps: f32) -> Option<Self> {
    let triangles = Self::convex_triangles(vertices, eps)?;
    Some(Self { vertices: vertices.to_owned(), triangles })
  }

  pub fn map_points(&mut self, f: impl Fn(Point) -> Point) {
    for v in &mut self.vertices {
      *v = f(*v);
    }
  }

  pub fn save_to_stl(&self, path: &std::path::Path) -> Result<(), String> {
    let mut file =
      std::io::BufWriter::new(std::fs::File::create(&path).map_err(|e| {
        format!("Unable to open file {} for writing: {}", path.to_string_lossy(), e)
      })?);

    let triangle_iter = self.triangles.iter().map(|t| {
      let v0 = self.vertices[t[0] as usize];
      let v1 = self.vertices[t[1] as usize];
      let v2 = self.vertices[t[2] as usize];
      let n = cross(v1 - v0, v2 - v0).norm();
      let result = stl_io::Triangle {
        normal: stl_io::Normal::new([n.x, n.y, n.z]),
        vertices: [
          stl_io::Vertex::new([v0.x, v0.y, v0.z]),
          stl_io::Vertex::new([v1.x, v1.y, v1.z]),
          stl_io::Vertex::new([v2.x, v2.y, v2.z]),
        ],
      };

      result
    });

    stl_io::write_stl(&mut file, triangle_iter)
      .map_err(|e| format!("Failed to save stl to file {}: {}", path.to_string_lossy(), e))
  }

  pub fn load_from_stl(path: &std::path::Path) -> Result<Self, String> {
    let mut file =
      std::io::BufReader::new(std::fs::File::open(&path).map_err(|e| {
        format!("Unable to open file {} for reading: {}", path.to_string_lossy(), e)
      })?);

    let stl = stl_io::read_stl(&mut file)
      .map_err(|e| format!("Failed to load stl from file {}: {}", path.to_string_lossy(), e))?;

    let mut result = Self::new();
    for v in stl.vertices {
      result.vertices.push(Point{x: v[0], y: v[1], z: v[2]});
    }

    result.triangles.reserve(stl.faces.len());
    for t in stl.faces {
      result.triangles.push([
        t.vertices[0] as u32,
        t.vertices[1] as u32,
        t.vertices[2] as u32,
      ]);
    }

    Ok(result)
  }

  pub fn cuboid(tx: usize, ty: usize, tz: usize, cell_size: f32) -> Self {
    let mut m = FxHashMap::default();
    let mut vertices = Vec::new();
    let mut nv = |x: usize, y: usize, z: usize| {
      let inserted = m.insert((x, y, z), vertices.len() as u32);
      assert!(inserted.is_none(), "This coordinated is used before!");
      vertices.push(Point {
        x: x as f32 * cell_size,
        y: y as f32 * cell_size,
        z: z as f32 * cell_size,
      });
    };

    for y in 0..=ty {
      for x in 0..=tx {
        nv(x, y, 0);
        nv(x, y, tz);
      }
    }
    for z in 1..tz {
      for x in 0..=tx {
        nv(x, 0, z);
        nv(x, ty, z);
      }
      for y in 1..ty {
        nv(0, y, z);
        nv(tx, y, z);
      }
    }

    let p = |x: usize, y: usize, z: usize| -> u32 { m[&(x, y, z)] };
    let mut triangles = Vec::new();
    let mut t = |v0, v1, v2| {
      triangles.push([v0, v1, v2]);
    };

    let mut qr = |triangles: &mut Vec<Triangle>, v0, v1, v2, v3, odd| {
      if odd & 1 == 0 {
        triangles.push([v0, v3, v1]);
        triangles.push([v1, v3, v2]);
      } else {
        triangles.push([v0, v2, v1]);
        triangles.push([v0, v3, v2]);
      }
    };

    let mut q = |triangles: &mut Vec<Triangle>, v0, v1, v2, v3, odd| {
      if odd & 1 == 0 {
        triangles.push([v0, v1, v3]);
        triangles.push([v1, v2, v3]);
      } else {
        triangles.push([v0, v1, v2]);
        triangles.push([v0, v2, v3]);
      }
    };

    for t0 in 0..ty {
      for t1 in 0..tx {
        qr(
          &mut triangles,
          p(t0, t1, 0),
          p(t0 + 1, t1, 0),
          p(t0 + 1, t1 + 1, 0),
          p(t0, t1 + 1, 0),
          t0 + t1,
        );
        q(
          &mut triangles,
          p(t0, t1, tz),
          p(t0 + 1, t1, tz),
          p(t0 + 1, t1 + 1, tz),
          p(t0, t1 + 1, tz),
          t0 + t1,
        );
        qr(
          &mut triangles,
          p(0, t0, t1),
          p(0, t0 + 1, t1),
          p(0, t0 + 1, t1 + 1),
          p(0, t0, t1 + 1),
          t0 + t1,
        );
        q(
          &mut triangles,
          p(tx, t0, t1),
          p(tx, t0 + 1, t1),
          p(tx, t0 + 1, t1 + 1),
          p(tx, t0, t1 + 1),
          t0 + t1,
        );
        qr(
          &mut triangles,
          p(t1, 0, t0),
          p(t1, 0, t0 + 1),
          p(t1 + 1, 0, t0 + 1),
          p(t1 + 1, 0, t0),
          t0 + t1,
        );
        q(
          &mut triangles,
          p(t1, ty, t0),
          p(t1, ty, t0 + 1),
          p(t1 + 1, ty, t0 + 1),
          p(t1 + 1, ty, t0),
          t0 + t1,
        );
      }
    }

    let mut result = Self::new();
    result.vertices = vertices;
    result.triangles = triangles;
    result
  }

  pub fn damaged_cuboid(tx: usize, ty: usize, tz: usize, cell_size: f32) -> Self {
    let mut m = FxHashMap::default();
    let mut vertices = Vec::new();
    let mut nv = |x: usize, y: usize, z: usize| {
      let inserted = m.insert((x, y, z), vertices.len() as u32);
      assert!(inserted.is_none(), "This coordinated is used before!");
      vertices.push(Point {
        x: x as f32 * cell_size,
        y: y as f32 * cell_size,
        z: z as f32 * cell_size,
      });
    };

    for y in 0..=ty {
      for x in 0..=tx {
        nv(x, y, 0);
        nv(x, y, tz);
      }
    }
    for z in 1..tz {
      for x in 0..=tx {
        nv(x, 0, z);
        nv(x, ty, z);
      }
      for y in 1..ty {
        nv(0, y, z);
        nv(tx, y, z);
      }
    }

    vertices[m[&(0, 0, 2)] as usize] += Point { x: cell_size * 0.5, y: cell_size * 0.5, z: 0.0 };

    let p = |x: usize, y: usize, z: usize| -> u32 { m[&(x, y, z)] };
    let mut triangles = Vec::new();
    let mut t = |v0, v1, v2| {
      triangles.push([v0, v1, v2]);
    };

    let mut qr = |triangles: &mut Vec<Triangle>, v0, v1, v2, v3, odd| {
      if odd & 1 == 0 {
        triangles.push([v0, v3, v1]);
        triangles.push([v1, v3, v2]);
      } else {
        triangles.push([v0, v2, v1]);
        triangles.push([v0, v3, v2]);
      }
    };

    let mut q = |triangles: &mut Vec<Triangle>, v0, v1, v2, v3, odd| {
      if odd & 1 == 0 {
        triangles.push([v0, v1, v3]);
        triangles.push([v1, v2, v3]);
      } else {
        triangles.push([v0, v1, v2]);
        triangles.push([v0, v2, v3]);
      }
    };

    for t0 in 0..ty {
      for t1 in 0..tx {
        qr(
          &mut triangles,
          p(t0, t1, 0),
          p(t0 + 1, t1, 0),
          p(t0 + 1, t1 + 1, 0),
          p(t0, t1 + 1, 0),
          t0 + t1,
        );
        q(
          &mut triangles,
          p(t0, t1, tz),
          p(t0 + 1, t1, tz),
          p(t0 + 1, t1 + 1, tz),
          p(t0, t1 + 1, tz),
          t0 + t1,
        );
        qr(
          &mut triangles,
          p(0, t0, t1),
          p(0, t0 + 1, t1),
          p(0, t0 + 1, t1 + 1),
          p(0, t0, t1 + 1),
          t0 + t1,
        );
        q(
          &mut triangles,
          p(tx, t0, t1),
          p(tx, t0 + 1, t1),
          p(tx, t0 + 1, t1 + 1),
          p(tx, t0, t1 + 1),
          t0 + t1,
        );
        qr(
          &mut triangles,
          p(t1, 0, t0),
          p(t1, 0, t0 + 1),
          p(t1 + 1, 0, t0 + 1),
          p(t1 + 1, 0, t0),
          t0 + t1,
        );
        q(
          &mut triangles,
          p(t1, ty, t0),
          p(t1, ty, t0 + 1),
          p(t1 + 1, ty, t0 + 1),
          p(t1 + 1, ty, t0),
          t0 + t1,
        );
      }
    }

    let mut result = Self::new();
    result.vertices = vertices;
    result.triangles = triangles;
    result
  }

  pub fn cylinder(tp: usize, th: usize, cell_size: f32) -> Self {
    let mut m = FxHashMap::default();
    let mut vertices = Vec::new();

    let br = tp as f32 / 2.0 / std::f32::consts::PI;
    let ba = 2.0 * std::f32::consts::PI / tp as f32;

    vertices.push(Point { x: 0.0, y: 0.0, z: 0.0 });
    vertices.push(Point { x: 0.0, y: 0.0, z: th as f32 * cell_size });

    for y in 0..=th + 2 {
      for x in 0..tp {
        let inserted = m.insert((x, y), vertices.len() as u32);
        assert!(inserted.is_none(), "This coordinated is used before!");
        let h = (y.clamp(1, th + 1) - 1) as f32 * cell_size;
        let r = br * cell_size * if y == 0 || y == th + 2 { 0.5 } else { 1.0 };
        let a = x as f32 * ba;
        vertices.push(Point { x: a.cos() * r, y: a.sin() * r, z: h });
      }
    }

    let p = |x: usize, y: usize| -> u32 { m[&(x, y)] };
    let mut triangles = Vec::new();
    let mut t = |v0, v1, v2| {
      triangles.push([v0, v1, v2]);
    };

    for y in 0..th + 2 {
      for x in 0..tp {
        let nx = (x + 1) % tp;
        t(p(x, y), p(nx, y), p(x, y + 1));
        t(p(x, y + 1), p(nx, y), p(nx, y + 1));
      }
    }

    for x in 0..tp {
      let nx = (x + 1) % tp;
      t(p(nx, 0), p(x, 0), 0);
      t(p(x, th + 2), p(nx, th + 2), 1);
    }

    let mut result = Self::new();
    result.vertices = vertices;
    result.triangles = triangles;
    result
  }

  pub fn rounded_cylinder(tp: usize, th: usize, cell_size: f32) -> Self {
    let mut m = FxHashMap::default();
    let mut vertices = Vec::new();

    let br = tp as f32 / 2.0 / std::f32::consts::PI;
    let ba = 2.0 * std::f32::consts::PI / tp as f32;

    vertices.push(Point { x: 0.0, y: 0.0, z: 0.0 });
    vertices.push(Point { x: 0.0, y: 0.0, z: th as f32 * cell_size });

    for y in 0..=th {
      for x in 0..tp {
        let inserted = m.insert((x, y), vertices.len() as u32);
        assert!(inserted.is_none(), "This coordinated is used before!");
        let h = y as f32 * cell_size;
        let r = cell_size * if y == 0 || y == th { br - 1.0 } else { br };
        let a = x as f32 * ba;
        vertices.push(Point { x: a.cos() * r, y: a.sin() * r, z: h });
      }
    }

    let p = |x: usize, y: usize| -> u32 { m[&(x, y)] };
    let mut triangles = Vec::new();
    let mut t = |v0, v1, v2| {
      triangles.push([v0, v1, v2]);
    };

    for y in 0..th {
      for x in 0..tp {
        let nx = (x + 1) % tp;
        t(p(x, y), p(nx, y), p(x, y + 1));
        t(p(x, y + 1), p(nx, y), p(nx, y + 1));
      }
    }

    for x in 0..tp {
      let nx = (x + 1) % tp;
      t(p(nx, 0), p(x, 0), 0);
      t(p(x, th), p(nx, th), 1);
    }

    let mut result = Self::new();
    result.vertices = vertices;
    result.triangles = triangles;
    result
  }

  pub fn cone(tp: usize, th: usize, cell_size: f32) -> Self {
    let mut m = FxHashMap::default();
    let mut vertices = Vec::new();

    let br = tp as f32 / 2.0 / std::f32::consts::PI;
    let ba = 2.0 * std::f32::consts::PI / tp as f32;

    vertices.push(Point { x: 0.0, y: 0.0, z: 0.0 });
    vertices.push(Point { x: 0.0, y: 0.0, z: th as f32 * cell_size });

    for y in 1..=th {
      for x in 0..tp {
        let inserted = m.insert((x, y), vertices.len() as u32);
        assert!(inserted.is_none(), "This coordinated is used before!");
        let h = y as f32 * cell_size;
        let r = cell_size * br * y as f32 / th as f32;
        let a = x as f32 * ba;
        vertices.push(Point { x: a.cos() * r, y: a.sin() * r, z: h });
      }
    }

    let p = |x: usize, y: usize| -> u32 { m[&(x, y)] };
    let mut triangles = Vec::new();
    let mut t = |v0, v1, v2| {
      triangles.push([v0, v1, v2]);
    };

    for y in 1..th {
      for x in 0..tp {
        let nx = (x + 1) % tp;
        t(p(x, y), p(nx, y), p(x, y + 1));
        t(p(x, y + 1), p(nx, y), p(nx, y + 1));
      }
    }

    for x in 0..tp {
      let nx = (x + 1) % tp;
      t(p(nx, 1), p(x, 1), 0);
      t(p(x, th), p(nx, th), 1);
    }

    let mut result = Self::new();
    result.vertices = vertices;
    result.triangles = triangles;
    result
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn convex_floating_pituh() {
    let v2c = [
      Point { x: 17.226563, y: -17.226563, z: -17.5 },
      Point { x: 17.226563, y: -17.5, z: -17.226563 },
      Point { x: 17.226563, y: -17.226563, z: -17.226563 },
      Point { x: 17.5, y: -17.226563, z: -17.226563 },
      Point { x: 17.226563, y: -17.235107, z: -17.5 },
      Point { x: 17.226563, y: -17.5, z: -17.235107 },
      Point { x: 17.235107, y: -17.226563, z: -17.5 },
      Point { x: 17.235107, y: -17.5, z: -17.226563 },
      Point { x: 17.5, y: -17.226563, z: -17.235107 },
      Point { x: 17.5, y: -17.235107, z: -17.226563 },
    ];
    let mut test = Model::convex(&v2c, 0.0).unwrap();
    test.validate_and_delete_small_groups();
  }
}
