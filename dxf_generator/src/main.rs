#![allow(unused)]

use common::contour::*;
use common::points2d::*;
use rand::Rng;
use rand::SeedableRng;
use std::io::Write;
use std::time::Instant;

use fxhash::FxHashMap;

mod brake_tool_creator;
type PartCreator = brake_tool_creator::BrakeToolCreator;

pub struct ImgBuffer {
  v: Vec<u8>,
  size_x: usize,
  size_y: usize,
}

impl ImgBuffer {
  pub fn new(size_x: usize, size_y: usize) -> Self {
    Self { v: vec![255u8; size_x * size_y * 4], size_x, size_y }
  }

  pub fn put(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
    let index = (x + y * self.size_x) * 4;
    self.v[index] = color.0;
    self.v[index + 1] = color.1;
    self.v[index + 2] = color.2;
  }

  pub fn save(&self, filename: &str) -> Result<(), String> {
    let path = std::path::Path::new(filename);
    let file = std::fs::File::create(path)
      .map_err(|e| format!("fail to create file {filename} because of {e}"))?;
    let ref mut w = std::io::BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, self.size_x as u32, self.size_y as u32); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
      .write_header()
      .map_err(|e| format!("fail to write header to {filename} because of {e}"))?;
    writer
      .write_image_data(&self.v)
      .map_err(|e| format!("fail to write image data to {filename} because of {e}"))?;
    Ok(())
  }
}

struct ColoredContourSet {
  contours: ContourSet,
  basic_color: (u8, u8, u8),
  aabb: Vec<AABB>,
}

struct MinDist<T> {
  dist: f32,
  data: T,
}

impl<T> MinDist<T> {
  fn apply(dst: &mut Option<Self>, dist: f32, data: T) {
    match dst {
      None => *dst = Some(Self { dist, data }),
      Some(Self { dist: old_dist, data: _ }) => {
        if *old_dist >= 0.0 && dist < *old_dist || dist <= 0.0 && dist > *old_dist {
          *dst = Some(Self { dist, data })
        }
      }
    }
  }

  fn perfect(data: T) -> Option<Self> {
    Some(Self { dist: 0.0, data })
  }
}

impl ColoredContourSet {
  fn load(path: &std::path::Path, basic_color: (u8, u8, u8), shift: Point, angle: f32) -> Self {
    let mut contours = ContourSet::load_from_dxf(path).unwrap();
    contours.rotate(angle);
    contours.translate(shift);
    let mut aabb = Vec::new();
    for part in &contours.parts {
      aabb.push(part.get_aabb(&contours.points));
    }

    Self { contours, basic_color, aabb }
  }

  fn dist(&self, treshhold: f32, p: Point) -> Option<MinDist<usize>> {
    let mut result = None;
    for (i, part) in self.contours.parts.iter().enumerate() {
      if self.aabb[i].rounded(treshhold).contains(p) {
        let inside = part.contains(&self.contours.points, p);
        for c in &part.contours {
          let mut prev = self.contours.points[*c.points.last().unwrap() as usize];
          for &pi in &c.points {
            let next = self.contours.points[pi as usize];
            let dist = dist_pl(p, prev, next);
            let dist = if inside { -dist } else { dist };
            MinDist::apply(&mut result, dist, i);
            prev = next;
          }
        }
      }
    }
    result
  }
}

struct FillPattern {
  v: Vec<u8>,
  start_x: usize,
  start_y: usize,
  size_x: usize,
  size_y: usize,
}

impl FillPattern {
  pub fn new(start_x: usize, start_y: usize, size_x: usize, size_y: usize) -> Self {
    Self { v: vec![255u8; size_x * size_y], start_x, start_y, size_x, size_y }
  }

  pub fn put(&mut self, x: usize, y: usize, value: u8) {
    let index = (x + y * self.size_x);
    self.v[index] = value;
  }

  pub fn get_rel(&self, x: usize, y: usize) -> u8 {
    let index = (x - self.start_x + (y - self.start_y) * self.size_x);
    self.v[index]
  }

