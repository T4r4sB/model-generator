use winapi::shared::windef::*;
use winapi::um::errhandlingapi::*;
use winapi::um::libloaderapi::*;
use winapi::um::wingdi::*;
use winapi::um::winuser::*;

use crate::errors::*;
use crate::resources::*;
use common::matrix::*;
use common::model::*;
use common::points3d::*;

unsafe fn create_shader(shader: gl::types::GLenum, source: &[u8]) -> Result<Shader, String> {
  let shader = unsafe { Shader::new(shader) };
  unsafe { gl::ShaderSource(shader.get(), 1, [source.as_ptr().cast()].as_ptr(), std::ptr::null()) };
  unsafe { gl::CompileShader(shader.get()) };

  let max_length = 4096usize;
  let mut buf: Vec<u8> = vec![0u8; max_length];
  let mut length = 0;
  unsafe {
    gl::GetShaderInfoLog(shader.get(), max_length as i32, &mut length, buf.as_mut_ptr() as *mut _)
  };
  if length > 0 {
    let log = String::from_utf8_lossy(&buf[..length as usize]).to_string();
    return Err(log);
  }

  Ok(shader)
}

unsafe fn create_program(
  vertex_shader: Shader,
  fragment_shader: Shader,
) -> Result<Program, String> {
  let program = unsafe { Program::new() };

  unsafe { gl::AttachShader(program.get(), vertex_shader.get()) };
  unsafe { gl::AttachShader(program.get(), fragment_shader.get()) };
  unsafe { gl::LinkProgram(program.get()) };

  let max_length = 4096usize;
  let mut buf: Vec<u8> = vec![0u8; max_length];
  let mut length = 0;
  unsafe {
    gl::GetProgramInfoLog(program.get(), max_length as i32, &mut length, buf.as_mut_ptr() as *mut _)
  };
  if length > 0 {
    let log = String::from_utf8_lossy(&buf[..length as usize]).to_string();
    return Err(log);
  }
  Ok(program)
}

fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static std::ffi::CStr> {
  unsafe {
    let s = gl::GetString(variant);
    (!s.is_null()).then(|| std::ffi::CStr::from_ptr(s.cast()))
  }
}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 100
precision mediump float;

uniform mat4 proj;
uniform mat4 view;
uniform float shift;

attribute vec3 position;
attribute vec3 center;
attribute vec3 color;

varying vec3 v_color;
varying vec3 v_pos;
varying vec3 v_initial_pos;

