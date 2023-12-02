use dxf::entities::*;
use dxf::Drawing;
use std::collections::HashMap;

pub type PartIndex = u32;
const BAD_INDEX: PartIndex = 0xFFFFFFFF;

#[derive(Debug, Clone, Copy)]
pub struct Point2D(pub f32, pub f32);

#[derive(Debug, Clone)]
pub struct Contour {
    points: Vec<Point2D>,
}

impl Contour {
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    pub fn smooth(&mut self) {
        let mut new_points = Vec::with_capacity(self.points.len());
        for i in 0..self.points.len() {
            let next = (i + 1) % self.points.len();
            let smoothed = Point2D(
                self.points[i].0 * 0.5 + self.points[next].0 * 0.5,
                self.points[i].1 * 0.5 + self.points[next].1 * 0.5,
            );
            new_points.push(smoothed);
        }
        self.points = new_points;
    }

    pub fn shift(&mut self, shift: Point2D) {
        for p in &mut self.points {
            p.0 += shift.0;
            p.1 += shift.1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Contours {
    contours: Vec<Contour>,
}

impl Contours {
    pub fn new() -> Self {
        Self {
            contours: Vec::new(),
        }
    }

    pub fn points_count(&self) -> usize {
        let mut result = 0;
        for c in &self.contours {
            result += c.points.len();
        }
        result
    }

    pub fn smooth(&mut self) {
        for c in &mut self.contours {
            c.smooth();
        }
    }

    pub fn shift(&mut self, shift: Point2D) {
        for c in &mut self.contours {
            c.shift(shift)
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.contours.extend(other.contours)
    }

    pub fn save_to_dxf(&self, path: &std::path::Path) -> Result<(), String> {
        let mut drawing = Drawing::new();
        for contour in &self.contours {
            for i in 0..contour.points.len() {
                fn point2d_to_dxf(pt: Point2D) -> dxf::Point {
                    dxf::Point {
                        x: pt.0 as f64,
                        y: pt.1 as f64,
                        z: 0.0,
                    }
                }

                let i2 = (i + 1) % contour.points.len();
                let pt1 = point2d_to_dxf(contour.points[i]);
                let pt2 = point2d_to_dxf(contour.points[i2]);
                drawing.add_entity(Entity::new(EntityType::Line(Line::new(pt1, pt2))));
            }
        }
        drawing.save_file(path).map_err(|e| {
            format!(
                "Unable to open file {} for writing: {}",
                path.to_string_lossy(),
                e
            )
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ContourCell {
    index: PartIndex,
    pos: Point2D,
    v_mz: u32,
    v_pz: u32,
    v_zm: u32,
    v_zp: u32,
}

impl ContourCell {
    fn new() -> Self {
        Self {
            index: 0,
            pos: Point2D(0.0, 0.0),
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
    points: Vec<Point2D>,
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

    fn index_to_point(size: usize, contour_size: f32, x: usize, y: usize) -> Point2D {
        let scale = contour_size / (size as f32 - 1.0);
        Point2D(
            x as f32 * scale * 0.5 - contour_size * 0.5,
            y as f32 * scale * 0.5 - contour_size * 0.5,
        )
    }

    fn center_of_cell(size: usize, contour_size: f32, x: usize, y: usize) -> Point2D {
        Self::index_to_point(size, contour_size, x * 2 - 1, y * 2 - 1)
    }

    fn corner_of_cell(size: usize, contour_size: f32, x: usize, y: usize) -> Point2D {
        Self::index_to_point(size, contour_size, x * 2, y * 2)
    }

    fn fill_cell(
        size: usize,
        contour_size: f32,
        cell: &mut ContourCell,
        x: usize,
        y: usize,
        part_f: &dyn Fn(Point2D) -> PartIndex,
    ) {
        cell.pos = Self::corner_of_cell(size, contour_size, x, y);
        cell.index = part_f(cell.pos);
    }

    fn root(
        mut p1: Point2D,
        mut p2: Point2D,
        tries: usize,
        part_index: PartIndex,
        part_f: &dyn Fn(Point2D) -> PartIndex,
    ) -> Point2D {
        let mut i = 0;
        loop {
            i += 1;
            let mid = Point2D((p1.0 + p2.0) * 0.5, (p1.1 + p2.1) * 0.5);
            if i >= tries {
                return mid;
            }
            if part_f(mid) == part_index {
                p1 = mid;
            } else {
                p2 = mid;
            }
        }
    }

    fn index_of_new_point(points: &mut Vec<Point2D>, pt: Point2D) -> u32 {
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
        part_f: &dyn Fn(Point2D) -> PartIndex,
    ) -> HashMap<PartIndex, Contours> {
        if self.size == 0 {
            return HashMap::new();
        }

        let cells = &mut self.cells[..];
        let sz = self.size;

        let mut result = HashMap::new();

        macro_rules! fill_mid {
            ($c: expr, $field: ident, $p1: expr, $p2: expr) => {
                if cells[$c].index != 0 {
                    let pt = Self::root($p1, $p2, self.tries, cells[$c].index, part_f);
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
                            let pt1 =
                                Self::root(center, cells[$c].pos, self.tries, center_index, part_f);
                            $dst1 = Self::index_of_new_point(&mut self.points, pt1);
                            let pt2 =
                                Self::root(cells[$c].pos, center, self.tries, c_index, part_f);
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
                let mut contours = Contours::new();

                while let Some(&key) = edges.keys().next() {
                    let mut new_contour = Contour::new();
                    let mut current = key;
                    loop {
                        new_contour.points.push(self.points[current as usize]);
                        current = edges.remove(&current).unwrap();
                        if current == key {
                            break;
                        }
                    }

                    contours.contours.push(new_contour);
                }

                (model_index, contours)
            })
            .collect()
    }
}
