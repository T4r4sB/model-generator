use common::points2d::*;
use common::solid::*;
use num::Float;
use png::InterlaceInfo::Adam7;

//use common::slots_and_holes::*;

const ERR: f32 = 0.1;

const PI: f32 = std::f32::consts::PI;

fn sqr(x: f32) -> f32 {
  x * x
}

enum GearCouple {
  Inner3Euro,
  Inner6,
  Adapter7Bolt,
}

enum GearInfill {
  None,
  Grid,
  Holes,
  Spiral,
}

enum GearInfillImpl {
  None,
  Grid(InfillGridData),
  Holes(InfillHolesData),
  Spiral(InfillSpiralData),
}

struct GearContour {
  tc: usize,
  pin_r: f32,
  edge: f32,
  pin_c_r: f32,
  pairing_r: f32,
  circles: Vec<Point>,
  couple: GearCouple,
  infill: GearInfillImpl,
}

impl GearContour {
  fn new(tc: usize, couple: GearCouple, infill: GearInfill) -> Self {
    let pin_r = 8.51 * 0.5;
    let edge = 12.7;
    let pin_c_r = edge * 0.5 / (PI / tc as f32).sin();
    let circles = (0..tc)
      .map(|i| {
        let a = i as f32 * 2.0 * PI / tc as f32;
        let (s, c) = a.sin_cos();
        Point { x: c * pin_c_r, y: s * pin_c_r }
      })
      .collect();

    let pairing_r = (sqr(pin_c_r) - sqr(edge * 0.5) + sqr(edge * 0.5 - pin_r)).sqrt();

    let infill = match infill {
      GearInfill::None => GearInfillImpl::None,
      GearInfill::Grid => GearInfillImpl::Grid(InfillGridData::new(tc, Self::get_minr(&couple))),
      GearInfill::Holes => GearInfillImpl::Holes(InfillHolesData::new(tc, Self::get_minr(&couple))),
      GearInfill::Spiral => {
        GearInfillImpl::Spiral(InfillSpiralData::new(tc, Self::get_minr(&couple)))
      }
    };

    Self { tc, pin_r, edge, pin_c_r, pairing_r, circles, couple, infill }
  }

  fn get_minr(couple: &GearCouple) -> f32 {
    match couple {
      GearCouple::Inner3Euro => DriverHole3EuroData::minr(),
      GearCouple::Inner6 => DriverHole6Data::minr(),
      GearCouple::Adapter7Bolt => Adapter7BoltsData::minr(),
    }
  }

  fn contains(&self, pos: Point) -> bool {
    let r = pos.len();
    if r > self.pin_c_r + self.pin_r {
      return false;
    } else if r > self.pairing_r {
      let mut match_c = 0;
      for &c in &self.circles {
        if (pos - c).len() < self.edge - self.pin_r {
          match_c += 1;
        }
      }
      if match_c < 2 {
        return false;
      }
    } else {
      for &c in &self.circles {
        if (pos - c).len() < self.pin_r {
          return false;
        }
      }
    }
    return true;
  }

  fn inner_r(&self) -> f32 {
    self.pin_c_r - self.pin_r - 5.0
  }
}

struct DriverHole3EuroData {
  p0: Point,
  p1: Point,
  p2: Point,
}

impl DriverHole3EuroData {
  fn new() -> Self {
    let r0 = 34.9 * 0.5;
    let cr = 3.0;
    let cd = 1.5;
    let r1 = r0 + cr - cd;
    let sq3 = 0.75.sqrt();

    let p0 = Point { x: r1, y: 0.0 };
    let p1 = Point { x: -r1 * 0.5, y: r1 * sq3 };
    let p2 = Point { x: -r1 * 0.5, y: -r1 * sq3 };
    Self { p0, p1, p2 }
  }

  fn minr() -> f32 {
    21.5
  }

