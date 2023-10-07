use crate::model::*;
use crate::points3d::*;
use std::collections::HashMap;

pub type PartIndex = u32;
const BAD_INDEX: PartIndex = 0xFFFFFFFF;

#[derive(Debug, Clone, Copy)]
pub struct SolidCell {
    index: PartIndex,
    pos: Point,
    v_pzz: u32,
    v_mzz: u32,
    v_zpz: u32,
    v_zmz: u32,
    v_zzp: u32,
    v_zzm: u32,
    v_ppz: u32,
    v_mmz: u32,
    v_pzp: u32,
    v_mzm: u32,
    v_zpp: u32,
    v_zmm: u32,
    v_ppp: u32,
    v_mmm: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SolidCellEdges {}

impl SolidCell {
    pub fn new() -> Self {
        Self {
            index: 0,
            pos: Point::zero(),
            v_pzz: BAD_INDEX,
            v_mzz: BAD_INDEX,
            v_zpz: BAD_INDEX,
            v_zmz: BAD_INDEX,
            v_zzp: BAD_INDEX,
            v_zzm: BAD_INDEX,
            v_ppz: BAD_INDEX,
            v_mmz: BAD_INDEX,
            v_pzp: BAD_INDEX,
            v_mzm: BAD_INDEX,
            v_zpp: BAD_INDEX,
            v_zmm: BAD_INDEX,
            v_ppp: BAD_INDEX,
            v_mmm: BAD_INDEX,
        }
    }
}
pub struct SolidLayer {
    size: usize,
    cells: Vec<SolidCell>,
}

impl SolidLayer {
    pub fn new_zero(size: usize, solid_size: f32, z: f32) -> Self {
        let mut cells = vec![SolidCell::new(); (size + 1) * (size + 1)];
        let mut idx = 0;
        for y in 0..=size {
            for x in 0..=size {
                let pos = Point {
                    x: (x as f32 / size as f32 - 0.5) * solid_size,
                    y: (y as f32 / size as f32 - 0.5) * solid_size,
                    z,
                };
                cells[idx].index = 0;
                cells[idx].pos = pos;
                idx += 1;
            }
        }

        Self { size, cells }
    }

    pub fn fill_by(&mut self, z: f32, part_f: &dyn Fn(Point) -> PartIndex) {
        let mut idx = 0;
        for y in 0..=self.size {
            for x in 0..=self.size {
                self.cells[idx].v_pzz = BAD_INDEX;
                self.cells[idx].v_mzz = BAD_INDEX;
                self.cells[idx].v_zpz = BAD_INDEX;
                self.cells[idx].v_zmz = BAD_INDEX;
                self.cells[idx].v_zzp = BAD_INDEX;
                self.cells[idx].v_zzm = BAD_INDEX;
                self.cells[idx].v_ppz = BAD_INDEX;
                self.cells[idx].v_mmz = BAD_INDEX;
                self.cells[idx].v_pzp = BAD_INDEX;
                self.cells[idx].v_mzm = BAD_INDEX;
                self.cells[idx].v_zpp = BAD_INDEX;
                self.cells[idx].v_zmm = BAD_INDEX;
                self.cells[idx].v_ppp = BAD_INDEX;
                self.cells[idx].v_mmm = BAD_INDEX;

                self.cells[idx].pos.z = z;
                if y == 0 || y == self.size || x == 0 || x == self.size {
                    self.cells[idx].index = 0;
                } else {
                    self.cells[idx].index = part_f(self.cells[idx].pos);
                }

                idx += 1;
            }
        }
    }