  pub fn make_cbble(&mut self, rng: &mut impl Rng) {
    let cnt = std::cmp::max(1, self.size_x * self.size_y / 400);
    let mut core_points = Vec::new();
    for _ in 0..cnt {
      let pt = Point {
        x: rng.random_range(0.0..self.size_x as f32 + 1.0),
        y: rng.random_range(0.0..self.size_y as f32 + 1.0),
      };
      core_points.push(pt);
    }
    for y in 0..self.size_y {
      for x in 0..self.size_x {
        let p = Point { x: x as f32, y: y as f32 };
        let mut d1 = f32::INFINITY;
        let mut d2 = f32::INFINITY;
        for &c in &core_points {
          let d = (p - c).len();
          if d < d1 {
            d2 = d1;
            d1 = d;
          } else if d < d2 {
            d2 = d;
          }
        }
        let d = (d1 - d2).abs();
        self.put(x, y, f32::min(255.0, d * 127.0) as u8);
      }
    }
  }

  pub fn make_waves(&mut self, rng: &mut impl Rng) {
    let cnt = std::cmp::max(1, self.size_x * self.size_y / 400);
    let mut core_points = Vec::new();
    for _ in 0..cnt {
      let pt = Point {
        x: rng.random_range(0.0..self.size_x as f32 + 1.0),
        y: rng.random_range(0.0..self.size_y as f32 + 1.0),
      };
      core_points.push(pt);
    }
    for y in 0..self.size_y {
      for x in 0..self.size_x {
        let p = Point { x: x as f32, y: y as f32 };
        let mut sum = 0.0;

        for &c in &core_points {
          let d = (p - c).len() / 10.0;
          sum += d.cos();
        }
        sum /= cnt as f32;
        self.put(x, y, f32::clamp(sum * 255.0 + 240.0, 224.0, 255.0) as u8);
      }
    }
  }
}

struct WindowSize {
  x1: usize,
  y1: usize,
  x2: usize,
  y2: usize,
}

impl WindowSize {
  fn new(x: usize, y: usize) -> Self {
    Self { x1: x, y1: y, x2: x + 1, y2: y + 1 }
  }

  fn apply(&mut self, x: usize, y: usize) {
    self.x1 = std::cmp::min(self.x1, x);
    self.y1 = std::cmp::min(self.y1, y);
    self.x2 = std::cmp::max(self.x2, x + 1);
    self.y2 = std::cmp::max(self.y2, y + 1);
  }

  fn create_fill_pattern(&self) -> FillPattern {
    FillPattern::new(self.x1, self.y1, self.x2 - self.x1, self.y2 - self.y1)
  }
}

