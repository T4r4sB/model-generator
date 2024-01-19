use crate::points3d::*;
use crate::solid::*;
use rand::Rng;

pub struct Node {
    pos: Point,
    r: f32,
}
pub struct TreeCreator {
    rng: rand::rngs::ThreadRng,
    nodes: Vec<Node>,
    wall: Vec<crate::points2d::Point>,
    h: Vec<i32>,
}

pub fn sqr(x: f32) -> f32 {
    x * x
}

impl TreeCreator {
    fn random_point(&mut self) -> Point {
        let h: f32 = self.rng.gen_range(-1.0..1.0);
        let r = (1.0 - h * h).sqrt();
        let h: f32 = self.rng.gen_range(-1.0..1.0);
        let a: f32 = self.rng.gen_range(0.0..std::f32::consts::PI * 2.0);
        let (s, c) = a.sin_cos();
        Point { x: r * c, y: r * s, z: h }
    }

    fn generate(&mut self, pos: Point, dn: Point, ttn: usize, dl: f32) {
        let l = dn.len();
        if l < 0.5 {
            self.nodes
                .push(Node { pos, r: self.rng.gen_range(3.0..9.0) });
            return;
        }
        self.nodes.push(Node { pos, r: l * 2.0 });

        let ttn = if ttn == 0 {
            self.rng.gen_range(5..10)
        } else {
            ttn - 1
        };

        let branch = ttn == 0;
        let delta = self.random_point();
        let delta = if branch {
            delta.scale(l * 0.7)
        } else {
            delta.scale(l * 0.05)
        };
        let ndn = dn + delta;
        let ndn = ndn.norm().scale(l * dl);
        let dl = if branch {
            self.rng.gen_range(0.96..0.99)
        } else {
            dl
        };
        self.generate(pos + ndn, ndn, ttn, dl);
        if branch {
            let ndn = dn - delta;
            let ndn = ndn.norm().scale(l * dl);
            let dl = self.rng.gen_range(0.96..0.99);
            self.generate(pos + ndn, ndn, ttn, dl);
        }
    }

    pub fn new() -> Self {
        let mut result = Self { rng: rand::thread_rng(), nodes: Vec::new(), wall: Vec::new(), h: Vec::new() };
        result.generate(
            Point { x: 0.0, y: 0.0, z: -18.0 },
            Point { x: 0.0, y: 0.0, z: 1.5 },
            0,
            0.98,
        );
        result
            .wall
            .push(crate::points2d::Point { x: 30.0, y: -50.0 });
        result
            .wall
            .push(crate::points2d::Point { x: 60.0, y: 25.0 });
        result
            .wall
            .push(crate::points2d::Point { x: -30.0, y: 40.0 });
        result
            .wall
            .push(crate::points2d::Point { x: -60.0, y: 0.0 });
        result
            .wall
            .push(crate::points2d::Point { x: -40.0, y: -40.0 });
        for w in &mut result.wall {
            *w = w.scale(0.5);
        }
        result.h.resize(result.wall.len(), 0);
        for h in &mut result.h {
            *h += result.rng.gen_range(-1 ..=2) * 2;
        }
        result
    }

    pub fn get_part_index(&self, pos: Point) -> PartIndex {
        if pos.x.abs() > 129.999 || pos.y.abs() > 129.999 || pos.z.abs() > 129.999 {
            return 0;
        }
        if pos.z < -20.0 {
            return 0;
        }

        let proj = crate::points2d::Point { x: pos.x, y: pos.y };

        let layer_shift = f32::min(0.0, (((pos.z + 100.0) * 0.5).fract() - 0.5).abs() - 0.2);
        let odd_level = ((pos.z + 100.0) * 0.25 + 0.25).fract() < 0.5;

        let mut in_footing = true;
        let mut prev = *self.wall.last().unwrap() - proj;
        for &w in &self.wall {
            let cur = w - proj;
            if crate::points2d::cross(prev, cur) < 0.0 {
                in_footing = false;
                break;
            }
            prev = cur;
        }

        if pos.z < -16.0 {
            if in_footing {
                return 1;
            }
        } else {
            for &w in &self.wall {
                if (w - proj).len() < 5.0 {
                    return 0;
                }
            }
        }

        for wi in 0..self.wall.len() - 1 {
            let p1 = self.wall[wi];
            let p2 = self.wall[wi + 1];
            let dst = crate::points2d::dist_pl(proj, p1, p2);

            let mut brick_part = crate::points2d::dot(proj - p1, p2 - p1) / (p2 - p1).len() * 0.25;
            if odd_level {
                brick_part += 0.5;
            }

            let brick_part = f32::min(0.0, (brick_part.fract() - 0.5).abs() - 0.2);
            let w = 8.0;
            let w = w + 3.0 * f32::min(brick_part, layer_shift);

            if pos.z < 1.0 && w - dst > (pos.z + 20.0) * 0.2 {
                return 1;
            }
        }

        for i in 0 .. self.wall.len() {
            let w = self.wall[i];
            let dw = w - proj;
            let a = f32::atan2(dw.y, dw.x) + std::f32::consts::PI * 2.0;
            let a_part = a / std::f32::consts::PI * 4.0;

            let h = self.h[i] as f32 + 16.0;
            
            let dst = dw.len();
            let w = if pos.z < h - 12.0 {
                12.0
            } else if pos.z < h - 6.0 {
                (pos.z - (h - 12.0)) * 2.0 / 3.0 + 12.0
            } else {
                16.0
            };

            let mut brick_part = a / std::f32::consts::PI * 8.0;
            if odd_level {
                brick_part += 0.5;
            }

            let brick_part = f32::min(0.0, (brick_part.fract() - 0.5).abs() - 0.2);
            let w = w + 3.0 * f32::min(brick_part, layer_shift);

            if pos.z < h - 2.0 && w - dst > (pos.z + 20.0) * 0.2 {
                if pos.z < h - 5.0 || (dst > 6.0 && a_part.fract() < 0.5) {
                    return 1;
                }
            }
        }

        for node in &self.nodes {
            let mut delta = pos - node.pos;
            delta.z *= 1.3;
            if delta.sqr_len() < sqr(node.r) {
                return 1;
            }
        }

        let ely = sqr(pos.y) + sqr(pos.z + 5.0) * 0.25;
        let elx = sqr(pos.x) + sqr(pos.z + 5.0) * 0.25;

        if pos.x.abs() > 8.0 && pos.x.abs() < 10.0 && ely > sqr(7.5) && ely < sqr(10.5) {
            return 1;
        }
        if pos.y.abs() > 8.0 && pos.y.abs() < 10.0 && elx > sqr(7.5) && elx < sqr(10.5) {
            return 1;
        }

        if pos.z < -10.0 && pos.z > (pos.x.abs() + pos.y.abs()) - 20.0 {
            return 1;
        }

        return 0;
    }
}