  fn inside(&self, pos: Point) -> bool {
    let r0 = 34.9 * 0.5;
    let cr = 3.0;
    pos.len() < r0
      && (pos - self.p0).len() > cr
      && (pos - self.p1).len() > cr
      && (pos - self.p2).len() > cr
  }
}

struct DriverHole6Data {
  p0: Point,
  p1: Point,
  p2: Point,
}

impl DriverHole6Data {
  fn new() -> Self {
    let r0 = 39.8 * 0.5;
    let cr = 3.1366;
    let cd = 1.4;
    let r1 = r0 + cr - cd;
    let sq3 = 0.75.sqrt();

    let p0 = Point { x: r1, y: 0.0 };
    let p1 = Point { x: -r1 * 0.5, y: r1 * sq3 };
    let p2 = Point { x: -r1 * 0.5, y: -r1 * sq3 };
    Self { p0, p1, p2 }
  }

  fn minr() -> f32 {
    25.0
  }

  fn inside(&self, pos: Point) -> bool {
    let r0 = 39.8 * 0.5;
    let cr = 3.1366;

    pos.len() < r0
      && (pos - self.p0).len() > cr
      && (pos - self.p1).len() > cr
      && (pos - self.p2).len() > cr
      && (pos + self.p0).len() > cr
      && (pos + self.p1).len() > cr
      && (pos + self.p2).len() > cr
  }
}

struct Adapter7BoltsData {
  bolt_holes: [Point; 7],
  bolt_holes_in: [Point; 7],
}

impl Adapter7BoltsData {
  fn new() -> Self {
    let bolt_r = 65.0 * 0.5;
    let bolt_in_r = 18.95;

    let bolt_holes = std::array::from_fn(|i| {
      Point::from_angle(i as f32 * 2.0 * PI / 7.0).scale(bolt_r)
      // r_in: 2.5, r_out: 5.5
    });

    let bolt_holes_in = std::array::from_fn(|i| {
      Point::from_angle(i as f32 * 2.0 * PI / 7.0).scale(bolt_in_r)
      // r_in: 0.0, r_out: 3.0
    });

    Self { bolt_holes, bolt_holes_in }
  }

  fn minr() -> f32 {
    40.0
  }

  fn inside(&self, pos: Point) -> bool {
    let bolt_r = 65.0 * 0.5;
    for &bh in &self.bolt_holes {
      let l = (pos - bh).len();
      if l < 2.5 + ERR {
        return true;
      }
      if l < 5.5 {
        return false;
      }
    }
    pos.len() < bolt_r - 1.0
  }
}

struct InfillGridData {
  root_in: Vec<Point>,
  root_out: Vec<Point>,
  step: usize,
  maxr: f32,
  minr: f32,
  buf: std::cell::RefCell<Vec<f32>>,
}

impl InfillGridData {
  fn new(tc: usize, minr: f32) -> Self {
    let maxr = tc as f32 * 12.7 / PI / 2.0 - 10.0;
    let round;
    let step;

    if minr > maxr - 1.0 {
      round = 0;
      step = 0;
    } else {
      round = tc / 3 - 1;
      let ma = (minr / maxr).acos();
      println!("pure ma={ma}, step = {}", ma / (PI * 2.0 / round as f32) * 2.0);
      step = (ma / (PI * 2.0 / round as f32) * 2.0 + 0.5).floor() as usize;
    }

    let cnt = round + step;

    let root_out =
      (0..cnt).map(|i| Point::from_angle(i as f32 / round as f32 * 2.0 * PI).scale(maxr)).collect();

    let root_in = (0..cnt)
      .map(|i| {
        Point::from_angle((i as f32 + step as f32 * 0.5) / round as f32 * 2.0 * PI).scale(minr)
      })
      .collect();

    let mut buf = Vec::<f32>::new();
    buf.resize(round * 2 + 2, 0.0);
    let buf = std::cell::RefCell::new(buf);

    Self { root_in, root_out, step, maxr, minr, buf }
  }

