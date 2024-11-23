use crate::points2d::*;
use fxhash::FxHashSet;

pub const PI: f32 = std::f32::consts::PI;

pub fn sqr(x: f32) -> f32 {
  x * x
}

/*
fn sm_min(x: &[f32]) -> f32 {
  let mut result = f32::MAX;
  for &x in x {
    result = f32::min(result, x);
  }
  result
}

fn sm_max(x: &[f32]) -> f32 {
  let mut result = f32::MIN;
  for &x in x {
    result = f32::max(result, x);
  }
  result
}*/

fn sm_min(x: &[f32]) -> f32 {
  let mut result = 0.0;
  for x in x {
    result += x.powi(-8);
  }
  return result.powf(-0.125);
}

fn sm_max(x: &[f32]) -> f32 {
  let mut result = 0.0;
  for x in x {
    result += x.powi(8);
  }
  return result.powf(0.125);
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

#[derive(Copy, Clone, Debug)]
pub struct HoleID(usize);

#[derive(Copy, Clone, Debug)]
pub struct HoleArcID(usize);

#[derive(Copy, Clone, Debug)]
pub struct ConnectorID(usize);

#[derive(Copy, Clone, Debug)]
pub struct SlotID(usize);

#[derive(Copy, Clone, Debug)]
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
    if !hole.has_gauntlet_width {
      hole.gauntlet_width = hole.border;
    }
    if hole.r > 0.0 {
      hole.r += self.error;
    } else if hole.r < 0.0 {
      hole.r -= self.error;
    }
    if hole.border > 0.0 {
      hole.border -= self.error * 1.0;
    }
    if hole.gauntlet_width > 0.0 {
      hole.gauntlet_width -= self.error * 1.0;
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

  fn border_func(&self, pt: Point) -> f32 {
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

    (sqr(x) + sqr(y)).sqrt() / self.border
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
  fn new(pt1: Point, pt2: Point) -> Self {
    let n = (pt2 - pt1).norm().perp();
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
    let start = LinearFunc::new(complex_mul(pt2, params.angle1), complex_mul(pt1, params.angle1));
    let end = LinearFunc::new(complex_mul(pt1, params.angle2), complex_mul(pt2, params.angle2));
    let mid = LinearFunc::new(
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
      let p = get_p(d, p);

      let l = p.len();
      if l == 0.0 {
        Point::X.scale(dl)
      } else {
        p.scale(1.0 + dl / l)
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
        println!("suka {:?}, {:?}, {:?}, {:?}, {:?}", ptm, pt1, pt2, pt3, pt4);
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
  gauntlet_width: f32,
  has_border: bool,
  has_gauntlet_width: bool,
}

impl Hole {
  pub fn new(center: Point, r: f32) -> Self {
    Self {
      center,
      r,
      border: 0.0,
      gauntlet_width: 0.0,
      has_border: false,
      has_gauntlet_width: false,
    }
  }

  pub fn new_no_border(center: Point, r: f32) -> Self {
    Self {
      center,
      r,
      border: 0.0,
      gauntlet_width: 0.0,
      has_border: true,
      has_gauntlet_width: false,
    }
  }

  pub fn new_solid(center: Point, r: f32) -> Self {
    Self {
      center,
      r: 0.0,
      border: r,
      gauntlet_width: 0.0,
      has_border: true,
      has_gauntlet_width: false,
    }
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

  pub fn gauntlet_width(mut self, gauntlet_width: f32) -> Self {
    self.gauntlet_width = gauntlet_width;
    self.has_gauntlet_width = true;
    self
  }

  fn hole(&self, pt: Point) -> bool {
    self.r > 0.0 && (pt - self.center).sqr_len() < sqr(self.r)
  }

  fn border_func(&self, pt: Point) -> f32 {
    if self.border == 0.0 {
      return 2.0;
    }
    (pt - self.center).len() / (self.border + self.r)
  }

  fn figure_contour_part(&self) -> FigureContourPart {
    FigureContourPart { center: self.center, r: self.gauntlet_width }
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

  fn border_func(&self, pt: Point) -> f32 {
    if self.border == 0.0 {
      return 2.0;
    }
    let pt = pt - self.center;

    if !self.pt_inside_angle(pt) {
      f32::min(
        (pt - self.angle1).len() / (self.border + self.hole_r),
        (pt - self.angle2).len() / (self.border + self.hole_r),
      )
    } else {
      (pt.len() - self.arc_r).abs() / (self.border + self.hole_r)
    }
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
      if h.border_func(corrected_pt) < 1.0 {
        return true;
      }
    }
    for &s in &self.slots {
      let s = builder.get_slot(s);
      if s.border_func(corrected_pt) < 1.0 {
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

impl FigureContourPart {
  fn in_gauntlet(&self, prev: &FigureContourPart, pt: Point) -> f32 {
    let w_mid = (prev.r + self.r) * 0.5;
    if w_mid < 0.01 {
      return 2.0;
    };
    let delta = self.center - prev.center;
    let rel_p = pt - prev.center;
    let len = delta.len();
    assert!(len > 0.01);
    let inv_len = len.recip();

    let t = dot(rel_p, delta) * inv_len;
    let t_check = f32::max(0.0, (t - len * 0.5).abs() - len * 0.5 + w_mid) / w_mid;

    let d = cross(rel_p, delta).abs() * inv_len;
    let lin = d / (prev.r + t * inv_len * (self.r - prev.r));

    return sm_max(&[t_check, lin]);
  }
}

#[derive(Debug, Copy, Clone)]
pub enum Layout {
  Begin,
  Middle,
  End,
}

#[derive(Debug, Copy, Clone)]
pub enum AnyID {
  HoleID(HoleID),
  HoleArcID(HoleArcID, Layout),
  SlotID(SlotID, Layout),
  SlotArcID(SlotArcID, Layout),
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
pub struct FigureContour {
  positions: Vec<FigureContourPart>,
  kind: FigureContourKind,
}

impl FigureContour {
  pub fn contains(&self, point: Point) -> bool {
    if self.kind != FigureContourKind::Filled || self.positions.is_empty() {
      return false;
    }
    let mut c_in = 0;
    let mut c_out = 0;
    let mut prev = self.positions[self.positions.len() - 1].center - point;
    for i in 0..self.positions.len() {
      let cur = self.positions[i].center - point;
      if prev.y >= 0.0 && cur.y < 0.0 && cross(prev, cur) <= 0.0 {
        c_in += 1;
      } else if prev.y < 0.0 && cur.y >= 0.0 && cross(prev, cur) > 0.0 {
        c_out += 1;
      }
      prev = cur;
    }
    c_in != c_out
  }
}

#[derive(Debug)]
pub struct Figure {
  contours: Vec<FigureContour>,
  holes: Vec<HoleID>,
  hole_arcs: Vec<HoleArcID>,
  slots: Vec<SlotID>,
  slot_arcs: Vec<SlotArcID>,
  name: Option<String>,
  count: usize,
}

impl Figure {
  pub fn new(builder: &Builder, contours: &[FigureContourPreparingInfo]) -> Self {
    let mut holes = Vec::<HoleID>::new();
    let mut hole_arcs = Vec::<HoleArcID>::new();
    let mut slots = Vec::<SlotID>::new();
    let mut slot_arcs = Vec::<SlotArcID>::new();

    let contours = contours
      .iter()
      .map(|c| FigureContour {
        positions: c
          .positions
          .iter()
          .map(|&p| match p {
            AnyID::HoleID(h) => {
              holes.push(h);
              builder.get_hole(h).figure_contour_part()
            }
            AnyID::HoleArcID(h, layout) => {
              hole_arcs.push(h);
              builder.get_hole_arc(h).figure_contour_part(layout)
            }
            AnyID::SlotID(s, layout) => {
              slots.push(s);
              builder.get_slot(s).figure_contour_part(layout)
            }
            AnyID::SlotArcID(s, layout) => {
              slot_arcs.push(s);
              builder.get_slot_arc(s).figure_contour_part(layout)
            }
          })
          .collect(),
        kind: c.kind,
      })
      .collect();

    holes.sort_by_key(|h| h.0);
    holes.dedup_by_key(|h| h.0);
    hole_arcs.sort_by_key(|h| h.0);
    hole_arcs.dedup_by_key(|h| h.0);
    slots.sort_by_key(|s| s.0);
    slots.dedup_by_key(|s| s.0);
    slot_arcs.sort_by_key(|s| s.0);
    slot_arcs.dedup_by_key(|s| s.0);

    let name = None;
    let count = 1;

    Self { contours, holes, hole_arcs, slots, slot_arcs, name, count }
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

    for c in &self.contours {
      if c.contains(pt) {
        return true;
      }
    }

    let mut v = Vec::new();
    for &h in &self.holes {
      let h = builder.get_hole(h);
      v.push(h.border_func(pt));
    }
    for &ha in &self.hole_arcs {
      let h = builder.get_hole_arc(ha);
      v.push(h.border_func(pt));
    }
    for &s in &self.slots {
      let s = builder.get_slot(s);
      v.push(s.border_func(pt));
    }

    for c in &self.contours {
      let mut prev = c.positions.last().unwrap();
      let mut check_g = c.kind != FigureContourKind::Chain;
      for h in &c.positions {
        if check_g {
          let g = h.in_gauntlet(prev, pt);
          assert!(!g.is_nan());
          v.push(g);
        }
        prev = h;
        check_g = true;
      }
    }

    sm_min(&v) < 1.0
  }
}
