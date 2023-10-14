use crate::model::*;
use crate::points3d::*;
use std::collections::HashMap;

pub type PartIndex = u32;
const BAD_INDEX: PartIndex = 0xFFFFFFFF;

#[derive(Debug, Clone, Copy)]
pub struct SolidCell {
    index: PartIndex,
    pos: Point,
    // intermediate edge points
    // p = +1 (plus), m = -1 (minus), z = 0 (zero)
    v_pzz: u32,
    v_mzz: u32,
    v_zpz: u32,
    v_zmz: u32,
    v_zzp: u32,
    v_zzm: u32,
    v_mmm: u32,
    v_mmp: u32,
    v_mpm: u32,
    v_mpp: u32,
    v_pmm: u32,
    v_pmp: u32,
    v_ppm: u32,
    v_ppp: u32,
    // intermediate plane points
    // p = +1 (plus), m = -1 (minus), z = 0 (zero)
    // q = +2 (next letter after p), n = -2 (next letter after m)
    w_mmz: u32,
    w_mpz: u32,
    w_pmz: u32,
    w_ppz: u32,
    w_mzm: u32,
    w_mzp: u32,
    w_pzm: u32,
    w_pzp: u32,
    w_zmm: u32,
    w_zmp: u32,
    w_zpm: u32,
    w_zpp: u32,
    w_mmn: u32,
    w_mnm: u32,
    w_nmm: u32,
    w_mmq: u32,
    w_mnp: u32,
    w_nmp: u32,
    w_mpn: u32,
    w_mqm: u32,
    w_npm: u32,
    w_mpq: u32,
    w_mqp: u32,
    w_npp: u32,
    w_pmn: u32,
    w_pnm: u32,
    w_qmm: u32,
    w_pmq: u32,
    w_pnp: u32,
    w_qmp: u32,
    w_ppn: u32,
    w_pqm: u32,
    w_qpm: u32,
    w_ppq: u32,
    w_pqp: u32,
    w_qpp: u32,
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
            v_mmm: BAD_INDEX,
            v_mmp: BAD_INDEX,
            v_mpm: BAD_INDEX,
            v_mpp: BAD_INDEX,
            v_pmm: BAD_INDEX,
            v_pmp: BAD_INDEX,
            v_ppm: BAD_INDEX,
            v_ppp: BAD_INDEX,
            w_mmz: BAD_INDEX,
            w_mpz: BAD_INDEX,
            w_pmz: BAD_INDEX,
            w_ppz: BAD_INDEX,
            w_mzm: BAD_INDEX,
            w_mzp: BAD_INDEX,
            w_pzm: BAD_INDEX,
            w_pzp: BAD_INDEX,
            w_zmm: BAD_INDEX,
            w_zmp: BAD_INDEX,
            w_zpm: BAD_INDEX,
            w_zpp: BAD_INDEX,
            w_mmn: BAD_INDEX,
            w_mnm: BAD_INDEX,
            w_nmm: BAD_INDEX,
            w_mmq: BAD_INDEX,
            w_mnp: BAD_INDEX,
            w_nmp: BAD_INDEX,
            w_mpn: BAD_INDEX,
            w_mqm: BAD_INDEX,
            w_npm: BAD_INDEX,
            w_mpq: BAD_INDEX,
            w_mqp: BAD_INDEX,
            w_npp: BAD_INDEX,
            w_pmn: BAD_INDEX,
            w_pnm: BAD_INDEX,
            w_qmm: BAD_INDEX,
            w_pmq: BAD_INDEX,
            w_pnp: BAD_INDEX,
            w_qmp: BAD_INDEX,
            w_ppn: BAD_INDEX,
            w_pqm: BAD_INDEX,
            w_qpm: BAD_INDEX,
            w_ppq: BAD_INDEX,
            w_pqp: BAD_INDEX,
            w_qpp: BAD_INDEX,
        }
    }
}
#[derive(Default)]
pub struct SolidLayer {
    cells: Vec<SolidCell>,
}

impl SolidLayer {
    pub fn get_coord(size: usize, solid_size: f32, x: usize, odd: bool) -> f32 {
        let scale = solid_size / (size as f32 - 1.5);
        let shift = if odd {
            -solid_size * 0.5
        } else {
            (-solid_size - scale) * 0.5
        };
        x as f32 * scale + shift
    }

