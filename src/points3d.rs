#[derive(Debug, Default, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point {
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn sqr_len(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn len(self) -> f32 {
        self.sqr_len().sqrt()
    }

    pub fn scale(self, factor: f32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }

    pub fn norm(self) -> Self {
        self.scale(self.len().recip())
    }

    pub fn any_perp(self) -> Self {
        if self.x.abs() < self.y.abs() && self.x.abs() < self.z.abs() {
            return Self {
                x: 0.0,
                y: self.z,
                z: -self.y,
            };
        }

        if self.y.abs() < self.z.abs() {
            return Self {
                x: -self.z,
                y: 0.0,
                z: self.x,
            };
        }

        Self {
            x: self.y,
            y: -self.x,
            z: 0.0,
        }
    }
}

impl std::ops::Add for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::AddAssign for Point {
    fn add_assign(&mut self, rhs: Point) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl std::ops::Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::SubAssign for Point {
    fn sub_assign(&mut self, rhs: Point) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

pub fn dot(lhs: Point, rhs: Point) -> f32 {
    lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z
}

pub fn cross(lhs: Point, rhs: Point) -> Point {
    Point {
        x: lhs.y * rhs.z - lhs.z * rhs.y,
        y: lhs.z * rhs.x - lhs.x * rhs.z,
        z: lhs.x * rhs.y - lhs.y * rhs.x,
    }
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
