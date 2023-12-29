use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::points3d::*;
use fxhash::{FxHashMap, FxHashSet};

// use u32 because of memory optimization
#[derive(Debug, Clone, Copy)]
pub struct Triangle(pub u32, pub u32, pub u32);

#[derive(Debug, Clone, Default)]
pub struct Model {
    pub vertices: Vec<Point>,
    pub triangles: Vec<Triangle>,
    pub free_vertices: Vec<u32>,
}

#[derive(Debug, Default)]
pub struct MeshTopology {
    edge_to_face: FxHashMap<(u32, u32), (u32, u32)>,
    face_adj: Vec<[u32; 3]>,
}

#[derive(Debug, Clone, Default)]
pub struct ArrayBuffer {
    pub v: Vec<f32>,
    pub i: Vec<u32>,
}

impl Model {
    pub fn new() -> Self {
        Self { vertices: Vec::new(), triangles: Vec::new(), free_vertices: Vec::new() }
    }

    pub fn add_vertex(&mut self, p: Point) -> u32 {
        if let Some(i) = self.free_vertices.pop() {
            self.vertices[i as usize] = p;
            return i;
        }
        self.vertices.push(p);
        self.vertices.len() as u32 - 1
    }

    pub fn write_to_buffer(&self, buffer: &mut ArrayBuffer, color: u32) {
        buffer.v.reserve(buffer.v.len() + self.vertices.len() * 6);
        buffer.i.reserve(buffer.i.len() + self.triangles.len() * 3);

        let old_v_len = (buffer.v.len() / 6) as u32;

        for v in &self.vertices {
            buffer.v.push(v.x);
            buffer.v.push(v.y);
            buffer.v.push(v.z);
            buffer.v.push((color >> 16 & 0xff) as f32 / 255.0);
            buffer.v.push((color >> 8 & 0xff) as f32 / 255.0);
            buffer.v.push((color & 0xff) as f32 / 255.0);
        }
        for t in &self.triangles {
            buffer.i.push(old_v_len + t.0);
            buffer.i.push(old_v_len + t.1);
            buffer.i.push(old_v_len + t.2);
        }
    }

    pub fn append(&mut self, other: Self) {
        self.triangles
            .reserve(self.triangles.len() + other.triangles.len());
        self.vertices
            .reserve(self.vertices.len() + other.vertices.len());
        let shift = self.vertices.len() as u32;
        for t in other.triangles {
            self.triangles
                .push(Triangle(t.0 + shift, t.1 + shift, t.2 + shift));
        }
        for v in other.vertices {
            self.vertices.push(v);
        }
    }

