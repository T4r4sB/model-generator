use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;
use std::ops::DerefMut;

pub struct SphereCreator {
    points: Vec<Point>,
}

pub fn sqr(x: f32) -> f32 {
    x * x
}

impl SphereCreator {
    pub fn new() -> Self {
        let mut points = Vec::new();
        for i in 0..1 {
            let a = i as f32 * std::f32::consts::PI / 5.0;
            let r = 20.0;
            points.push(Point { x: a.cos() * r, y: a.sin() * r, z: 0.0 });
            points.push(Point { x: -a.cos() * r, y: -a.sin() * r, z: 0.0 });
        }

        Self { points }
    }

    pub fn faces(&self) -> usize {
        0
    }

    pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
        0
    }

    pub fn get_part_index(&self, pos: Point) -> PartIndex {
        if pos.z < 0.0 || pos.z > 8.0 {
            return 0;
        }
        if sqr(pos.x) + sqr(pos.y) > sqr(30.0 - pos.z) {
            return 0;
        }

        for (i, &p) in self.points.iter().enumerate() {
            let d = sqr(pos.x - p.x) + sqr(pos.y - p.y);
            if d < sqr(1.35) || d < sqr(f32::max(0.0, 3.0 - pos.z)) {
                return (1 + i) as PartIndex;
            }
            if d < sqr(1.45) || d < sqr(f32::max(0.0, 3.2 - pos.z)) {
                return 0;
            }
        }

        if pos.z > 3.0 {
            return 0;
        }

        return 100;
    }
}