    pub fn filled(
        size: usize,
        solid_size: f32,
        z: f32,
        odd: bool,
        part_f: &dyn Fn(Point) -> PartIndex,
    ) -> Self {
        let mut cells = vec![SolidCell::new(); size * size];
        let mut idx = 0;

        for y in 0..size {
            for x in 0..size {
                let pos = Point {
                    x: Self::get_coord(size, solid_size, x, odd),
                    y: Self::get_coord(size, solid_size, y, odd),
                    z,
                };
                cells[idx].index = part_f(pos);
                cells[idx].pos = pos;
                idx += 1;
            }
        }

        Self { cells }
    }

    pub fn lift(&self, z: f32, part_f: &dyn Fn(Point) -> PartIndex) -> Self {
        let mut cells = vec![SolidCell::new(); self.cells.len()];

        for i in 0..cells.len() {
            cells[i].pos = self.cells[i].pos;
            cells[i].pos.z = z;
            cells[i].index = part_f(cells[i].pos);
        }

        Self { cells }
    }
}

pub struct ModelCreator {
    size: usize,
    solid_size: f32,
    models: HashMap<u32, Model>,
    prev_layer: SolidLayer,
    cur_layer: SolidLayer,
    next_layer: SolidLayer,
    used_numbers: Vec<u32>,
    last_z: usize,
    last_odd: bool,
    tries: usize,
}

impl ModelCreator {
    pub fn new(
        size: usize,
        solid_size: f32,
        tries: usize,
        part_f: &dyn Fn(Point) -> PartIndex,
    ) -> Self {
        let mut result = Self {
            size,
            solid_size,
            models: HashMap::new(),
            prev_layer: SolidLayer::default(),
            cur_layer: SolidLayer::default(),
            next_layer: SolidLayer::default(),
            used_numbers: Vec::new(),
            last_z: 0,
            last_odd: true,
            tries,
        };

        result.cur_layer = result.filled_layer(0, false, part_f);
        result.next_layer = result.filled_layer(0, true, part_f);

        result
    }

    pub fn get_models(self) -> HashMap<u32, Model> {
        self.models
    }

    fn filled_layer(&self, z: usize, odd: bool, part_f: &dyn Fn(Point) -> PartIndex) -> SolidLayer {
        SolidLayer::filled(
            self.size,
            self.solid_size,
            SolidLayer::get_coord(self.size, self.solid_size, z, odd),
            odd,
            part_f,
        )
    }

    pub fn finished(&self) -> bool {
        self.last_z == self.size - 1 && self.last_odd
    }

    fn fill_tetrahedron(
        model: &mut Model,
        part_f: &dyn Fn(Point) -> u32,
        tries: usize,
        model_index: PartIndex,
        (c0, c0_inside, e01, e02, e03, p012, p023, p031): (
            Point,
            bool,
            &mut u32,
            &mut u32,
            &mut u32,
            &mut u32,
            &mut u32,
            &mut u32,
        ),
        (c1, c1_inside, e10, e12, e13, p102, p123, p130): (
            Point,
            bool,
            &mut u32,
            &mut u32,
            &mut u32,
            &mut u32,
            &mut u32,
            &mut u32,
        ),
        (c2, c2_inside, e20, e21, e23, p201, p213, p230): (
            Point,
            bool,
            &mut u32,
            &mut u32,
            &mut u32,
            &mut u32,
            &mut u32,
            &mut u32,
        ),
        (c3, c3_inside, e30, e31, e32, p301, p312, p320): (
            Point,
            bool,
            &mut u32,
            &mut u32,
            &mut u32,
            &mut u32,
            &mut u32,
            &mut u32,
        ),
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
            model.triangles.push(Triangle(v0, v1, v2));
        };

        let it = |model: &mut Model,
                  corner: Point,
                  c0: Point,
                  e0: &mut u32,
                  p01: &mut u32,
                  c1: Point,
                  e1: &mut u32,
                  p12: &mut u32,
                  c2: Point,
                  e2: &mut u32,
                  p20: &mut u32| {
            if *e0 == BAD_INDEX {
                *e0 = model.add_vertex(root(corner, c0)) as u32;
            }
            if *e1 == BAD_INDEX {
                *e1 = model.add_vertex(root(corner, c1)) as u32;
            }
            if *e2 == BAD_INDEX {
                *e2 = model.add_vertex(root(corner, c2)) as u32;
            }
            if *p01 == BAD_INDEX {
                *p01 = model.add_vertex(root(corner, (c0 + c1).scale(0.5))) as u32;
            }
            if *p12 == BAD_INDEX {
                *p12 = model.add_vertex(root(corner, (c1 + c2).scale(0.5))) as u32;
            }
            if *p20 == BAD_INDEX {
                *p20 = model.add_vertex(root(corner, (c2 + c0).scale(0.5))) as u32;
            }

            add_t(model, *e0, *p01, *p20);
            add_t(model, *e1, *p12, *p01);
            add_t(model, *e2, *p20, *p12);
            add_t(model, *p01, *p12, *p20);
        };

