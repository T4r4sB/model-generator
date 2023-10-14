use winit::event_loop::EventLoopBuilder;

use std::ffi::{CStr, CString};
use std::num::NonZeroU32;
use std::ops::Deref;

use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

use raw_window_handle::HasRawWindowHandle;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::SwapInterval;

use winit::dpi::PhysicalPosition;
use winit::event::ElementState::Pressed;

use glutin_winit::{self, DisplayBuilder, GlWindow};

mod points3d;
use crate::points3d::*;
mod matrix;
use crate::matrix::*;
mod model;
use crate::model::*;
mod solid;
use crate::solid::*;
mod part_creator;

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

    pub use Gles2 as Gl;
}

pub fn gl_main(event_loop: winit::event_loop::EventLoop<()>) {
    // Only windows requires the window to be present before creating the display.
    // Other platforms don't really need one.
    //
    // XXX if you don't care about running on android or so you can safely remove
    // this condition and always pass the window builder.
    let window_builder = if cfg!(wgl_backend) {
        Some(WindowBuilder::new().with_transparent(true))
    } else {
        None
    };

    // The template will match only the configurations supporting rendering
    // to windows.
    //
    // XXX We force transparency only on macOS, given that EGL on X11 doesn't
    // have it, but we still want to show window. The macOS situation is like
    // that, because we can query only one config at a time on it, but all
    // normal platforms will return multiple configs, so we can find the config
    // with transparency ourselves inside the `reduce`.
    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(cfg!(cgl_backend));

    let display_builder = DisplayBuilder::new().with_window_builder(window_builder);

    let (mut window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            // Find the config with the maximum number of samples, so our triangle will
            // be smooth.
            configs
                .reduce(|accum, config| {
                    let transparency_check = config.supports_transparency().unwrap_or(false)
                        & !accum.supports_transparency().unwrap_or(false);

                    if transparency_check || config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .unwrap();

    println!("Picked a config with {} samples", gl_config.num_samples());

    let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());

    // XXX The display could be obtained from the any object created by it, so we
    // can query it from the config.
    let gl_display = gl_config.display();

    // The context creation part. It can be created before surface and that's how
    // it's expected in multithreaded + multiwindow operation mode, since you
    // can send NotCurrentContext, but not Surface.
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

    // Since glutin by default tries to create OpenGL core context, which may not be
    // present we should try gles.
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);

    // There are also some old devices that support neither modern OpenGL nor GLES.
    // To support these we can try and create a 2.1 context.
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
        .build(raw_window_handle);

    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(&gl_config, &fallback_context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(&gl_config, &legacy_context_attributes)
                            .expect("failed to create context")
                    })
            })
    });

    let mut state = None;
    let mut renderer = None;
    event_loop.run(move |event, window_target, control_flow| {
        // control_flow.set_wait();
        match event {
            Event::Resumed => {
                #[cfg(android_platform)]
                println!("Android window available");

                let window = window.take().unwrap_or_else(|| {
                    let window_builder = WindowBuilder::new().with_transparent(true);
                    glutin_winit::finalize_window(window_target, window_builder, &gl_config)
                        .unwrap()
                });

                let attrs = window.build_surface_attributes(<_>::default());
                let gl_surface = unsafe {
                    gl_config
                        .display()
                        .create_window_surface(&gl_config, &attrs)
                        .unwrap()
                };

                // Make it current.
                let gl_context = not_current_gl_context
                    .take()
                    .unwrap()
                    .make_current(&gl_surface)
                    .unwrap();

                // The context needs to be current for the Renderer to set up shaders and
                // buffers. It also performs function loading, which needs a current context on
                // WGL.
                renderer.get_or_insert_with(|| Renderer::new(&gl_display));

                // Try setting vsync.
                if let Err(res) = gl_surface
                    .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                {
                    eprintln!("Error setting vsync: {res:?}");
                }

                assert!(state.replace((gl_context, gl_surface, window)).is_none());
            }
            Event::Suspended => {
                // This event is only raised on Android, where the backing NativeWindow for a GL
                // Surface can appear and disappear at any moment.
                println!("Android window removed");

                // Destroy the GL Surface and un-current the GL Context before ndk-glue releases
                // the window back to the system.
                let (gl_context, ..) = state.take().unwrap();
                assert!(not_current_gl_context
                    .replace(gl_context.make_not_current().unwrap())
                    .is_none());
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        // Some platforms like EGL require resizing GL surface to update the size
                        // Notable platforms here are Wayland and macOS, other don't require it
                        // and the function is no-op, but it's wise to resize it for portability
                        // reasons.
                        if let Some((gl_context, gl_surface, _)) = &state {
                            gl_surface.resize(
                                gl_context,
                                NonZeroU32::new(size.width).unwrap(),
                                NonZeroU32::new(size.height).unwrap(),
                            );
                            let renderer = renderer.as_mut().unwrap();
                            renderer.resize(size.width as i32, size.height as i32);
                        }
                    }
                }
                WindowEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                } => {
                    let renderer = renderer.as_mut().unwrap();
                    match input.virtual_keycode {
                        Some(winit::event::VirtualKeyCode::W) => {
                            renderer.forward(input.state == Pressed)
                        }
                        Some(winit::event::VirtualKeyCode::S) => {
                            renderer.back(input.state == Pressed)
                        }
                        Some(winit::event::VirtualKeyCode::A) => {
                            renderer.left(input.state == Pressed)
                        }
                        Some(winit::event::VirtualKeyCode::D) => {
                            renderer.right(input.state == Pressed)
                        }
                        Some(winit::event::VirtualKeyCode::LControl) => {
                            renderer.down(input.state == Pressed)
                        }
                        Some(winit::event::VirtualKeyCode::Space) => {
                            renderer.up(input.state == Pressed)
                        }
                        _ => {}
                    }
                }
                WindowEvent::MouseInput {
                    device_id,
                    state,
                    button,
                    modifiers: _,
                } => {
                    let renderer = renderer.as_mut().unwrap();
                    match button {
                        winit::event::MouseButton::Left => {
                            renderer.lbutton(state == Pressed);
                        }
                        winit::event::MouseButton::Right => {
                            renderer.rbutton(state == Pressed);
                        }
                        _ => {}
                    }
                }
                WindowEvent::CursorMoved {
                    device_id,
                    position,
                    modifiers: _,
                } => {
                    let renderer = renderer.as_mut().unwrap();
                    renderer.position(position);
                }
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                }
                _ => (),
            },
            Event::RedrawEventsCleared => {
                if let Some((gl_context, gl_surface, window)) = &state {
                    let renderer = renderer.as_mut().unwrap();
                    renderer.draw();
                    window.request_redraw();

                    gl_surface.swap_buffers(gl_context).unwrap();
                }
            }
            _ => (),
        }
    })
}

