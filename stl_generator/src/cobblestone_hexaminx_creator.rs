use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

use std::cell::RefCell;

const PI: f32 = std::f32::consts::PI;

struct Piece {
  bounds_pos: &'static [usize],
  bounds_neg: &'static [usize],
}

#[derive(Copy, Clone)]
pub enum SubGroup {
  Group(usize),
  Piece(usize),
}

struct Group {
  splitter: usize,
  lhs: SubGroup,
  rhs: SubGroup,
}

const BASIC: &[Point] = &[
  Point { x: 0.0, y: -0.52573115, z: -0.85065085 },
  Point { x: 0.0, y: -0.52573115, z: 0.85065085 },
  Point { x: 0.0, y: 0.52573115, z: -0.85065085 },
  Point { x: 0.0, y: 0.52573115, z: 0.85065085 },
  Point { x: -0.85065085, y: 0.0, z: -0.52573115 },
  Point { x: 0.85065085, y: 0.0, z: -0.52573115 },
  Point { x: -0.85065085, y: 0.0, z: 0.52573115 },
  Point { x: 0.85065085, y: 0.0, z: 0.52573115 },
  Point { x: -0.52573115, y: -0.85065085, z: 0.0 },
  Point { x: -0.52573115, y: 0.85065085, z: 0.0 },
  Point { x: 0.52573115, y: -0.85065085, z: 0.0 },
  Point { x: 0.52573115, y: 0.85065085, z: 0.0 },
];

