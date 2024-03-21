use crate::points2d::*;
use crate::solid::*;

pub fn sqr(x: f32) -> f32 {
  x * x
}

struct Slot {
  n1: Point,
  c1: f32,
  n2: Point,
  c2: f32,
}

impl Slot {
  fn new(pt1: Point, pt2: Point, width: f32) -> Self {
    let delta = pt2 - pt1;
    let n1 = delta.perp().norm().scale(width.recip());
    let c1 = 0.5 - dot(pt1, n1);
    let n2 = delta.scale(delta.sqr_len().recip());
    let c2 = -dot(n2, pt1);
    Self { n1, c1, n2, c2 }
  }

  fn contains(&self, pt: Point) -> bool {
    let f1 = dot(pt, self.n1) + self.c1;
    if f1 < 0.0 || f1 > 1.0 {
      return false;
    }
    let f2 = dot(pt, self.n2) + self.c2;
    if f2 < 0.0 || f2 > 1.0 {
      return false;
    }
    true
  }
}

struct Corner {
  position: Point,
  radius: f32,
}

struct Contour {
  corners: Vec<Corner>,
}

impl Contour {
  fn contains(&self, pt: Point) -> bool {
    if self.corners.len() < 3 {
      return false;
    }
    let mut prev = self.corners.last().unwrap().position - pt;
    let mut c_in = 0;
    let mut c_out = 0;
    for c in &self.corners {
      let cur = c.position - pt;
      if prev.y >= 0.0 && cur.y < 0.0 && cross(prev, cur) <= 0.0 {
        c_in += 1;
      } else if prev.y < 0.0 && cur.y >= 0.0 && cross(prev, cur) > 0.0 {
        c_out += 1;
      }
      prev = cur;
    }
    c_in < c_out
  }
}

struct Circle {
  position: Point,
  radius: f32,
}

impl Circle {
  fn contains(&self, pt: Point) -> bool {
    (pt - self.position).sqr_len() < sqr(self.radius)
  }
}

struct Figure {
  outer: Contour,
  inner: Vec<Contour>,
  slots: Vec<Slot>,
  holes: Vec<Circle>,
}

impl Figure {
  fn contains(&self, pt: Point) -> bool {
    self.outer.contains(pt)
      && self.inner.iter().all(|i| !i.contains(pt))
      && self.slots.iter().all(|s| !s.contains(pt))
      && self.holes.iter().all(|h| !h.contains(pt))
  }
}

pub struct BrakeHandleCreator {
  figures: Vec<Figure>,
}