#[derive(Debug, Default)]
pub struct InputState {
    forward: bool,
    back: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    lbutton: bool,
    rbutton: bool,
    prev_position: PhysicalPosition<f64>,
}

#[derive(Debug, Default)]
pub struct CameraPosition {
    position: Point,
    anglex: f32,
    anglez: f32,
}

pub struct Renderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    veo: gl::types::GLuint,
    proj_matrix: Matrix,
    view_matrix: Matrix,
    index_count: usize,
    prev_time: std::time::Instant,
    input_state: InputState,
    camera_position: CameraPosition,
    gl: gl::Gl,
}

impl Renderer {
    pub fn new<D: GlDisplay>(gl_display: &D) -> Self {
        unsafe {
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            gl.Enable(gl::DEPTH_TEST);

            gl.Enable(gl::CULL_FACE);
            gl.CullFace(gl::FRONT);

            if let Some(renderer) = get_gl_string(&gl, gl::RENDERER) {
                println!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(&gl, gl::VERSION) {
                println!("OpenGL Version {}", version.to_string_lossy());
            }

            if let Some(shaders_version) = get_gl_string(&gl, gl::SHADING_LANGUAGE_VERSION) {
                println!("Shaders version on {}", shaders_version.to_string_lossy());
            }

            let mut length = 0;
            let max_length = 4096usize;
            let mut buf = vec![0u8; max_length];

            let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE);

            gl.GetShaderInfoLog(
                vertex_shader,
                max_length as i32,
                &mut length,
                buf.as_mut_ptr() as *mut _,
            );
            let log = String::from_utf8_lossy(&buf[..length as usize]);
            println!("vertex shader log = {}", log);

            let fragment_shader = create_shader(&gl, gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);

            gl.GetShaderInfoLog(
                fragment_shader,
                max_length as i32,
                &mut length,
                buf.as_mut_ptr() as *mut _,
            );
            let log = String::from_utf8_lossy(&buf[..length as usize]);
            println!("fragment shader log = {}", log);

            let program = gl.CreateProgram();
            gl.AttachShader(program, vertex_shader);
            gl.AttachShader(program, fragment_shader);
            gl.LinkProgram(program);

            gl.GetProgramInfoLog(
                program,
                max_length as i32,
                &mut length,
                buf.as_mut_ptr() as *mut _,
            );
            let log = String::from_utf8_lossy(&buf[..length as usize]);
            println!("program log = {}", log);

            gl.UseProgram(program);

            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            let mut vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut vao);

            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);

            let mut veo = std::mem::zeroed();
            gl.GenBuffers(1, &mut veo);