    pub fn fill_zero(&mut self, z: f32) {
        self.fill_by(z, &|_| 0);
    }
}

pub struct ModelCreator {
    size: usize,
    solid_size: f32,
    models: HashMap<u32, Model>,
    prev_layer: SolidLayer,
    next_layer: SolidLayer,
    used_numbers: Vec<u32>,
    last_z: usize,
    tries: usize,
}

impl ModelCreator {
    pub fn new(size: usize, solid_size: f32, tries: usize) -> Self {
        Self {
            size,
            solid_size,
            models: HashMap::new(),
            prev_layer: SolidLayer::new_zero(size, solid_size, Self::first_z(solid_size)),
            next_layer: SolidLayer::new_zero(size, solid_size, Self::first_z(solid_size)),
            used_numbers: Vec::new(),
            last_z: 0,
            tries,
        }
    }

    pub fn get_models(self) -> HashMap<u32, Model> {
        self.models
    }

    fn first_z(solid_size: f32) -> f32 {
        -0.5 * solid_size
    }

    fn count_z(&self) -> f32 {
        (self.last_z as f32 / self.size as f32 - 0.5) * self.solid_size
    }

    pub fn finished(&self) -> bool {
        self.last_z == self.size
    }

    fn fill_tetrahedron(
        model: &mut Model,
        part_f: &dyn Fn(Point) -> u32,
        tries: usize,
        model_index: PartIndex,
        (c0, c0_inside, e01, e02, e03): (Point, bool, &mut u32, &mut u32, &mut u32),
        (c1, c1_inside, e10, e12, e13): (Point, bool, &mut u32, &mut u32, &mut u32),
        (c2, c2_inside, e20, e21, e23): (Point, bool, &mut u32, &mut u32, &mut u32),
        (c3, c3_inside, e30, e31, e32): (Point, bool, &mut u32, &mut u32, &mut u32),
    ) {
        let root = |mut p1: Point, mut p2: Point| {
            let mut i = 0;
            loop {
                i += 1;
                let mid = (p1 + p2).scale(0.5);
                if i >= tries {
                    return mid;
                }
                if part_f(mid) == model_index {
                    p1 = mid;
                } else {
                    p2 = mid;
                }
            }
        };

        let add_t = |model: &mut Model, v0: u32, v1: u32, v2: u32| {
            model
                .triangles
                .push(Triangle(v0 as usize, v1 as usize, v2 as usize));
        };

        let it = |model: &mut Model,
                  corner: Point,
                  c0: Point,
                  p0: &mut u32,
                  c1: Point,
                  p1: &mut u32,
                  c2: Point,
                  p2: &mut u32| {
            if *p0 == BAD_INDEX {
                *p0 = model.add_vertex(root(corner, c0)) as u32;
            }
            if *p1 == BAD_INDEX {
                *p1 = model.add_vertex(root(corner, c1)) as u32;
            }
            if *p2 == BAD_INDEX {
                *p2 = model.add_vertex(root(corner, c2)) as u32;
            }

            add_t(model, *p0, *p1, *p2);
        };

        let ot = |model: &mut Model,
                  corner: Point,
                  c0: Point,
                  p0: &mut u32,
                  c1: Point,
                  p1: &mut u32,
                  c2: Point,
                  p2: &mut u32| {
            if *p0 == BAD_INDEX {
                *p0 = model.add_vertex(root(c0, corner)) as u32;
            }
            if *p1 == BAD_INDEX {
                *p1 = model.add_vertex(root(c1, corner)) as u32;
            }
            if *p2 == BAD_INDEX {
                *p2 = model.add_vertex(root(c2, corner)) as u32;
            }

            add_t(model, *p0, *p1, *p2);
        };

        let q = |model: &mut Model,
                 c0: Point,
                 p0: &mut u32,
                 c1: Point,
                 p1: &mut u32,
                 c2: Point,
                 p2: &mut u32,
                 c3: Point,
                 p3: &mut u32| {
            if *p0 == BAD_INDEX {
                *p0 = model.add_vertex(root(c0, c1)) as u32;
            }
            if *p1 == BAD_INDEX {
                *p1 = model.add_vertex(root(c2, c1)) as u32;
            }
            if *p2 == BAD_INDEX {
                *p2 = model.add_vertex(root(c2, c3)) as u32;
            }
            if *p3 == BAD_INDEX {
                *p3 = model.add_vertex(root(c0, c3)) as u32;
            }

            let center = (model.vertices[*p0 as usize]
                + model.vertices[*p1 as usize]
                + model.vertices[*p2 as usize]
                + model.vertices[*p3 as usize])
                .scale(0.25);

            let center_ok = part_f(center) == model_index;

            
            if center_ok {
                add_t(model, *p0, *p1, *p2);
                add_t(model, *p0, *p2, *p3);
            } else {
                add_t(model, *p0, *p1, *p3);
                add_t(model, *p3, *p1, *p2);
            }
        };

        if c0_inside {
            if c1_inside {
                if c2_inside {
                    if c3_inside {
                        // skip
                    } else {
                        // 012|3
                        ot(model, c3, c0, e03, c1, e13, c2, e23);
                    }
                } else {
                    if c3_inside {
                        // 013|2
                        ot(model, c2, c0, e02, c3, e32, c1, e12);
                    } else {
                        // 01|23
                        q(model, c0, e03, c3, e13, c1, e12, c2, e02);
                    }
                }
            } else {
                if c2_inside {
                    if c3_inside {
                        // 023|1
                        ot(model, c1, c0, e01, c2, e21, c3, e31);
                    } else {
                        // 02|13
                        q(model, c0, e01, c1, e21, c2, e23, c3, e03);
                    }
                } else {
                    if c3_inside {
                        // 03|12
                        q(model, c0, e02, c2, e32, c3, e31, c1, e01);
                    } else {
                        // 0|123
                        it(model, c0, c1, e01, c2, e02, c3, e03);
                    }
                }
            }
        } else {
            if c1_inside {
                if c2_inside {
                    if c3_inside {
                        // 123|0
                        ot(model, c0, c1, e10, c3, e30, c2, e20);
                    } else {
                        // 12|03
                        q(model, c1, e13, c3, e23, c2, e20, c0, e10);
                    }
                } else {
                    if c3_inside {
                        // 13|02
                        q(model, c1, e10, c0, e30, c3, e32, c2, e12);
                    } else {
                        // 1|023
                        it(model, c1, c0, e10, c3, e13, c2, e12);
                    }
                }
            } else {
                if c2_inside {
                    if c3_inside {
                        // 23|01
                        q(model, c2, e21, c1, e31, c3, e30, c0, e20);
                    } else {
                        // 2|013
                        it(model, c2, c0, e20, c1, e21, c3, e23);
                    }
                } else {
                    if c3_inside {
                        // 3|012
                        it(model, c3, c0, e30, c2, e32, c1, e31);
                    } else {
                        // skip
                    }
                }
            }
        }
    }

