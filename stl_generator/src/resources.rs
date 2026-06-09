use winapi::shared::windef::*;
use winapi::um::errhandlingapi::*;
use winapi::um::wingdi::*;
use winapi::um::winuser::*;

use crate::errors::*;

pub struct GLContext {
    id: HGLRC,
}

impl GLContext {
    pub unsafe fn new(dc: winapi::shared::windef::HDC) -> APIResult<Self> {
        Ok(Self {
            id: unsafe { run_api!(wglCreateContext(dc))? },
        })
    }

    pub fn get(&self) -> HGLRC {
        self.id
    }
}

impl Drop for GLContext {
    fn drop(&mut self) {
        unsafe {
            if let Err(_) = run_api!(wglDeleteContext(self.id)) {
                // We cant pass this error anywhere, because it is a destructor
            }
        }
    }
}

pub struct HDC {
    hwnd: HWND,
    dc: winapi::shared::windef::HDC,
}

impl HDC {
    pub unsafe fn new(hwnd: HWND) -> APIResult<Self> {
        Ok(Self {
            hwnd,
            dc: unsafe { run_api!(GetDC(hwnd))? },
        })
    }

    pub fn get_dc(&self) -> winapi::shared::windef::HDC {
        self.dc
    }
}

impl Drop for HDC {
    fn drop(&mut self) {
        unsafe {
            if let Err(_) = run_api!(ReleaseDC(self.hwnd, self.dc)) {
                // We cant pass this error anywhere, because it is a destructor
            }
        }
    }
}

pub struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    pub unsafe fn new(shader: gl::types::GLenum) -> Self {
        Self {
            id: unsafe { gl::CreateShader(shader) },
        }
    }

    pub fn get(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

pub struct Program {
    id: gl::types::GLuint,
}

impl Program {
    pub unsafe fn new() -> Self {
        Self {
            id: unsafe { gl::CreateProgram() },
        }
    }

    pub fn get(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub struct VertexArray {
    id: gl::types::GLuint,
}

impl VertexArray {
    pub unsafe fn new() -> Self {
        let mut id = 0;
        unsafe { gl::GenVertexArrays(1, &mut id) };
        Self { id }
    }

    pub fn get(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

pub struct Buffer {
    id: gl::types::GLuint,
}

impl Buffer {
    pub unsafe fn new() -> Self {
        let mut id = 0;
        unsafe { gl::GenBuffers(1, &mut id) };
        Self { id }
    }

    pub fn get(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}
