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