impl BrakeHandleCreator {
  pub fn new() -> Self {
    const HOLE_DISP: f32 = 3.0;
    let figures = vec![
      Figure {
        outer: Contour {
          corners: vec![
            Corner { position: Point { x: -42.6, y: -3.2 }, radius: 0.0 },
            Corner { position: Point { x: 0.0, y: -60.0 }, radius: 0.0 },
            Corner { position: Point { x: 0.0, y: 0.0 }, radius: 0.0 },
            Corner { position: Point { x: 8.0, y: 0.0 }, radius: 0.0 },
            Corner { position: Point { x: 8.0, y: 20.0 }, radius: 0.0 },
            Corner { position: Point { x: -6.0, y: 46.0 }, radius: 0.0 },
            Corner { position: Point { x: -18.0, y: 46.0 }, radius: 0.0 },
            Corner { position: Point { x: -18.0, y: 20.0 }, radius: 0.0 },
            Corner { position: Point { x: -26.6, y: 8.8 }, radius: 0.0 },
          ],
        },
        inner: vec![],
        slots: vec![
          Slot::new(Point { x: -17.0, y: 4.8 }, Point { x: -17.0, y: 15.2 }, 2.2),
          Slot::new(Point { x: -17.0, y: 30.8 }, Point { x: -17.0, y: 43.2 }, 2.2),
          Slot::new(Point { x: -9.0, y: 30.8 }, Point { x: -9.0, y: 43.2 }, 2.2),
          Slot::new(Point { x: -32.2, y: -1.0 }, Point { x: -21.8, y: -1.0 }, 2.2),
          Slot::new(Point { x: -39.4, y: -0.8 }, Point { x: -29.8, y: 6.4 }, 4.2),
        ],
        holes: vec![
          Circle { position: Point { x: -22.5, y: 4.5 }, radius: 2.6 },
          Circle { position: Point { x: 0.0, y: -60.0 }, radius: 60.0 },
          Circle { position: Point { x: HOLE_DISP, y: 7.0 }, radius: 2.6 },
          Circle { position: Point { x: HOLE_DISP, y: 16.0 }, radius: 2.6 },
          Circle { position: Point { x: -HOLE_DISP, y: 22.0 }, radius: 2.6 },
        ],
      },
      Figure {
        outer: Contour {
          corners: vec![
            Corner { position: Point { x: -8.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: -5.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: -5.0, y: -7.0 }, radius: 0.0 },
            Corner { position: Point { x: 5.0, y: -7.0 }, radius: 0.0 },
            Corner { position: Point { x: 5.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: 8.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: 8.0, y: 5.0 }, radius: 0.0 },
            Corner { position: Point { x: 5.0, y: 5.0 }, radius: 0.0 },
            Corner { position: Point { x: 5.0, y: 7.0 }, radius: 0.0 },
            Corner { position: Point { x: -5.0, y: 7.0 }, radius: 0.0 },
            Corner { position: Point { x: -5.0, y: 5.0 }, radius: 0.0 },
            Corner { position: Point { x: -8.0, y: 5.0 }, radius: 0.0 },
          ],
        },
        inner: vec![],
        slots: vec![],
        holes: vec![],
      },
      Figure {
        outer: Contour {
          corners: vec![
            Corner { position: Point { x: -8.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: -5.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: -5.0, y: -7.0 }, radius: 0.0 },
            Corner { position: Point { x: 5.0, y: -7.0 }, radius: 0.0 },
            Corner { position: Point { x: 5.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: 21.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: 21.0, y: -7.0 }, radius: 0.0 },
            Corner { position: Point { x: 33.0, y: -7.0 }, radius: 0.0 },
            Corner { position: Point { x: 33.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: 36.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: 36.0, y: 5.0 }, radius: 0.0 },
            Corner { position: Point { x: 33.0, y: 5.0 }, radius: 0.0 },
            Corner { position: Point { x: 33.0, y: 7.0 }, radius: 0.0 },
            Corner { position: Point { x: 21.0, y: 7.0 }, radius: 0.0 },
            Corner { position: Point { x: 21.0, y: 5.0 }, radius: 0.0 },
            Corner { position: Point { x: 5.0, y: 5.0 }, radius: 0.0 },
            Corner { position: Point { x: 5.0, y: 7.0 }, radius: 0.0 },
            Corner { position: Point { x: -5.0, y: 7.0 }, radius: 0.0 },
            Corner { position: Point { x: -5.0, y: 5.0 }, radius: 0.0 },
            Corner { position: Point { x: -8.0, y: 5.0 }, radius: 0.0 },
          ],
        },
        inner: vec![],
        slots: vec![],
        holes: vec![Circle { position: Point { x: 27.0, y: 0.0 }, radius: 5.1 }],
      },
      Figure {
        outer: Contour {
          corners: vec![
            Corner { position: Point { x: 18.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: 21.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: 21.0, y: -7.0 }, radius: 0.0 },
            Corner { position: Point { x: 33.0, y: -7.0 }, radius: 0.0 },
            Corner { position: Point { x: 33.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: 36.0, y: -5.0 }, radius: 0.0 },
            Corner { position: Point { x: 36.0, y: 5.0 }, radius: 0.0 },
            Corner { position: Point { x: 33.0, y: 5.0 }, radius: 0.0 },
            Corner { position: Point { x: 33.0, y: 7.0 }, radius: 0.0 },
            Corner { position: Point { x: 21.0, y: 7.0 }, radius: 0.0 },
            Corner { position: Point { x: 21.0, y: 5.0 }, radius: 0.0 },
            Corner { position: Point { x: 18.0, y: 5.0 }, radius: 0.0 },
          ],
        },
        inner: vec![],
        slots: vec![],
        holes: vec![Circle { position: Point { x: 27.0, y: 0.0 }, radius: 5.1 }],
      },
      Figure {
        outer: Contour {
          corners: vec![
            Corner { position: Point { x: -8.0, y: 28.0 }, radius: 0.0 },
            Corner { position: Point { x: -10.0, y: 26.0 }, radius: 0.0 },
            Corner { position: Point { x: -10.0, y: 22.0 }, radius: 0.0 },
            Corner { position: Point { x: -2.0, y: 4.0 }, radius: 0.0 },
            Corner { position: Point { x: 0.0, y: 2.0 }, radius: 0.0 },
            Corner { position: Point { x: 6.0, y: 2.0 }, radius: 0.0 },
            Corner { position: Point { x: 25.0, y: 22.0 }, radius: 0.0 },
            Corner { position: Point { x: 45.0, y: 37.0 }, radius: 0.0 },
            Corner { position: Point { x: 135.0, y: 57.0 }, radius: 0.0 },
            Corner { position: Point { x: 140.0, y: 57.0 }, radius: 0.0 },
            Corner { position: Point { x: 145.0, y: 55.0 }, radius: 0.0 },
            Corner { position: Point { x: 150.0, y: 57.0 }, radius: 0.0 },
            Corner { position: Point { x: 150.0, y: 63.0 }, radius: 0.0 },
            Corner { position: Point { x: 145.0, y: 65.0 }, radius: 0.0 },
            Corner { position: Point { x: -8.0, y: 45.0 }, radius: 0.0 },
          ],
        },
        inner: vec![],
        slots: vec![Slot::new(
          Point { x: -14.0, y: 37.0 },
          Point { x: 0.0, y: 37.0 },
          2.0,
        )],
        holes: vec![
          Circle { position: Point { x: HOLE_DISP, y: 7.0 }, radius: 3.1 },
          Circle { position: Point { x: HOLE_DISP, y: 16.0 }, radius: 3.1 },
          Circle { position: Point { x: -HOLE_DISP, y: 22.0 }, radius: 3.1 },
          Circle { position: Point { x: 0.0, y: 37.0 }, radius: 3.6 },
          Circle { position: Point { x: 10.0, y: 37.0 }, radius: 2.6 },
          Circle { position: Point { x: 38.0, y: 41.0 }, radius: 2.6 },
          Circle { position: Point { x: 145.0, y: 60.0 }, radius: 2.6 },
        ],
      },
      Figure {
        outer: Contour {
          corners: vec![
            Corner { position: Point { x: -8.0, y: 28.0 }, radius: 0.0 },
            Corner { position: Point { x: -10.0, y: 26.0 }, radius: 0.0 },
            Corner { position: Point { x: -10.0, y: 22.0 }, radius: 0.0 },
            Corner { position: Point { x: -2.0, y: 4.0 }, radius: 0.0 },
            Corner { position: Point { x: 0.0, y: 2.0 }, radius: 0.0 },
            Corner { position: Point { x: 6.0, y: 2.0 }, radius: 0.0 },
            Corner { position: Point { x: 25.0, y: 22.0 }, radius: 0.0 },
            Corner { position: Point { x: 45.0, y: 37.0 }, radius: 0.0 },
            Corner { position: Point { x: 40.0, y: 51.0 }, radius: 0.0 },
            Corner { position: Point { x: -8.0, y: 45.0 }, radius: 0.0 },
          ],
        },
        inner: vec![],
        slots: vec![],
        holes: vec![
          Circle { position: Point { x: HOLE_DISP, y: 7.0 }, radius: 3.1 },
          Circle { position: Point { x: -HOLE_DISP, y: 12.0 }, radius: 3.1 },
          Circle { position: Point { x: HOLE_DISP, y: 17.0 }, radius: 3.1 },
          Circle { position: Point { x: -HOLE_DISP, y: 22.0 }, radius: 3.1 },
          Circle { position: Point { x: 0.0, y: 37.0 }, radius: 3.6 },
          Circle { position: Point { x: 10.0, y: 37.0 }, radius: 2.6 },
          Circle { position: Point { x: 38.0, y: 41.0 }, radius: 2.6 },
          Circle { position: Point { x: 145.0, y: 55.0 }, radius: 2.6 },
        ],
      },
      Figure {
        outer: Contour {
          corners: vec![
            Corner { position: Point { x: 7.0, y: 26.0 }, radius: 0.0 },
            Corner { position: Point { x: 134.0, y: 55.0 }, radius: 0.0 },
            Corner { position: Point { x: 135.0, y: 57.0 }, radius: 0.0 },
            Corner { position: Point { x: 140.0, y: 57.0 }, radius: 0.0 },
            Corner { position: Point { x: 145.0, y: 55.0 }, radius: 0.0 },
            Corner { position: Point { x: 150.0, y: 57.0 }, radius: 0.0 },
            Corner { position: Point { x: 150.0, y: 63.0 }, radius: 0.0 },
            Corner { position: Point { x: 145.0, y: 65.0 }, radius: 0.0 },
            Corner { position: Point { x: 140.0, y: 75.0 }, radius: 0.0 },
            Corner { position: Point { x: 9.0, y: 55.0 }, radius: 0.0 },
            Corner { position: Point { x: 4.0, y: 45.0 }, radius: 0.0 },
          ],
        },
        inner: vec![],
        slots: vec![],
        holes: vec![
          Circle { position: Point { x: 10.0, y: 37.0 }, radius: 2.6 },
          Circle { position: Point { x: 38.0, y: 41.0 }, radius: 2.6 },
          Circle { position: Point { x: 145.0, y: 60.0 }, radius: 2.6 },
        ],
      },
    ];
    Self { figures }
  }

  pub fn faces(&self) -> usize {
    self.figures.len()
  }

  pub fn get_sticker_index(&self, pos: Point, current_normal: usize) -> PartIndex {
    let mut r = self.figures[current_normal].contains(pos);
    if current_normal == 6 {
      r = r
        || self.figures[current_normal]
          .contains(pos.reflect(Point { x: 140.0, y: 75.0 }, Point { x: 9.0, y: 55.0 }))
          ;
    }
    r as PartIndex
  }

  pub fn get_part_index(&self, pos: crate::points3d::Point) -> PartIndex {
    return 0;
  }
}
