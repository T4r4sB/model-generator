use crate::points3d::*;

pub const PI: f32 = std::f32::consts::PI;

fn sqr(x: f32) -> f32 {
  x * x
}

pub fn get_groove(r: f32, info: &[f32], bevel: f32) -> (f32, f32, bool) {
  assert!(info.len() % 2 == 1);
  let mut i = 0;

  const HOLE_W: f32 = 0.2;
  const BEVEL_W: f32 = 0.2;
  const BEVEL_D: f32 = BEVEL_W + HOLE_W;

  while i + 2 < info.len() {
    let v = info[i];
    let b = info[i + 1];
    let nv = info[i + 2];
    if v < nv {
      if r < b - BEVEL_D {
        return (v, v, false);
      } else if r < b - HOLE_W {
        return (v, v + (r - (b - BEVEL_D)) * bevel / BEVEL_W, false);
      } else if r < b + HOLE_W {
        return (v, nv, true);
      } else if r < b + BEVEL_D {
        return (nv - ((b + BEVEL_D) - r) * bevel / BEVEL_W, nv, false);
      } else {
        i += 2;
      }
    } else {
      if r < b - BEVEL_D {
        return (v, v, false);
      } else if r < b - HOLE_W {
        return (v - (r - (b - BEVEL_D)) * bevel / BEVEL_W, v, false);
      } else if r < b + HOLE_W {
        return (nv, v, true);
      } else if r < b + BEVEL_D {
        return (nv, nv + ((b + BEVEL_D) - r) * bevel / BEVEL_W, false);
      } else {
        i += 2;
      }
    }
  }

  (info[i], info[i], false)
}

pub fn get_diag_groove(r: f32, info: &[f32]) -> (f32, f32, bool) {
  assert!(info.len() % 2 == 0);
  assert!(!info.is_empty());

  let mut i = 0;

  const HOLE_W: f32 = 0.1;

  if r < info[0] {
    return (info[1], info[1], false);
  }

  while i + 3 < info.len() {
    let b = info[i];
    let v = info[i + 1];
    let nb = info[i + 2];
    let nv = info[i + 3];
    if v < nv {
      if r > nb {
        i += 2;
      } else {
        let diag = (nv - v) / (nb - b - HOLE_W);
        if r < b + HOLE_W {
          return (v, v + diag * (r - b), false);
        } else if r < nb - HOLE_W {
          return (v + diag * (r - b - HOLE_W), v + diag * (r - b), false);
        } else {
          return (v + diag * (r - b - HOLE_W), nv, false);
        }
      }
    } else {
      if r > nb {
        i += 2;
      } else {
        let diag = (nv - v) / (nb - b - HOLE_W);
        if r < b + HOLE_W {
          return (v + diag * (r - b), v, false);
        } else if r < nb - HOLE_W {
          return (v + diag * (r - b), v + diag * (r - b - HOLE_W), false);
        } else {
          return (nv, v + diag * (r - b - HOLE_W), false);
        }
      }
    }
  }

  (info[i + 1], info[i + 1], false)
}

pub fn in_spiral(
  pt: Point,
  n: Point,
  n1: Point,
  n2: Point,
  d: f32,
  depth: f32,
  width: f32,
) -> bool {
  if d > depth {
    return false;
  }

  let r = pt.len();

  let inv_r = r.recip();
  let x = dot(pt, n1) * inv_r;
  let y = dot(pt, n2) * inv_r;
  let a = f32::atan2(y, x);
  let spiral_f = a / PI * 12.0 - r * 0.2;
  (spiral_f - spiral_f.floor() - 0.5).abs() < (1.0 - d / depth) * width
}

pub fn spheric_circle_r_cos(cosa: f32, cosb: f32, cosc: f32) -> f32 {
  ((sqr(cosa) + sqr(cosb) + sqr(cosc) - 1.0 - 2.0 * cosa * cosb * cosc)
    / (sqr(cosa)
      + sqr(cosb)
      + sqr(cosc)
      + 2.0 * (cosa + cosb + cosc - cosa * cosb - cosb * cosc - cosc * cosa)
      - 3.0))
    .sqrt()
}

pub fn half_cos(cosa: f32) -> f32 {
  ((cosa + 1.0) * 0.5).sqrt()
}

pub fn find_square(n1: Point, n2: Point) -> Point {
  let mid = (n1 + n2).scale(0.5);
  let dn = n1 - mid;
  mid + cross(mid, dn).scale(mid.len().recip())
}

pub struct Basis(pub Point, pub Point, pub Point);

pub fn to_basis(a: Point) -> Basis {
  let a2 = a.any_perp().norm();
  let a3 = cross(a, a2);
  Basis(a, a2, a3)
}
