use crate::points2d::*;
use fxhash::FxHashSet;

pub const PI: f32 = std::f32::consts::PI;

pub fn sqr(x: f32) -> f32 {
  x * x
}

pub fn in_riga_logo(pos: Point) -> bool {
  let pos = pos + Point { x: 8.0, y: 2.5 };
  if dist_pl(pos, Point { x: 0.5, y: 0.5 }, Point { x: 0.5, y: 4.0 }) < 0.5 {
    return true;
  }
  if dist_pl(pos, Point { x: 1.0, y: 4.0 }, Point { x: 1.5, y: 4.0 }) < 1.0 {
    return true;
  }
  if (pos - Point { x: 1.5, y: 3.5 }).len() < 1.5 {
    return true;
  }
  if dist_pl(pos, Point { x: 1.0, y: 2.0 }, Point { x: 2.5, y: 0.5 }) < 0.5 {
    return true;
  }

  if dist_pl(pos, Point { x: 5.5, y: 0.5 }, Point { x: 5.5, y: 4.5 }) < 0.5 {
    return true;
  }

  if pos.y > 3.5 {
    let c = Point { x: 9.5, y: 3.5 };
    let l = (pos - c).len();
    if l > 0.5 && l < 1.5 {
      return true;
    }
  }
  if pos.y < 1.5 {
    let c = Point { x: 9.5, y: 1.5 };
    let l = (pos - c).len();
    if l > 0.5 && l < 1.5 {
      return true;
    }
  }
  if dist_pl(pos, Point { x: 8.5, y: 1.5 }, Point { x: 8.5, y: 3.5 }) < 0.5 {
    return true;
  }
  if (pos - Point { x: 10.5, y: 1.5 }).len() < 0.5 {
    return true;
  }
  if (pos - Point { x: 10.5, y: 3.5 }).len() < 0.5 {
    return true;
  }
  if (pos.x - 10.5).abs() < 0.5 && (pos.y - 0.75).abs() < 0.75 {
    return true;
  }

  if dist_pl(pos, Point { x: 13.5, y: 0.5 }, Point { x: 14.5, y: 4.5 }) < 0.5 {
    return true;
  }
  if dist_pl(pos, Point { x: 15.5, y: 0.5 }, Point { x: 14.5, y: 4.5 }) < 0.5 {
    return true;
  }

  false
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HoleID(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HoleArcID(usize);

#[derive(Copy, Clone, Debug)]
pub struct ConnectorID(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotID(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotArcID(usize);

#[derive(Copy, Clone, Debug)]
pub struct FigureID(usize);

pub struct Builder {
  default_border: f32,
  default_material_thickness: f32,
  error: f32,

  holes: Vec<Hole>,
  hole_arcs: Vec<HoleArc>,
  slots: Vec<Slot>,
  slot_arcs: Vec<SlotArc>,
  connectors: Vec<Connector>,
  figures: Vec<Figure>,
}

impl Builder {
  pub fn new(default_border: f32, default_material_thickness: f32, error: f32) -> Self {
    Self {
      default_border,
      default_material_thickness,
      error,
      holes: vec![],
      hole_arcs: vec![],
      slots: vec![],
      slot_arcs: vec![],
      connectors: vec![],
      figures: vec![],
    }
  }

  pub fn get_hole(&self, hole_id: HoleID) -> &Hole {
    &self.holes[hole_id.0]
  }

  pub fn get_hole_arc(&self, hole_arc_id: HoleArcID) -> &HoleArc {
    &self.hole_arcs[hole_arc_id.0]
  }

  pub fn get_slot(&self, slot_id: SlotID) -> &Slot {
    &self.slots[slot_id.0]
  }

  pub fn get_slot_arc(&self, slot_arc_id: SlotArcID) -> &SlotArc {
    &self.slot_arcs[slot_arc_id.0]
  }

  pub fn get_connector(&self, connector_id: ConnectorID) -> &Connector {
    &self.connectors[connector_id.0]
  }

  pub fn get_figure(&self, figure_id: FigureID) -> &Figure {
    &self.figures[figure_id.0]
  }

  pub fn add_hole(&mut self, mut hole: Hole) -> HoleID {
    let result = HoleID(self.holes.len());
    if !hole.has_border {
      hole.border = self.default_border;
    }
    if !hole.has_oval_width {
      hole.oval_width = hole.border;
    }
    if hole.r > 0.0 {
      hole.r += self.error;
    } else if hole.r < 0.0 {
      hole.r -= self.error;
    }
    if hole.border > 0.0 {
      hole.border -= self.error * 1.0;
    }
    if hole.oval_width > 0.0 {
      hole.oval_width -= self.error * 1.0;
    }
    self.holes.push(hole);
    result
  }

  pub fn add_hole_arc(&mut self, hole_arc: HoleArc) -> HoleArcID {
    let result = HoleArcID(self.hole_arcs.len());
    self.hole_arcs.push(hole_arc);
    result
  }

  pub fn add_slot(&mut self, mut slot: Slot) -> SlotID {
    let result = SlotID(self.slots.len());
    if !slot.has_border {
      slot.border = self.default_border;
    }
    if !slot.has_width {
      slot.width = self.default_material_thickness;
    }

    slot.direction = slot.direction.norm();
    slot.error = self.error;

    self.slots.push(slot);
    result
  }

  pub fn add_slot_arc(&mut self, slot_arc: SlotArc) -> SlotArcID {
    let result = SlotArcID(self.slot_arcs.len());
    self.slot_arcs.push(slot_arc);
    result
  }

  pub fn add_figure(&mut self, figure: Figure) -> FigureID {
    let result = FigureID(self.figures.len());
    self.figures.push(figure);
    result
  }

  pub fn add_connector(&mut self, mut connector: Connector) -> ConnectorID {
    let result = ConnectorID(self.connectors.len());
    if !connector.has_couple_size_top {
      connector.couple_size_top = self.default_material_thickness;
    }
    if !connector.has_couple_size_bottom {
      connector.couple_size_bottom = self.default_material_thickness;
    }

    connector.length -= self.error * 2.0;
    for p in &mut connector.protrusions {
      p.1 -= 2.0 * self.error;
    }
    for e in &mut connector.extra_layers_bottom {
      e.2 -= 2.0 * self.error;
    }
    for e in &mut connector.extra_layers_top {
      e.2 -= 2.0 * self.error;
    }

    self.connectors.push(connector);
    result
  }

  pub fn contour_count(&self) -> usize {
    self.figures.len() + self.connectors.len()
  }

  pub fn get_material_thickness(&self, index: usize) -> f32 {
    self.default_material_thickness
  }

  pub fn sticker_is_figure_id(&self, index: usize, figure_id: FigureID) -> bool {
    figure_id.0 == index
  }

  pub fn sticker_is_connector_id(&self, index: usize, connector_id: ConnectorID) -> bool {
    connector_id.0 == index + self.figures.len()
  }

  pub fn contains(&self, index: usize, pos: Point) -> bool {
    if index < self.figures.len() {
      self.get_figure(FigureID(index)).contains(pos, self)
    } else {
      self.get_connector(ConnectorID(index - self.figures.len())).contains(pos, self)
    }
  }

  pub fn aabb(&self, index: usize) -> AABB {
    if index < self.figures.len() {
      self.get_figure(FigureID(index)).aabb.rounded(0.501)
    } else {
      self.get_connector(ConnectorID(index - self.figures.len())).aabb().rounded(0.001)
    }
  }

  pub fn get_name(&self, index: usize) -> Option<&str> {
    if index < self.figures.len() {
      self.get_figure(FigureID(index)).name.as_ref().map(String::as_str)
    } else {
      self.get_connector(ConnectorID(index - self.figures.len())).name.as_ref().map(String::as_str)
    }
  }

  pub fn get_count(&self, index: usize) -> usize {
    if index < self.figures.len() {
      self.get_figure(FigureID(index)).count
    } else {
      self.get_connector(ConnectorID(index - self.figures.len())).count
    }
  }
}

#[derive(Debug, Clone)]
pub struct Slot {
  start: Point,
  direction: Point,
  width: f32,
  border: f32,
  length: f32,
  protrusions: Vec<(f32, f32)>,
  has_border: bool,
  has_width: bool,
  error: f32,
}

impl Slot {
  pub fn new(start: Point, direction: Point, length: f32, protrusions: &[(f32, f32)]) -> Self {
    Self {
      start,
      direction,
      length,
      protrusions: protrusions.to_vec(),
      width: 0.0,
      has_width: false,
      border: 0.0,
      has_border: false,
      error: 0.0,
    }
  }

  pub fn new_no_border(
    start: Point,
    direction: Point,
    length: f32,
    protrusions: &[(f32, f32)],
  ) -> Self {
    Self {
      start,
      direction,
      length,
      protrusions: protrusions.to_vec(),
      width: 0.0,
      has_width: false,
      border: 0.0,
      has_border: true,
      error: 0.0,
    }
  }

  pub fn rotate(mut self, center: Point, angle: f32) -> Self {
    let a = Point::from_angle(angle);
    self.start = complex_mul((self.start - center), a) + center;
    self.direction = complex_mul(self.direction, a);
    self
  }

  pub fn border(mut self, border: f32) -> Self {
    self.border = border;
    self.has_border = true;
    self
  }

  pub fn width(mut self, width: f32) -> Self {
    self.width = width;
    self.has_width = true;
    self
  }

  fn hmin(&self) -> f32 {
    self.border
  }

  fn hmax(&self) -> f32 {
    self.length - self.border
  }

  fn dir_proj(&self, pt: Point) -> f32 {
    dot(pt - self.start, self.direction)
  }

  fn normal_proj(&self, pt: Point) -> f32 {
    cross(pt - self.start, self.direction)
  }

  fn hole(&self, pt: Point) -> bool {
    let f1 = self.normal_proj(pt);
    if f1.abs() > self.width * 0.5 + self.error {
      return false;
    }
    let f2 = self.dir_proj(pt);
    for &(x1, x2) in &self.protrusions {
      if f2 > x1 - self.error && f2 < x2 + self.error {
        return true;
      }
    }
    false
  }

  fn dist(&self, pt: Point) -> f32 {
    if self.border == 0.0 {
      return 2.0;
    }
    let f1 = self.normal_proj(pt);
    let w = self.width * 0.5 - self.error;
    let x = if f1 < -w {
      f1 + w
    } else if f1 < w {
      0.0
    } else {
      f1 - w
    };

    let f2 = self.dir_proj(pt);
    let h1 = self.hmin() + self.error;
    let h2 = self.hmax() - self.error;

    let mut m1 = h1;
    let mut m2 = h2;

    if h1 > h2 {
      m1 = (h1 + h2) * 0.5;
      m2 = m1;
    }

    let y = if f2 < m1 {
      h1 - f2
    } else if f2 < m2 {
      0.0
    } else {
      f2 - h2
    };

    (sqr(x) + sqr(y)).sqrt() - self.border
  }

  fn aabb(&self) -> AABB {
    let h1 = self.hmin() + self.error;
    let h2 = self.hmax() - self.error;

    let mut m1 = h1;
    let mut m2 = h2;

    if h1 > h2 {
      m1 = (h1 + h2) * 0.5;
      m2 = m1;
    }

    let get_p =
      |a: f32, b: f32| self.start + self.direction.scale(a) + self.direction.perp().scale(b);

    let p0 = get_p(h1, (self.width * 0.5 - self.error));
    let p1 = get_p(h2, (self.width * 0.5 - self.error));
    let p2 = get_p(h1, -(self.width * 0.5 - self.error));
    let p3 = get_p(h2, -(self.width * 0.5 - self.error));
    AABB::from(&[p0, p1, p2, p3]).rounded(self.border)
  }

  fn figure_contour_part(&self, layout: Layout) -> FigureContourPart {
    let center = match layout {
      Layout::Begin => self.start,
      Layout::Middle => self.start + self.direction.scale(self.length * 0.5),
      Layout::End => self.start + self.direction.scale(self.length),
    };

    FigureContourPart { center, r: self.border }
  }
}

#[derive(Debug)]
struct LinearFunc {
  n: Point,
  c: f32,
}

impl LinearFunc {
  // create normalized linear function gives 0 in both points
  fn new_00_norm(pt1: Point, pt2: Point) -> Self {
    let n = (pt2 - pt1).norm().perp();
    let c = -dot(pt1, n);
    Self { n, c }
  }

  // create  linear function gives 0 in first point and 1 in second
  fn new_01(pt1: Point, pt2: Point) -> Self {
    let n = pt2 - pt1;
    let n = n.scale(1.0 / n.sqr_len());
    let c = -dot(pt1, n);
    Self { n, c }
  }

  fn get(&self, x: Point) -> f32 {
    dot(self.n, x) + self.c
  }
}

#[derive(Debug)]
struct ControlLine {
  start: LinearFunc,
  end: LinearFunc,
  mid: LinearFunc,
  r_in: f32,
  r_out: f32,
}

struct ControlLineParams {
  angle1: Point,
  angle2: Point,
  angle_mid_with_cos: Point,
  center: Point,
}

impl ControlLine {
  fn new(params: &ControlLineParams, pt1: Point, pt2: Point) -> Self {
    let pt1 = pt1 - params.center;
    let pt2 = pt2 - params.center;

    let r_in = pt1.len();
    let r_out = pt2.len();
    let start =
      LinearFunc::new_00_norm(complex_mul(pt2, params.angle1), complex_mul(pt1, params.angle1));
    let end =
      LinearFunc::new_00_norm(complex_mul(pt1, params.angle2), complex_mul(pt2, params.angle2));
    let mid = LinearFunc::new_00_norm(
      complex_mul(pt1, params.angle_mid_with_cos),
      complex_mul(pt2, params.angle_mid_with_cos),
    );
    Self { start, end, mid, r_in, r_out }
  }

  fn hole(&self, pt: Point) -> bool {
    let r = pt.len();
    if r < self.r_in || r > self.r_out {
      return false;
    }

    if self.mid.get(pt) > 0.0 {
      self.start.get(pt) > 0.0
    } else {
      self.end.get(pt) > 0.0
    }
  }
}

#[derive(Debug)]
pub struct SlotArc {
  start1: Point,
  start2: Point,
  direction1: Point,
  direction2: Point,
  width: f32,
  border: f32,
  length: f32,
  round: f32,
  center: Point,
  geom_center: Point,
  control_lines: Vec<ControlLine>,
}

impl SlotArc {
  pub fn new_no_border(
    builder: &Builder,
    slot_id: SlotID,
    slot_dist: f32,
    center: Point,
    angle1: f32,
    angle2: f32,
  ) -> Self {
    let slot = builder.get_slot(slot_id);
    let border = 0.0;
    let round = 0.0;

    let angle_mid = (angle1 + angle2) * 0.5;
    let angle1 = Point::from_angle(angle1);
    let angle2 = Point::from_angle(angle2);
    let angle_mid = Point::from_angle(angle_mid);

    let get_p = |d: f32, p: f32| -> Point {
      slot.start + slot.direction.scale(d) + slot.direction.perp().scale(p)
    };

    let get_c = |d: f32, p: f32, dl: f32| -> Point {
      let p = get_p(d, p) - center;

      let l = p.len();
      if l == 0.0 {
        Point::X.scale(dl) + center
      } else {
        p.scale(1.0 + dl / l) + center
      }
    };

    let start = get_p(0.0, 0.0);

    let start1 = complex_mul(start - center, angle1) + center;
    let start2 = complex_mul(start - center, angle2) + center;
    let direction1 = complex_mul(slot.direction, angle1);
    let direction2 = complex_mul(slot.direction, angle2);
    let width = slot.width;
    let length = slot.length;

    let cos_half = angle_mid.x * angle1.x + angle_mid.y * angle1.y;
    let params =
      ControlLineParams { angle1, angle2, angle_mid_with_cos: angle_mid.scale(cos_half), center };

    let d_dir = dot(center - slot.start, slot.direction);
    let d_perp = cross(center - slot.start, slot.direction);

    let mut control_lines = Vec::<ControlLine>::new();

    let geom_center = complex_mul(get_p(length * 0.5, 0.0) - center, angle_mid) + center;

    if d_perp < -width * 0.5 {
      if d_dir < 0.0 {
        let pt1 = get_c(0.0, -width * 0.5, -slot_dist);
        let pt2 = get_c(length, width * 0.5, slot_dist);
        control_lines.push(ControlLine::new(&params, pt1, pt2));
      } else if d_dir < length {
        let ptm = get_c(d_dir, -width * 0.5, -slot_dist);
        let pt1 = get_c(0.0, width * 0.5, slot_dist);
        let pt2 = get_c(length, width * 0.5, slot_dist);
        control_lines.push(ControlLine::new(&params, ptm, pt1));
        control_lines.push(ControlLine::new(&params, ptm, pt2));
      } else {
        let pt1 = get_c(length, -width * 0.5, -slot_dist);
        let pt2 = get_c(0.0, width * 0.5, slot_dist);
        control_lines.push(ControlLine::new(&params, pt1, pt2));
      }
    } else if d_perp < width * 0.5 {
      if d_dir < 0.0 {
        let ptm = get_c(0.0, d_perp, -slot_dist);
        let pt1 = get_c(length, -width * 0.5, slot_dist);
        let pt2 = get_c(length, width * 0.5, slot_dist);
        control_lines.push(ControlLine::new(&params, ptm, pt1));
        control_lines.push(ControlLine::new(&params, ptm, pt2));
      } else if d_dir < length {
        let ptm = get_c(d_dir, d_perp, 0.0);
        let pt1 = get_c(0.0, -width * 0.5, slot_dist);
        let pt2 = get_c(0.0, width * 0.5, slot_dist);
        let pt3 = get_c(length, -width * 0.5, slot_dist);
        let pt4 = get_c(length, width * 0.5, slot_dist);
        control_lines.push(ControlLine::new(&params, ptm, pt1));
        control_lines.push(ControlLine::new(&params, ptm, pt2));
        control_lines.push(ControlLine::new(&params, ptm, pt3));
        control_lines.push(ControlLine::new(&params, ptm, pt4));
      } else {
        let ptm = get_c(length, d_perp, -slot_dist);
        let pt1 = get_c(0.0, -width * 0.5, slot_dist);
        let pt2 = get_c(0.0, width * 0.5, slot_dist);
        control_lines.push(ControlLine::new(&params, ptm, pt1));
        control_lines.push(ControlLine::new(&params, ptm, pt2));
      }
    } else {
      if d_dir < 0.0 {
        let pt1 = get_c(0.0, width * 0.5, -slot_dist);
        let pt2 = get_c(length, -width * 0.5, slot_dist);
        control_lines.push(ControlLine::new(&params, pt1, pt2));
      } else if d_dir < length {
        let ptm = get_c(d_dir, width * 0.5, -slot_dist);
        let pt1 = get_c(0.0, -width * 0.5, slot_dist);
        let pt2 = get_c(length, -width * 0.5, slot_dist);
        control_lines.push(ControlLine::new(&params, ptm, pt1));
        control_lines.push(ControlLine::new(&params, ptm, pt2));
      } else {
        let pt1 = get_c(length, width * 0.5, -slot_dist);
        let pt2 = get_c(0.0, -width * 0.5, slot_dist);
        control_lines.push(ControlLine::new(&params, pt1, pt2));
      }
    }

    Self {
      start1,
      start2,
      direction1,
      direction2,
      width,
      border,
      length,
      center,
      round: slot_dist,
      geom_center,
      control_lines,
    }
  }

  fn hole(&self, pt: Point) -> bool {
    let hole = |start, dir| -> bool {
      let d_dir = dot(pt - start, dir);
      let d_perp = cross(pt - start, dir);
      let d = if d_dir < 0.0 {
        -d_dir
      } else if d_dir < self.length {
        0.0
      } else {
        d_dir - self.length
      };

      let p = if d_perp < -self.width * 0.5 {
        -d_perp - self.width * 0.5
      } else if d_perp < self.width * 0.5 {
        0.0
      } else {
        d_perp - self.width * 0.5
      };

      sqr(d) + sqr(p) <= sqr(self.round)
    };

    if hole(self.start1, self.direction1) || hole(self.start2, self.direction2) {
      return true;
    }

    for cl in &self.control_lines {
      if cl.hole(pt - self.center) {
        return true;
      }
    }

    false
  }

  fn figure_contour_part(&self, layout: Layout) -> FigureContourPart {
    let center = match layout {
      Layout::Begin => self.start1 + self.direction1.scale(self.length * 0.5),
      Layout::Middle => self.geom_center,
      Layout::End => self.start2 + self.direction2.scale(self.length * 0.5),
    };

    FigureContourPart { center, r: self.border }
  }
}

#[derive(Debug, Clone)]
pub struct Hole {
  center: Point,
  r: f32,
  border: f32,
  oval_width: f32,
  has_border: bool,
  has_oval_width: bool,
}

impl Hole {
  pub fn new(center: Point, r: f32) -> Self {
    Self { center, r, border: 0.0, oval_width: 0.0, has_border: false, has_oval_width: false }
  }

  pub fn new_no_border(center: Point, r: f32) -> Self {
    Self { center, r, border: 0.0, oval_width: 0.0, has_border: true, has_oval_width: false }
  }

  pub fn new_solid(center: Point, r: f32) -> Self {
    Self { center, r: 0.0, border: r, oval_width: 0.0, has_border: true, has_oval_width: false }
  }

  pub fn rotate(mut self, center: Point, angle: f32) -> Self {
    let a = Point::from_angle(angle);
    self.center = complex_mul((self.center - center), a) + center;
    self
  }

  pub fn border(mut self, border: f32) -> Self {
    self.border = border;
    self.has_border = true;
    self
  }

  pub fn oval_width(mut self, oval_width: f32) -> Self {
    self.oval_width = oval_width;
    self.has_oval_width = true;
    self
  }

  fn hole(&self, pt: Point) -> bool {
    self.r > 0.0 && (pt - self.center).sqr_len() < sqr(self.r)
  }

  fn dist(&self, pt: Point) -> f32 {
    if self.border == 0.0 {
      return 2.0;
    }
    (pt - self.center).len() - (self.border + self.r)
  }

  fn aabb(&self) -> AABB {
    AABB::around(self.center, self.border + self.r)
  }

  fn figure_contour_part(&self) -> FigureContourPart {
    FigureContourPart { center: self.center, r: self.oval_width }
  }
}

#[derive(Debug)]
pub struct HoleArc {
  center: Point,
  angle1: Point,
  angle2: Point,
  angle_mid: Point,
  hole_r: f32,
  border: f32,
  arc_r: f32,
  big_arc: i32,
}

impl HoleArc {
  pub fn new(
    builder: &Builder,
    hole_id: HoleID,
    hole_dist: f32,
    center: Point,
    angle1: f32,
    angle2: f32,
  ) -> Self {
    let hole = builder.get_hole(hole_id);
    let angle_mid = (angle1 + angle2) * 0.5;
    let dc = hole.center - center;
    let big_arc = (angle2 > angle1 + PI) as i32;
    let angle1 = Point::from_angle(angle1);
    let angle2 = Point::from_angle(angle2);
    let angle_mid = Point::from_angle(angle_mid);
    let angle1 = complex_mul(dc, angle1);
    let angle2 = complex_mul(dc, angle2);
    let angle_mid = complex_mul(dc, angle_mid);
    let arc_r = dc.len();
    Self {
      center,
      angle1,
      angle2,
      angle_mid,
      hole_r: hole.r + hole_dist,
      border: hole.border,
      arc_r,
      big_arc,
    }
  }

  pub fn border(mut self, border: f32) -> Self {
    self.border = border;
    self
  }

  fn pt_inside_angle(&self, pt: Point) -> bool {
    (cross(self.angle1, pt) > 0.0) as i32 + (cross(pt, self.angle2) > 0.0) as i32 + self.big_arc
      >= 2
  }

  fn hole(&self, pt: Point) -> bool {
    let pt = pt - self.center;
    if (pt - self.angle1).sqr_len() < sqr(self.hole_r) {
      return true;
    }
    if (pt - self.angle2).sqr_len() < sqr(self.hole_r) {
      return true;
    }
    if !self.pt_inside_angle(pt) {
      return false;
    }

    let sl = pt.sqr_len();
    sl > sqr(self.arc_r - self.hole_r) && sl < sqr(self.arc_r + self.hole_r)
  }

  fn dist(&self, pt: Point) -> f32 {
    if self.border == 0.0 {
      return 2.0;
    }
    let pt = pt - self.center;

    if !self.pt_inside_angle(pt) {
      f32::min(
        (pt - self.angle1).len() - (self.border + self.hole_r),
        (pt - self.angle2).len() - (self.border + self.hole_r),
      )
    } else {
      (pt.len() - self.arc_r).abs() - (self.border + self.hole_r)
    }
  }

  fn aabb(&self) -> AABB {
    let mut result = AABB::from(&[self.center + self.angle1, self.center + self.angle2]);
    for p in [Point::X, Point::Y, -Point::X, -Point::Y] {
      if self.pt_inside_angle(p) {
        result = result.with(self.center + p.scale(self.arc_r))
      }
    }

    result.rounded(self.border + self.hole_r)
  }

  fn figure_contour_part(&self, layout: Layout) -> FigureContourPart {
    let center = match layout {
      Layout::Begin => self.center + self.angle1,
      Layout::Middle => self.center + self.angle_mid,
      Layout::End => self.center + self.angle2,
    };
    FigureContourPart { center, r: self.border }
  }
}

#[derive(Debug)]
pub struct Connector {
  width: f32,
  length: f32,
  error_shift: f32,
  protrusions: Vec<(f32, f32)>,
  couple_size_top: f32,
  couple_size_bottom: f32,
  holes: Vec<HoleID>,
  slots: Vec<SlotID>,
  has_couple_size_top: bool,
  has_couple_size_bottom: bool,

  extra_layers_top: Vec<(f32, f32, f32)>,
  extra_layers_bottom: Vec<(f32, f32, f32)>,

  name: Option<String>,
  count: usize,
}

impl Connector {
  pub fn new(builder: &Builder, slot_id: SlotID, width: f32) -> Self {
    let slot = builder.get_slot(slot_id);
    let length = slot.length;
    let protrusions = slot.protrusions.clone();
    let error_shift = builder.error;
    let name = None;
    let count = 1;

    Self {
      width,
      length,
      error_shift,
      protrusions,
      couple_size_top: 0.0,
      couple_size_bottom: 0.0,
      holes: vec![],
      slots: vec![],
      has_couple_size_top: false,
      has_couple_size_bottom: false,
      extra_layers_top: vec![],
      extra_layers_bottom: vec![],
      name,
      count,
    }
  }

  pub fn name(mut self, name: String) -> Self {
    self.name = Some(name);
    self
  }

  pub fn count(mut self, count: usize) -> Self {
    self.count = count;
    self
  }

  pub fn couple_size(mut self, couple_size: f32) -> Self {
    self.couple_size_top = couple_size;
    self.couple_size_bottom = couple_size;
    self.has_couple_size_top = true;
    self.has_couple_size_bottom = true;
    self
  }

  pub fn couple_size_top(mut self, couple_size_top: f32) -> Self {
    self.couple_size_top = couple_size_top;
    self.has_couple_size_top = true;
    self
  }

  pub fn couple_size_bottom(mut self, couple_size_bottom: f32) -> Self {
    self.couple_size_bottom = couple_size_bottom;
    self.has_couple_size_bottom = true;
    self
  }

  pub fn holes(mut self, holes: &[HoleID]) -> Self {
    self.holes = holes.to_vec();
    self
  }

  pub fn slots(mut self, slots: &[SlotID]) -> Self {
    self.slots = slots.to_vec();
    self
  }

  pub fn extra_layers_top(mut self, extra_layers_top: &[(f32, f32, f32)]) -> Self {
    self.extra_layers_top = extra_layers_top.to_vec();
    self
  }

  pub fn extra_layers_bottom(mut self, extra_layers_bottom: &[(f32, f32, f32)]) -> Self {
    self.extra_layers_bottom = extra_layers_bottom.to_vec();
    self
  }

  pub fn aabb(&self) -> AABB {
    let x1 = 0.0;
    let x2 = self.length;

    let top_w = self.width * 0.5 + self.couple_size_top;
    let bottom_w = -self.width * 0.5 - self.couple_size_bottom;

    let mut y1 = bottom_w;
    let mut y2 = top_w;

    // assume extra layers are not intersected
    for &(w, _, _) in &self.extra_layers_top {
      y2 = f32::max(y2, top_w + w);
    }

    for &(w, _, _) in &self.extra_layers_bottom {
      y1 = f32::max(y1, bottom_w - w);
    }

    AABB { x1, y1, x2, y2 }
  }

  pub fn contains(&self, pt: Point, builder: &Builder) -> bool {
    let corrected_pt = Point { x: pt.x + self.error_shift, y: pt.y };
    for &h in &self.holes {
      let h = builder.get_hole(h);
      if h.hole(corrected_pt) {
        return false;
      }
    }
    for &s in &self.slots {
      let s = builder.get_slot(s);
      if s.hole(corrected_pt) {
        return false;
      }
    }

    for &h in &self.holes {
      let h = builder.get_hole(h);
      if h.dist(corrected_pt) < 0.0 {
        return true;
      }
    }
    for &s in &self.slots {
      let s = builder.get_slot(s);
      if s.dist(corrected_pt) < 0.0 {
        return true;
      }
    }

    if pt.x < 0.0 || pt.x >= self.length {
      return false;
    }

    let mut top_w = self.width * 0.5;
    let mut bottom_w = -self.width * 0.5;

    for &(x1, x2) in &self.protrusions {
      if pt.x > x1 && pt.x < x2 {
        top_w += self.couple_size_top;
        bottom_w -= self.couple_size_bottom;
      }
    }

    for &(w, x1, x2) in &self.extra_layers_top {
      if pt.x > x1 && pt.x < x2 {
        top_w += w;
      }
    }

    for &(w, x1, x2) in &self.extra_layers_bottom {
      if pt.x > x1 && pt.x < x2 {
        bottom_w -= w;
      }
    }

    pt.y < top_w && pt.y > bottom_w
  }
}

#[derive(Debug)]
struct FigureContourPart {
  center: Point,
  r: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layout {
  Begin,
  Middle,
  End,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnyID {
  HoleID(HoleID),
  HoleArcID(HoleArcID, Layout),
  SlotID(SlotID, Layout),
  SlotArcID(SlotArcID, Layout),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct OvalKey(AnyID, AnyID);

#[derive(Debug)]
struct Oval {
  first: FigureContourPart,
  second: FigureContourPart,
  // some helper fields to improve computations
  splitter: LinearFunc,
  case1: LinearFunc,
  dist1: LinearFunc,
  case2: LinearFunc,
  dist2: LinearFunc,
}

impl Oval {
  fn new(first: FigureContourPart, second: FigureContourPart) -> Option<Self> {
    let splitter = LinearFunc::new_00_norm(first.center, second.center);

    let delta = first.center - second.center;
    let tcos = (second.r - first.r) / delta.len();

    if tcos.abs() > 1.0 {
      return None;
    }

    let tsin = (1.0 - sqr(tcos)).sqrt();

    let delta = delta.norm();
    let dir1 = complex_mul(delta, Point { x: tcos, y: tsin });
    let f1 = first.center + dir1.scale(first.r);
    let s1 = second.center + dir1.scale(second.r);
    let dir2 = complex_mul(delta, Point { x: tcos, y: -tsin });
    let f2 = first.center + dir2.scale(first.r);
    let s2 = second.center + dir2.scale(second.r);

    let case1 = LinearFunc::new_01(f1, s1);
    let dist1 = LinearFunc::new_00_norm(f1, s1);
    let case2 = LinearFunc::new_01(f2, s2);
    let dist2 = LinearFunc::new_00_norm(s2, f2);

    Some(Self { first, second, splitter, case1, dist1, case2, dist2 })
  }

  fn dist(&self, pt: Point) -> f32 {
    if self.splitter.get(pt) > 0.0 {
      let case = self.case1.get(pt);
      if case < 0.0 {
        (pt - self.first.center).len() - self.first.r
      } else if case < 1.0 {
        self.dist1.get(pt)
      } else {
        (pt - self.second.center).len() - self.second.r
      }
    } else {
      let case = self.case2.get(pt);
      if case < 0.0 {
        (pt - self.first.center).len() - self.first.r
      } else if case < 1.0 {
        self.dist2.get(pt)
      } else {
        (pt - self.second.center).len() - self.second.r
      }
    }
  }

  fn aabb(&self) -> AABB {
    AABB::around(self.first.center, self.first.r)
      .combine(AABB::around(self.second.center, self.second.r))
  }
}

impl AnyID {
  pub fn to_any_id(self) -> AnyID {
    self
  }
}

impl HoleID {
  pub fn to_any_id(self) -> AnyID {
    AnyID::HoleID(self)
  }
}

impl HoleArcID {
  pub fn to_any_id(self) -> AnyID {
    AnyID::HoleArcID(self, Layout::Middle)
  }

  pub fn begin(self) -> AnyID {
    AnyID::HoleArcID(self, Layout::Begin)
  }

  pub fn end(self) -> AnyID {
    AnyID::HoleArcID(self, Layout::End)
  }
}

impl SlotID {
  pub fn to_any_id(self) -> AnyID {
    AnyID::SlotID(self, Layout::Middle)
  }

  pub fn begin(self) -> AnyID {
    AnyID::SlotID(self, Layout::Begin)
  }

  pub fn end(self) -> AnyID {
    AnyID::SlotID(self, Layout::End)
  }
}

impl SlotArcID {
  pub fn to_any_id(self) -> AnyID {
    AnyID::SlotArcID(self, Layout::Middle)
  }

  pub fn begin(self) -> AnyID {
    AnyID::SlotArcID(self, Layout::Begin)
  }

  pub fn end(self) -> AnyID {
    AnyID::SlotArcID(self, Layout::End)
  }
}

#[macro_export]
macro_rules! contour {
  ($($tt: expr),+ $(,)?) => {
    FigureContourPreparingInfo {
      positions: vec![$($tt.to_any_id()),+],
      kind: FigureContourKind::Contour,
    }
  }
}

pub use contour;

#[macro_export]
macro_rules! chain {
  ($($tt: expr),+  $(,)?) => {
    FigureContourPreparingInfo {
      positions: vec![$($tt.to_any_id()),+],
      kind: FigureContourKind::Chain,
    }
  }
}

pub use chain;

#[macro_export]
macro_rules! filled {
  ($($tt: expr),+  $(,)?) => {
    FigureContourPreparingInfo {
      positions: vec![$($tt.to_any_id()),+],
      kind: FigureContourKind::Filled,
    }
  }
}

pub use filled;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FigureContourKind {
  Chain,
  Contour,
  Filled,
}

#[derive(Debug)]
pub struct FigureContourPreparingInfo {
  pub positions: Vec<AnyID>,
  pub kind: FigureContourKind,
}

#[derive(Debug)]
pub struct Filled {
  corners: Vec<Point>,
}

impl Filled {
  pub fn contains(&self, point: Point) -> bool {
    let mut c_in = 0;
    let mut c_out = 0;
    let mut prev = self.corners[self.corners.len() - 1] - point;
    for i in 0..self.corners.len() {
      let cur = self.corners[i] - point;
      if prev.y >= 0.0 && cur.y < 0.0 && cross(prev, cur) <= 0.0 {
        c_in += 1;
      } else if prev.y < 0.0 && cur.y >= 0.0 && cross(prev, cur) > 0.0 {
        c_out += 1;
      }
      prev = cur;
    }
    c_in != c_out
  }

  fn aabb(&self) -> AABB {
    AABB::from(&self.corners)
  }
}

#[derive(Debug)]
pub struct Figure {
  filleds: Vec<Filled>,
  holes: Vec<HoleID>,
  hole_arcs: Vec<HoleArcID>,
  slots: Vec<SlotID>,
  slot_arcs: Vec<SlotArcID>,
  ovals: Vec<Oval>,
  name: Option<String>,
  aabb: AABB,
  count: usize,
}

impl Figure {
  fn get_part(builder: &Builder, part_id: AnyID) -> FigureContourPart {
    match part_id {
      AnyID::HoleID(h) => builder.get_hole(h).figure_contour_part(),
      AnyID::HoleArcID(h, layout) => builder.get_hole_arc(h).figure_contour_part(layout),
      AnyID::SlotID(s, layout) => builder.get_slot(s).figure_contour_part(layout),
      AnyID::SlotArcID(s, layout) => builder.get_slot_arc(s).figure_contour_part(layout),
    }
  }

  pub fn new(builder: &Builder, contours: &[FigureContourPreparingInfo]) -> Self {
    let mut holes = Vec::<HoleID>::new();
    let mut hole_arcs = Vec::<HoleArcID>::new();
    let mut slots = Vec::<SlotID>::new();
    let mut slot_arcs = Vec::<SlotArcID>::new();
    let mut filleds = Vec::<Filled>::new();
    let mut oval_keys = Vec::<OvalKey>::new();

    for c in contours {
      if c.positions.is_empty() {
        continue;
      }
      for p in &c.positions {
        match p {
          AnyID::HoleID(h) => {
            holes.push(*h);
          }
          AnyID::HoleArcID(h, _) => {
            hole_arcs.push(*h);
          }
          AnyID::SlotID(s, _) => {
            slots.push(*s);
          }
          AnyID::SlotArcID(s, _) => {
            slot_arcs.push(*s);
          }
        }
      }

      if c.kind == FigureContourKind::Filled {
        let mut corners = Vec::<Point>::new();
        for &p in &c.positions {
          corners.push(Self::get_part(builder, p).center);
        }
        filleds.push(Filled { corners });
      } else {
        let mut prev = if c.kind == FigureContourKind::Chain { None } else { c.positions.last() };
        for p in &c.positions {
          if let Some(prev) = prev {
            let mut g = if prev < p { OvalKey(*prev, *p) } else { OvalKey(*p, *prev) };
            oval_keys.push(g);
          }
          prev = Some(p);
        }
      }
    }

    holes.sort();
    holes.dedup();
    hole_arcs.sort();
    hole_arcs.dedup();
    slots.sort();
    slots.dedup();
    slot_arcs.sort();
    slot_arcs.dedup();
    oval_keys.sort();
    oval_keys.dedup();

    let ovals: Vec<Oval> = oval_keys
      .iter()
      .filter_map(|&OvalKey(p1, p2)| {
        let p1 = Self::get_part(builder, p1);
        let p2 = Self::get_part(builder, p2);
        Oval::new(p1, p2)
      })
      .collect();

    let name = None;
    let count = 1;

    let mut aabb = AABB::empty();
    for f in &filleds {
      aabb = aabb.combine(f.aabb());
    }
    for &h in &holes {
      aabb = aabb.combine(builder.get_hole(h).aabb());
    }
    for &ha in &hole_arcs {
      aabb = aabb.combine(builder.get_hole_arc(ha).aabb());
    }
    for &s in &slots {
      aabb = aabb.combine(builder.get_slot(s).aabb());
    }
    for o in &ovals {
      aabb = aabb.combine(o.aabb());
    }

    Self { holes, hole_arcs, slots, slot_arcs, filleds, ovals, name, aabb, count }
  }

  pub fn name(mut self, name: String) -> Self {
    self.name = Some(name);
    self
  }

  pub fn count(mut self, count: usize) -> Self {
    self.count = count;
    self
  }

  pub fn contains(&self, pt: Point, builder: &Builder) -> bool {
    if !self.aabb.contains(pt) {
      return false;
    }
    for &h in &self.holes {
      let h = builder.get_hole(h);
      if h.hole(pt) {
        return false;
      }
    }
    for &ha in &self.hole_arcs {
      let ha = builder.get_hole_arc(ha);
      if ha.hole(pt) {
        return false;
      }
    }
    for &s in &self.slots {
      let s = builder.get_slot(s);
      if s.hole(pt) {
        return false;
      }
    }
    for &s in &self.slot_arcs {
      let s = builder.get_slot_arc(s);
      if s.hole(pt) {
        return false;
      }
    }

    for c in &self.filleds {
      if c.contains(pt) {
        return true;
      }
    }

    let mut sumd = 0.0;
    let maxd = 0.5;

    let mut use_dist = |d: f32| {
      if d < 0.0 {
        return true;
      }
      if d < maxd {
        sumd += sqr(sqr(1.0 - d / maxd));
      }
      return false;
    };

    for &h in &self.holes {
      let h = builder.get_hole(h);
      if use_dist(h.dist(pt)) {
        return true;
      }
    }
    for &ha in &self.hole_arcs {
      let ha = builder.get_hole_arc(ha);
      if use_dist(ha.dist(pt)) {
        return true;
      }
    }
    for &s in &self.slots {
      let s = builder.get_slot(s);
      if use_dist(s.dist(pt)) {
        return true;
      }
    }
    for o in &self.ovals {
      if use_dist(o.dist(pt)) {
        return true;
      }
    }

    sumd > 1.0
  }
}