  fn inside(&self, pos: Point) -> bool {
    let r = pos.len();
    let mut buf = self.buf.borrow_mut();
    buf[0] = r - self.minr;
    buf[1] = self.maxr - r;
    for i in 0..self.root_in.len() - self.step {
      buf[i * 2 + 2] = dist_pl(pos, self.root_out[i], self.root_in[i]) - 3.0;
      buf[i * 2 + 3] = dist_pl(pos, self.root_in[i], self.root_out[i + self.step]) - 3.0;
    }
    buf.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    if buf[0] > 0.0 {
      let rr = 2.0;
      if sqr(rr - f32::min(buf[0], rr)) + sqr(rr - f32::min(buf[1], rr)) < sqr(rr) {
        return true;
      }
    }
    return false;
  }
}

struct InfillHolesData {
  roots: Vec<(Point, f32)>,
}

impl InfillHolesData {
  fn new(tc: usize, minr: f32) -> Self {
    let maxr = tc as f32 * 12.7 / PI / 2.0 - 8.0;
    let maxhr = f32::min(5.0, (maxr - minr) * 0.5);
    let minhr = 1.5;

    let cnt_in_row;
    if maxhr < minhr {
      cnt_in_row = 0;
    } else {
      cnt_in_row = (tc as f32 / (minhr + maxhr) * 2.0) as usize;
    }

    let minhc = minr + minhr;
    let maxhc = maxr - maxhr;

    let groups = 3 + (std::cmp::max(tc, 21) - 21) / 2;

    let roots = (0..cnt_in_row * groups)
      .map(|i| {
        let sp = i / cnt_in_row;
        let stage = (i % cnt_in_row) as f32 / (cnt_in_row - 1) as f32;
        let stage = (stage + stage * stage * 0.4) / 1.4;
        let hr = minhr + (maxhr - minhr) * stage;
        let cr = minhc + (maxhc - minhc) * stage;
        let a0 = sp as f32 * 2.0 * PI / groups as f32;
        let a1 = a0 + PI * 2.0 / 3.0 - (maxhr + maxhr) / minr;
        let a = a0 + (a1 - a0) * stage;
        let p = Point::from_angle(a).scale(cr);
        (p, hr)
      })
      .collect();

    Self { roots }
  }

  fn inside(&self, pos: Point) -> bool {
    for r in &self.roots {
      if (pos - r.0).len() < r.1 {
        return true;
      }
    }
    false
  }
}

struct InfillSpiralData {
  chain: [Point; 4],
}

impl InfillSpiralData {
  fn new(tc: usize, minr: f32) -> Self {
    let maxr = tc as f32 * 12.7 / PI / 2.0 - 20.0;
    let ba = PI * 2.0 / 7.0 * 30.0;
    let chain = [
      Point { x: minr + 5.0, y: -ba * 0.5 },
      Point { x: minr + 5.0, y: 0.0 },
      Point { x: maxr + 5.0, y: 0.0 },
      Point { x: maxr + 5.0, y: ba * 0.5 },
    ];
    Self { chain }
  }

  fn inside(&self, pos: Point) -> bool {
    let ba = PI * 2.0 / 7.0;
    let a = f32::atan2(pos.y, pos.x) - ba * 3.5;
    let r = pos.len();
    for t in 0..=7 {
      let polar = Point { x: r, y: (a + ba * t as f32) * 30.0 };
      let mut pp0 = f32::INFINITY;
      let mut pp1 = f32::INFINITY;
      for i in 1..self.chain.len() {
        let c0 = self.chain[i - 1];
        let c1 = self.chain[i];

        let pp = dist_pl(polar, c0, c1) - 2.5;
        if pp < 0.0 {
          return true;
        }
        let pp = if dot(c1 - c0, polar - c0) > 0.0 && dot(c0 - c1, polar - c1) > 0.0 {
          pp
        } else {
          f32::INFINITY
        };

        if pp < pp0 {
          pp1 = pp0;
          pp0 = pp;
        } else if pp < pp1 {
          pp1 = pp;
        }
      }
      let rr = 10.0;
      if pp0 < rr && pp1 < rr && sqr(rr - pp0) + sqr(rr - pp1) > sqr(rr) {
        return true;
      }
    }

    false
  }
}

