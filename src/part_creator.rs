use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::collections::HashMap;

pub struct PartCreator {
    axis: Vec<Point>,

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
}

fn sqr(x: f32) -> f32 {
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
            },
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

        let cone_angle = 0.876;
        let screw_diam = 3.0;
        let head_diam = 5.5;
        let thread_diam = 2.5;

        Self {
            axis,
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
        }
    }

    pub fn get_part_index(&self, pos: Point) -> u32 {
        let r = pos.len();
        if pos.x.abs() > 35.0 || pos.y.abs() > 35.0 || pos.z.abs() > 35.0 {
            return 0;
        }
        let wall = pos.x.abs() > 34.5 || pos.y.abs() > 34.5 || pos.z.abs() > 34.5;

        for a in &self.axis {
            let diam = if r > 30.6 {
                self.head_diam
            } else {
                self.thread_diam
            };
            if dot(pos, *a) > 0.0 && cross(pos, *a).sqr_len() < sqr(diam * 0.5) {
                return 0;
            }
        }

        let index = 255;

        if r < 26.6 {
            for a in &self.axis {
                if dot(pos, *a) > 0.0 && cross(pos, *a).sqr_len() < sqr(self.screw_diam * 0.5 + 3.0)
                {
                    return index;
                }
            }

            return 0;
        }

        if r < 28.6 {
            let x = pos.x / 7.1;
            let y = pos.y / 7.1;
            let z = pos.z / 7.1;

            if ((x - x.floor() - 0.5).abs() < 0.24 && pos.x.abs() < 20.0)
                || ((y - y.floor() - 0.5).abs() < 0.24 && pos.y.abs() < 20.0)
                || ((z - z.floor() - 0.5).abs() < 0.24 && pos.z.abs() < 20.0)
            {
                return index;
            }

            return 0;
        }

        let mut index = 0;

        macro_rules! match_axis {
            ($pos: expr, $a: expr, $cone_angle: expr, $id: expr) => {
                let pos = $pos;
                let a = $a;
                let mut cone_angle = $cone_angle;
                let id = $id;

                let mut cone_angle_in = cone_angle - 0.003;
                if r < 33.0 && r > 30.70 {
                    cone_angle_in -= 0.024;
                }

                if r < 32.85 && r > 30.85 {
                    cone_angle -= 0.024;
                }

                let p1 = a.any_perp().norm();
                let p2 = cross(a, p1);
                let spiral_a = f32::atan2(dot(pos, p1), dot(pos, p2)) / std::f32::consts::PI;

                if dot(pos, a) > 0.0 {
                    let sin = cross(pos, a).len() / r;
                    if sin < cone_angle_in {
                        let in_spiral = r * 0.2 - spiral_a * 15.0;
                        let in_spiral = in_spiral - in_spiral.floor();
                        let in_spiral = f32::max(in_spiral * (0.2 - in_spiral) * 0.6, 0.0);
                        if wall || sin < cone_angle_in - in_spiral {
                            index += 1 << id;
                        } else {
                            return 0;
                        }
                    } else if sin > cone_angle {
                        let in_spiral = r * 0.2 + spiral_a * 15.0;
                        let in_spiral = in_spiral - in_spiral.floor();
                        let in_spiral = f32::max(in_spiral * (0.2 - in_spiral) * 0.6, 0.0);

                        if wall || sin > cone_angle_in + in_spiral {
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

        for i in 0..self.axis.len() {
            match_axis!(pos, self.axis[i], self.cone_angle, i);
        }

        if index == 0 {
            return 0;
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

        return index;
    }
}