const SPLITTERS: &[Point] = &[
  Point { x: 0.0, y: -0.52573115, z: 0.85065085 },
  Point { x: 0.18596187, y: -0.979333, z: -0.07952996 },
  Point { x: 0.88952744, y: -0.314644, z: 0.3312704 },
  Point { x: 0.314644, y: -0.3312704, z: 0.88952744 },
  Point { x: -0.4599526, y: -0.8099975, z: 0.3637963 },
  Point { x: 0.52573115, y: -0.85065085, z: 0.0 },
  Point { x: 0.69506675, y: -0.664689, z: 0.2739907 },
  Point { x: 0.8099975, y: -0.36379635, z: 0.4599526 },
  Point { x: 0.14530851, y: -0.77459663, z: -0.61553675 },
  Point { x: 0.3312704, y: -0.88952744, z: -0.31464413 },
  Point { x: 0.85065085, y: 0.0, z: -0.52573115 },
  Point { x: -0.18596187, y: -0.97933304, z: -0.07952998 },
  Point { x: -0.88952744, y: -0.31464413, z: 0.3312704 },
  Point { x: -0.31464413, y: -0.3312704, z: 0.88952744 },
  Point { x: 0.4599526, y: -0.8099975, z: 0.36379635 },
  Point { x: -0.4599526, y: -0.8099975, z: -0.36379635 },
  Point { x: -0.27399072, y: -0.69506675, z: 0.66468894 },
  Point { x: -0.2739907, y: -0.69506675, z: -0.664689 },
  Point { x: -0.52573115, y: -0.85065085, z: 0.0 },
  Point { x: 0.664689, y: 0.2739907, z: -0.69506675 },
  Point { x: -0.88952744, y: -0.314644, z: -0.3312704 },
  Point { x: -0.18596187, y: -0.979333, z: 0.07952996 },
  Point { x: -0.7745967, y: -0.6155367, z: -0.14530852 },
  Point { x: 0.77459663, y: -0.61553675, z: -0.14530851 },
  Point { x: 0.36379635, y: 0.4599526, z: -0.8099975 },
  Point { x: 0.88952744, y: -0.31464413, z: -0.3312704 },
  Point { x: 0.18596187, y: -0.97933304, z: 0.07952998 },
  Point { x: 0.0, y: -0.52573115, z: -0.85065085 },
  Point { x: 0.88952744, y: 0.314644, z: -0.3312704 },
  Point { x: 0.3637963, y: -0.4599526, z: -0.8099975 },
  Point { x: 0.66468894, y: -0.27399072, z: -0.69506675 },
  Point { x: 0.7745967, y: 0.6155367, z: -0.14530852 },
  Point { x: -0.664689, y: -0.2739907, z: -0.69506675 },
  Point { x: -0.3637963, y: 0.4599526, z: -0.8099975 },
  Point { x: 0.0, y: 0.52573115, z: -0.85065085 },
  Point { x: -0.18596187, y: 0.97933304, z: 0.07952998 },
  Point { x: -0.88952744, y: 0.31464413, z: -0.3312704 },
  Point { x: -0.77459663, y: 0.61553675, z: -0.14530851 },
  Point { x: 0.18596187, y: 0.979333, z: 0.07952996 },
  Point { x: -0.52573115, y: 0.85065085, z: 0.0 },
  Point { x: -0.27399072, y: 0.69506675, z: -0.66468894 },
  Point { x: 0.4599526, y: 0.8099975, z: 0.3637963 },
  Point { x: 0.2739907, y: 0.69506675, z: -0.664689 },
  Point { x: 0.4599526, y: 0.8099975, z: -0.36379635 },
  Point { x: 0.27399072, y: 0.69506675, z: 0.66468894 },
  Point { x: 0.97933304, y: -0.07952998, z: 0.18596187 },
  Point { x: 0.6155367, y: 0.14530852, z: -0.7745967 },
  Point { x: 0.979333, y: -0.07952996, z: -0.18596187 },
  Point { x: 0.18467632, y: 0.9801276, z: -0.072419375 },
  Point { x: 0.9606612, y: 0.27068096, z: -0.062143937 },
  Point { x: -0.4599526, y: 0.8099975, z: -0.3637963 },
  Point { x: 0.314644, y: 0.3312704, z: -0.88952744 },
  Point { x: 0.52573115, y: 0.85065085, z: 0.0 },
  Point { x: 0.3637963, y: 0.4599526, z: 0.8099975 },
  Point { x: -0.664689, y: 0.2739907, z: 0.69506675 },
  Point { x: -0.3637963, y: -0.4599526, z: 0.8099975 },
  Point { x: 0.664689, y: -0.2739907, z: 0.69506675 },
  Point { x: 0.18596187, y: 0.97933304, z: -0.07952998 },
  Point { x: 0.88952744, y: 0.31464413, z: 0.3312704 },
  Point { x: 0.31464413, y: 0.3312704, z: 0.88952744 },
  Point { x: -0.4599526, y: 0.8099975, z: 0.36379635 },
  Point { x: -0.7745967, y: 0.6155367, z: 0.14530852 },
  Point { x: -0.18596187, y: 0.979333, z: -0.07952996 },
  Point { x: -0.88952744, y: 0.314644, z: 0.3312704 },
  Point { x: 0.0, y: 0.52573115, z: 0.85065085 },
  Point { x: 0.81379825, y: 0.3630365, z: 0.45380276 },
  Point { x: 0.48477432, y: -0.6226473, z: 0.6142511 },
  Point { x: -0.07952998, y: 0.18596187, z: 0.97933304 },
  Point { x: 0.33127043, y: -0.88952744, z: 0.314644 },
  Point { x: -0.07952996, y: -0.18596187, z: 0.979333 },
  Point { x: 0.14530852, y: -0.7745967, z: 0.6155367 },
  Point { x: 0.6155367, y: -0.14530852, z: 0.7745967 },
  Point { x: 0.97933304, y: 0.07952998, z: -0.18596187 },
  Point { x: 0.69506675, y: -0.66468894, z: -0.27399072 },
  Point { x: 0.8099975, y: 0.36379635, z: -0.4599526 },
  Point { x: 0.3312704, y: 0.88952744, z: 0.31464413 },
  Point { x: 0.66184896, y: 0.26912653, z: 0.6996621 },
  Point { x: 0.8099975, y: -0.36379626, z: -0.4599526 },
  Point { x: 0.9317353, y: -0.34641013, z: -0.10894708 },
  Point { x: 0.69506675, y: 0.664689, z: -0.2739907 },
  Point { x: 0.61709124, y: -0.15194936, z: 0.77208155 },
  Point { x: 0.89332813, y: 0.44696102, z: -0.04680313 },
  Point { x: 0.14530851, y: 0.77459663, z: 0.61553675 },
  Point { x: 0.85065085, y: 0.0, z: 0.52573115 },
  Point { x: -0.69506675, y: 0.664689, z: 0.2739907 },
  Point { x: 0.07952996, y: -0.18596187, z: -0.979333 },
  Point { x: -0.36379635, y: -0.4599526, z: -0.8099975 },
  Point { x: -0.6155367, y: -0.14530852, z: -0.7745967 },
  Point { x: -0.97933304, y: 0.07952998, z: 0.18596187 },
  Point { x: -0.5997051, y: -0.0036346614, z: -0.8002129 },
  Point { x: -0.9657476, y: 0.20110169, z: 0.16398029 },
  Point { x: -0.8099975, y: 0.36379635, z: 0.4599526 },
  Point { x: 0.07952998, y: 0.18596187, z: -0.97933304 },
  Point { x: -0.3312704, y: 0.88952744, z: -0.31464413 },
  Point { x: -0.9483615, y: -0.21184678, z: -0.23607484 },
  Point { x: -0.28806713, y: 0.2177279, z: -0.93252987 },
  Point { x: -0.69506675, y: -0.66468894, z: 0.27399072 },
  Point { x: -0.14530852, y: -0.7745967, z: -0.6155367 },
  Point { x: -0.85065085, y: 0.0, z: -0.52573115 },
  Point { x: -0.69506675, y: -0.664689, z: -0.2739907 },
  Point { x: -0.98012763, y: -0.072419465, z: 0.18467622 },
  Point { x: -0.27068105, y: -0.062143832, z: 0.9606612 },
  Point { x: -0.5997051, y: 0.0036346614, z: 0.8002129 },
  Point { x: -0.8099975, y: -0.36379635, z: -0.4599526 },
  Point { x: -0.9657476, y: -0.20110169, z: -0.16398029 },
  Point { x: -0.3312704, y: -0.88952744, z: 0.31464413 },
  Point { x: -0.33127043, y: 0.88952744, z: 0.314644 },
  Point { x: 0.07952996, y: 0.18596187, z: 0.979333 },
  Point { x: -0.14530852, y: 0.7745967, z: 0.6155367 },
  Point { x: -0.73426837, y: 0.34865648, z: 0.582485 },
  Point { x: -0.63008267, y: -0.6776805, z: 0.37913698 },
  Point { x: 0.07952998, y: -0.18596187, z: 0.97933304 },
  Point { x: -0.66184884, y: 0.26912656, z: 0.6996621 },
  Point { x: -0.9317352, y: -0.3464102, z: -0.108947 },
  Point { x: -0.94611514, y: -0.21772791, z: 0.2397095 },
  Point { x: -0.41235474, y: 0.5431173, z: 0.73142815 },
  Point { x: -0.69506675, y: 0.66468894, z: -0.27399072 },
  Point { x: -0.85065085, y: 0.0, z: 0.52573115 },
];