void main() {
    v_initial_pos = position;
    gl_Position = proj * (view * vec4(position + center * shift, 1.0));
    v_color = color;
    v_pos = gl_Position.xyz;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 130
precision mediump float;

varying vec3 v_color;
varying vec3 v_pos;
varying vec3 v_initial_pos;

void main() {
    vec3 normal = normalize(cross(dFdy(v_pos), dFdx(v_pos)));

    float factor = max(0.0, min(1.0, (250.0 - v_pos.z) * 0.01));
    vec3 white = vec3(0.8, 0.8, 0.8);
    vec3 color = v_color;
    int odd_x = fract(v_initial_pos.x * 0.5) < 0.5 ? 1 : 0;
    int odd_y = fract(v_initial_pos.y * 0.5) < 0.5 ? 1 : 0;
    int odd_z = fract(v_initial_pos.z * 0.5) < 0.5 ? 1 : 0;
    int odd = odd_x + odd_y + odd_z;
    /*
    if (odd == 1 || odd == 3) {
        color.x = fract(color.x + 0.5);
        color.y = fract(color.y + 0.5);
        color.z = fract(color.z + 0.5);
    }*/
    float n_factor = 1.0 + (normal.z - 1.0) * 0.3;
    color = color * n_factor;
    color = white + (color - white) * factor;
    gl_FragColor = vec4(color, 1.0);
}
\0";

#[derive(Debug, Default)]
pub struct InputState {
  pub forward: bool,
  pub back: bool,
  pub left: bool,
  pub right: bool,
  pub up: bool,
  pub down: bool,
  pub rbutton: bool,
  pub prev_position: (i32, i32),
}

#[derive(Debug, Default)]
pub struct CameraPosition {
  pub position: Point,
  pub anglex: f32,
  pub anglez: f32,
}

pub struct GLData {
  pub gl_context: GLContext,
  pub program: Program,
  pub vao: VertexArray,
  pub vbo: Buffer,
  pub veo: Buffer,

  pub proj_matrix: Matrix,
  pub view_matrix: Matrix,
  pub shift: f32,
  pub index_count: usize,
  pub prev_time: std::time::Instant,
  pub input_state: InputState,
  pub camera_position: CameraPosition,
}

pub fn init_gl(hwnd: HWND, models: &mut dyn Iterator<Item = (u32, &Model)>) -> APIResult<(GLData)> {
  unsafe {
    let pfd: PIXELFORMATDESCRIPTOR = PIXELFORMATDESCRIPTOR {
      nSize: core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
      nVersion: 1,
      dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
      iPixelType: PFD_TYPE_RGBA,
      cColorBits: 32,
      cAlphaBits: 8,
      cDepthBits: 32,
      cRedBits: 0,
      cRedShift: 0,
      cGreenBits: 0,
      cGreenShift: 0,
      cBlueBits: 0,
      cBlueShift: 0,
      cAlphaShift: 0,
      cAccumBits: 0,
      cAccumRedBits: 0,
      cAccumGreenBits: 0,
      cAccumBlueBits: 0,
      cAccumAlphaBits: 0,
      cStencilBits: 0,
      cAuxBuffers: 0,
      iLayerType: 0,
      bReserved: 0,
      dwLayerMask: 0,
      dwVisibleMask: 0,
      dwDamageMask: 0,
    };

    let dc = crate::resources::HDC::new(hwnd)?;
    let pf_id: i32 = run_api!(ChoosePixelFormat(dc.get_dc(), &pfd))?;
    run_api!(SetPixelFormat(dc.get_dc(), pf_id, &pfd))?;
    let gl_context = GLContext::new(dc.get_dc())?;
    run_api!(wglMakeCurrent(dc.get_dc(), gl_context.get()))?;

    let handle = run_api!(LoadLibraryA("Opengl32.dll\0".as_ptr() as *const i8))?;

    let loader_callback = |symbol: &str| {
      let s = std::ffi::CString::new(symbol).expect("String contains an internal null byte");
      let p = run_api!(GetProcAddress(handle, s.as_ptr()) as *const _).unwrap_or(std::ptr::null());
      if p == std::ptr::null() {
        eprintln!("Fail to load {symbol}");
      }
      p
    };
    let wgl_loader_callback = |symbol: &str| {
      let s = std::ffi::CString::new(symbol).expect("String contains an internal null byte");
      let p = run_api!(wglGetProcAddress(s.as_ptr()) as *const _).unwrap_or(std::ptr::null());
      if p == std::ptr::null() {
        eprintln!("Fail to load {symbol}");
      }
      p
    };

    gl::Clear::load_with(loader_callback);
    gl::ClearColor::load_with(loader_callback);
    gl::CullFace::load_with(loader_callback);
    gl::DrawElements::load_with(loader_callback);
    gl::Enable::load_with(loader_callback);
    gl::GetString::load_with(loader_callback);
    gl::Viewport::load_with(loader_callback);

    gl::AttachShader::load_with(wgl_loader_callback);
    gl::BindBuffer::load_with(wgl_loader_callback);
    gl::BindVertexArray::load_with(wgl_loader_callback);
    gl::BufferData::load_with(wgl_loader_callback);
    gl::CreateProgram::load_with(wgl_loader_callback);
    gl::CreateShader::load_with(wgl_loader_callback);
    gl::CompileShader::load_with(wgl_loader_callback);
    gl::DeleteBuffers::load_with(wgl_loader_callback);
    gl::DeleteProgram::load_with(wgl_loader_callback);
    gl::DeleteShader::load_with(wgl_loader_callback);
    gl::DeleteVertexArrays::load_with(wgl_loader_callback);
    gl::EnableVertexAttribArray::load_with(wgl_loader_callback);
    gl::GetAttribLocation::load_with(wgl_loader_callback);
    gl::GenBuffers::load_with(wgl_loader_callback);
    gl::GenVertexArrays::load_with(wgl_loader_callback);
    gl::GetProgramInfoLog::load_with(wgl_loader_callback);
    gl::GetShaderInfoLog::load_with(wgl_loader_callback);
    gl::GetUniformLocation::load_with(wgl_loader_callback);
    gl::LinkProgram::load_with(wgl_loader_callback);
    gl::ShaderSource::load_with(wgl_loader_callback);
    gl::Uniform1f::load_with(wgl_loader_callback);
    gl::GetUniformLocation::load_with(wgl_loader_callback);
    gl::UniformMatrix4fv::load_with(wgl_loader_callback);
    gl::UseProgram::load_with(wgl_loader_callback);
    gl::VertexAttribPointer::load_with(wgl_loader_callback);

    if let Some(renderer) = get_gl_string(gl::RENDERER) {
      println!("Running on {}", renderer.to_string_lossy());
    }
    if let Some(version) = get_gl_string(gl::VERSION) {
      println!("OpenGL Version {}", version.to_string_lossy());
    }
    if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
      println!("Shaders version on {}", shaders_version.to_string_lossy());
    }

    let vertex_shader = create_shader(gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE).map_err(|e| {
      eprintln!("Failed to create vertex shader: {e}");
      APIResultCode::user()
    })?;
    let fragment_shader =
      create_shader(gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE).map_err(|e| {
        eprintln!("Failed to create fragment shader: {e}");
        APIResultCode::user()
      })?;
    let program = create_program(vertex_shader, fragment_shader).map_err(|e| {
      eprintln!("Failed to create program: {e}");
      APIResultCode::user()
    })?;
    let vao = VertexArray::new();
    let vbo = Buffer::new();
    let veo = Buffer::new();

    let mut array_buffer = ArrayBuffer::default();
    while let Some((color, m)) = models.next() {
      m.write_to_buffer(&mut array_buffer, color);
    }

    println!(
      "models written to big buffer size {} vertices, {} indices",
      array_buffer.v.len(),
      array_buffer.i.len(),
    );

    gl::BindVertexArray(vao.get());
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo.get());
    gl::BufferData(
      gl::ARRAY_BUFFER,
      (array_buffer.v.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
      array_buffer.v.as_ptr() as *const _,
      gl::STATIC_DRAW,
    );

    let pos_attrib =
      gl::GetAttribLocation(program.get(), b"position\0".as_ptr() as *const _) as gl::types::GLuint;
    let center_attrib =
      gl::GetAttribLocation(program.get(), b"center\0".as_ptr() as *const _) as gl::types::GLuint;
    let color_attrib =
      gl::GetAttribLocation(program.get(), b"color\0".as_ptr() as *const _) as gl::types::GLuint;
    let f32_size = std::mem::size_of::<f32>() as gl::types::GLsizei;
    gl::VertexAttribPointer(pos_attrib, 3, gl::FLOAT, 0, 9 * f32_size, std::ptr::null());
    gl::VertexAttribPointer(center_attrib, 3, gl::FLOAT, 0, 9 * f32_size, (3 * f32_size) as _);
    gl::VertexAttribPointer(color_attrib, 3, gl::FLOAT, 0, 9 * f32_size, (6 * f32_size) as _);

    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, veo.get());
    gl::BufferData(
      gl::ELEMENT_ARRAY_BUFFER,
      (array_buffer.i.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
      array_buffer.i.as_ptr() as *const _,
      gl::STATIC_DRAW,
    );
    gl::EnableVertexAttribArray(pos_attrib);
    gl::EnableVertexAttribArray(center_attrib);
    gl::EnableVertexAttribArray(color_attrib);

    gl::Enable(gl::DEPTH_TEST);
    gl::Enable(gl::CULL_FACE);
    gl::CullFace(gl::BACK);

    let mut camera_position = CameraPosition::default();
    camera_position.position.z = 190.0;

    let mut rect = RECT { left: 0, top: 0, right: 100, bottom: 100 };
    run_api!(GetClientRect(hwnd, &mut rect))?;
    let aspect = (rect.right - rect.left) as f32 / std::cmp::max(1, rect.bottom - rect.top) as f32;

    let proj_matrix = Matrix::new_proj(std::f32::consts::FRAC_PI_8, aspect, 1000.0, 0.01);
    let view_matrix = Matrix::new_view(Point { x: 0.0, y: 0.0, z: 1.0 }, 0.0, 0.0);
    let shift = 0.0;
    let index_count = array_buffer.i.len();
    let prev_time = std::time::Instant::now();
    let input_state = InputState::default();

    Ok(GLData {
      gl_context,
      program,
      vao,
      vbo,
      veo,
      proj_matrix,
      view_matrix,
      shift,
      index_count,
      prev_time,
      input_state,
      camera_position,
    })
  }
}