fn create_mockup() {
  let contours_paths = std::path::Path::new("D:\\")
    .join("RustProj")
    .join("model_generator")
    .join("stl_generator")
    .join("contours");
  let cb_path = contours_paths.join("CobblestoneSkewb");
  let sd_path = contours_paths.join("Sixdecaminx");

  let a = [
    ColoredContourSet::load(
      &cb_path.join("WHITE.dxf"),
      (255, 255, 255),
      Point { x: -160.0, y: -100.0 },
      0.0,
    ),
    ColoredContourSet::load(
      &cb_path.join("RED.dxf"),
      (255, 0, 0),
      Point { x: -96.0, y: -100.0 },
      0.0,
    ),
    ColoredContourSet::load(
      &cb_path.join("BLUE.dxf"),
      (0, 128, 255),
      Point { x: -32.0, y: -100.0 },
      0.0,
    ),
    ColoredContourSet::load(
      &cb_path.join("GREEN.dxf"),
      (0, 255, 0),
      Point { x: 32.0, y: -100.0 },
      0.0,
    ),
    ColoredContourSet::load(
      &cb_path.join("ORANGE.dxf"),
      (255, 128, 0),
      Point { x: 96.0, y: -100.0 },
      0.0,
    ),
    ColoredContourSet::load(
      &cb_path.join("YELLOW.dxf"),
      (255, 255, 0),
      Point { x: 160.0, y: -100.0 },
      0.0,
    ),
    ColoredContourSet::load(
      &sd_path.join("x4.dxf"),
      (255, 255, 255),
      Point { x: -160.0, y: -40.0 },
      -35.0 * std::f32::consts::PI / 180.0,
    ),
    ColoredContourSet::load(
      &sd_path.join("x4.dxf"),
      (192, 192, 192),
      Point { x: -120.0, y: -40.0 },
      145.0 * std::f32::consts::PI / 180.0,
    ),
    ColoredContourSet::load(
      &sd_path.join("x4.dxf"),
      (255, 255, 0),
      Point { x: -70.0, y: -40.0 },
      -35.0 * std::f32::consts::PI / 180.0,
    ),
    ColoredContourSet::load(
      &sd_path.join("x4.dxf"),
      (96, 48, 0),
      Point { x: -30.0, y: -40.0 },
      145.0 * std::f32::consts::PI / 180.0,
    ),
    ColoredContourSet::load(
      &sd_path.join("x12.dxf"),
      (0, 255, 0),
      Point { x: 20.0, y: -40.0 },
      -45.0 * std::f32::consts::PI / 180.0,
    ),
    
    ColoredContourSet::load(
      &sd_path.join("x12.dxf"),
      (0, 255, 224),
      Point { x: 60.0, y: -40.0 },
      135.0 * std::f32::consts::PI / 180.0,
    ),
    ColoredContourSet::load(
      &sd_path.join("x12.dxf"),
      (0, 96, 96),
      Point { x: 100.0, y: -40.0 },
      -45.0 * std::f32::consts::PI / 180.0,
    ),
    ColoredContourSet::load(
      &sd_path.join("x12.dxf"),
      (192, 255, 192),
      Point { x: 140.0, y: -40.0 },
      135.0 * std::f32::consts::PI / 180.0,
    ),
    
    ColoredContourSet::load(
      &sd_path.join("x12.dxf"),
      (255, 128, 0),
      Point { x: -160.0, y: 15.0 },
      -45.0 * std::f32::consts::PI / 180.0,
    ),
    ColoredContourSet::load(
      &sd_path.join("x12.dxf"),
      (255, 0, 0),
      Point { x: -120.0, y: 15.0 },
      135.0 * std::f32::consts::PI / 180.0,
    ),
    
    ColoredContourSet::load(
      &sd_path.join("x12.dxf"),
      (0, 64, 255),
      Point { x: -80.0, y: 15.0 },
      -45.0 * std::f32::consts::PI / 180.0,
    ),
    ColoredContourSet::load(
      &sd_path.join("x12.dxf"),
      (0, 192, 192),
      Point { x: -40.0, y: 15.0 },
      135.0 * std::f32::consts::PI / 180.0,
    ),
    ColoredContourSet::load(
      &sd_path.join("x12.dxf"),
      (128, 192, 255),
      Point { x: 0.0, y: 15.0 },
      -45.0 * std::f32::consts::PI / 180.0,
    ),
    ColoredContourSet::load(
      &sd_path.join("x12.dxf"),
      (255, 255, 224),
      Point { x: 40.0, y: 15.0 },
      -45.0 * std::f32::consts::PI / 180.0,
    ),
    ColoredContourSet::load(
      &sd_path.join("x12.dxf"),
      (255, 128, 255),
      Point { x: 80.0, y: 15.0 },
      135.0 * std::f32::consts::PI / 180.0,
    ),
    ColoredContourSet::load(
      &sd_path.join("x12.dxf"),
      (128, 0, 255),
      Point { x: 120.0, y: 15.0 },
      -45.0 * std::f32::consts::PI / 180.0,
    ),
  ];

  let mut img = ImgBuffer::new(5000, 3500);
  let mut sizes = FxHashMap::default();

  let mut dists = Vec::new();

  for iy in 0..img.size_y {
    let mut row = Vec::new();
    let y = (img.size_y as f32 * 0.5 - iy as f32) / 300.0 * 25.4;
    for ix in 0..img.size_x {
      let x = (ix as f32 - img.size_x as f32 * 0.5) as f32 / 300.0 * 25.4;
      let pt = Point { x, y };
      let mut dist = None;
      for (i, a) in a.iter().enumerate() {
        if let Some(cd) = a.dist(1.0, pt) {
          sizes.entry((i, cd.data)).or_insert_with(|| WindowSize::new(ix, iy)).apply(ix, iy);
          MinDist::apply(&mut dist, cd.dist, (i, cd.data));
        }
      }
      row.push(dist);
    }
    dists.push(row);
    println!("dists: processed {iy} of {}", img.size_y);
  }

  let mut fills: FxHashMap<_, FillPattern> =
    sizes.into_iter().map(|(key, v)| (key, v.create_fill_pattern())).collect();

  let mut rng = rand::rngs::StdRng::from_os_rng();

  let count = fills.len();
  for (i, (key, value)) in fills.iter_mut().enumerate() {
    if key.0 < 6 {
      value.make_cbble(&mut rng);
    } else {
      value.make_waves(&mut rng);
    }
    println!("fill images: processed {i} of {count}");
  }

  for iy in 0..img.size_y {
    for ix in 0..img.size_x {
      if let Some(pd) = &dists[iy][ix] {
        if pd.dist < 1.0 {
          let fill = fills.get(&pd.data).unwrap();
          let mut color = a[pd.data.0].basic_color;
          let factor = if pd.dist > -0.5 { 192 } else { fill.get_rel(ix, iy) };

          if pd.data.0 < 6 {
            color.0 = (color.0 as i32 * factor as i32 / 255) as u8;
            color.1 = (color.1 as i32 * factor as i32 / 255) as u8;
            color.2 = (color.2 as i32 * factor as i32 / 255) as u8;
          } else {
            color.0 -= std::cmp::min(color.0, 255 - factor);
            color.1 -= std::cmp::min(color.1, 255 - factor);
            color.2 -= std::cmp::min(color.2, 255 - factor);
          }
          img.put(ix, iy, color)
        }
      }
    }
    println!("output: processed {iy} of {}", img.size_y);
  }

  img.save("common.png").unwrap();

  let mut common = ContourSet::new();
  for a in a {
    common.append(a.contours);
  }
  common.save_to_dxf(std::path::Path::new("common.dxf")).unwrap();
}

