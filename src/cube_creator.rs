use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;
use std::ops::DerefMut;

pub struct CubeCreator {
    axis: Vec<Point>,
    normals: Vec<Point>,
    n_basis: Vec<(Point, Point)>,
    l: Vec<f32>,

    screw_diam: f32,
    head_diam: f32,
    thread_diam: f32,

    axis_dst: RefCell<Vec<f32>>,
}

pub fn sqr(x: f32) -> f32 {
    x * x
}

impl CubeCreator {
    pub fn new() -> Self {
        let axis = vec![
            Point { x: -1.0, y: 0.0, z: 0.0 },
            Point { x: 0.0, y: -1.0, z: 0.0 },
            Point { x: 0.0, y: 0.0, z: -1.0 },
            Point { x: 1.0, y: 0.0, z: 0.0 },
            Point { x: 0.0, y: 1.0, z: 0.0 },
            Point { x: 0.0, y: 0.0, z: 1.0 },
        ];

        let l = vec![6.0, 6.0, 6.0, 6.0, 13.0, 13.0];

        let normals = axis.clone();

        let n_basis = normals
            .iter()
            .map(|&n| {
                let n1 = n.any_perp().norm();
                let n2 = cross(n, n1).norm();
                (n1, n2)
            })
            .collect();

        let cone_angle = 0.883;
        let screw_diam = 3.0;
        let head_diam = 5.6;
        let thread_diam = 2.5;

        let axis_dst = RefCell::new(Vec::new());

        Self { axis, normals, n_basis, l, screw_diam, head_diam, thread_diam, axis_dst }
    }

    pub fn faces(&self) -> usize {
        self.normals.len()
    }

    pub fn get_part_index(&self, pos: Point) -> PartIndex {
        self.get_part_index_impl(pos, self.normals.len())
    }

    pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
        let n = self.normals[current_normal];
        let (n1, n2) = self.n_basis[current_normal];
        let pos = n.scale(35.0 / n.sqr_len()) + n1.scale(pos.x) + n2.scale(pos.y);
        let result = self.get_part_index_impl(pos, current_normal);
        (result > 0) as PartIndex
    }

    pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
        let r = pos.len();
        if pos.x.abs() > 64.999 || pos.y.abs() > 64.999 || pos.z.abs() > 64.999 {
            return 0;
        }

        let sticker = current_normal < self.normals.len();

        let mut cup = false;
        for i in 0..self.normals.len() {
            let n = self.normals[i];
            let d = dot(n, pos);
            let center_dist = if sticker { 18.499 } else { 18.999 } + self.l[i];
            if i != current_normal && d > center_dist {
                return 0;
            }
            if d > self.l[i] + 18.0 {
                cup = true;
            }
        }

        if !cup {
            for i in 0..self.axis.len() {
                let a = self.axis[i];
                let diam = if r > 12.5 {
                    self.head_diam
                } else if r > 10.0 {
                    self.screw_diam
                } else {
                    self.thread_diam
                };
                if dot(pos, a) > 0.0 && cross(pos, a).sqr_len() < sqr(diam * 0.5) {
                    return 0;
                }
            }
        }

        if r < 10.0 {
            if r > 9.2 {
                return 0;
            }
            return 1024;
        }

        let r_in = 12.0;
        let r_out = 23.7;

        let mut axis_dst_h = self.axis_dst.borrow_mut();
        let axis_dst = axis_dst_h.deref_mut();

        let mut pure_f = |pos: Point, r: f32| -> (PartIndex, f32, f32) {
            axis_dst.clear();
            let mut index: PartIndex = 0;
            let radius = f32::min(6.0, 1.5 * sqr(r / r_in));

            for i in 0..self.axis.len() {
                let a = self.axis[i];
                let w = self.l[i];
                let d = dot(pos, a);

                if d < 0.0 {
                    continue;
                }

                let axis_border;

                if r < r_in {
                    axis_border = 1.5;
                } else {
                    axis_border = f32::min(w, 5.0 * (r / r_in) + sqr(r - r_in) * 0.012);
                }

                if d > axis_border {
                    if d - axis_border < 6.0 {
                        axis_dst.push(d - axis_border);
                    }
                    index += 1 << i;
                } else {
                    if axis_border - d < 6.0 {
                        axis_dst.push(axis_border - d);
                    }
                }
            }

            axis_dst.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let mut shortest_dist = axis_dst.get(0).copied().unwrap_or(6.0);
            let mut shortest_dist_top = axis_dst.get(0).copied().unwrap_or(6.0);

            if axis_dst.len() >= 2 {
                shortest_dist = radius
                    - (sqr(f32::max(0.0, radius - axis_dst[0]))
                        + sqr(f32::max(0.0, radius - axis_dst[1])))
                    .sqrt();
                if shortest_dist < 0.0 {
                    return (0, 0.0, 0.0);
                }

                shortest_dist_top = 6.0
                    - (sqr(f32::max(0.0, 6.0 - axis_dst[0]))
                        + sqr(f32::max(0.0, 6.0 - axis_dst[1])))
                    .sqrt();
            }

            if sticker && shortest_dist < 0.5 {
                return (0, 0.0, 0.0);
            }

            return (index, shortest_dist, shortest_dist_top);
        };

        let (mut index, shortest_dist, shortest_dist_top) = pure_f(pos, r);
        if index == 0 {
            return 0;
        }

        /*
        let delta = 0.2 + f32::max(0.0, 2.0 - shortest_dist) * 0.25;

        fn change_longest_coord(p: Point, r: f32, delta: f32, dl: Point) -> Point {
            if p.x.abs() - dl.x > p.y.abs() - dl.y && p.x.abs() - dl.x > p.z.abs() - dl.z {
                let nx = (sqr(r + delta) - sqr(p.y) - sqr(p.z)).sqrt() * p.x.signum();
                Point { x: nx, y: p.y, z: p.z }
            } else if p.y.abs() - dl.y > p.z.abs() - dl.z {
                let ny = (sqr(r + delta) - sqr(p.x) - sqr(p.z)).sqrt() * p.y.signum();
                Point { x: p.x, y: ny, z: p.z }
            } else {
                let nz = (sqr(r + delta) - sqr(p.x) - sqr(p.y)).sqrt() * p.z.signum();
                Point { x: p.x, y: p.y, z: nz }
            }
        }

        let pl = Point {
            x: if r < r_out || pos.x < 0.0 { 0.0 } else { 0.0 },
            y: if r < r_out || pos.y < 0.0 { 0.0 } else { 7.0 },
            z: if r < r_out || pos.z < 0.0 { 0.0 } else { 7.0 },
        };

        if pure_f(change_longest_coord(pos, r, -delta, pl), r - delta).0 != index
            || pure_f(change_longest_coord(pos, r, delta, pl), r + delta).0 != index
        {
            return 0;
        }*/

        if (index & (index - 1)) == 0 {
            if r < r_in {
                return 0;
            }
            let j = index.ilog2() as usize;
            let d = dot(pos, self.axis[j]);
            let l = self.l[j];
            if d > l + 13.0 {
                if d > l + 17.0 || (shortest_dist_top > 2.1 && d > l + 13.3) {
                    index += 64;
                } else if d > l + 16.8 || shortest_dist_top > 2.0 {
                    return 0;
                }
            }
        }

        return index;
    }
}
