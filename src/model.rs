use rand::seq::SliceRandom;

use crate::points3d::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy)]
pub struct Triangle(pub usize, pub usize, pub usize);

#[derive(Debug, Clone, Default)]
pub struct Model {
    pub vertices: Vec<Point>,
    pub triangles: Vec<Triangle>,
}

#[derive(Debug, Default)]
pub struct MeshTopology {
    edge_to_face: HashMap<(usize, usize), (usize, usize)>,
    face_adj: Vec<[usize; 3]>,
}

#[derive(Debug, Clone, Default)]
pub struct ArrayBuffer {
    pub v: Vec<f32>,
    pub i: Vec<u32>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
        }
    }

    pub fn write_to_buffer(&self, buffer: &mut ArrayBuffer, color: u32) {
        buffer.v.reserve(buffer.v.len() + self.vertices.len() * 6);
        buffer.i.reserve(buffer.i.len() + self.triangles.len() * 3);

        println!(
            "make flat model of {} v and {} t",
            self.vertices.len(),
            self.triangles.len()
        );
        let old_v_len = buffer.v.len() / 6;

        for v in &self.vertices {
            buffer.v.push(v.x);
            buffer.v.push(v.y);
            buffer.v.push(v.z);
            buffer.v.push((color >> 16 & 0xff) as f32 / 255.0);
            buffer.v.push((color >> 8 & 0xff) as f32 / 255.0);
            buffer.v.push((color & 0xff) as f32 / 255.0);
        }
        for t in &self.triangles {
            buffer.i.push((old_v_len + t.0) as u32);
            buffer.i.push((old_v_len + t.1) as u32);
            buffer.i.push((old_v_len + t.2) as u32);
        }
    }

    pub fn append(&mut self, other: Self) {
        self.triangles
            .reserve(self.triangles.len() + other.triangles.len());
        self.vertices
            .reserve(self.vertices.len() + other.vertices.len());
        for t in other.triangles {
            self.triangles.push(Triangle(
                t.0 + self.vertices.len(),
                t.1 + self.vertices.len(),
                t.2 + self.vertices.len(),
            ));
        }
        for v in other.vertices {
            self.vertices.push(v);
        }
    }

    pub fn smooth(&mut self, delta: f32) {
        let mut positions = vec![(Point::zero(), 0); self.vertices.len()];
        for t in &self.triangles {
            positions[t.0].0 += self.vertices[t.1];
            positions[t.0].0 += self.vertices[t.2];
            positions[t.0].1 += 2;
            positions[t.1].0 += self.vertices[t.2];
            positions[t.1].0 += self.vertices[t.0];
            positions[t.1].1 += 2;
            positions[t.2].0 += self.vertices[t.0];
            positions[t.2].0 += self.vertices[t.1];
            positions[t.2].1 += 2;
        }

        for i in 0..self.vertices.len() {
            let dp = positions[i].0.scale((positions[i].1 as f32).recip()) - self.vertices[i];
            self.vertices[i] += dp.scale(delta);
        }
    }

    pub fn get_topology(&self) -> MeshTopology {
        let mut edge_to_face = HashMap::<(usize, usize), (usize, usize)>::new();
        const BAD_INDEX: usize = usize::MAX;
        let mut make_edge = |t, v1, v2| {
            if v1 < v2 {
                let e = edge_to_face
                    .entry((v1, v2))
                    .or_insert((BAD_INDEX, BAD_INDEX));
                if e.0 != BAD_INDEX {
                    panic!(
                        "fail with edge {v1} to {v2} while add {t}: {:?}, first elem still used",
                        e
                    );
                }
                e.0 = t;
            } else {
                let e = edge_to_face
                    .entry((v2, v1))
                    .or_insert((BAD_INDEX, BAD_INDEX));
                if e.1 != BAD_INDEX {
                    panic!(
                        "fail with edge {v1} to {v2} while add {t}: {:?}, second elem still used",
                        e
                    );
                }
                e.1 = t;
            }
        };

        for i in 0..self.triangles.len() {
            let t = self.triangles[i];
            make_edge(i, t.0, t.1);
            make_edge(i, t.1, t.2);
            make_edge(i, t.2, t.0);
        }

        let mut bad = false;
        for ((v1, v2), (f1, f2)) in &edge_to_face {
            if *f1 == BAD_INDEX || *f2 == BAD_INDEX {
                println!("v1={}, v2={}, f1={}, f2={}", v1, v2, f1, f2);
                bad = true;
            }
        }
        if bad {
            println!("self.triangles={:?}", self.triangles);
            panic!("mesh is incomplete");
        }

        let mut face_adj = Vec::<[usize; 3]>::new();
        face_adj.resize(self.triangles.len(), [BAD_INDEX; 3]);

        let use_edge = |dst: &mut usize, v1, v2| {
            if v1 < v2 {
                *dst = edge_to_face.get(&(v1, v2)).unwrap().1;
            } else {
                *dst = edge_to_face.get(&(v2, v1)).unwrap().0;
            }
        };

        for i in 0..self.triangles.len() {
            let t = self.triangles[i];
            use_edge(&mut face_adj[i][0], t.0, t.1);
            use_edge(&mut face_adj[i][1], t.1, t.2);
            use_edge(&mut face_adj[i][2], t.2, t.0);
        }

        MeshTopology {
            edge_to_face,
            face_adj,
        }
    }

    pub fn flip(&mut self, model_index: u32, part_f: &dyn Fn(Point) -> u32) -> usize {
        let top = self.get_topology();

        let mut new_triangles = Vec::<Triangle>::new();

        let mut used_f = Vec::<bool>::new();
        used_f.resize(self.triangles.len(), false);

        let mut control_edges: HashSet<(usize, usize)> = top.edge_to_face.keys().copied().collect();
        let mut result = 0;

        for (&e, &f) in &top.edge_to_face {
            if used_f[f.0] || used_f[f.1] {
                continue;
            }

            fn opp_v(e: (usize, usize), t: Triangle) -> usize {
                if e.0 == t.0 && e.1 == t.1 {
                    t.2
                } else if e.0 == t.1 && e.1 == t.2 {
                    t.0
                } else if e.0 == t.2 && e.1 == t.0 {
                    t.1
                } else {
                    panic!("edge {:?} is not from triangle {:?}", e, t)
                }
            }

            if dot(self.get_normal(self.triangles[f.0]), self.get_normal(self.triangles[f.1])) > 0.5 {
                continue;
            }

            let e2 = opp_v(e, self.triangles[f.0]);
            let e3 = opp_v((e.1, e.0), self.triangles[f.1]);

            let control_edge = if e2 < e3 { (e2, e3) } else { (e3, e2) };
            if control_edges.contains(&control_edge) {
                continue;
            }

            let v0 = self.vertices[e.0];
            let v1 = self.vertices[e.1];
            let v2 = self.vertices[e2];
            let v3 = self.vertices[e3];
            let have_center = part_f((v0 + v1 + v2 + v3).scale(0.25)) == model_index;
            let pos = dot(v3 - v0, cross(v1 - v0, v2 - v0)) > 0.0;
            if have_center == pos {
                used_f[f.0] = true;
                used_f[f.1] = true;
                new_triangles.push(Triangle(e.0, e3, e2));
                new_triangles.push(Triangle(e.1, e2, e3));
                control_edges.insert(control_edge);
                result += 1;
            }
        }

        for i in 0..used_f.len() {
            if !used_f[i] {
                new_triangles.push(self.triangles[i]);
            }
        }
        self.triangles = new_triangles;
        result
    }

    pub fn double(
        &mut self,
        width: f32,
        eps: f32,
        model_index: u32,
        part_f: &dyn Fn(Point) -> u32,
        count: usize,
        count_p: usize,
    ) {
        let top = self.get_topology();

        let mut middles = HashMap::<(usize, usize), usize>::new();

        for (&e, &f) in &top.edge_to_face {
            let mut e0 = self.vertices[e.0];
            let mut e1 = self.vertices[e.1];
            let mut m_cur = (e0 + e1).scale(0.5);

            let delta = self.get_normal(self.triangles[f.0]) + self.get_normal(self.triangles[f.1]);
            let l = delta.len();
            if l == 0.0 {
            } else {
                let delta = delta.scale(width / l);

                let mut rm = find_root(part_f, m_cur - delta, m_cur + delta, model_index, count);
                let mut dist = (rm - m_cur).sqr_len();

                for _ in 0..count_p {
                    let m1 = (e0 + m_cur).scale(0.5);
                    let rm1 = find_root(part_f, m1 - delta, m1 + delta, model_index, count);
                    let dist1 = (rm1 - m1).sqr_len();

                    if dist1 > dist {
                        e1 = m_cur;
                        m_cur = m1;
                        rm = rm1;
                        dist = dist1;
                        continue;
                    }

                    let m2 = (m_cur + e1).scale(0.5);
                    let rm2 = find_root(part_f, m2 - delta, m2 + delta, model_index, count);
                    let dist2 = (rm2 - m2).sqr_len();

                    if dist2 > dist {
                        e0 = m_cur;
                        m_cur = m2;
                        rm = rm2;
                        dist = dist2;
                        continue;
                    }
                }

                if dist > eps {
                    let nv = self.vertices.len();
                    self.vertices.push(rm);
                    middles.insert(e, nv);
                    middles.insert((e.1, e.0), nv);
                }
            }
        }

        let mut new_triangles = Vec::<Triangle>::new();
        for &t in &self.triangles {
            let m0 = middles.get(&(t.1, t.2));
            let m1 = middles.get(&(t.2, t.0));
            let m2 = middles.get(&(t.0, t.1));

            if let Some(&m0) = m0 {
                if let Some(&m1) = m1 {
                    if let Some(&m2) = m2 {
                        new_triangles.push(Triangle(t.0, m2, m1));
                        new_triangles.push(Triangle(m0, t.2, m1));
                        new_triangles.push(Triangle(m0, m2, t.1));
                        new_triangles.push(Triangle(m0, m1, m2));
                    } else {
                        new_triangles.push(Triangle(m1, m0, t.2));
                        new_triangles.push(Triangle(m0, m1, t.0));
                        new_triangles.push(Triangle(m0, t.0, t.1));
                    }
                } else {
                    if let Some(&m2) = m2 {
                        new_triangles.push(Triangle(m0, m2, t.1));
                        new_triangles.push(Triangle(m2, m0, t.2));
                        new_triangles.push(Triangle(m2, t.2, t.0));
                    } else {
                        new_triangles.push(Triangle(m0, t.0, t.1));
                        new_triangles.push(Triangle(t.0, m0, t.2));
                    }
                }
            } else {
                if let Some(&m1) = m1 {
                    if let Some(&m2) = m2 {
                        new_triangles.push(Triangle(m2, m1, t.0));
                        new_triangles.push(Triangle(m1, m2, t.1));
                        new_triangles.push(Triangle(m1, t.1, t.2));
                    } else {
                        new_triangles.push(Triangle(m1, t.1, t.2));
                        new_triangles.push(Triangle(t.1, m1, t.0));
                    }
                } else {
                    if let Some(&m2) = m2 {
                        new_triangles.push(Triangle(m2, t.2, t.0));
                        new_triangles.push(Triangle(t.2, m2, t.1));
                    } else {
                        new_triangles.push(t);
                    }
                }
            }
        }

        self.triangles = new_triangles;
    }

    pub fn optimize(&mut self, width: f32, model_index: u32, f: &dyn Fn(Point) -> u32) -> bool {
        let mut deleted_triangles = Vec::<bool>::new();
        deleted_triangles.resize(self.triangles.len(), false);

        let mut t_of_v = Vec::<Vec<usize>>::new();
        t_of_v.resize_with(self.vertices.len(), || Vec::new());

        let mut edges = HashSet::<(usize, usize)>::new();
        let mut additional_triangles = Vec::<Triangle>::new();

        for i in 0..self.triangles.len() {
            let t = self.triangles[i];
            t_of_v[t.0].push(i);
            t_of_v[t.1].push(i);
            t_of_v[t.2].push(i);
            if !edges.insert((t.0, t.1)) {
                panic!("edge {}:{} is used twice!", t.0, t.1);
            }
            if !edges.insert((t.1, t.2)) {
                panic!("edge {}:{} is used twice!", t.1, t.2);
            }
            if !edges.insert((t.2, t.0)) {
                panic!("edge {}:{} is used twice!", t.2, t.0);
            }
        }

        let mut rng = rand::thread_rng();
        let mut v_indices: Vec<usize> = (0..self.vertices.len()).collect();
        v_indices.shuffle(&mut rng);

        'check_v: for i in v_indices {
            let t = &t_of_v[i];
            if t.is_empty() {
                continue;
            }

            let mut next_v = HashMap::<usize, usize>::new();
            for &t in t {
                if deleted_triangles[t] {
                    continue 'check_v;
                }
                let t = self.triangles[t];
                if t.0 == i {
                    if next_v.insert(t.1, t.2).is_some() {
                        println!("fail with {i}");
                        for &t in &t_of_v[i] {
                            println!("t={:?}", self.triangles[t]);
                        }
                        panic!("edge {}:{} is used before!", t.1, t.2);
                    }
                } else if t.1 == i {
                    if next_v.insert(t.2, t.0).is_some() {
                        println!("fail with {i}");
                        for &t in &t_of_v[i] {
                            println!("t={:?}", self.triangles[t]);
                        }
                        panic!("edge {}:{} is used before!", t.2, t.0);
                    }
                } else if t.2 == i {
                    if next_v.insert(t.0, t.1).is_some() {
                        println!("fail with {i}");
                        for &t in &t_of_v[i] {
                            println!("t={:?}", self.triangles[t]);
                        }
                        panic!("edge {}:{} is used before!", t.0, t.1);
                    }
                }
            }

            let validate = |t: Triangle| -> bool {
                let n = self.get_normal(t);

                let validate_p = |p: Point| -> bool {
                    if f(p) == model_index {
                        f(p + n.scale(width)) != model_index
                    } else {
                        f(p - n.scale(width)) == model_index
                    }
                };

                let v0 = self.vertices[t.0];
                let v1 = self.vertices[t.1];
                let v2 = self.vertices[t.2];

                validate_p(v0)
                    && validate_p(v1)
                    && validate_p(v2)
                    && validate_p((v0 + v1).scale(0.5))
                    && validate_p((v0 + v2).scale(0.5))
                    && validate_p((v1 + v2).scale(0.5))
            };

            for (&v, &nv) in &next_v {
                let start = v;
                let start_next = nv;
                let mut n1 = nv;
                let mut n2 = next_v.get(&n1).copied().unwrap();
                let mut ok = true;
                while n2 != start {
                    if n1 != start_next && edges.contains(&(start, n1)) {
                        ok = false;
                        break;
                    }

                    if !validate(Triangle(start, n1, n2)) {
                        ok = false;
                        break;
                    }

                    n1 = n2;
                    n2 = next_v.get(&n1).copied().unwrap();
                }

                if ok {
                    let start = v;
                    let start_next = nv;
                    let mut n1 = nv;
                    let mut n2 = next_v.get(&n1).copied().unwrap();
                    while n2 != start {
                        if n1 != start_next {
                            if !edges.insert((start, n1)) {
                                panic!("edge {}:{} is used twice!", start, n1);
                            }
                            if !edges.insert((n1, start)) {
                                panic!("edge {}:{} is used twice!", start, n1);
                            }
                        }

                        additional_triangles.push(Triangle(start, n1, n2));
                        n1 = n2;
                        n2 = next_v.get(&n1).copied().unwrap();
                    }

                    for &t in t {
                        deleted_triangles[t] = true;
                    }

                    continue 'check_v;
                }
            }
        }

        let result = !additional_triangles.is_empty();

        for i in 0..self.triangles.len() {
            if !deleted_triangles[i] {
                additional_triangles.push(self.triangles[i]);
            }
        }

        self.triangles = additional_triangles;
        result
    }

    pub fn delete_unused_v(&mut self) {
        const BAD_INDEX: usize = usize::MAX;
        let mut mapping = Vec::<usize>::new();
        mapping.resize(self.vertices.len(), usize::MAX);

        let mut result = Vec::<Point>::new();

        let mut use_v = |v: &mut usize| {
            if mapping[*v] == BAD_INDEX {
                mapping[*v] = result.len();
                result.push(self.vertices[*v]);
            }

            *v = mapping[*v];
        };

        for t in &mut self.triangles {
            use_v(&mut t.0);
            use_v(&mut t.1);
            use_v(&mut t.2);
        }

        self.vertices = result;
    }

    fn get_normal(&self, t: Triangle) -> Point {
        cross(
            self.vertices[t.1] - self.vertices[t.0],
            self.vertices[t.2] - self.vertices[t.0],
        )
        .norm()
    }

    /// panics if mesh is not closed
    pub fn validate_and_delete_small_groups(&mut self) {
        let topology = self.get_topology();

        let mut face_group = Vec::<usize>::new();
        face_group.resize(self.triangles.len(), 0);

        let mut for_visit = Vec::<usize>::new();
        let mut last_group = 0;

        for i in 0..face_group.len() {
            if face_group[i] == 0 {
                last_group += 1;
                for_visit.push(i);
                while let Some(f) = for_visit.pop() {
                    face_group[f] = last_group;
                    for &f in &topology.face_adj[f] {
                        if face_group[f] == 0 {
                            for_visit.push(f);
                        }
                    }
                }
            }
        }

        let mut counts = Vec::<usize>::new();
        counts.resize(last_group, 0);
        for f in &face_group {
            counts[*f - 1] += 1;
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
            if face_group[i] == max_index {
                new_t.push(self.triangles[i]);
            }
        }
        self.triangles = new_t;
    }

    pub fn split_by(self, t_to_i: &dyn Fn(usize) -> usize) -> Vec<Model> {
        #[derive(Default)]
        struct Mapping {
            v: HashMap<usize, usize>, // vertex_id -> new_vertex_id
            m: Model,
        }

        let mut mappings = HashMap::<usize, Mapping>::new(); // model_id -> mapping

        for i in 0..self.triangles.len() {
            let ti = t_to_i(i);
            let mapping = mappings.entry(ti).or_default();
            let t = self.triangles[i];

            let mut use_mapping = |v| {
                *mapping.v.entry(v).or_insert_with(|| {
                    let result = mapping.m.vertices.len();
                    mapping.m.vertices.push(self.vertices[v]);
                    result
                })
            };

            let t0 = use_mapping(t.0);
            let t1 = use_mapping(t.1);
            let t2 = use_mapping(t.2);
            mapping.m.triangles.push(Triangle(t0, t1, t2));
        }

        mappings.into_iter().map(|(_, m)| m.m).collect()
    }

    pub fn out_of_center(&mut self, factor: f32) {
        let mut sum = Point::zero();
        for v in &self.vertices {
            sum += *v;
        }
        sum = sum.scale(1.0 / self.vertices.len() as f32);
        for v in &mut self.vertices {
            *v += sum.scale(factor);
        }
    }

    fn add_to_convex(triangles: &[Triangle], vertices: &[Point], vi: usize) -> Vec<Triangle> {
        // add_to_convex stuff
        fn flip2(edges: &mut HashSet<(usize, usize)>, t0: usize, t1: usize) {
            if !edges.remove(&(t1, t0)) {
                edges.insert((t0, t1));
            }
        }

        fn flip3(edges: &mut HashSet<(usize, usize)>, t0: usize, t1: usize, t2: usize) {
            flip2(edges, t0, t1);
            flip2(edges, t1, t2);
            flip2(edges, t2, t0);
        }

        fn validate(edges: &HashSet<(usize, usize)>, buffer: &mut HashMap<usize, usize>) -> bool {
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

        let mut vols = Vec::<(f32, usize)>::new();

        for i in 0..triangles.len() {
            let t = triangles[i];
            let vol = dot(
                vertices[vi] - vertices[t.0],
                cross(vertices[t.1] - vertices[t.0], vertices[t.2] - vertices[t.0]),
            );
            vols.push((vol, i));
        }

        vols.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut split = 0;
        while split < vols.len() && vols[split].0 < 0.0 {
            split += 1;
        }

        let mut edges = HashSet::<(usize, usize)>::new();
        for i in 0..split {
            let t = triangles[vols[i].1];
            flip3(&mut edges, t.2, t.1, t.0);
        }

        let mut buffer = HashMap::<usize, usize>::new();

        if !validate(&edges, &mut buffer) {
            assert!(split > 0);
            assert!(split < vols.len() - 1);

            let mut s_down = split - 1;
            let mut s_up = split;

            let mut edges_up = edges.clone();

            loop {
                let t = triangles[vols[s_down].1];
                // reversed
                flip3(&mut edges, t.0, t.1, t.2);
                if validate(&edges, &mut buffer) {
                    split = s_down;
                    break;
                }
                assert!(s_down > 0);
                s_down -= 1;

                let t = triangles[vols[s_up].1];
                flip3(&mut edges_up, t.2, t.1, t.0);
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
            new_t.push(triangles[vols[i].1]);
        }

        for &(v0, v1) in &edges {
            new_t.push(Triangle(v0, v1, vi));
        }
        new_t
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

        result.push(Triangle(0, i1, i2));
        result.push(Triangle(0, i2, i3));
        result.push(Triangle(0, i3, i1));
        result.push(Triangle(i3, i2, i1));

        for i in 1..vertices.len() {
            if i == i1 || i == i2 || i == i3 {
                continue;
            }
            result = Self::add_to_convex(&result, vertices, i);
        }

        Some(result)
    }

    pub fn convex(vertices: &[Point], eps: f32) -> Option<Self> {
        let triangles = Self::convex_triangles(vertices, eps)?;
        Some(Self {
            vertices: vertices.to_owned(),
            triangles,
        })
    }

    pub fn save_to_stl(&self, path: &std::path::Path) -> Result<(), String> {
        let mut file = std::io::BufWriter::new(std::fs::File::create(&path).map_err(|e| {
            format!(
                "Unable to open file {} for writing: {}",
                path.to_string_lossy(),
                e
            )
        })?);

        let triangle_iter = self.triangles.iter().map(|t| {
            let v0 = self.vertices[t.0];
            let v1 = self.vertices[t.1];
            let v2 = self.vertices[t.2];
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

        stl_io::write_stl(&mut file, triangle_iter).map_err(|e| {
            format!(
                "Failed to save stl to file {}: {}",
                path.to_string_lossy(),
                e
            )
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convex_floating_pituh() {
        let v2c = [
            Point {
                x: 17.226563,
                y: -17.226563,
                z: -17.5,
            },
            Point {
                x: 17.226563,
                y: -17.5,
                z: -17.226563,
            },
            Point {
                x: 17.226563,
                y: -17.226563,
                z: -17.226563,
            },
            Point {
                x: 17.5,
                y: -17.226563,
                z: -17.226563,
            },
            Point {
                x: 17.226563,
                y: -17.235107,
                z: -17.5,
            },
            Point {
                x: 17.226563,
                y: -17.5,
                z: -17.235107,
            },
            Point {
                x: 17.235107,
                y: -17.226563,
                z: -17.5,
            },
            Point {
                x: 17.235107,
                y: -17.5,
                z: -17.226563,
            },
            Point {
                x: 17.5,
                y: -17.226563,
                z: -17.235107,
            },
            Point {
                x: 17.5,
                y: -17.235107,
                z: -17.226563,
            },
        ];
        let mut test = Model::convex(&v2c, 0.0).unwrap();
        test.validate_and_delete_small_groups();
    }
}