fn main() {
  let start = Instant::now();
  let part_creator = PartCreator::new();

  let mut total_length = 0.0;
  let mut total_square = 0.0;

  let mut look_together = ContourSet::new();

  for i in 0..part_creator.faces() {
    let aabb = part_creator.aabb(i).unwrap_or(AABB::around_zero(200.0));

    let mut cc = ContourCreator::new(aabb, 0.2, 20);

    let name = part_creator.get_name(i).map(|s| s.to_string()).unwrap_or(format!("part_{i}"));
    print!("generate {name} in aabb {:?}...", aabb);
    std::io::stdout().flush().unwrap();

    let contours = cc.make_contour(&|p| part_creator.get_sticker_index(p, i));
    let h = part_creator.get_height(i);

    let thickness = h;
    let count = part_creator.get_count(i);
    let full_name = format!("(THICK={thickness}, AMOUNT={count}) {name}");

    let single_i = contours.len() == 1;
    for (index, mut cc) in contours {
      cc.optimize(0.001);
      cc.remove_trash();

      if name == "gear1" || name == "gear2" {
        println!("append together");
        look_together.append(cc.clone());
      }

      let full_name = if single_i { full_name.clone() } else { format!("{full_name}_{index}") };

      let square = cc.get_square();
      let length = cc.get_length();

      total_length += length * count as f32;
      total_square += square * count as f32;

      println!(
        "\rsave {full_name} ({} points, {square} square, {length} length) to dxf...",
        cc.points_count()
      );
      if let Err(msg) =
        cc.save_to_dxf(&std::path::Path::new("contours").join(format!("{full_name}.dxf")))
      {
        println!("{}", msg);
      }

      let ex = cc.extrude(h);

      let single_j = ex.len() == 1;
      for j in 0..ex.len() {
        let full_name = if single_j { full_name.clone() } else { format!("{full_name}_{j}") };

        if let Err(msg) =
          ex[j].save_to_stl(&std::path::Path::new("extruded").join(format!("{full_name}.stl")))
        {
          println!("{}", msg);
        }
      }
    }
  }

  let _ = look_together.save_to_dxf(&std::path::Path::new("contours").join(("TOGETHER.dxf")));

  println!("total {total_length} length, {total_square} square");
  println!("time {}", start.elapsed().as_millis() as f32 / 1000.0);
}
