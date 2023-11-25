use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;

#[derive(Debug, Default, Clone)]
struct NearAxis {
    dist: f32,
    pos: Point,
}
pub struct PartCreator {
    axis: Vec<Point>,
    normals: Vec<Point>,

    p6: Point,
    p7: Point,

    p26_4: Point,
    p62_4: Point,
    p36_4: Point,
    p63_4: Point,
    p46_3: Point,
    p65_3: Point,
    p32_3: Point,

    cone_angle: f32,
    screw_diam: f32,
    head_diam: f32,
    thread_diam: f32,

    axis_pos: RefCell<Vec<NearAxis>>,
    axis_neg: RefCell<Vec<NearAxis>>,

    long_edges: Vec<u32>,
}

pub fn sqr(x: f32) -> f32 {
    x * x
}

impl PartCreator {
    pub fn new() -> Self {
        let sqrt3 = 3.0f32.sqrt();
        let sqrt15 = 15.0f32.sqrt();

        let axis = vec![
            Point {
                x: -sqrt3 / 3.0,
                y: -sqrt3 / 3.0,
                z: -sqrt3 / 3.0,
            },
            Point {
                x: -sqrt3 / 3.0,
                y: -sqrt3 / 3.0,
                z: sqrt3 / 3.0,
            },
            Point {
                x: -sqrt3 / 3.0,
                y: sqrt3 / 3.0,
                z: -sqrt3 / 3.0,
            },
            Point {
                x: -sqrt3 / 3.0,
                y: sqrt3 / 3.0,
                z: sqrt3 / 3.0,
            },
            Point {
                x: sqrt3 / 3.0,
                y: -sqrt3 / 3.0,
                z: -sqrt3 / 3.0,
            },
            Point {
                x: sqrt3 / 3.0,
                y: -sqrt3 / 3.0,
                z: sqrt3 / 3.0,
            },
            Point {
                x: (sqrt15 + sqrt3) / 6.0,
                y: (sqrt15 - sqrt3) / 6.0,
                z: 0.0,
            }
        ];

        let p6 = Point {
            x: sqrt3 / 3.0,
            y: sqrt3 / 3.0,
            z: -sqrt3 / 3.0,
        };
        let p7 = Point {
            x: sqrt3 / 3.0,
            y: sqrt3 / 3.0,
            z: sqrt3 / 3.0,
        };

        fn find4(n1: Point, n2: Point) -> Point {
            let mid = (n1 + n2).scale(0.5);
            let dn = n1 - mid;
            mid + cross(mid, dn).scale(mid.len().recip())
        }

        fn find3(n1: Point, n2: Point) -> Point {
            let mid = (n1 + n2).scale(0.5);
            let dn = n1 - mid;
            let mut result = mid + cross(mid, dn).scale(mid.len().recip() * 3.0f32.sqrt());
            let l = dn.len() * 3.0f32.sqrt();
            for _ in 0..99 {
                result = (mid + (result - mid).norm().scale(l)).norm();
            }

            result
        }

        let p26_4 = find4(axis[2], axis[6]);
        let p62_4 = find4(axis[6], axis[2]);
        let p36_4 = find4(axis[3], axis[6]);
        let p63_4 = find4(axis[6], axis[3]);

        let p46_3 = find3(axis[4], axis[6]);
        let p65_3 = find3(axis[6], axis[5]);
        let p32_3 = find3(axis[3], axis[2]);

        let normals = vec![
            Point {
                x: -1.0,
                y: 0.0,
                z: 0.0,
            },
            Point {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            Point {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            Point {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            },
            (axis[2] + axis[6]).norm(),
            (axis[3] + axis[6]).norm(),
            (axis[4] + axis[5] + axis[6]).norm().scale(0.92),
            (axis[4] + axis[5] + axis[6]).norm().scale(-0.92),
        ];

        let cone_angle = 0.883;
        let screw_diam = 3.0;
        let head_diam = 5.6;
        let thread_diam = 2.5;

        let axis_pos = RefCell::new(Vec::new());
        let axis_neg = RefCell::new(Vec::new());

        let long_edges = vec![
            1 << 0 | 1 << 3,
            1 << 1 | 1 << 2,
            1 << 0 | 1 << 5,
            1 << 1 | 1 << 4,
            1 << 2 | 1 << 4,
            1 << 3 | 1 << 5,
            1 << 2 | 1 << 6,
            1 << 3 | 1 << 6,
            1 << 0 | 1 << 7,
            1 << 1 | 1 << 7,
        ];

        Self {
            axis,
            normals,
            p6,
            p7,
            p26_4,
            p62_4,
            p36_4,
            p63_4,
            p46_3,
            p65_3,
            p32_3,
            cone_angle,
            screw_diam,
            head_diam,
            thread_diam,
            axis_pos,
            axis_neg,
            long_edges,
        }
    }

    pub fn get_part_index(&self, pos: Point) -> PartIndex {
        let r = pos.len();
        if pos.x.abs() > 64.999 || pos.y.abs() > 64.999 || pos.z.abs() > 64.999 {
            return 0;
        }

        let mut wall = false;
        let mut cup = false;
        let mut core = true;
        for &n in &self.normals {
            let d = dot(n, pos);
            if d > 34.999 {
                return 0;
            }
            if d > 33.5 {
                wall = true;
                cup = true;
            } else if d > 33.3 {
                core = false;
            }
        }

        if !cup {
            for a in &self.axis {
                let diam = if r > 30.6 {
                    self.head_diam
                } else if r > 28.6 {
                    self.screw_diam
                } else {
                    self.thread_diam
                };
                if dot(pos, *a) > 0.0 && cross(pos, *a).sqr_len() < sqr(diam * 0.5) {
                    if r > 38.0 && cross(pos, *a).sqr_len() < sqr((diam - 0.3) * 0.5) {
                        cup = true;
                    } else {
                        return 0;
                    }
                }
            }
        }

        if r <= 28.6 {
            let mut index = 255;
            if pos.x < 0.0 {
                index -= 1;
            }
            if pos.y < 0.0 {
                index -= 2;
            }

            if pos.x > -2.0 && pos.x < 3.0 && sqr(pos.z.abs() - 11.3) + sqr(pos.y - 11.3) < sqr(1.9)
            {
                return 254;
            }
            if pos.x > 0.0 && pos.x < 5.0 && sqr(pos.z.abs() - 11.3) + sqr(pos.y - 11.3) < sqr(2.1)
            {
                return 0;
            }
            if pos.x > -3.0 && pos.x < 2.0 && sqr(pos.z.abs() - 11.3) + sqr(pos.y + 11.3) < sqr(1.9)
            {
                return 253;
            }
            if pos.x < 0.0 && pos.x > -5.0 && sqr(pos.z.abs() - 11.3) + sqr(pos.y + 11.3) < sqr(2.1)
            {
                return 0;
            }
            if pos.y > -3.0 && pos.y < 2.0 && sqr(pos.z.abs() - 11.3) + sqr(pos.x - 11.3) < sqr(1.9)
            {
                return 255;
            }
            if pos.y < 0.0 && pos.y > -5.0 && sqr(pos.z.abs() - 11.3) + sqr(pos.x - 11.3) < sqr(2.1)
            {
                return 0;
            }
            if pos.y > -2.0 && pos.y < 3.0 && sqr(pos.z.abs() - 11.3) + sqr(pos.x + 11.3) < sqr(1.9)
            {
                return 252;
            }
            if pos.y > 0.0 && pos.y < 5.0 && sqr(pos.z.abs() - 11.3) + sqr(pos.x + 11.3) < sqr(2.1)
            {
                return 0;
            }

            if pos.x.abs() < 0.2 {
                return 0;
            }
            if pos.y.abs() < 0.2 {
                return 0;
            }
            return index;
        }

        let mut index = 0;
        let mut axis_pos = self.axis_pos.borrow_mut();
        let mut axis_neg = self.axis_neg.borrow_mut();
        axis_pos.clear();
        axis_neg.clear();

        let mut sharp_in = false;
        let mut sharp_out = false;
        macro_rules! match_axis {
            ($pos: expr, $a: expr, $cone_angle: expr, $id: expr) => {
                let pos = $pos;
                let a = $a;
                let mut cone_angle = $cone_angle;
                let id = $id;

                let mut cone_angle_in = cone_angle;
                if r > 33.70 {
                } else if r > 33.30 {
                    cone_angle_in -= (33.70 - r) * 0.01;
                } else if r > 32.90 {
                    cone_angle_in -= 0.050;
                    sharp_in = true;
                } else if r > 30.90 {
                    cone_angle_in -= 0.050;
                    cone_angle -= 0.050;
                    sharp_in = true;
                    sharp_out = true;
                } else if r > 30.50 {
                    cone_angle_in -= 0.050;
                    sharp_in = true;
                } else if r > 30.10 {
                    cone_angle_in -= (r - 30.10) * 0.015;
                } else {
                }

                let p1 = a.any_perp().norm();
                let p2 = cross(a, p1);
                let spiral_a = f32::atan2(dot(pos, p1), dot(pos, p2)) / std::f32::consts::PI;

                if dot(pos, a) > 0.0 {
                    let sin = cross(pos, a).len() / r;
                    if sin < cone_angle_in {
                        let curvyness = 16.0;
                        let d = 1.0 - (cone_angle_in - sin) * curvyness;
                        if d > 0.0 {
                            axis_pos.push(NearAxis { dist: d, pos: a });
                        }

                        let in_spiral = r * 0.2 - spiral_a * 15.0;
                        let in_spiral = in_spiral - in_spiral.floor();
                        let in_spiral = f32::max(in_spiral * (0.2 - in_spiral) * 0.6, 0.0);
                        if sharp_in || wall || sin < cone_angle_in - in_spiral {
                            index += 1 << id;
                        } else {
                            return 0;
                        }
                    } else if sin > cone_angle {
                        let curvyness = 16.0;
                        let d = 1.0 - (sin - cone_angle) * curvyness;
                        if d > 0.0 {
                            axis_neg.push(NearAxis { dist: d, pos: a });
                        }

                        let in_spiral = r * 0.2 + spiral_a * 15.0;
                        let in_spiral = in_spiral - in_spiral.floor();
                        let in_spiral = f32::max(in_spiral * (0.2 - in_spiral) * 0.6, 0.0);

                        if sharp_out || wall || sin > cone_angle_in + in_spiral {
                            // nothing
                        } else {
                            return 0;
                        }
                    } else {
                        return 0;
                    }
                }
            };
        }

        for i in 0..self.axis.len()-1 {
            match_axis!(pos, self.axis[i], self.cone_angle, i);
        }

        if index == 1 << 2 | 1 << 3 | 1 << 6 {
            return 0;
        }
        if index == 1 << 2 | 1 << 4 | 1 << 6 {
            return 0;
        }
        if index == 1 << 3 | 1 << 5 | 1 << 6 {
            return 0;
        }

        if index != 1 << 2 | 1 << 4 && index != 1 << 3 | 1 << 5 && index != 1 << 2 | 1 << 3 {
            match_axis!(pos, self.axis[6], self.cone_angle, 6);
        }

        if index == 1 << 2 | 1 << 6 {
            match_axis!(pos, self.p26_4, self.cone_angle, self.axis.len());
            match_axis!(pos, self.p62_4, self.cone_angle, self.axis.len() + 1);
        }
        if index == 1 << 3 | 1 << 6 {
            match_axis!(pos, self.p36_4, self.cone_angle, self.axis.len());
            match_axis!(pos, self.p63_4, self.cone_angle, self.axis.len() + 1);
        }
        if index == 1 << 4 | 1 << 6 {
            match_axis!(pos, self.p46_3, self.cone_angle, self.axis.len());
        }
        if index == 1 << 5 | 1 << 6 {
            match_axis!(pos, self.p65_3, self.cone_angle, self.axis.len());
        }
        if index == 1 << 2 | 1 << 3 {
            match_axis!(pos, self.p32_3, self.cone_angle, self.axis.len());
        }
        if index & (1 << 0) != 0 {
            match_axis!(pos, self.p6, self.cone_angle, self.axis.len());
        }
        if index & (1 << 1) != 0 {
            match_axis!(pos, self.p7, self.cone_angle, self.axis.len());
        }

        for i in 0..self.axis.len() {
            if index == (1 << i) {
                if cup {
                    index += 1 << self.axis.len();
                } else if !core {
                    return 0;
                }
            }
        }

        let validate = |ap: &[f32], k: f32| -> bool {
            let d1 = f32::max(0.0, 1.0 - k * (1.0 - ap[0]));
            let d2 = f32::max(0.0, 1.0 - k * (1.0 - ap[1]));
            return sqr(d1) + sqr(d2) < 1.0;
        };

        if axis_pos.len() == 2
            && !validate(
                &[axis_pos[0].dist, axis_pos[1].dist],
                if sharp_in { 3.0 } else { 1.0 },
            )
        {
            return 0;
        }

        if axis_neg.len() == 2
            && !validate(
                &[axis_neg[0].dist, axis_neg[1].dist],
                if sharp_out { 1.0 } else { 3.0 },
            )
        {
            return 0;
        }

        let in_long = self.long_edges.iter().any(|&e| (index & e) == e);

        if axis_pos.len() == 1 && axis_neg.len() == 1 {
            if !validate(
                &[axis_pos[0].dist, axis_neg[0].dist],
                if in_long { 1.0 } else { 3.0 },
            ) {
                return 0;
            }
        }

        return index;
    }
}
