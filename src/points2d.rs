#[derive(Debug, Default, Clone, Copy)]
pub struct Point {
  pub x: f32,
  pub y: f32,
}

impl Point {
  pub fn zero() -> Self {
    Self { x: 0.0, y: 0.0 }
  }

  pub fn from_angle(a: f32) -> Self {
    let (s, c) = a.sin_cos();
    Self { x: c, y: s }
  }

  pub fn sqr_len(self) -> f32 {
    self.x * self.x + self.y * self.y
  }

  pub fn len(self) -> f32 {
    self.sqr_len().sqrt()
  }

  pub fn scale(self, factor: f32) -> Self {
    Self { x: self.x * factor, y: self.y * factor }
  }

  pub fn norm(self) -> Self {
    self.scale(self.len().recip())
  }

  pub fn perp(self) -> Self {
    Self { x: self.y, y: -self.x }
  }

  pub fn reflect(self, pt1: Point, pt2: Point) -> Self {
    let delta = (pt2 - pt1).perp().norm();
    let d = dot(delta, self - pt1);
    self - delta.scale(d * 2.0)
  }
}

impl std::ops::Add for Point {
  type Output = Point;

  fn add(self, rhs: Point) -> Point {
    Point { x: self.x + rhs.x, y: self.y + rhs.y }
  }
}

impl std::ops::AddAssign for Point {
  fn add_assign(&mut self, rhs: Point) {
    self.x += rhs.x;
    self.y += rhs.y;
  }
}

impl std::ops::Sub for Point {
  type Output = Point;

  fn sub(self, rhs: Point) -> Point {
    Point { x: self.x - rhs.x, y: self.y - rhs.y }
  }
}

impl std::ops::SubAssign for Point {
  fn sub_assign(&mut self, rhs: Point) {
    self.x -= rhs.x;
    self.y -= rhs.y;
  }
}

impl std::ops::Neg for Point {
  type Output = Point;

  fn neg(self) -> Point {
    Point { x: -self.x, y: -self.y }
  }
}

pub fn dot(lhs: Point, rhs: Point) -> f32 {
  lhs.x * rhs.x + lhs.y * rhs.y
}

pub fn cross(lhs: Point, rhs: Point) -> f32 {
  lhs.x * rhs.y - lhs.y * rhs.x
}

pub fn find_root(
  f: &dyn Fn(Point) -> u32,
  mut pos1: Point,
  mut pos2: Point,
  target: u32,
  tries: usize,
) -> Point {
  let mut i = 0;
  loop {
    i += 1;
    let mid = (pos1 + pos2).scale(0.5);
    if i >= tries {
      return mid;
    }
    let r = f(mid);
    if r == target {
      pos1 = mid;
    } else {
      pos2 = mid;
    }
  }
}

pub fn dist_pl(p: Point, p1: Point, p2: Point) -> f32 {
  let p12 = p2 - p1;
  let l = p12.len();
  let p1 = p1 - p;
  let p2 = p2 - p;
  if l == 0.0 {
    return p2.len();
  }
  let p12 = p12.scale(l.recip());
  let d = dot(p2, p12);
  if d <= 0.0 {
    p2.len()
  } else if d >= l {
    p1.len()
  } else {
    cross(p1, p12).abs()
  }
}
