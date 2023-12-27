use crate::points2d::*;
use dxf::entities::*;
use dxf::Drawing;
use std::collections::HashMap;

pub type PartIndex = u32;
const BAD_INDEX: PartIndex = 0xFFFFFFFF;

#[derive(Debug, Clone)]
pub struct Contour {
    points: Vec<Point>,
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
        let last = *self.points.last().unwrap();
        for i in 0..self.points.len() - 2 {
            result += cross(self.points[i] - last, self.points[i + 1] - last);
        }
        result
    }

    pub fn contains(&self, p: Point) -> bool {
        if self.points.is_empty() {
            return false;
        }
        let mut c_in = 0;
        let mut c_out = 0;
        for i in 0..self.points.len() {
            let pi = self.points[i] - p;
            let pi1 = self.points[(i + 1) % self.points.len()] - p;
            if pi.y >= 0.0 && pi1.y < 0.0 && cross(pi, pi1) < 0.0 {
                c_in += 1;
            } else if pi.y < 0.0 && pi1.y >= 0.0 && cross(pi, pi1) > 0.0 {
                c_out += 1;
            }
        }
        c_in < c_out
    }

    pub fn split_to_triangles_if_convex(self) -> Vec<Self> {
        let mut result = Vec::new();

        if self.points.len() < 3 {
            return result;
        }

        let last = *self.points.last().unwrap();
        for i in 0..self.points.len() - 2 {
            result.push(Contour {
                points: vec![
                    last,
                    self.points[i],
                    self.points[(i + 1) % self.points.len()],
                ],
            });
        }
        result
    }

    pub fn smooth(&mut self) {
        let mut new_points = Vec::with_capacity(self.points.len());
        for i in 0..self.points.len() {
            let next = (i + 1) % self.points.len();
            let smoothed = (self.points[i] + self.points[next]).scale(0.5);
            new_points.push(smoothed);
        }
        self.points = new_points;
    }

    pub fn optimize(&mut self, eps: f32) {
        let ok = |i1: usize, i2: usize| {
            if i1 == i2 {
                return false;
            }
            let mut i = i1;
            loop {
                i += 1;
                if i == self.points.len() {
                    i = 0;
                }
                if i == i2 {
                    return true;
                }
                if dist_pl(self.points[i], self.points[i1], self.points[i2]) > eps {
                    return false;
                }
            }
        };

        let mut v = Vec::<(usize, usize)>::new(); // point index, next
        for i in 0..self.points.len() {
            v.push((i, (i + 1) % self.points.len()));
        }

        let mut i = 0;
        let mut ni = v[i].1;
        let mut nni = v[ni].1;
        let mut lv: Option<usize> = None;
        loop {
            let next = v[nni].1;
            if i == next {
                break;
            }
            if ok(i, nni) {
                v[i].1 = nni;
                lv = None;
            } else {
                i = ni;
                if lv == Some(i) {
                    break;
                }
                lv = lv.or(Some(i));
            }
            ni = nni;
            nni = next;
        }

        let si = i;
        let mut fixed_points = Vec::new();
        loop {
            fixed_points.push(self.points[v[i].0]);
            i = v[i].1;
            if i == si {
                break;
            }
        }
        self.points = fixed_points;
    }

    pub fn shift(&mut self, shift: Point) {
        for p in &mut self.points {
            *p += shift;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Contours {
    contours: Vec<Contour>,
}

impl Contours {
    pub fn new() -> Self {
        Self { contours: Vec::new() }
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

    pub fn optimize(&mut self, eps: f32) {
        for c in &mut self.contours {
            c.optimize(eps)
        }
    }

    pub fn shift(&mut self, shift: Point) {
        for c in &mut self.contours {
            c.shift(shift)
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.contours.extend(other.contours)
    }

    fn split_by(mut self, c1: usize, p1: usize, c2: usize, p2: usize) -> Vec<Self> {
        if c1 == c2 {
            let src = &self.contours[c1].points;
            let (v1, v2) = if p1 < p2 { (p1, p2) } else { (p2, p1) };
            let new_c1 = Contour { points: src[v1..=v2].to_vec() };
            let new_c2 = Contour { points: [&src[v2..], &src[..=v1]].concat() };
            self.contours = [
                &self.contours[..c1],
                &[new_c1, new_c2],
                &self.contours[c2 + 1..],
            ]
            .concat();
            self.split_to_connected_areas()
        } else {
            let src1 = &self.contours[c1].points;
            let src2 = &self.contours[c2].points;
            let new_c =
                Contour { points: [&src1[..=p1], &src2[p2..], &src2[..=p2], &src1[p1..]].concat() };
            let (c1, c2) = if c1 < c2 { (c1, c2) } else { (c2, c1) };
            self.contours = [
                &self.contours[..c1],
                &self.contours[c1 + 1..c2],
                &self.contours[c2 + 1..],
                &[new_c],
            ]
            .concat();
            // here we dont create new connected parts
            vec![self]
        }
    }

    fn find_bad_angle(&self) -> Option<(usize, usize)> {
        for ci in 0..self.contours.len() {
            let c = &self.contours[ci];
            if c.points.len() < 3 {
                continue;
            }

            for i in 0..c.points.len() {
                let ni = (i + 1) % c.points.len();
                let nni = (ni + 1) % c.points.len();
                if cross(c.points[nni] - c.points[ni], c.points[i] - c.points[ni]) < 0.0 {
                    return Some((ci, ni));
                }
            }
        }
        None
    }

    fn find_pair_for_bad_angle(&self, c: usize, p: usize) -> (usize, usize) {
        const EPS: f32 = 1.0e-5;
        let ps0 = &self.contours[c].points;
        let p0 = ps0[p];
        let p1 = ps0[(p + ps0.len() - 1) % ps0.len()] - p0;
        let p2 = ps0[(p + 1) % ps0.len()] - p0;
        let bisect = -p1.norm() - p2.norm();

        // stage1: find closest edge crossing bisect
        let mut mind = (Point::zero(), Point::zero(), f32::INFINITY, false);
        for ci in 0..self.contours.len() {
            let cp = &self.contours[ci].points;
            for i in 0..cp.len() {
                let ni = (i + 1) % cp.len();
                if ci == c && (i == p || ni == p) {
                    continue;
                }
                let pi = cp[i] - p0;
                let pi1 = cp[ni] - p0;
                let ci = cross(pi, bisect);
                let ci1 = cross(pi1, bisect);
                if ci >= 0.0 && ci1 < 0.0 {
                    let di = dot(pi, bisect);
                    let di1 = dot(pi1, bisect);
                    let d = (di1 * ci - di * ci1) / (ci - ci1);
                    if d > EPS && d < mind.2 {
                        mind = (pi, pi1, d, true);
                    }
                }
            }
        }

        assert!(mind.3);

        // stage2: find closest point in triangle
        let mut closest = (0, 0, f32::INFINITY, false);
        for ci in 0..self.contours.len() {
            let cp = &self.contours[ci].points;
            for i in 0..cp.len() {
                let pi = cp[i] - p0;
                if cross(pi - mind.0, pi - mind.1) < -EPS
                    || (cross(pi, p1) <= EPS && cross(p2, pi) <= EPS)
                {
                    continue;
                }

                let d = cross(pi, bisect).abs();
                if d < closest.2 {
                    closest = (ci, i, d, true);
                }
            }
        }

        assert!(closest.3);

        (closest.0, closest.1)
    }

    pub fn split_to_connected_areas(mut self) -> Vec<Self> {
        let squares: Vec<(f32, i32)> = self
            .contours
            .iter()
            .map(|c| {
                if c.points.is_empty() {
                    (0.0, 0)
                } else {
                    let sq = c.get_square();
                    (sq, sq.signum() as i32)
                }
            })
            .collect();

        let mut insides = Vec::new();
        insides.resize(self.contours.len(), Vec::new());

        for i in 0..self.contours.len() {
            if squares[i].1 == 1 {
                insides[i].push(i);
                continue;
            }
            if squares[i].1 == 0 {
                continue;
            }
            let pt0 = self.contours[i].points[0];
            let mut inside = (self.contours.len(), f32::INFINITY, false);
            for j in 0..self.contours.len() {
                if i == j || squares[j].1 != 1 || !self.contours[j].contains(pt0) {
                    continue;
                }

                if inside.1 > squares[j].0 {
                    inside = (j, squares[j].0, true);
                }
            }
            if inside.2 {
                insides[inside.0].push(i);
            }
        }

        insides
            .into_iter()
            .filter(|i| !i.is_empty())
            .map(|i| Self {
                contours: i
                    .into_iter()
                    .map(|i| std::mem::replace(&mut self.contours[i], Contour::new()))
                    .collect(),
            })
            .collect()
    }

    pub fn split_to_triangles_if_convex(self) -> Self {
        let mut result = Contours::new();
        for c in self.contours {
            result.contours.extend(c.split_to_triangles_if_convex());
        }
        result
    }

    pub fn split_to_triangles(self) -> Self {
        let mut result_before = self.split_to_connected_areas();
        let mut result_after = Vec::new();

        let mut output = Contours::new();
        loop {
            for r in result_before {
                if let Some((c, i)) = r.find_bad_angle() {
                    let (c2, i2) = r.find_pair_for_bad_angle(c, i);
                    result_after.extend(r.split_by(c, i, c2, i2));
                } else {
                    output
                        .contours
                        .extend(r.split_to_triangles_if_convex().contours);
                }
            }

            if result_after.is_empty() {
                break;
            }
            result_before = std::mem::take(&mut result_after);
        }

        output
    }

    pub fn get_square(&self) -> f32 {
        let mut result = 0.0;
        for c in &self.contours {
            result += c.get_square()
        }
        result
    }

    pub fn save_to_dxf(&self, path: &std::path::Path) -> Result<(), String> {
        let mut drawing = Drawing::new();
        for contour in &self.contours {
            for i in 0..contour.points.len() {
                fn point2d_to_dxf(pt: Point) -> dxf::Point {
                    dxf::Point { x: pt.x as f64, y: pt.y as f64, z: 0.0 }
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
    pos: Point,
    v_mz: u32,
    v_pz: u32,
    v_zm: u32,
    v_zp: u32,
}

impl ContourCell {
    fn new() -> Self {
        Self {
            index: 0,
            pos: Point { x: 0.0, y: 0.0 },
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
    points: Vec<Point>,
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

    fn index_to_point(size: usize, contour_size: f32, x: usize, y: usize) -> Point {
        let scale = contour_size / (size as f32 - 1.0);
        Point {
            x: x as f32 * scale * 0.5 - contour_size * 0.5,
            y: y as f32 * scale * 0.5 - contour_size * 0.5,
        }
    }

    fn center_of_cell(size: usize, contour_size: f32, x: usize, y: usize) -> Point {
        Self::index_to_point(size, contour_size, x * 2 - 1, y * 2 - 1)
    }

    fn corner_of_cell(size: usize, contour_size: f32, x: usize, y: usize) -> Point {
        Self::index_to_point(size, contour_size, x * 2, y * 2)
    }

    fn fill_cell(
        size: usize,
        contour_size: f32,
        cell: &mut ContourCell,
        x: usize,
        y: usize,
        part_f: &dyn Fn(Point) -> PartIndex,
    ) {
        cell.pos = Self::corner_of_cell(size, contour_size, x, y);
        cell.index = part_f(cell.pos);
    }

    fn index_of_new_point(points: &mut Vec<Point>, pt: Point) -> u32 {
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
        part_f: &dyn Fn(Point) -> PartIndex,
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
                    let pt = find_root(part_f, $p1, $p2, cells[$c].index, self.tries);
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
                                find_root(part_f, center, cells[$c].pos, center_index, self.tries);
                            $dst1 = Self::index_of_new_point(&mut self.points, pt1);
                            let pt2 = find_root(part_f, cells[$c].pos, center, c_index, self.tries);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize_contour4() {
        let mut c = Contour {
            points: vec![
                Point { x: 1.0, y: 0.0 },
                Point { x: 0.0, y: 1.0 },
                Point { x: -1.0, y: 0.0 },
                Point { x: 0.0, y: -1.0 },
            ],
        };

        c.optimize(0.5);
        assert_eq!(c.points.len(), 4);
        c.optimize(1.5);
        assert_eq!(c.points.len(), 3);
    }

    #[test]
    fn test_split_contour4() {
        let c = Contours {
            contours: vec![Contour {
                points: vec![
                    Point { x: 1.0, y: 0.0 },
                    Point { x: 0.0, y: 1.0 },
                    Point { x: -1.0, y: 0.0 },
                    Point { x: 0.0, y: -1.0 },
                ],
            }],
        };

        let cc = c.split_by(0, 1, 0, 3);
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
        let c = Contours {
            contours: vec![
                Contour {
                    points: vec![
                        Point { x: 11.0, y: 0.0 },
                        Point { x: 10.0, y: 1.0 },
                        Point { x: 9.0, y: 0.0 },
                        Point { x: 10.0, y: -1.0 },
                    ],
                },
                Contour {
                    points: vec![
                        Point { x: -9.0, y: 0.0 },
                        Point { x: -10.0, y: 1.0 },
                        Point { x: -11.0, y: 0.0 },
                        Point { x: -10.0, y: -1.0 },
                    ],
                },
            ],
        };

        let cc = c.split_by(0, 2, 1, 0);
        assert_eq!(cc.len(), 1);
        let c = &cc[0];
        assert_eq!(c.contours.len(), 1);
        assert_eq!(c.contours[0].points.len(), 10);
    }

    #[test]
    fn test_contains_contour4() {
        let c = Contour {
            points: vec![
                Point { x: 1.0, y: 0.0 },
                Point { x: 0.0, y: 1.0 },
                Point { x: -1.0, y: 0.0 },
                Point { x: 0.0, y: -1.0 },
            ],
        };

        assert!(c.contains(Point { x: 0.0, y: 0.0 }));
        assert!(!c.contains(Point { x: 2.0, y: 0.0 }));
    }

    #[test]
    fn test_bad_angle() {
        let c = Contours {
            contours: vec![Contour {
                points: vec![
                    Point { x: 1.0, y: 0.0 },
                    Point { x: -2.0, y: 1.0 },
                    Point { x: -1.0, y: 0.0 },
                    Point { x: -2.0, y: -1.0 },
                ],
            }],
        };

        assert_eq!(c.find_bad_angle(), Some((0, 2)));
    }

    #[test]
    fn test_pair_for_bad_angle() {
        let c = Contours {
            contours: vec![Contour {
                points: vec![
                    Point { x: 1.0, y: 0.0 },
                    Point { x: -2.0, y: 1.0 },
                    Point { x: -1.0, y: 0.0 },
                    Point { x: -2.0, y: -1.0 },
                ],
            }],
        };

        assert_eq!(c.find_pair_for_bad_angle(0, 2), (0, 0));
    }
    
    #[test]
    fn test_split_to_triangles() {
        let c = Contours {
            contours: vec![
                Contour {
                    points: vec![
                        Point { x: -1.0, y: 0.0 },
                        Point { x: 0.0, y: 1.0 },
                        Point { x: 1.0, y: 0.0 },
                        Point { x: 0.0, y: -1.0 },
                    ],
                },
                Contour {
                    points: vec![
                        Point { x: 2.0, y: 0.0 },
                        Point { x: 0.0, y: 2.0 },
                        Point { x: -2.0, y: 0.0 },
                        Point { x: 0.0, y: -2.0 },
                    ],
                },
            ],
        };

        let cc = c.split_to_triangles();

        assert_eq!(cc.contours.len(), 8);
    }

    #[test]
    fn test_connection_2contour4() {
        let c = Contours {
            contours: vec![
                Contour {
                    points: vec![
                        Point { x: 11.0, y: 0.0 },
                        Point { x: 10.0, y: 1.0 },
                        Point { x: 9.0, y: 0.0 },
                        Point { x: 10.0, y: -1.0 },
                    ],
                },
                Contour {
                    points: vec![
                        Point { x: -9.0, y: 0.0 },
                        Point { x: -10.0, y: 1.0 },
                        Point { x: -11.0, y: 0.0 },
                        Point { x: -10.0, y: -1.0 },
                    ],
                },
            ],
        };

        let cc = c.split_to_connected_areas();
        assert_eq!(cc.len(), 2);
        assert_eq!(cc[0].contours.len(), 1);
        assert_eq!(cc[1].contours.len(), 1);
    }

    #[test]
    fn test_connection_2contour4_inside() {
        let c = Contours {
            contours: vec![
                Contour {
                    points: vec![
                        Point { x: -1.0, y: 0.0 },
                        Point { x: 0.0, y: 1.0 },
                        Point { x: 1.0, y: 0.0 },
                        Point { x: 0.0, y: -1.0 },
                    ],
                },
                Contour {
                    points: vec![
                        Point { x: 2.0, y: 0.0 },
                        Point { x: 0.0, y: 2.0 },
                        Point { x: -2.0, y: 0.0 },
                        Point { x: 0.0, y: -2.0 },
                    ],
                },
            ],
        };

        let cc = c.split_to_connected_areas();
        assert_eq!(cc.len(), 1);
        assert_eq!(cc[0].contours.len(), 2);
    }
}
