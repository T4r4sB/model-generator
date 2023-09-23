use crate::model::*;
use crate::points3d::*;
use std::collections::HashMap;

const BAD_INDEX: u32 = 0xFFFFFFFF;

#[derive(Debug, Clone, Copy)]
pub struct SolidCell {
    index: u32,
    pos: Point,
}

impl SolidCell {
    pub fn new() -> Self {
        Self {
            index: 0,
            pos: Point::zero(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SolidEdge {
    near_v_index: u32,
    far_v_index: u32,
}

impl SolidEdge {
    pub fn new() -> Self {
        Self {
            near_v_index: BAD_INDEX,
            far_v_index: BAD_INDEX,
        }
    }
}

pub struct SolidLayer {
    size: usize,
    cells: Vec<SolidCell>,
    edges_x: Vec<SolidEdge>,
    edges_y: Vec<SolidEdge>,
    edges_z: Vec<SolidEdge>,
}

impl SolidLayer {
    pub fn new_zero(size: usize, solid_size: f32, z: f32) -> Self {
        let mut cells = vec![SolidCell::new(); (size + 1) * (size + 1)];
        let edges_x = vec![SolidEdge::new(); (size + 1) * (size + 1)];
        let edges_y = vec![SolidEdge::new(); (size + 1) * (size + 1)];
        let edges_z = vec![SolidEdge::new(); (size + 1) * (size + 1)];
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

        Self {
            size,
            cells,
            edges_x,
            edges_y,
            edges_z,
        }
    }

    pub fn fill_by(&mut self, z: f32, part_f: &dyn Fn(Point) -> u32) {
        let mut idx = 0;
        for y in 0..=self.size {
            for x in 0..=self.size {
                self.cells[idx].pos.z = z;
                if y == 0 || y == self.size || x == 0 || x == self.size {
                    self.cells[idx].index = 0;
                } else {
                    self.cells[idx].index = part_f(self.cells[idx].pos);
                }

                self.edges_x[idx] = SolidEdge::new();
                self.edges_y[idx] = SolidEdge::new();
                self.edges_z[idx] = SolidEdge::new();

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

    fn use_layers(&mut self, part_f: &dyn Fn(Point) -> u32) {
        let mut vertices_for_convex = Vec::new();
        let mut vindices_for_convex = Vec::new();
        let mut layouts_for_convex = Vec::new();

        for y in 0..self.size {
            for x in 0..self.size {
                let idx = y * (self.size + 1) + x;
                let corner_index = self.prev_layer.cells[idx].index;

                if corner_index != self.prev_layer.cells[idx + 1].index
                    || corner_index != self.prev_layer.cells[idx + self.size + 1].index
                    || corner_index != self.prev_layer.cells[idx + self.size + 2].index
                    || corner_index != self.next_layer.cells[idx].index
                    || corner_index != self.next_layer.cells[idx + 1].index
                    || corner_index != self.next_layer.cells[idx + self.size + 1].index
                    || corner_index != self.next_layer.cells[idx + self.size + 2].index
                {
                    self.used_numbers.clear();
                    let mut use_number = |i| {
                        if i != 0 && !self.used_numbers.contains(&i) {
                            self.used_numbers.push(i);
                        }
                    };

                    use_number(corner_index);
                    use_number(self.prev_layer.cells[idx + 1].index);
                    use_number(self.prev_layer.cells[idx + self.size + 1].index);
                    use_number(self.prev_layer.cells[idx + self.size + 2].index);
                    use_number(self.next_layer.cells[idx].index);
                    use_number(self.next_layer.cells[idx + 1].index);
                    use_number(self.next_layer.cells[idx + self.size + 1].index);
                    use_number(self.next_layer.cells[idx + self.size + 2].index);

                    for &index in &self.used_numbers {
                        vertices_for_convex.clear();
                        vindices_for_convex.clear();
                        layouts_for_convex.clear();
                        let model = self.models.entry(index).or_insert(Model::new());

                        let reversed = false;
                        let mut use_cell = |cell: &SolidCell, layout: u16| {
                            if (cell.index == index) == !reversed {
                                vertices_for_convex.push(cell.pos);
                                vindices_for_convex.push(BAD_INDEX);
                                layouts_for_convex.push(layout);
                            }
                        };

                        use_cell(&self.prev_layer.cells[idx], 0x000);
                        use_cell(&self.prev_layer.cells[idx + 1], 0x003);
                        use_cell(&self.prev_layer.cells[idx + self.size + 1], 0x030);
                        use_cell(&self.prev_layer.cells[idx + self.size + 2], 0x033);
                        use_cell(&self.next_layer.cells[idx], 0x300);
                        use_cell(&self.next_layer.cells[idx + 1], 0x303);
                        use_cell(&self.next_layer.cells[idx + self.size + 1], 0x330);
                        use_cell(&self.next_layer.cells[idx + self.size + 2], 0x333);

                        let mut use_edge =
                            |model: &mut Model,
                             cell1: &SolidCell,
                             cell2: &SolidCell,
                             edge: &mut SolidEdge,
                             layout: u16| {
                                if cell1.index == index && cell2.index != index {
                                    if edge.near_v_index == BAD_INDEX {
                                        let p = find_root(
                                            part_f, cell1.pos, cell2.pos, index, self.tries,
                                        );
                                        edge.near_v_index = model.vertices.len() as u32;
                                        model.vertices.push(p);
                                    }
                                    vertices_for_convex
                                        .push(model.vertices[edge.near_v_index as usize]);
                                    vindices_for_convex.push(edge.near_v_index);
                                    layouts_for_convex.push(layout);
                                    return;
                                }
                                if cell2.index == index && cell1.index != index {
                                    if edge.far_v_index == BAD_INDEX {
                                        let p = find_root(
                                            part_f, cell2.pos, cell1.pos, index, self.tries,
                                        );
                                        edge.far_v_index = model.vertices.len() as u32;
                                        model.vertices.push(p);
                                    }
                                    vertices_for_convex
                                        .push(model.vertices[edge.far_v_index as usize]);
                                    vindices_for_convex.push(edge.far_v_index);
                                    layouts_for_convex.push(layout);
                                }
                            };

                        use_edge(
                            model,
                            &self.prev_layer.cells[idx],
                            &self.prev_layer.cells[idx + 1],
                            &mut self.prev_layer.edges_x[idx],
                            0x001,
                        );

                        use_edge(
                            model,
                            &self.prev_layer.cells[idx],
                            &self.prev_layer.cells[idx + self.size + 1],
                            &mut self.prev_layer.edges_y[idx],
                            0x010,
                        );

                        use_edge(
                            model,
                            &self.prev_layer.cells[idx],
                            &self.next_layer.cells[idx],
                            &mut self.prev_layer.edges_z[idx],
                            0x100,
                        );
                        //
                        use_edge(
                            model,
                            &self.prev_layer.cells[idx + 1],
                            &self.prev_layer.cells[idx + self.size + 2],
                            &mut self.prev_layer.edges_y[idx + 1],
                            0x013,
                        );

                        use_edge(
                            model,
                            &self.prev_layer.cells[idx + 1],
                            &self.next_layer.cells[idx + 1],
                            &mut self.prev_layer.edges_z[idx + 1],
                            0x103,
                        );

                        use_edge(
                            model,
                            &self.prev_layer.cells[idx + self.size + 1],
                            &self.prev_layer.cells[idx + self.size + 2],
                            &mut self.prev_layer.edges_x[idx + self.size + 1],
                            0x031,
                        );

                        use_edge(
                            model,
                            &self.prev_layer.cells[idx + self.size + 1],
                            &self.next_layer.cells[idx + self.size + 1],
                            &mut self.prev_layer.edges_z[idx + self.size + 1],
                            0x130,
                        );

                        use_edge(
                            model,
                            &self.next_layer.cells[idx],
                            &self.next_layer.cells[idx + 1],
                            &mut self.next_layer.edges_x[idx],
                            0x301,
                        );

                        use_edge(
                            model,
                            &self.next_layer.cells[idx],
                            &self.next_layer.cells[idx + self.size + 1],
                            &mut self.next_layer.edges_y[idx],
                            0x310,
                        );

                        //
                        use_edge(
                            model,
                            &self.prev_layer.cells[idx + self.size + 2],
                            &self.next_layer.cells[idx + self.size + 2],
                            &mut self.prev_layer.edges_z[idx + self.size + 2],
                            0x133,
                        );

                        use_edge(
                            model,
                            &self.next_layer.cells[idx + 1],
                            &self.next_layer.cells[idx + self.size + 2],
                            &mut self.next_layer.edges_y[idx + 1],
                            0x313,
                        );

                        use_edge(
                            model,
                            &self.next_layer.cells[idx + self.size + 1],
                            &self.next_layer.cells[idx + self.size + 2],
                            &mut self.next_layer.edges_x[idx + self.size + 1],
                            0x331,
                        );

                        let triangles = Model::convex_triangles(&vertices_for_convex, 0.0).unwrap();

                        for t in triangles {
                            let l_and = layouts_for_convex[t.0]
                                & layouts_for_convex[t.1]
                                & layouts_for_convex[t.2];
                            let l_or = layouts_for_convex[t.0]
                                | layouts_for_convex[t.1]
                                | layouts_for_convex[t.2];

                            if (l_or & 0x300) == 0
                                || (l_or & 0x030) == 0
                                || (l_or & 0x003) == 0
                                || (l_and & 0x300) == 0x300
                                || (l_and & 0x030) == 0x030
                                || (l_and & 0x003) == 0x003
                            {
                                continue;
                            }

                            if reversed {
                                model.triangles.push(Triangle(
                                    vindices_for_convex[t.2] as usize,
                                    vindices_for_convex[t.1] as usize,
                                    vindices_for_convex[t.0] as usize,
                                ));
                            } else {
                                model.triangles.push(Triangle(
                                    vindices_for_convex[t.0] as usize,
                                    vindices_for_convex[t.1] as usize,
                                    vindices_for_convex[t.2] as usize,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn fill_next_layer(&mut self, part_f: &dyn Fn(Point) -> u32) {
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
