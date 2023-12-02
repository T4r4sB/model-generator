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
        Point {
            x: r * c,
            y: r * s,
            z: h,
        }
    }

    fn generate(&mut self, pos: Point, dn: Point, ttn: usize, dl: f32) {
        let l = dn.len();
        if l < 0.5 {
            self.nodes.push(Node {
                pos,
                r: self.rng.gen_range(3.0..9.0),
            });
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
        let mut result = Self {
            rng: rand::thread_rng(),
            nodes: Vec::new(),
        };
        result.generate(
            Point {
                x: 0.0,
                y: 0.0,
                z: -18.0,
            },
            Point {
                x: 0.0,
                y: 0.0,
                z: 1.5,
            },
            0,
            0.98,
        );
        result
    }

    pub fn get_part_index(&self, pos: Point) -> PartIndex {
        if pos.x.abs() > 34.999 || pos.y.abs() > 34.999 || pos.z.abs() > 34.999 {
            return 0;
        }
        if pos.z < -20.0 {
            return 0;
        }

        for node in &self.nodes {
            let mut delta = pos - node.pos;
            delta.z *= 1.3;
            if delta.sqr_len() < sqr(node.r) {
                return 1;
            }
        }

        if pos.z < -16.0 && pos.x.abs() < 10.0 && pos.y.abs() < 10.0 {
            return 1;
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
