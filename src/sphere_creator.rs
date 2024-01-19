use crate::model::*;
use crate::points3d::*;
use crate::solid::*;

use std::cell::RefCell;
use std::ops::DerefMut;

pub struct SphereCreator {
}

pub fn sqr(x: f32) -> f32 {
    x * x
}

impl SphereCreator {
    pub fn new() -> Self {
   

        Self {  }
    }

    pub fn faces(&self) -> usize {
        0
    }

    pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
        0
    }

    pub fn get_part_index(&self, pos: Point) -> PartIndex {
        if pos.z < -3.0 || pos.z > 3.0 {
            return 0;
        }
        
        if pos.x.abs() < 10.0 {
            if dist_pl(pos, Point{x: 10.0, y: 17.0, z:0.0}, Point{x: -10.0, y: 17.0, z:0.0}) < 3.0 {
                return 1;
            }
            
            if dist_pl(pos, Point{x: -5.0, y: -17.0, z:0.0}, Point{x: -10.0, y: -17.0, z:0.0}) < 3.0 {
                return 1;
            }

            if dist_pl(pos, Point{x: 5.0, y: -17.0, z:0.0}, Point{x: 10.0, y: -17.0, z:0.0}) < 3.0 {
                return 1;
            }
        }

        if pos.x > 10.0 {
            let r = (sqr(pos.x - 10.0) + sqr(pos.y)).sqrt();
            if sqr(r - 17.0) + sqr(pos.z) < sqr(3.0) {
                return 1;
            }
        }

        if pos.x < -10.0 {
            let r = (sqr(pos.x + 10.0) + sqr(pos.y)).sqrt();
            if sqr(r - 17.0) + sqr(pos.z) < sqr(3.0) {
                return 1;
            }
        }

        return 0;
    }
}