const PIECES: &[Piece] = &[
  Piece { bounds_pos: &[], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 2, 3, 8, 10] },
  Piece { bounds_pos: &[0], bounds_neg: &[2, 7, 8, 9, 10] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[10, 11] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 11] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[8, 10] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[2, 4, 6, 7, 9, 10] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 9] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[7, 9] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[9, 10] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 7, 8, 9] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[9, 10] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 10] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[8, 9] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[9, 10] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[8, 10] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[7, 9] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[1, 6, 7, 8, 9, 10] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[6, 8] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[1, 9] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[7, 9] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[8, 9] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[7, 9] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[8, 9] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[9, 10] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[7, 10] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[8, 9] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[1, 4, 5, 6, 7] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[7, 8] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[6, 8] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[4, 7] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[6, 7] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 7] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[6, 7] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 3, 4, 5] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[5, 6] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 6] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[4, 5] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[2, 3, 4, 5, 6, 7] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[4, 6] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[6, 8] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[7, 8] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[3, 6] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 5] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[4, 5] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[6, 7] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[5, 7] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[4, 6] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 2, 3, 4, 5] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[5, 6] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 6] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 5] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[4, 5] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 5] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[3, 4] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[4, 5] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[3, 5] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 4] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[2, 3, 4, 5, 6, 7] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 7] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[6, 7] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[5, 6] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[6, 7] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[6, 7] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[4, 7] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[5, 6] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[3, 5] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[4, 5] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[3, 4] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 4] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[3, 4] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[3, 5] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[4, 5] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2, 3, 4, 5, 6] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[0, 2] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[1, 4] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 5] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[4, 5] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[1, 3] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 3] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[4, 5] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[3, 5] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 4] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 3] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0], bounds_neg: &[0, 1, 2, 3, 4, 5] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[3, 4] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[0, 5] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[4, 5] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[3, 4] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 4] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[3, 4] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 3] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[3, 4] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 4] },
  Piece { bounds_pos: &[0, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1], bounds_neg: &[2, 3] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[0, 3] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[1, 2] },
  Piece { bounds_pos: &[1, 2, 3], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
  Piece { bounds_pos: &[0, 1, 2], bounds_neg: &[] },
];

use SubGroup::*;