            let part_creator = part_creator::PartCreator::new();
            let part_func = &|p| part_creator.get_part_index(p);

            //let part_func = &|p: Point| ((p - Point{x: -0.0, y:0.0, z:0.0}).len() < 35.0 ) as u32;
            //let part_func = &|p: Point| (p.len() < 35.0 && p.len() > 20.0 && (p.y*p.y + p.z*p.z).sqrt() > 15.0) as u32;

            //let part_func =
            //    &|p: Point| (p.x.abs() < 34.9 && p.y.abs() < 34.9 && p.z.abs() < 34.9) as u32;

            //let part_func =
            //    &|p: Point| (p.x.abs() < 34.999 && (p.y*p.y + p.z*p.z).sqrt() < 15.0) as u32;

            println!("usize={}", std::mem::size_of::<usize>());

            let start = std::time::Instant::now();
            let mut mc = ModelCreator::new(64, 70.0, 20, part_func);
            let width = 0.05;
            while !mc.finished() {
                mc.fill_next_layer(part_func);
            }

            let end_layers = std::time::Instant::now();

            let mut max_v = 0;
            let mut sum_v = 0;

            let mut max_v_after = 0;
            let mut sum_v_after = 0;

            let mut models = mc.get_models();
           
            for (&m_index, m) in &mut models {
                sum_v += m.vertices.len();
                max_v = std::cmp::max(max_v, m.vertices.len());
                m.validate_and_delete_small_groups();
                let smooth_cnt = 0;
                for i in 0..smooth_cnt {
                    m.smooth(0.1);
                    println!("make model smooth, progress [{i}/{smooth_cnt}]");
                }
                println!("tcount before = {}", m.triangles.len());
                //m.optimize(width, 0.9999, 10, 0.9);
                println!("tcount after {}", m.triangles.len());
                m.delete_unused_v();
                //m.out_of_center(1.0);

                sum_v_after += m.vertices.len();
                max_v_after = std::cmp::max(max_v_after, m.vertices.len());

                println!(
                    "save {m_index} to stl... {} vertices {} triangles",
                    m.vertices.len(),
                    m.triangles.len()
                );
                if let Err(msg) =
                    m.save_to_stl(std::path::Path::new(&format!("output_{}.stl", m_index)))
                {
                    println!("{}", msg);
                }
            }

            let end_opt = std::time::Instant::now();

            println!(
                "models created, sum_v={}, max_v={}, after: sum_v={}, max_v={}",
                sum_v, max_v, sum_v_after, max_v_after
            );

            println!(
                "layers time: {:?}, opt time: {:?}",
                end_layers - start,
                end_opt - end_layers
            );

            let mut array_buffer = ArrayBuffer::default();
            for (m_index, m) in &models {
                m.write_to_buffer(&mut array_buffer, (m_index + 1).wrapping_mul(0x274381) as u32);
            }

            println!("models written to big buffer");