    fn use_layers(&mut self, part_f: &dyn Fn(Point) -> u32) {
        let pl = self.prev_layer.cells.as_mut_slice();
        let nl = self.next_layer.cells.as_mut_slice();

        for y in 0..self.size {
            for x in 0..self.size {
                let c = y * (self.size + 1) + x;
                let cx = c + 1;
                let cy = c + self.size + 1;
                let cxy = c + self.size + 2;
                let corner_index = pl[c].index;

                if corner_index != pl[cx].index
                    || corner_index != pl[cy].index
                    || corner_index != pl[cxy].index
                    || corner_index != nl[c].index
                    || corner_index != nl[cx].index
                    || corner_index != nl[cy].index
                    || corner_index != nl[cxy].index
                {
                    self.used_numbers.clear();
                    let mut use_number = |i| {
                        if i != 0 && !self.used_numbers.contains(&i) {
                            self.used_numbers.push(i);
                        }
                    };

                    use_number(corner_index);
                    use_number(pl[cx].index);
                    use_number(pl[cy].index);
                    use_number(pl[cxy].index);
                    use_number(nl[c].index);
                    use_number(nl[cx].index);
                    use_number(nl[cy].index);
                    use_number(nl[cxy].index);

                    for &model_index in &self.used_numbers {
                        let model = self.models.entry(model_index).or_insert(Model::new());

                        macro_rules! vertex {
                            ($l: expr, $c: expr, $e0: ident, $e1: ident, $e2: ident) => {
                                (
                                    $l[$c].pos,
                                    $l[$c].index == model_index,
                                    &mut $l[$c].$e0,
                                    &mut $l[$c].$e1,
                                    &mut $l[$c].$e2,
                                )
                            };
                        }

                        Self::fill_tetrahedron(
                            model,
                            part_f,
                            self.tries,
                            model_index,
                            vertex!(pl, c, v_pzz, v_ppz, v_ppp),
                            vertex!(pl, cx, v_mzz, v_zpz, v_zpp),
                            vertex!(pl, cxy, v_mmz, v_zmz, v_zzp),
                            vertex!(nl, cxy, v_mmm, v_zmm, v_zzm),
                        );

                        Self::fill_tetrahedron(
                            model,
                            part_f,
                            self.tries,
                            model_index,
                            vertex!(pl, c, v_zpz, v_zpp, v_ppp),
                            vertex!(pl, cy, v_zmz, v_zzp, v_pzp),
                            vertex!(nl, cy, v_zmm, v_zzm, v_pzz),
                            vertex!(nl, cxy, v_mmm, v_mzm, v_mzz),
                        );
                        
                        Self::fill_tetrahedron(
                            model,
                            part_f,
                            self.tries,
                            model_index,
                            vertex!(pl, c, v_zzp, v_pzp, v_ppp),
                            vertex!(nl, c, v_zzm, v_pzz, v_ppz),
                            vertex!(nl, cx, v_mzm, v_mzz, v_zpz),
                            vertex!(nl, cxy, v_mmm, v_mmz, v_zmz),
                        );

                        Self::fill_tetrahedron(
                            model,
                            part_f,
                            self.tries,
                            model_index,
                            vertex!(pl, c, v_ppz, v_zpz, v_ppp),
                            vertex!(pl, cxy, v_mmz, v_mzz, v_zzp),
                            vertex!(pl, cy, v_zmz, v_pzz, v_pzp),
                            vertex!(nl, cxy, v_mmm, v_zzm, v_mzm),
                        );

                        Self::fill_tetrahedron(
                            model,
                            part_f,
                            self.tries,
                            model_index,
                            vertex!(pl, c, v_zpp, v_zzp, v_ppp),
                            vertex!(nl, cy, v_zmm, v_zmz, v_pzz),
                            vertex!(nl, c, v_zzm, v_zpz, v_ppz),
                            vertex!(nl, cxy, v_mmm, v_mzz, v_mmz),
                        );

                        Self::fill_tetrahedron(
                            model,
                            part_f,
                            self.tries,
                            model_index,
                            vertex!(pl, c, v_pzp, v_pzz, v_ppp),
                            vertex!(nl, cx, v_mzm, v_zzm, v_zpz),
                            vertex!(pl, cx, v_mzz, v_zzp, v_zpp),
                            vertex!(nl, cxy, v_mmm, v_zmz, v_zmm),
                        );
                    }
                }
            }
        }
    }

    pub fn fill_next_layer(&mut self, part_f: &dyn Fn(Point) -> PartIndex, width: f32) {
        std::mem::swap(&mut self.next_layer, &mut self.prev_layer);
        self.last_z += 1;
        let z = self.count_z();
        if self.finished() {
            self.next_layer.fill_zero(z);
        } else {
            self.next_layer.fill_by(z, part_f);
        }
        self.use_layers(part_f);

        println!("processed [{}/{}] layers", self.last_z, self.size);
    }
}