        let ot = |model: &mut Model,
                  corner: Point,
                  c0: Point,
                  e0: &mut u32,
                  p01: &mut u32,
                  c1: Point,
                  e1: &mut u32,
                  p12: &mut u32,
                  c2: Point,
                  e2: &mut u32,
                  p20: &mut u32| {
            if *e0 == BAD_INDEX {
                *e0 = model.add_vertex(root(c0, corner)) as u32;
            }
            if *e1 == BAD_INDEX {
                *e1 = model.add_vertex(root(c1, corner)) as u32;
            }
            if *e2 == BAD_INDEX {
                *e2 = model.add_vertex(root(c2, corner)) as u32;
            }
            if *p01 == BAD_INDEX {
                *p01 = model.add_vertex(root((c0 + c1).scale(0.5), corner)) as u32;
            }
            if *p12 == BAD_INDEX {
                *p12 = model.add_vertex(root((c1 + c2).scale(0.5), corner)) as u32;
            }
            if *p20 == BAD_INDEX {
                *p20 = model.add_vertex(root((c2 + c0).scale(0.5), corner)) as u32;
            }

            add_t(model, *e0, *p01, *p20);
            add_t(model, *e1, *p12, *p01);
            add_t(model, *e2, *p20, *p12);
            add_t(model, *p01, *p12, *p20);
        };

        let q = |model: &mut Model,
                 c0: Point,
                 e0: &mut u32,
                 p01: &mut u32,
                 c1: Point,
                 e1: &mut u32,
                 p12: &mut u32,
                 c2: Point,
                 e2: &mut u32,
                 p23: &mut u32,
                 c3: Point,
                 e3: &mut u32,
                 p30: &mut u32| {
            if *e0 == BAD_INDEX {
                *e0 = model.add_vertex(root(c0, c1)) as u32;
            }
            if *e1 == BAD_INDEX {
                *e1 = model.add_vertex(root(c2, c1)) as u32;
            }
            if *e2 == BAD_INDEX {
                *e2 = model.add_vertex(root(c2, c3)) as u32;
            }
            if *e3 == BAD_INDEX {
                *e3 = model.add_vertex(root(c0, c3)) as u32;
            }
            if *p01 == BAD_INDEX {
                *p01 = model.add_vertex(root((c2 + c0).scale(0.5), c1)) as u32;
            }
            if *p12 == BAD_INDEX {
                *p12 = model.add_vertex(root(c2, (c1 + c3).scale(0.5))) as u32;
            }
            if *p23 == BAD_INDEX {
                *p23 = model.add_vertex(root((c2 + c0).scale(0.5), c3)) as u32;
            }
            if *p30 == BAD_INDEX {
                *p30 = model.add_vertex(root(c0, (c1 + c3).scale(0.5))) as u32;
            }

            add_t(model, *e0, *p01, *p30);
            add_t(model, *e1, *p12, *p01);
            add_t(model, *e2, *p23, *p12);
            add_t(model, *e3, *p30, *p23);

            let v0 = model.vertices[*p01 as usize];
            let v1 = model.vertices[*p12 as usize];
            let v2 = model.vertices[*p23 as usize];
            let v3 = model.vertices[*p30 as usize];

            let center = (v0 + v1 + v2 + v3).scale(0.25);
            let vol_ok = dot(v1 - v0, cross(v2 - v0, v3 - v0)) > 0.0;

            let center_ok = part_f(center) == model_index;

            if center_ok != vol_ok {
                add_t(model, *p01, *p12, *p23);
                add_t(model, *p23, *p30, *p01);
            } else {
                add_t(model, *p01, *p12, *p30);
                add_t(model, *p23, *p30, *p12);
            }
        };