const GROUPS: &[Group] = &[
  Group { splitter: 0, lhs: Piece(0), rhs: Piece(1) },
  Group { splitter: 1, lhs: Piece(4), rhs: Piece(5) },
  Group { splitter: 2, lhs: Piece(3), rhs: Group(1) },
  Group { splitter: 3, lhs: Piece(6), rhs: Piece(7) },
  Group { splitter: 4, lhs: Group(2), rhs: Group(3) },
  Group { splitter: 0, lhs: Piece(2), rhs: Group(4) },
  Group { splitter: 5, lhs: Group(0), rhs: Group(5) },
  Group { splitter: 6, lhs: Piece(8), rhs: Piece(9) },
  Group { splitter: 7, lhs: Piece(11), rhs: Piece(12) },
  Group { splitter: 8, lhs: Group(8), rhs: Piece(13) },
  Group { splitter: 6, lhs: Piece(10), rhs: Group(9) },
  Group { splitter: 9, lhs: Group(7), rhs: Group(10) },
  Group { splitter: 10, lhs: Group(6), rhs: Group(11) },
  Group { splitter: 11, lhs: Piece(16), rhs: Piece(17) },
  Group { splitter: 12, lhs: Piece(15), rhs: Group(13) },
  Group { splitter: 13, lhs: Piece(18), rhs: Piece(19) },
  Group { splitter: 14, lhs: Group(14), rhs: Group(15) },
  Group { splitter: 0, lhs: Piece(14), rhs: Group(16) },
  Group { splitter: 15, lhs: Piece(21), rhs: Piece(22) },
  Group { splitter: 16, lhs: Piece(20), rhs: Group(18) },
  Group { splitter: 4, lhs: Piece(23), rhs: Piece(24) },
  Group { splitter: 17, lhs: Group(19), rhs: Group(20) },
  Group { splitter: 0, lhs: Group(21), rhs: Piece(25) },
  Group { splitter: 5, lhs: Group(17), rhs: Group(22) },
  Group { splitter: 18, lhs: Group(12), rhs: Group(23) },
  Group { splitter: 19, lhs: Piece(26), rhs: Piece(27) },
  Group { splitter: 20, lhs: Piece(29), rhs: Piece(30) },
  Group { splitter: 21, lhs: Piece(28), rhs: Group(26) },
  Group { splitter: 22, lhs: Group(25), rhs: Group(27) },
  Group { splitter: 10, lhs: Piece(32), rhs: Piece(33) },
  Group { splitter: 5, lhs: Piece(31), rhs: Group(29) },
  Group { splitter: 23, lhs: Piece(34), rhs: Piece(35) },
  Group { splitter: 24, lhs: Group(31), rhs: Piece(36) },
  Group { splitter: 19, lhs: Group(30), rhs: Group(32) },
  Group { splitter: 25, lhs: Group(28), rhs: Group(33) },
  Group { splitter: 21, lhs: Piece(38), rhs: Piece(39) },
  Group { splitter: 23, lhs: Piece(37), rhs: Group(35) },
  Group { splitter: 5, lhs: Piece(40), rhs: Piece(41) },
  Group { splitter: 18, lhs: Group(36), rhs: Group(37) },
  Group { splitter: 22, lhs: Group(38), rhs: Piece(42) },
  Group { splitter: 25, lhs: Group(39), rhs: Piece(43) },
  Group { splitter: 26, lhs: Group(34), rhs: Group(40) },
  Group { splitter: 27, lhs: Group(24), rhs: Group(41) },
  Group { splitter: 28, lhs: Piece(46), rhs: Piece(47) },
  Group { splitter: 29, lhs: Piece(45), rhs: Group(43) },
  Group { splitter: 30, lhs: Piece(48), rhs: Piece(49) },
  Group { splitter: 31, lhs: Group(44), rhs: Group(45) },
  Group { splitter: 10, lhs: Piece(44), rhs: Group(46) },
  Group { splitter: 29, lhs: Piece(51), rhs: Piece(52) },
  Group { splitter: 32, lhs: Piece(50), rhs: Group(48) },
  Group { splitter: 10, lhs: Piece(53), rhs: Piece(54) },
  Group { splitter: 33, lhs: Group(50), rhs: Piece(55) },
  Group { splitter: 19, lhs: Group(49), rhs: Group(51) },
  Group { splitter: 27, lhs: Group(47), rhs: Group(52) },
  Group { splitter: 34, lhs: Group(42), rhs: Group(53) },
  Group { splitter: 35, lhs: Piece(58), rhs: Piece(59) },
  Group { splitter: 36, lhs: Piece(57), rhs: Group(55) },
  Group { splitter: 37, lhs: Piece(60), rhs: Piece(61) },
  Group { splitter: 38, lhs: Group(56), rhs: Group(57) },
  Group { splitter: 34, lhs: Piece(56), rhs: Group(58) },
  Group { splitter: 39, lhs: Group(54), rhs: Group(59) },
  Group { splitter: 40, lhs: Piece(62), rhs: Piece(63) },
  Group { splitter: 41, lhs: Piece(65), rhs: Piece(66) },
  Group { splitter: 42, lhs: Piece(64), rhs: Group(62) },
  Group { splitter: 34, lhs: Group(63), rhs: Piece(67) },
  Group { splitter: 43, lhs: Piece(68), rhs: Piece(69) },
  Group { splitter: 44, lhs: Group(64), rhs: Group(65) },
  Group { splitter: 39, lhs: Group(61), rhs: Group(66) },
  Group { splitter: 45, lhs: Piece(71), rhs: Piece(72) },
  Group { splitter: 46, lhs: Piece(70), rhs: Group(68) },
  Group { splitter: 47, lhs: Group(67), rhs: Group(69) },
  Group { splitter: 46, lhs: Piece(74), rhs: Piece(75) },
  Group { splitter: 40, lhs: Piece(73), rhs: Group(71) },
  Group { splitter: 48, lhs: Piece(76), rhs: Piece(77) },
  Group { splitter: 49, lhs: Group(72), rhs: Group(73) },
  Group { splitter: 50, lhs: Group(74), rhs: Piece(78) },
  Group { splitter: 47, lhs: Group(75), rhs: Piece(79) },
  Group { splitter: 51, lhs: Group(70), rhs: Group(76) },
  Group { splitter: 52, lhs: Group(60), rhs: Group(77) },
  Group { splitter: 53, lhs: Piece(82), rhs: Piece(83) },
  Group { splitter: 54, lhs: Piece(81), rhs: Group(79) },
  Group { splitter: 55, lhs: Piece(84), rhs: Piece(85) },
  Group { splitter: 56, lhs: Group(80), rhs: Group(81) },
  Group { splitter: 0, lhs: Piece(80), rhs: Group(82) },
  Group { splitter: 57, lhs: Piece(87), rhs: Piece(88) },
  Group { splitter: 58, lhs: Piece(86), rhs: Group(84) },
  Group { splitter: 59, lhs: Piece(89), rhs: Piece(90) },
  Group { splitter: 60, lhs: Group(85), rhs: Group(86) },
  Group { splitter: 52, lhs: Group(83), rhs: Group(87) },
  Group { splitter: 61, lhs: Piece(92), rhs: Piece(93) },
  Group { splitter: 57, lhs: Piece(91), rhs: Group(89) },
  Group { splitter: 62, lhs: Piece(94), rhs: Piece(95) },
  Group { splitter: 63, lhs: Group(90), rhs: Group(91) },
  Group { splitter: 52, lhs: Group(92), rhs: Piece(96) },
  Group { splitter: 39, lhs: Group(88), rhs: Group(93) },
  Group { splitter: 64, lhs: Group(78), rhs: Group(94) },
  Group { splitter: 65, lhs: Piece(99), rhs: Piece(100) },
  Group { splitter: 66, lhs: Piece(98), rhs: Group(96) },
  Group { splitter: 67, lhs: Piece(97), rhs: Group(97) },
  Group { splitter: 67, lhs: Piece(102), rhs: Piece(103) },
  Group { splitter: 68, lhs: Group(99), rhs: Piece(104) },
  Group { splitter: 69, lhs: Piece(101), rhs: Group(100) },
  Group { splitter: 70, lhs: Group(98), rhs: Group(101) },
  Group { splitter: 71, lhs: Piece(106), rhs: Piece(107) },
  Group { splitter: 72, lhs: Piece(105), rhs: Group(103) },
  Group { splitter: 73, lhs: Piece(108), rhs: Piece(109) },
  Group { splitter: 70, lhs: Group(104), rhs: Group(105) },
  Group { splitter: 0, lhs: Group(106), rhs: Piece(110) },
  Group { splitter: 5, lhs: Group(102), rhs: Group(107) },
  Group { splitter: 74, lhs: Piece(112), rhs: Piece(113) },
  Group { splitter: 73, lhs: Piece(111), rhs: Group(109) },
  Group { splitter: 5, lhs: Group(110), rhs: Piece(114) },
  Group { splitter: 10, lhs: Group(108), rhs: Group(111) },
  Group { splitter: 74, lhs: Piece(116), rhs: Piece(117) },
  Group { splitter: 75, lhs: Piece(115), rhs: Group(113) },
  Group { splitter: 76, lhs: Piece(118), rhs: Piece(119) },
  Group { splitter: 77, lhs: Group(115), rhs: Piece(120) },
  Group { splitter: 78, lhs: Group(114), rhs: Group(116) },
  Group { splitter: 79, lhs: Group(112), rhs: Group(117) },
  Group { splitter: 75, lhs: Piece(122), rhs: Piece(123) },
  Group { splitter: 67, lhs: Piece(121), rhs: Group(119) },
  Group { splitter: 69, lhs: Group(120), rhs: Piece(124) },
  Group { splitter: 80, lhs: Piece(125), rhs: Piece(126) },
  Group { splitter: 81, lhs: Group(121), rhs: Group(122) },
  Group { splitter: 79, lhs: Group(123), rhs: Piece(127) },
  Group { splitter: 82, lhs: Group(118), rhs: Group(124) },
  Group { splitter: 83, lhs: Group(95), rhs: Group(125) },
  Group { splitter: 84, lhs: Piece(128), rhs: Piece(129) },
  Group { splitter: 85, lhs: Group(127), rhs: Piece(130) },
  Group { splitter: 36, lhs: Piece(132), rhs: Piece(133) },
  Group { splitter: 86, lhs: Piece(131), rhs: Group(129) },
  Group { splitter: 27, lhs: Group(130), rhs: Piece(134) },
  Group { splitter: 34, lhs: Group(128), rhs: Group(131) },
  Group { splitter: 87, lhs: Piece(136), rhs: Piece(137) },
  Group { splitter: 88, lhs: Piece(135), rhs: Group(133) },
  Group { splitter: 18, lhs: Group(132), rhs: Group(134) },
  Group { splitter: 89, lhs: Piece(139), rhs: Piece(140) },
  Group { splitter: 90, lhs: Piece(138), rhs: Group(136) },
  Group { splitter: 39, lhs: Piece(141), rhs: Piece(142) },
  Group { splitter: 34, lhs: Group(137), rhs: Group(138) },
  Group { splitter: 91, lhs: Group(139), rhs: Piece(143) },
  Group { splitter: 92, lhs: Group(140), rhs: Piece(144) },
  Group { splitter: 93, lhs: Group(135), rhs: Group(141) },
  Group { splitter: 27, lhs: Piece(146), rhs: Piece(147) },
  Group { splitter: 18, lhs: Piece(145), rhs: Group(143) },
  Group { splitter: 92, lhs: Piece(148), rhs: Piece(149) },
  Group { splitter: 94, lhs: Group(145), rhs: Piece(150) },
  Group { splitter: 95, lhs: Group(144), rhs: Group(146) },
  Group { splitter: 96, lhs: Group(147), rhs: Piece(151) },
  Group { splitter: 97, lhs: Group(142), rhs: Group(148) },
  Group { splitter: 98, lhs: Group(126), rhs: Group(149) },
  Group { splitter: 99, lhs: Piece(152), rhs: Piece(153) },
  Group { splitter: 91, lhs: Piece(155), rhs: Piece(156) },
  Group { splitter: 96, lhs: Piece(154), rhs: Group(152) },
  Group { splitter: 18, lhs: Group(153), rhs: Piece(157) },
  Group { splitter: 98, lhs: Group(151), rhs: Group(154) },
  Group { splitter: 100, lhs: Piece(159), rhs: Piece(160) },
  Group { splitter: 101, lhs: Piece(158), rhs: Group(156) },
  Group { splitter: 102, lhs: Piece(161), rhs: Piece(162) },
  Group { splitter: 103, lhs: Group(158), rhs: Piece(163) },
  Group { splitter: 104, lhs: Group(157), rhs: Group(159) },
  Group { splitter: 105, lhs: Group(155), rhs: Group(160) },
  Group { splitter: 64, lhs: Piece(165), rhs: Piece(166) },
  Group { splitter: 39, lhs: Piece(164), rhs: Group(162) },
  Group { splitter: 106, lhs: Piece(167), rhs: Piece(168) },
  Group { splitter: 107, lhs: Group(163), rhs: Group(164) },
  Group { splitter: 108, lhs: Group(161), rhs: Group(165) },
  Group { splitter: 0, lhs: Piece(170), rhs: Piece(171) },
  Group { splitter: 64, lhs: Piece(169), rhs: Group(167) },
  Group { splitter: 109, lhs: Piece(172), rhs: Piece(173) },
  Group { splitter: 110, lhs: Group(168), rhs: Group(169) },
  Group { splitter: 105, lhs: Group(170), rhs: Piece(174) },
  Group { splitter: 108, lhs: Group(171), rhs: Piece(175) },
  Group { splitter: 111, lhs: Group(166), rhs: Group(172) },
  Group { splitter: 112, lhs: Piece(177), rhs: Piece(178) },
  Group { splitter: 113, lhs: Piece(176), rhs: Group(174) },
  Group { splitter: 114, lhs: Piece(179), rhs: Piece(180) },
  Group { splitter: 115, lhs: Group(175), rhs: Group(176) },
  Group { splitter: 108, lhs: Group(177), rhs: Piece(181) },
  Group { splitter: 103, lhs: Group(178), rhs: Piece(182) },
  Group { splitter: 116, lhs: Group(173), rhs: Group(179) },
  Group { splitter: 117, lhs: Group(150), rhs: Group(180) },
];