            gl.BindVertexArray(vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (array_buffer.v.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                array_buffer.v.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let pos_attrib = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            let color_attrib = gl.GetAttribLocation(program, b"color\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl.VertexAttribPointer(
                color_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (3 * std::mem::size_of::<f32>()) as _,
            );

            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, veo);
            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (array_buffer.i.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                array_buffer.i.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl.EnableVertexAttribArray(color_attrib as gl::types::GLuint);

            let mut camera_position = CameraPosition::default();
            camera_position.position.z = 90.0;

            Self {
                program,
                vao,
                vbo,
                veo,
                proj_matrix: Matrix::new_proj(std::f32::consts::FRAC_PI_8, 1.0, 1000.0, 0.01),
                view_matrix: Matrix::new_view(
                    Point {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    0.0,
                    0.0,
                ),
                index_count: array_buffer.i.len(),
                prev_time: std::time::Instant::now(),
                input_state: InputState::default(),
                camera_position,
                gl,
            }
        }
    }

    pub fn draw(&mut self) {
        unsafe {
            self.gl.UseProgram(self.program);
            let proj_matrix_location = self
                .gl
                .GetUniformLocation(self.program, b"proj\0".as_ptr() as *const _);
            self.gl.UniformMatrix4fv(
                proj_matrix_location,
                1,
                gl::FALSE,
                self.proj_matrix.as_ptr() as *const _,
            );

            let current_time = std::time::Instant::now();
            let dt = (current_time - self.prev_time).as_secs_f32();
            self.prev_time = current_time;

            let delta = dt * 10.0;

            if self.input_state.forward {
                self.camera_position.position += Point {
                    x: -delta * self.camera_position.anglez.sin(),
                    y: 0.0,
                    z: -delta * self.camera_position.anglez.cos(),
                };
            }

            if self.input_state.back {
                self.camera_position.position += Point {
                    x: delta * self.camera_position.anglez.sin(),
                    y: 0.0,
                    z: delta * self.camera_position.anglez.cos(),
                };
            }

            if self.input_state.left {
                self.camera_position.position += Point {
                    x: delta * self.camera_position.anglez.cos(),
                    y: 0.0,
                    z: -delta * self.camera_position.anglez.sin(),
                };
            }

            if self.input_state.right {
                self.camera_position.position += Point {
                    x: -delta * self.camera_position.anglez.cos(),
                    y: 0.0,
                    z: delta * self.camera_position.anglez.sin(),
                };
            }

            if self.input_state.up {
                self.camera_position.position += Point {
                    x: 0.0,
                    y: -delta,
                    z: 0.0,
                };
            }

            if self.input_state.down {
                self.camera_position.position += Point {
                    x: 0.0,
                    y: delta,
                    z: 0.0,
                };
            }

            let position = self.camera_position.position;

            let angle = self.camera_position.anglez;
            let angle_x = self.camera_position.anglex;
            self.view_matrix = Matrix::new_view(position, angle, angle_x);

            let view_matrix_location = self
                .gl
                .GetUniformLocation(self.program, b"view\0".as_ptr() as *const _);
            self.gl.UniformMatrix4fv(
                view_matrix_location,
                1,
                gl::FALSE,
                self.view_matrix.as_ptr() as *const _,
            );

            self.gl.ClearColor(1.0, 1.0, 1.0, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            self.gl.DrawElements(
                gl::TRIANGLES,
                self.index_count as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
            self.proj_matrix = Matrix::new_proj(
                std::f32::consts::FRAC_PI_4,
                width as f32 / height as f32,
                1000.0,
                0.01,
            );
        }
    }

    pub fn forward(&mut self, set: bool) {
        self.input_state.forward = set;
    }
    pub fn back(&mut self, set: bool) {
        self.input_state.back = set;
    }
    pub fn left(&mut self, set: bool) {
        self.input_state.left = set;
    }
    pub fn right(&mut self, set: bool) {
        self.input_state.right = set;
    }
    pub fn up(&mut self, set: bool) {
        self.input_state.up = set;
    }
    pub fn down(&mut self, set: bool) {
        self.input_state.down = set;
    }
    pub fn lbutton(&mut self, set: bool) {
        self.input_state.lbutton = set;
    }
    pub fn rbutton(&mut self, set: bool) {
        self.input_state.rbutton = set;
    }
    pub fn position(&mut self, position: PhysicalPosition<f64>) {
        if self.input_state.rbutton {
            let dz = position.x - self.input_state.prev_position.x;
            self.camera_position.anglez += (dz * 0.001) as f32;
            let dx = position.y - self.input_state.prev_position.y;
            self.camera_position.anglex -= (dx * 0.001) as f32;
        }
        self.input_state.prev_position = position;
    }
}

impl Deref for Renderer {
    type Target = gl::Gl;

    fn deref(&self) -> &Self::Target {
        &self.gl
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.program);
            self.gl.DeleteBuffers(1, &self.vbo);
            self.gl.DeleteBuffers(1, &self.veo);
        }
    }
}

unsafe fn create_shader(
    gl: &gl::Gl,
    shader: gl::types::GLenum,
    source: &[u8],
) -> gl::types::GLuint {
    let shader = gl.CreateShader(shader);
    gl.ShaderSource(
        shader,
        1,
        [source.as_ptr().cast()].as_ptr(),
        std::ptr::null(),
    );
    gl.CompileShader(shader);
    shader
}

fn get_gl_string(gl: &gl::Gl, variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl.GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 100
precision mediump float;

uniform mat4 proj;
uniform mat4 view;

attribute vec3 position;
attribute vec3 color;

varying vec3 v_color;
varying vec3 v_pos;
varying vec3 v_initial_pos;

void main() {
    v_initial_pos = position;
    gl_Position = proj * (view * vec4(position, 1.0));
    v_color = color;
    v_pos = gl_Position.xyz;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 100
precision mediump float;

varying vec3 v_color;
varying vec3 v_pos;
varying vec3 v_initial_pos;

void main() {
    vec3 normal = normalize(cross(dFdy(v_pos), dFdx(v_pos)));

    float factor = max(0.0, min(1.0, (160.0 - v_pos.z) * 0.01));
    vec3 white = vec3(1.0, 1.0, 1.0);
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

    color = color * (-normal.z);
    color = white + (color - white) * factor;
    gl_FragColor = vec4(color, 1.0);
}
\0";

fn main() {
    gl_main(EventLoopBuilder::new().build())
}
