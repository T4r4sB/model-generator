use common::common_for_twisty_puzzles::*;
use common::model::*;
use common::points3d::*;
use common::solid::*;
use num::Float;

pub fn sqr(x: f32) -> f32 {
  x * x
}

pub struct BeggarCreator {}

impl BeggarCreator {
  pub fn new() -> Self {
    Self {}
  }

  pub fn faces(&self) -> usize {
    0
  }

  pub fn get_part_index(&self, pos: Point) -> PartIndex {
    self.get_part_index_impl(pos, 1)
  }

  pub fn get_height(&self, current_normal: usize) -> f32 {
    0.6
  }

  pub fn get_count(&self, current_normal: usize) -> usize {
    1
  }

  pub fn get_name(&self, current_normal: usize) -> Option<&str> {
    None
  }

  pub fn get_sticker_index(&self, pos: crate::points2d::Point, current_normal: usize) -> PartIndex {
    0
  }

  pub fn get_quality() -> usize {
    128
  }

  pub fn get_size() -> f32 {
    120.0
  }

  pub fn get_part_index_impl(&self, pos: Point, current_normal: usize) -> PartIndex {
    if pos.x.abs() > 44.999 || pos.y.abs() > 44.999 || pos.z.abs() > 44.999 {
      return 0;
    }

    let p_min = -25.0;
    let p_max = 30.0;
    let split = 10.0;

    let match_p =
      |pos: Point, max: [f32; 3], data: [i8; 9], colors: [i8; 3]| -> Option<PartIndex> {
        let bx = [max[0] - (p_max - p_min), -split, split, max[0]];
        let by = [max[1] - (p_max - p_min), -split, split, max[1]];
        if pos.z < split || pos.z > max[2] {
          return None;
        }

        if pos.z < max[2] - 1.0 {
          let r = if pos.z > split + 4.0 { 3.2 } else { 1.5 };
          if sqr(pos.x) + sqr(pos.y) < sqr(r) {
            return None;
          }
        }

        let r = 1.7;
        let w = 2.5;
        let mut rdx = f32::NEG_INFINITY;
        let mut rdy = f32::NEG_INFINITY;
        let mut rdiag = f32::NEG_INFINITY;
        let mut dz = f32::min(pos.z - split, max[2] - pos.z);
        let mut r_to_sq = f32::INFINITY;

        let mut validate = |mut dx, mut dy, mut dz, rdx: &mut _, rdy: &mut _| {
          let sharpx = pos.x > -split && pos.x < split && pos.z < split + w + 1.0;
          let sharpy = pos.y > -split && pos.y < split && pos.z < split + w + 1.0;
          *rdx = f32::max(*rdx, dx);
          *rdy = f32::max(*rdy, dy);
          if sharpx && dx + dz < w || sharpy && dy + dz < w {
            return false;
          }
          sqr(r - f32::min(dx, r)) + sqr(r - f32::min(dy, r)) + sqr(r - f32::min(dz, r)) < sqr(r)
        };

        let mut result = false;

        for j in 0..3 {
          for i in 0..3 {
            let d = i + (2 - j) * 3;
            if data[d] == 0 {
              continue;
            }

            r_to_sq = f32::min(
              r_to_sq,
              f32::max(
                (pos.x - (bx[i] + bx[i + 1]) * 0.5).abs(),
                (pos.y - (by[j] + by[j + 1]) * 0.5).abs(),
              ),
            );

            if pos.x > bx[i] && pos.x < bx[i + 1] && pos.y > by[j] && pos.y < by[j + 1] {
              let dx = f32::min(pos.x - bx[i], bx[i + 1] - pos.x);
              let dy = f32::min(pos.y - by[j], by[j + 1] - pos.y);
              result |= validate(dx, dy, dz, &mut rdx, &mut rdy);
            }

            if i < 2 {
              if data[d + 1] != 0 {
                if pos.x > bx[i] && pos.x < bx[i + 2] && pos.y > by[j] && pos.y < by[j + 1] {
                  let dx = f32::min(pos.x - bx[i], bx[i + 2] - pos.x);
                  let dy = f32::min(pos.y - by[j], by[j + 1] - pos.y);
                  result |= validate(dx, dy, dz, &mut rdx, &mut rdy);
                }
              }
            }

            if j < 2 {
              if data[d - 3] != 0 {
                if pos.x > bx[i] && pos.x < bx[i + 1] && pos.y > by[j] && pos.y < by[j + 2] {
                  let dx = f32::min(pos.x - bx[i], bx[i + 1] - pos.x);
                  let dy = f32::min(pos.y - by[j], by[j + 2] - pos.y);
                  result |= validate(dx, dy, dz, &mut rdx, &mut rdy);
                }
              }
            }

            let match_diag = |dw, d1, d3, rdiag: &mut _| -> bool {
              let mut x1 = (w - dw) * 0.5.sqrt();
              let mut x2 = (w + dw) * 0.5.sqrt();

              if data[d1] != 0 {
                x1 = f32::max(x1, r);
              }
              if data[d3] != 0 {
                x2 = f32::max(x2, r);
              }
              let dx = f32::min(x1, x2);
              *rdiag = f32::max(*rdiag, dx);

              return dx > 0.0 && sqr(r - f32::min(r, dx)) + sqr(r - f32::min(dz, r)) < sqr(r);
            };

            if (pos.x - bx[i + 1]).abs() < r + w + w && (pos.y - by[j + 1]).abs() < r + w + w {
              if i < 2 && j < 2 {
                if data[d - 2] != 0 {
                  result |=
                    match_diag((pos.x - pos.y) - (bx[i + 1] - by[j + 1]), d + 1, d - 3, &mut rdiag);
                }
              }
            }

            if (pos.x - bx[i]).abs() < r + w + w && (pos.y - by[j + 1]).abs() < r + w + w {
              if i > 0 && j < 2 {
                if data[d - 4] != 0 {
                  result |=
                    match_diag((bx[i] + by[j + 1]) - (pos.x + pos.y), d - 1, d - 3, &mut rdiag);
                }
              }
            }
          }
        }

        if result {
          if r_to_sq < 4.15 && pos.z > max[2] - 5.2 {
            if r_to_sq < 4.0 && pos.z > max[2] - 5.0 || dz < 2.0 {
              return Some((7 - colors[2]) as PartIndex * 10 + 2);
            } else {
              return None;
            }
          }

          if rdx < 2.2 || rdy < 2.2 || dz < 2.2 {
            if dz >= 2.2 && rdiag > 0.0 {
              return Some(0);
            } else if dz >= 2.0 && rdiag > -0.2 {
              return None;
            } else if rdiag > 0.0 {
              rdx = f32::INFINITY;
              rdy = f32::INFINITY;
            }

            let getv = |c, v: f32| -> PartIndex {
              for i in 0..3 {
                if v < (bx[i] + bx[i + 1]) * 0.5 {
                  return colors[c] as PartIndex * 10 + i as PartIndex;
                } else if v < bx[i + 1] {
                  return (7 - colors[c] as PartIndex) * 10 + i as PartIndex;
                }
              }
              return (7 - colors[c]) as PartIndex * 10 + 2;
            };

            if rdx < 2.0 || rdy < 2.0 || dz < 2.0 {
              if rdx < rdy - 0.3 && rdx < dz - 0.3 {
                return if rdiag > -0.2 { None } else { Some(getv(0, pos.x)) };
              } else if rdy < rdx - 0.3 && rdy < dz - 0.3 {
                return if rdiag > -0.2 { None } else { Some(getv(1, pos.y)) };
              } else if dz < rdx - 0.3 && dz < rdy - 0.3 {
                return Some(getv(2, pos.z));
              } else {
                return None;
              }
            } else {
              return None;
            }
          }
          return Some(0);
        }

        return None;
      };

    if pos.y < 0.0 {
      //  return 0;
    }

    let p1;
    let p2;
    let p3;
    let p4;
    let p5;
    let p6;

    let by_color = false;

    if by_color {
      p1 = Point { x: -pos.x, y: -pos.y, z: pos.z };
      p2 = Point { x: pos.y, y: -pos.z, z: -pos.x };
      p3 = Point { x: -pos.z, y: pos.x, z: -pos.y };
      p4 = Point { x: -pos.z, y: pos.y, z: pos.x };
      p5 = Point { x: -pos.z, y: -pos.x, z: pos.y };
      p6 = Point { x: -pos.y, y: -pos.x, z: -pos.z };
    } else {
      p1 = Point { x: pos.x, y: pos.y, z: pos.z };
      p2 = Point { x: -pos.y, y: pos.z, z: -pos.x };
      p3 = Point { x: pos.x, y: pos.z, z: -pos.y };
      p4 = Point { x: pos.y, y: pos.z, z: pos.x };
      p5 = Point { x: -pos.x, y: pos.z, z: pos.y };
      p6 = Point { x: pos.x, y: -pos.y, z: -pos.z };
    }

    if let Some(f) = match_p(p1, [p_max, p_max, -p_min], [1, 0, 0, 0, 1, 0, 0, 0, 0], [6, 2, 4]) {
      return 1 + f * 10;
    }
    if let Some(f) = match_p(p2, [-p_min, -p_min, -p_min], [0, 0, 0, 0, 1, 0, 0, 0, 1], [5, 3, 6]) {
      return 2 + f * 10;
    }
    if let Some(f) = match_p(p3, [p_max, -p_min, -p_min], [0, 1, 0, 0, 1, 0, 0, 0, 0], [3, 1, 2]) {
      return 3 + f * 10;
    }
    if let Some(f) = match_p(p4, [p_max, -p_min, p_max], [1, 0, 0, 1, 1, 1, 0, 0, 0], [3, 5, 1]) {
      return 4 + f * 10;
    }
    if let Some(f) = match_p(p5, [-p_min, -p_min, p_max], [0, 0, 0, 0, 1, 0, 0, 0, 1], [3, 6, 5]) {
      return 5 + f * 10;
    }
    if let Some(f) = match_p(p6, [p_max, -p_min, p_max], [0, 1, 1, 0, 1, 0, 0, 0, 0], [2, 6, 3]) {
      return 6 + f * 10;
    }

    let split = 10.0;
    let dx = f32::max(0.0, split - pos.x.abs());
    let dy = f32::max(0.0, split - pos.y.abs());
    let dz = f32::max(0.0, split - pos.z.abs());
    if sqr(2.0 - f32::min(2.0, dx)) + sqr(2.0 - f32::min(2.0, dy)) + sqr(2.0 - f32::min(2.0, dz))
      < sqr(2.0)
    {
      if sqr(pos.x) + sqr(pos.y) > sqr(1.25)
        && sqr(pos.y) + sqr(pos.z) > sqr(1.25)
        && sqr(pos.z) + sqr(pos.x) > sqr(1.25)
      {
        return 7;
      }
    }

    return 0;
  }
}