#[derive(Debug, Default, Clone)]
struct NearAxis {
  dist: f32,
  pos: Point,
}

pub struct HexaminxCreator {
  groove: Vec<f32>,
  normals: Vec<Point>,
  axis_pos: RefCell<Vec<(f32, Point)>>,
  axis_neg: RefCell<Vec<(f32, Point)>>,
}

pub fn sqr(x: f32) -> f32 {
  x * x
}

fn apply_indices<T: Copy>(v: &mut Vec<T>, indices: &[usize]) {
  for (new_index, &old_index) in indices.iter().enumerate() {
    v[new_index] = v[old_index];
  }
  v.truncate(indices.len());
}

impl HexaminxCreator {
  pub fn new() -> Self {
    let axis_pos = RefCell::new(Vec::new());
    let axis_neg = RefCell::new(Vec::new());

    let basic_angle = 2.0 * PI / 5.0;
    let basic_cos = basic_angle.cos();
    let edge = basic_cos / (1.0 - basic_cos);
    let minimal_angle = ((edge * 2.0 + 1.0) / 3.0).sqrt().acos();

    let ca2 = (basic_angle * 2.0 / 3.0).cos();
    let cba = -sqr(ca2) + (1.0 - sqr(ca2)) * edge;
    let maximal_angle = ((edge - cba) / (1.0 - cba)).sqrt().acos();
    let sphere_r = 4.0 / (maximal_angle - minimal_angle);


    let normals = [
      Point { x: -1.0, y: -1.0, z: -1.0 },
      Point { x: -1.0, y: -1.0, z: 1.0 },
      Point { x: -1.0, y: 1.0, z: -1.0 },
      Point { x: -1.0, y: 1.0, z: 1.0 },
      Point { x: 1.0, y: -1.0, z: -1.0 },
      Point { x: 1.0, y: -1.0, z: 1.0 },
      Point { x: 1.0, y: 1.0, z: -1.0 },
      Point { x: 1.0, y: 1.0, z: 1.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();
    
/*
    let normals = [
      Point { x: 0.0, y: 0.0, z: -1.0 },
      Point { x: 0.0, y: 0.0, z: 1.0 },
      Point { x: 0.0, y: -1.0, z: 0.0 },
      Point { x: 0.0, y: 1.0, z: 0.0 },
      Point { x: -1.0, y: 0.0, z: 0.0 },
      Point { x: 1.0, y: 0.0, z: 0.0 },
    ]
    .into_iter()
    .map(Point::norm)
    .collect();*/

    let groove = vec![
      (maximal_angle - 0.0 / sphere_r).cos(),
      sphere_r + 0.2,
      (maximal_angle - 3.0 / sphere_r).cos(),
      sphere_r + 2.6,
      (maximal_angle + 1.0 / sphere_r).cos(),
    ];

    Self { groove, normals, axis_pos, axis_neg }
  }

  pub fn faces(&self) -> usize {
    self.normals.len()
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, self.faces())
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    0.6
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    1
  }

  pub fn get_name(&self, current_normal: usize) -> Option<String> {
    None
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    let n0 = self.normals[current_normal];
    let n1 = n0.any_perp().norm();
    let n2 = cross(n0, n1);

    let last_groove = self.groove[self.groove.len() - 2];
    let sz = last_groove + 2.2;
    let p = n0.scale(sz) + n1.scale(pos.x) + n2.scale(pos.y);
    (self.get_part_index_impl(p, current_normal) > 0) as PartIndex
  }

  pub fn get_quality() -> usize {
    128
  }

  pub fn get_size() -> f32 {
    180.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    let r = pos.len();

    if pos.y < 0.0 {
      // return 0;
    }

    let sphere_r = self.groove[1] - 2.2;

    if r < sphere_r {
      if r > sphere_r - 0.2 || r < sphere_r - 5.2 {
        return 0;
      }
      for &a in BASIC {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < 1.5 {
          return 0;
        }
      }
      return PIECES.len() as PartIndex;
    }

    let mut out_core = false;
    let last_groove = self.groove[self.groove.len() - 2];
    let sz = last_groove + 2.2;

     panic!("sphere_r={sphere_r}, sz={sz}");

    let mut n_dists = Vec::new();
    for i in 0..self.normals.len() {
      if i == current_normal {
        continue;
      }
      let d = sz - dot(pos, self.normals[i]);
      if current_normal < self.normals.len() && d < 1.0 {
        return 0;
      }
      if d < 0.0 {
        return 0;
      }
      n_dists.push(d);
    }

    n_dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let out_r = 1.0;
    if sqr(out_r - f32::min(n_dists[0], out_r))
      + sqr(out_r - f32::min(n_dists[1], out_r))
      + sqr(out_r - f32::min(n_dists[2], out_r))
      > sqr(out_r)
    {
      return 0;
    }

    let (mut shift_out, mut shift_in, inter) = get_groove(r, &self.groove, 0.03);
    let dr = (self.groove[1] - 0.2) - r;
    if dr > 0.0 {
      shift_out += dr * 0.003;
      shift_in += dr * 0.003;
    }

    let mut axis_pos = self.axis_pos.borrow_mut();
    let mut axis_neg = self.axis_neg.borrow_mut();

    axis_pos.clear();
    axis_neg.clear();

    let mut spiral = false;

    let mut match_axis = |a: Point, shift_in: f32, shift_out: f32| {
      let c = dot(pos, a) / r;
      let check_in = c - shift_in;
      if check_in > 0.0 {
        axis_pos.push((check_in, a));
        return 1;
      } else {
        let check_out = shift_out - c;
        if check_out > 0.0 {
          axis_neg.push((check_out, a));
          return -1;
        } else {
          return 0;
        }
      }
    };

    let mut index: PartIndex = 0;
    let mut current_group = GROUPS.len() - 1;
    loop {
      let g = &GROUPS[current_group];

      let next = match match_axis(SPLITTERS[g.splitter], shift_in, shift_out) {
        -1 => g.lhs,
        1 => g.rhs,
        _ => return 0,
      };
      match next {
        SubGroup::Group(g) => {
          current_group = g;
          continue;
        }
        SubGroup::Piece(p) => {
          apply_indices(&mut axis_pos, PIECES[p].bounds_pos);
          apply_indices(&mut axis_neg, PIECES[p].bounds_neg);
          index = p as PartIndex;
          break;
        }
      }
    }

    let mut thick = false;

    if axis_pos.len() == 1 && r < sz - 3.0 {
      let hole_r = 1.25;
      for (i, &a) in BASIC.iter().enumerate() {
        let c = dot(pos, a) / r;
        let s = cross(pos, a).len();
        if c > 0.0 && s < hole_r {
          return 0;
        }
      }
    }


    if !thick {
      if spiral {
        //  return 0;
      }

      let sr = if r > self.groove[self.groove.len() - 2] + 0.2 && axis_pos.len() == 3 && axis_neg.len() == 0 {
        0.11f32
      } else {
        0.03f32
      };

      let mut in_sr = |a, b, d| {
        let r = sr * d;
        if a < r && b < r && sqr(r - a) + sqr(r - b) > sqr(r) {
          return true;
        }
        false
      };

      if current_normal < self.normals.len() {
        let hole = 0.006;
        for a in axis_pos.iter_mut() {
          if a.0 < hole {
            return 0;
          }
          a.0 -= hole;
        }
        for a in axis_neg.iter_mut() {
          if a.0 < hole {
            return 0;
          }
          a.0 -= hole;
        }
      }

      axis_pos.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
      axis_neg.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

      if axis_pos.len() >= 2 {
        let d = dot(axis_pos[0].1, axis_pos[1].1);
        if in_sr(axis_pos[0].0, axis_pos[1].0, 1.0) {
          return 0;
        }
      }
      if axis_neg.len() >= 2 {
        let d = dot(axis_neg[0].1, axis_neg[1].1);
        if in_sr(axis_neg[0].0, axis_neg[1].0, 1.0) {
          return 0;
        }
      }
      if !inter && axis_pos.len() >= 1 && axis_neg.len() >= 1 {
        let d = dot(axis_pos[0].1, axis_neg[0].1);
        if d <= 0.06 {
          if axis_pos.len() >= 2 {
            let d = dot(axis_pos[1].1, axis_neg[0].1);
            if d >= 0.06 {
              if in_sr(axis_pos[1].0, axis_neg[0].0, 1.0) {
                return 0;
              }
            }
          }
          if axis_neg.len() >= 2 {
            let d = dot(axis_pos[0].1, axis_neg[1].1);
            if d >= 0.06 {
              if in_sr(axis_pos[0].0, axis_neg[1].0, 1.0) {
                return 0;
              }
            }
          }
        } else {
          if in_sr(axis_pos[0].0, axis_neg[0].0, 1.0) {
            return 0;
          }
        }
      }
    }

    return index;
  }
}