        if c0_inside {
            if c1_inside {
                if c2_inside {
                    if c3_inside {
                        // skip
                    } else {
                        // 012|3
                        ot(model, c3, c0, e03, p301, c1, e13, p312, c2, e23, p320);
                    }
                } else {
                    if c3_inside {
                        // 031|2
                        ot(model, c2, c0, e02, p230, c3, e32, p213, c1, e12, p201);
                    } else {
                        // 01|23
                        q(
                            model, c0, e03, p301, c3, e13, p123, c1, e12, p201, c2, e02, p023,
                        );
                    }
                }
            } else {
                if c2_inside {
                    if c3_inside {
                        // 023|1
                        ot(model, c1, c0, e01, p102, c2, e21, p123, c3, e31, p130);
                    } else {
                        // 02|13
                        q(
                            model, c0, e01, p102, c1, e21, p213, c2, e23, p320, c3, e03, p031,
                        );
                    }
                } else {
                    if c3_inside {
                        // 03|12
                        q(
                            model, c0, e02, p230, c2, e32, p312, c3, e31, p130, c1, e01, p012,
                        );
                    } else {
                        // 0|123
                        it(model, c0, c1, e01, p012, c2, e02, p023, c3, e03, p031);
                    }
                }
            }
        } else {
            if c1_inside {
                if c2_inside {
                    if c3_inside {
                        // 132|0
                        ot(model, c0, c1, e10, p031, c3, e30, p023, c2, e20, p012);
                    } else {
                        // 12|03
                        q(
                            model, c1, e13, p312, c3, e23, p230, c2, e20, p012, c0, e10, p130,
                        );
                    }
                } else {
                    if c3_inside {
                        // 13|02
                        q(
                            model, c1, e10, p031, c0, e30, p320, c3, e32, p213, c2, e12, p102,
                        );
                    } else {
                        // 1|032
                        it(model, c1, c0, e10, p130, c3, e13, p123, c2, e12, p102);
                    }
                }
            } else {
                if c2_inside {
                    if c3_inside {
                        // 23|01
                        q(
                            model, c2, e21, p123, c1, e31, p301, c3, e30, p023, c0, e20, p201,
                        );
                    } else {
                        // 2|013
                        it(model, c2, c0, e20, p201, c1, e21, p213, c3, e23, p230);
                    }
                } else {
                    if c3_inside {
                        // 3|021
                        it(model, c3, c0, e30, p320, c2, e32, p312, c1, e31, p301);
                    } else {
                        // skip
                    }
                }
            }
        }
    }

    fn use_layers(&mut self, part_f: &dyn Fn(Point) -> u32) {
        let pl = self.prev_layer.cells.as_mut_slice();
        let cl = self.cur_layer.cells.as_mut_slice();
        let nl = self.next_layer.cells.as_mut_slice();

        let next_shift = self.last_odd as usize;
        let cur_shift = 1 - next_shift;

        for y in 0..self.size - 1 {
            for x in 0..self.size - 1 {
                let c = y * self.size + x;
                let cx = c + 1;
                let cy = c + self.size;
                let cxy = c + self.size + 1;
                let npc = if self.last_odd { c } else { cxy };

                let h1cur = if self.last_odd { cy } else { c };
                let h2cur = h1cur + 1;
                let v1next = if self.last_odd { c } else { cx };
                let v2next = v1next + self.size;

                let v1cur = if self.last_odd { cx } else { c };
                let v2cur = v1cur + self.size;
                let h1next = if self.last_odd { c } else { cy };
                let h2next = h1next + 1;

                let corner_index = pl[npc].index;

                if corner_index != cl[c].index
                    || corner_index != cl[cx].index
                    || corner_index != cl[cy].index
                    || corner_index != cl[cxy].index
                    || corner_index != nl[npc].index
                    || corner_index != cl[h1cur].index
                    || corner_index != cl[h2cur].index
                    || corner_index != cl[v1cur].index
                    || corner_index != cl[v2cur].index
                    || corner_index != nl[h1next].index
                    || corner_index != nl[h2next].index
                    || corner_index != nl[v1next].index
                    || corner_index != nl[v2next].index
                {
                    self.used_numbers.clear();
                    let mut use_number = |i| {
                        if i != 0 && !self.used_numbers.contains(&i) {
                            self.used_numbers.push(i);
                        }
                    };

                    use_number(corner_index);
                    use_number(cl[c].index);
                    use_number(cl[cx].index);
                    use_number(cl[cy].index);
                    use_number(cl[cxy].index);
                    use_number(nl[npc].index);
                    use_number(cl[h1cur].index);
                    use_number(cl[h2cur].index);
                    use_number(cl[v1cur].index);
                    use_number(cl[v2cur].index);
                    use_number(nl[h1next].index);
                    use_number(nl[h2next].index);
                    use_number(nl[v1next].index);
                    use_number(nl[v2next].index);

                    for &model_index in &self.used_numbers {
                        let model = self.models.entry(model_index).or_insert(Model::new());

                        macro_rules! vertex {
                            ($l: expr, $c: expr, $e0: ident, $e1: ident, $e2: ident, $p01: ident, $p12: ident, $p20: ident) => {
                                (
                                    $l[$c].pos,
                                    $l[$c].index == model_index,
                                    &mut $l[$c].$e0,
                                    &mut $l[$c].$e1,
                                    &mut $l[$c].$e2,
                                    &mut $l[$c].$p01,
                                    &mut $l[$c].$p12,
                                    &mut $l[$c].$p20,
                                )
                            };
                        }

                        Self::fill_tetrahedron(
                            model,
                            part_f,
                            self.tries,
                            model_index,
                            vertex!(pl, npc, v_mmp, v_pmp, v_zzp, w_zmp, w_pmq, w_mmq),
                            vertex!(cl, c, v_ppm, v_pzz, v_ppp, w_qpm, w_qpp, w_ppz),
                            vertex!(cl, cx, v_mpm, v_mzz, v_mpp, w_npm, w_npp, w_mpz),
                            vertex!(nl, npc, v_zzm, v_mmm, v_pmm, w_mmn, w_zmm, w_pmn),
                        );

                        Self::fill_tetrahedron(
                            model,
                            part_f,
                            self.tries,
                            model_index,
                            vertex!(pl, npc, v_pmp, v_ppp, v_zzp, w_pzp, w_ppq, w_pmq),
                            vertex!(cl, cx, v_mpm, v_zpz, v_mpp, w_mqm, w_mqp, w_mpz),
                            vertex!(cl, cxy, v_mmm, v_zmz, v_mmp, w_mnm, w_mnp, w_mmz),
                            vertex!(nl, npc, v_zzm, v_pmm, v_ppm, w_pmn, w_pzm, w_ppn),
                        );

                        Self::fill_tetrahedron(
                            model,
                            part_f,
                            self.tries,
                            model_index,
                            vertex!(pl, npc, v_ppp, v_mpp, v_zzp, w_zpp, w_mpq, w_ppq),
                            vertex!(cl, cxy, v_mmm, v_mzz, v_mmp, w_nmm, w_nmp, w_mmz),
                            vertex!(cl, cy, v_pmm, v_pzz, v_pmp, w_qmm, w_qmp, w_pmz),
                            vertex!(nl, npc, v_zzm, v_ppm, v_mpm, w_ppn, w_zpm, w_mpn),
                        );

                        Self::fill_tetrahedron(
                            model,
                            part_f,
                            self.tries,
                            model_index,
                            vertex!(pl, npc, v_mpp, v_mmp, v_zzp, w_mzp, w_mmq, w_mpq),
                            vertex!(cl, cy, v_pmm, v_zmz, v_pmp, w_pnm, w_pnp, w_pmz),
                            vertex!(cl, c, v_ppm, v_zpz, v_ppp, w_pqm, w_pqp, w_ppz),
                            vertex!(nl, npc, v_zzm, v_mpm, v_mmm, w_mpn, w_mzm, w_mmn),
                        );

                        Self::fill_tetrahedron(
                            model,
                            part_f,
                            self.tries,
                            model_index,
                            vertex!(cl, h1cur, v_pzz, v_ppp, v_pmp, w_qpp, w_pzp, w_qmp),
                            vertex!(cl, h2cur, v_mzz, v_mpp, v_mmp, w_npp, w_mzp, w_nmp),
                            vertex!(nl, v2next, v_mmm, v_pmm, v_zmz, w_zmm, w_pnm, w_mnm),
                            vertex!(nl, v1next, v_mpm, v_ppm, v_zpz, w_zpm, w_pqm, w_mqm),
                        );

                        Self::fill_tetrahedron(
                            model,
                            part_f,
                            self.tries,
                            model_index,
                            vertex!(cl, v1cur, v_zpz, v_mpp, v_ppp, w_mqp, w_zpp, w_pqp),
                            vertex!(cl, v2cur, v_zmz, v_mmp, v_pmp, w_mnp, w_zmp, w_pnp),
                            vertex!(nl, h1next, v_pmm, v_ppm, v_pzz, w_pzm, w_qpm, w_qmm),
                            vertex!(nl, h2next, v_mmm, v_mpm, v_mzz, w_mzm, w_npm, w_nmm),
                        );
                    }
                }
            }
        }
    }

    pub fn fill_next_layer(&mut self, part_f: &dyn Fn(Point) -> PartIndex) {
        self.prev_layer = std::mem::take(&mut self.cur_layer);
        self.cur_layer = std::mem::take(&mut self.next_layer);
        if self.last_odd {
            self.last_odd = false;
            self.last_z += 1;
        } else {
            self.last_odd = true;
        }

        let z = SolidLayer::get_coord(self.size, self.solid_size, self.last_z, self.last_odd);
        self.next_layer = self.prev_layer.lift(z, part_f);
        self.use_layers(part_f);

        println!("processed [{}/{}] layers", self.last_z, self.size);
    }
}