pub fn gl_update_and_draw(context: &mut crate::gl_window::Context) {
  unsafe {
    let gl_data = context.gl_data.as_mut().unwrap();

    gl::UseProgram(gl_data.program.get());
    let proj_matrix_location =
      gl::GetUniformLocation(gl_data.program.get(), b"proj\0".as_ptr() as *const _);
    gl::UniformMatrix4fv(
      proj_matrix_location,
      1,
      gl::FALSE,
      gl_data.proj_matrix.as_ptr() as *const _,
    );

    let current_time = std::time::Instant::now();
    let dt = (current_time - gl_data.prev_time).as_secs_f32();
    gl_data.prev_time = current_time;
    let delta = dt * 100.0;

    if gl_data.input_state.rbutton {
      let mut cursor = POINT { x: 0, y: 0 };
      GetPhysicalCursorPos(&mut cursor);
      let cursor_delta = (
        cursor.x - gl_data.input_state.prev_position.0,
        cursor.y - gl_data.input_state.prev_position.1,
      );

      gl_data.camera_position.anglez += cursor_delta.0 as f32 * 0.001;
      gl_data.camera_position.anglex -= cursor_delta.1 as f32 * 0.001;
      gl_data.input_state.prev_position = (cursor.x, cursor.y);
    }

    if gl_data.input_state.forward {
      gl_data.camera_position.position += Point {
        x: -delta * gl_data.camera_position.anglez.sin(),
        y: 0.0,
        z: -delta * gl_data.camera_position.anglez.cos(),
      };
    }

    if gl_data.input_state.back {
      gl_data.camera_position.position += Point {
        x: delta * gl_data.camera_position.anglez.sin(),
        y: 0.0,
        z: delta * gl_data.camera_position.anglez.cos(),
      };
    }

    if gl_data.input_state.left {
      gl_data.camera_position.position += Point {
        x: delta * gl_data.camera_position.anglez.cos(),
        y: 0.0,
        z: -delta * gl_data.camera_position.anglez.sin(),
      };
    }

    if gl_data.input_state.right {
      gl_data.camera_position.position += Point {
        x: -delta * gl_data.camera_position.anglez.cos(),
        y: 0.0,
        z: delta * gl_data.camera_position.anglez.sin(),
      };
    }

    if gl_data.input_state.up {
      gl_data.camera_position.position += Point { x: 0.0, y: -delta, z: 0.0 };
    }

    if gl_data.input_state.down {
      gl_data.camera_position.position += Point { x: 0.0, y: delta, z: 0.0 };
    }

    let position = gl_data.camera_position.position;

    let angle = gl_data.camera_position.anglez;
    let angle_x = gl_data.camera_position.anglex;
    gl_data.view_matrix = Matrix::new_view(position, angle, angle_x);

    let view_matrix_location =
      gl::GetUniformLocation(gl_data.program.get(), b"view\0".as_ptr() as *const _);
    gl::UniformMatrix4fv(
      view_matrix_location,
      1,
      gl::FALSE,
      gl_data.view_matrix.as_ptr() as *const _,
    );

    let shift_location =
      gl::GetUniformLocation(gl_data.program.get(), b"shift\0".as_ptr() as *const _);
    gl::Uniform1f(shift_location, gl_data.shift);

    gl::ClearColor(1.0, 1.0, 1.0, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    gl::DrawElements(gl::TRIANGLES, gl_data.index_count as i32, gl::UNSIGNED_INT, std::ptr::null());
  }
}