    pub fn smooth(&mut self, delta: f32) {
        let mut positions = vec![(Point::zero(), 0); self.vertices.len()];
        for t in &self.triangles {
            let t0 = t.0 as usize;
            let t1 = t.1 as usize;
            let t2 = t.2 as usize;
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

    pub fn get_topology(&self) -> MeshTopology {
        let mut edge_to_face = FxHashMap::<(u32, u32), (u32, u32)>::default();
        const BAD_INDEX: u32 = u32::MAX;
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
            make_edge(i as u32, t.0, t.1);
            make_edge(i as u32, t.1, t.2);
            make_edge(i as u32, t.2, t.0);
        }

        let mut bad = false;
        for ((v1, v2), (f1, f2)) in &edge_to_face {
            if *f1 == BAD_INDEX || *f2 == BAD_INDEX {
                println!("v1={}, v2={}, f1={}, f2={}", v1, v2, f1, f2);
                bad = true;
            }
        }
        if bad {
            panic!("mesh is incomplete");
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
            use_edge(&mut face_adj[i][0], t.0, t.1);
            use_edge(&mut face_adj[i][1], t.1, t.2);
            use_edge(&mut face_adj[i][2], t.2, t.0);
        }

        MeshTopology { edge_to_face, face_adj }
    }

    pub fn v_near_t(&self, v_index: u32, t: Triangle, eps: f32) -> bool {
        let v = self.vertices[v_index as usize];
        let v0 = v - self.vertices[t.0 as usize];
        let n = self.get_normal(t);

        if dot(v0, n).abs() > eps {
            return false;
        }

        let v1 = v - self.vertices[t.1 as usize];
        let v2 = v - self.vertices[t.2 as usize];

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

    pub fn optimize(&mut self, width: f32, group_dot: f32, min_group_size: u32, smooth_dot: f32) {
        let mut v_of_t = FxHashMap::<u32, Vec<u32>>::default();

        let mut group_of_t = Vec::<u32>::new();
        group_of_t.resize(self.triangles.len(), 0);
        let mut group_counts = Vec::<u32>::new();
        let top = self.get_topology();

        for ti in 0..self.triangles.len() {
            if group_of_t[ti] != 0 {
                continue;
            }

            let cn = self.get_normal(self.triangles[ti]);
            let mut stack = Vec::<u32>::new();
            stack.push(ti as u32);
            while let Some(cur_ti) = stack.pop() {
                for new_ti in top.face_adj[cur_ti as usize] {
                    if group_of_t[new_ti as usize] != 0 {
                        continue;
                    }

                    let nn = self.get_normal(self.triangles[new_ti as usize]);
                    if dot(nn, cn) <= group_dot {
                        continue;
                    }
                    if group_of_t[ti] == 0 {
                        group_counts.push(0);
                        group_of_t[ti] = group_counts.len() as u32;
                        *group_counts.last_mut().unwrap() += 1;
                    }

                    group_of_t[new_ti as usize] = group_counts.len() as u32;
                    *group_counts.last_mut().unwrap() += 1;
                    stack.push(new_ti);
                }
            }
        }

        for i in 0..group_of_t.len() {
            if group_of_t[i] != 0 && group_counts[group_of_t[i] as usize - 1] < min_group_size {
                group_of_t[i] = 0;
            }
        }

        drop(top);

        let mut mapping = FxHashMap::<Vec<u32>, u32>::default();
        let mut g_of_v = Vec::<Vec<u32>>::new();
        g_of_v.resize_with(self.vertices.len(), Default::default);

        for i in 0..self.triangles.len() {
            let t = self.triangles[i];
            g_of_v[t.0 as usize].push(group_of_t[i]);
            g_of_v[t.1 as usize].push(group_of_t[i]);
            g_of_v[t.2 as usize].push(group_of_t[i]);
        }

        let gi_of_v: Vec<u32> = g_of_v
            .iter_mut()
            .map(|g| {
                g.sort();
                g.dedup();
                let l = mapping.len();
                *mapping.entry(g.clone()).or_insert(l as u32)
            })
            .collect();

        fn is_subset(a: &[u32], b: &[u32]) -> bool {
            let mut i = 0;
            for &b in b {
                if i == a.len() {
                    return true;
                }

                if a[i] < b {
                    return false;
                }

                if a[i] > b {
                    continue;
                }

                if a[i] == b {
                    i += 1;
                }
            }

            i == a.len()
        }

        let mut subsets = Vec::<FxHashSet<u32>>::new();
        subsets.resize_with(g_of_v.len(), Default::default);

        println!("build inheritance...");
        for (g1, &gi1) in &mapping {
            for b in 0..=1 << g1.len() {
                let mut g2 = Vec::new();
                for i in 0..g1.len() {
                    if ((b >> i) & 1) == 1 {
                        g2.push(g1[i]);
                    }
                }
                if let Some(&gi2) = mapping.get(&g2) {
                    subsets[gi1 as usize].insert(gi2);
                }
            }
        }
        println!("/build inheritance...");
        drop(mapping);
        drop(g_of_v);

        let mut suka_count = 0;
        loop {
            println!("optimization state; tcount={}", self.triangles.len());
            let mut deleted_triangles = Vec::<bool>::new();
            deleted_triangles.resize(self.triangles.len(), false);

            let mut t_of_v = Vec::<Vec<u32>>::new();
            t_of_v.resize_with(self.vertices.len(), || Vec::new());

            let mut edges = FxHashSet::<(u32, u32)>::default();
            let mut additional_triangles = Vec::<Triangle>::new();

            let mut new_v_of_t = FxHashMap::<u32, Vec<u32>>::default();

            for i in 0..self.triangles.len() {
                let t = self.triangles[i];
                let t0 = t.0 as usize;
                let t1 = t.1 as usize;
                let t2 = t.2 as usize;
                t_of_v[t0].push(i as u32);
                t_of_v[t1].push(i as u32);
                t_of_v[t2].push(i as u32);
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

            let mut border_v = FxHashSet::default();
            for &(v1, v2) in &edges {
                if !edges.contains(&(v2, v1)) {
                    border_v.insert(v1);
                    border_v.insert(v2);
                }
            }

            //let mut rng = rand::thread_rng();
            let mut rng = rand::rngs::StdRng::seed_from_u64(2);
            let mut v_indices: Vec<u32> = (0..self.vertices.len() as u32)
                .filter(|i| !border_v.contains(i))
                .collect();
            v_indices.shuffle(&mut rng);

            'check_v: for i in v_indices {
                let t = &t_of_v[i as usize];
                if t.is_empty() {
                    continue;
                }

                let mut next_v = FxHashMap::<u32, u32>::default();
                let mut normals = Vec::<Point>::new();

                let mut control_v = FxHashMap::<u32, bool>::default();
                let mut v_of_new_t = FxHashMap::<u32, Vec<u32>>::default();

                control_v.insert(i, false);
                for &t in t {
                    if deleted_triangles[t as usize] {
                        continue 'check_v;
                    }

                    if let Some(v) = v_of_t.get(&t) {
                        for &v in v {
                            control_v.insert(v, false);
                        }
                    }

                    let t = self.triangles[t as usize];
                    if t.0 == i {
                        if next_v.insert(t.1, t.2).is_some() {
                            println!("fail with {i}");
                            for &t in &t_of_v[i as usize] {
                                println!("t={:?}", self.triangles[t as usize]);
                            }
                            panic!("edge {}:{} is used before!", t.1, t.2);
                        }
                    } else if t.1 == i {
                        if next_v.insert(t.2, t.0).is_some() {
                            println!("fail with {i}");
                            for &t in &t_of_v[i as usize] {
                                println!("t={:?}", self.triangles[t as usize]);
                            }
                            panic!("edge {}:{} is used before!", t.2, t.0);
                        }
                    } else if t.2 == i {
                        if next_v.insert(t.0, t.1).is_some() {
                            println!("fail with {i}");
                            for &t in &t_of_v[i as usize] {
                                println!("t={:?}", self.triangles[t as usize]);
                            }
                            panic!("edge {}:{} is used before!", t.0, t.1);
                        }
                    }

                    normals.push(self.get_normal(t));
                }

                let mut validate = |new: Triangle, old: Triangle| -> bool {
                    let p_new = self.get_perp(new);
                    let p_old = self.get_perp(old);
                    let l_new = p_new.len();
                    let l_old = p_old.len();
                    dot(p_new, p_old) > smooth_dot * l_new * l_old
                };

                for (&v, &nv) in &next_v {
                    let mut near_t = |t: Triangle, index: u32| {
                        for (&i, c) in &mut control_v {
                            if *c {
                                continue;
                            }

                            if self.v_near_t(i, t, width) {
                                *c = true;
                                v_of_new_t.entry(index).or_default().push(i);
                            }
                        }
                    };

                    if !subsets[gi_of_v[v as usize] as usize].contains(&gi_of_v[i as usize]) {
                        continue;
                    }

                    let mut n1 = nv;
                    let mut n2 = next_v.get(&n1).copied().unwrap();
                    let mut ok = true;
                    let mut cur_t = Triangle(v, n1, n2);

                    if !validate(Triangle(v, n1, n2), Triangle(i, v, nv)) {
                        ok = false;
                    }

                    while n2 != v {
                        if n1 != nv && edges.contains(&(v, n1)) {
                            // broken topology fix
                            ok = false;
                            break;
                        }

                        cur_t = Triangle(v, n1, n2);

                        if !validate(cur_t, Triangle(i, n1, n2)) {
                            ok = false;
                            break;
                        }

                        near_t(cur_t, n1);

                        n1 = n2;
                        n2 = next_v.get(&n1).copied().unwrap();
                    }

                    if !validate(cur_t, Triangle(i, n1, n2)) {
                        ok = false;
                    }

                    ok &= control_v.iter().all(|(_, &n)| n);

                    if ok {
                        suka_count += 1;
                        let mut n1 = nv;
                        let mut n2 = next_v.get(&n1).copied().unwrap();
                        while n2 != v {
                            if n1 != nv {
                                if !edges.insert((v, n1)) {
                                    panic!("edge {}:{} is used twice!", v, n1);
                                }
                                if !edges.insert((n1, v)) {
                                    panic!("edge {}:{} is used twice!", v, n1);
                                }
                            }

                            if let Some(v) = v_of_new_t.get(&n1) {
                                for &v in v {
                                    new_v_of_t
                                        .entry(additional_triangles.len() as u32)
                                        .or_default()
                                        .push(v);
                                }
                            }

                            additional_triangles.push(Triangle(v, n1, n2));
                            n1 = n2;
                            n2 = next_v.get(&n1).copied().unwrap();
                        }

                        for &t in t {
                            deleted_triangles[t as usize] = true;
                        }

                        self.free_vertices.push(i);

                        continue 'check_v;
                    }

                    for (_, c) in &mut control_v {
                        *c = false;
                    }
                    v_of_new_t.clear();
                }
            }

            let no_changes = additional_triangles.is_empty();

            // fix old triangle indices
            for t in 0..self.triangles.len() {
                if !deleted_triangles[t] {
                    if let Some(v) = v_of_t.remove(&(t as u32)) {
                        new_v_of_t
                            .entry(additional_triangles.len() as u32)
                            .or_default()
                            .extend(v);
                    }
                    additional_triangles.push(self.triangles[t]);
                }
            }

            v_of_t = new_v_of_t;
            self.triangles = additional_triangles;

            for (&t, vs) in &v_of_t {
                for &v in vs {
                    if !self.v_near_t(v, self.triangles[t as usize], width) {
                        panic!("fail with v={}, t={}", v, t);
                    }
                }
            }

            if no_changes {
                break;
            }
        }
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
            use_v(&mut t.0);
            use_v(&mut t.1);
            use_v(&mut t.2);
        }

        self.vertices = result;
        self.free_vertices.clear();
    }

    pub fn get_normal(&self, t: Triangle) -> Point {
        cross(
            self.vertices[t.1 as usize] - self.vertices[t.0 as usize],
            self.vertices[t.2 as usize] - self.vertices[t.0 as usize],
        )
        .norm()
    }

    pub fn get_perp(&self, t: Triangle) -> Point {
        cross(
            self.vertices[t.1 as usize] - self.vertices[t.0 as usize],
            self.vertices[t.2 as usize] - self.vertices[t.0 as usize],
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
                    for &f in &topology.face_adj[f as usize] {
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

    pub fn split_by(self, t_to_i: &dyn Fn(u32) -> u32) -> Vec<Model> {
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
                vertices[vi as usize] - vertices[t.0 as usize],
                cross(
                    vertices[t.1 as usize] - vertices[t.0 as usize],
                    vertices[t.2 as usize] - vertices[t.0 as usize],
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
            flip3(&mut edges, t.2, t.1, t.0);
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
                flip3(&mut edges, t.0, t.1, t.2);
                if validate(&edges, &mut buffer) {
                    split = s_down;
                    break;
                }
                assert!(s_down > 0);
                s_down -= 1;

                let t = triangles[vols[s_up].1 as usize];
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
            new_t.push(triangles[vols[i].1 as usize]);
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

        result.push(Triangle(0, i1 as u32, i2 as u32));
        result.push(Triangle(0, i2 as u32, i3 as u32));
        result.push(Triangle(0, i3 as u32, i1 as u32));
        result.push(Triangle(i3 as u32, i2 as u32, i1 as u32));

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
        Some(Self { vertices: vertices.to_owned(), triangles, free_vertices: Vec::new() })
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
            let v0 = self.vertices[t.0 as usize];
            let v1 = self.vertices[t.1 as usize];
            let v2 = self.vertices[t.2 as usize];
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
        })
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