pub struct ChaingearCreator {
  gears: Vec<GearContour>,
  h3e: DriverHole3EuroData,
  h6: DriverHole6Data,
  a7b: Adapter7BoltsData,
}

impl ChaingearCreator {
  pub fn new() -> Self {
    let gears = vec![
      GearContour::new(27, GearCouple::Inner3Euro, GearInfill::Spiral),
      GearContour::new(23, GearCouple::Inner3Euro, GearInfill::Grid),
    ];

    let h3e = DriverHole3EuroData::new();
    let h6 = DriverHole6Data::new();
    let a7b = Adapter7BoltsData::new();

    Self { gears, h3e, h6, a7b }
  }

  pub fn get_count(&self, part_index: usize) -> usize {
    1
  }

  pub fn aabb(&self, part_index: usize) -> Option<AABB> {
    let gear = &self.gears[part_index];
    Some(AABB::around_zero(gear.pin_c_r + gear.pin_r + 0.01))
  }

  pub fn faces(&self) -> usize {
    self.gears.len()
  }

  pub fn get_height(&self, part_index: usize) -> f32 {
    3.0
  }

  pub fn get_quality() -> usize {
    1
  }

  pub fn get_size() -> f32 {
    100.0
  }

  pub fn get_name(&self, part_index: usize) -> Option<String> {
    let g = &self.gears[part_index];
    let desc = match g.couple {
      GearCouple::Inner3Euro => format!("{}-rear-3pins-euro", g.tc),
      GearCouple::Inner6 => format!("{}-rear-6pins", g.tc),
      GearCouple::Adapter7Bolt => format!("{}-rear-adater7bolts", g.tc),
    };
    Some(desc)
  }

  pub fn get_sticker_index(&self, pos: Point, part_index: usize) -> PartIndex {
    let r = pos.len();
    if part_index < self.gears.len() {
      let g = &self.gears[part_index];

      let in_couple = match &g.couple {
        GearCouple::Inner3Euro => self.h3e.inside(pos),
        GearCouple::Inner6 => self.h6.inside(pos),
        GearCouple::Adapter7Bolt => self.a7b.inside(pos),
      };

      if in_couple {
        return 0;
      }

      let in_infill = match &g.infill {
        GearInfillImpl::None => false,
        GearInfillImpl::Grid(infill_grid_data) => infill_grid_data.inside(pos),
        GearInfillImpl::Holes(infill_holes_data) => infill_holes_data.inside(pos),
        GearInfillImpl::Spiral(infill_spiral_data) => infill_spiral_data.inside(pos),
      };

      if in_infill {
        return 0;
      }

      /*

      } else if part_index < 2 {

      } else {
        for i in 0..3 {
          let angle = f32::atan2(pos.y, pos.x) - i as f32 * 2.0 * PI / 3.0;
          let angle = angle.rem_euclid(2.0 * PI);
          let r = pos.len();
          let ak = 30.0;
          let a = angle * ak;

          let pp = Point { x: r, y: a };
          let pp1 = Point { x: 28.5, y: 0.6 * ak };
          let pp2 = Point { x: 28.5, y: 1.4 * ak };
          let pp3 = Point { x: 40.5, y: 2.6 * ak };
          let pp4 = Point { x: 40.5, y: 3.4 * ak };
          if dist_pl(pp, pp1, pp2) < sqr(1.0)
            || dist_pl(pp, pp2, pp3) < sqr(1.0)
            || dist_pl(pp, pp3, pp4) < sqr(1.0)
          {
            return 0;
          }
        }
      }
      */

      return g.contains(pos) as PartIndex;
    }

    0
  }

  pub fn get_part_index(&self, pos: common::points3d::Point) -> PartIndex {
    0
  }
}
