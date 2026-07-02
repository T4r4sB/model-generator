     use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::errhandlingapi::*;
use winapi::um::libloaderapi::*;
use winapi::um::wingdi::*;
use winapi::um::winuser::*;

use std::ffi::OsStr;
use std::mem::MaybeUninit;
use std::os::windows::ffi::*;

use crate::errors::*;
use crate::gl_utils::*;
use common::matrix::*;
use common::model::*;

pub struct Context {
  stopped: bool,
  pub gl_data: Option<GLData>,
}

unsafe fn maybe_window_proc(
  hwnd: HWND,
  msg: UINT,
  wparam: WPARAM,
  lparam: LPARAM,
) -> APIResult<LRESULT> {
  let get_context = || -> APIResult<&mut Context> {
    unsafe {
      let context: *mut Context = std::ptr::with_exposed_provenance_mut(run_api!(
        GetWindowLongPtrW(hwnd, GWL_USERDATA)
      )? as usize);
      let context: &mut Context = context.as_mut().ok_or(APIResultCode::user())?;
      Ok(context)
    }
  };

  match msg {
    WM_KEYDOWN => {
      if let Some(gl_data) = &mut get_context()?.gl_data {
        if wparam == 'W' as WPARAM {
          gl_data.input_state.forward = true;
        } else if wparam == 'S' as WPARAM {
          gl_data.input_state.back = true;
        } else if wparam == 'A' as WPARAM {
          gl_data.input_state.left = true;
        } else if wparam == 'D' as WPARAM {
          gl_data.input_state.right = true;
        } else if wparam == VK_SPACE as WPARAM {
          gl_data.input_state.up = true;
        } else if wparam == VK_CONTROL as WPARAM {
          gl_data.input_state.down = true;
        } else if wparam == VK_OEM_PLUS as WPARAM {
          gl_data.shift += 0.01;
        } else if wparam == VK_OEM_MINUS as WPARAM {
          gl_data.shift = f32::max(0.0, gl_data.shift - 0.01);
        }
      }
    }
    WM_KEYUP => {
      if let Some(gl_data) = &mut get_context()?.gl_data {
        if wparam == 'W' as WPARAM {
          gl_data.input_state.forward = false;
        } else if wparam == 'S' as WPARAM {
          gl_data.input_state.back = false;
        } else if wparam == 'A' as WPARAM {
          gl_data.input_state.left = false;
        } else if wparam == 'D' as WPARAM {
          gl_data.input_state.right = false;
        } else if wparam == VK_SPACE as WPARAM {
          gl_data.input_state.up = false;
        } else if wparam == VK_CONTROL as WPARAM {
          gl_data.input_state.down = false;
        }
      }
    }
    WM_RBUTTONDOWN => {
      if let Some(gl_data) = &mut get_context()?.gl_data {
        gl_data.input_state.rbutton = true;
        SetCapture(hwnd);
        let mut cursor = POINT { x: 0, y: 0 };
        GetPhysicalCursorPos(&mut cursor);
        gl_data.input_state.prev_position = (cursor.x, cursor.y);
      }
    }
    WM_RBUTTONUP => {
      if let Some(gl_data) = &mut get_context()?.gl_data {
        ReleaseCapture();
        gl_data.input_state.rbutton = false;
      }
    }
    WM_SIZE => {
      if let Some(gl_data) = &mut get_context()?.gl_data {
        let width = LOWORD(lparam as u32) as i32;
        let height = HIWORD(lparam as u32) as i32;
        unsafe {
          gl::Viewport(0, 0, width, height);
        }
        gl_data.proj_matrix =
          Matrix::new_proj(std::f32::consts::FRAC_PI_4, width as f32 / height as f32, 1000.0, 0.01);
      }
    }
    WM_DESTROY => {
      unsafe { PostQuitMessage(0) };
      get_context()?.stopped = true;
      return Ok(0);
    }
    _ => {}
  }

  Ok(unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) })
}

unsafe extern "system" fn window_proc(
  hwnd: HWND,
  msg: UINT,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match unsafe { maybe_window_proc(hwnd, msg, wparam, lparam) } {
    Ok(l_result) => return l_result,
    Err(_) => {
      // Do nothing, read message and continue
      return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }
  }
}

fn handle_message() -> bool {
  unsafe {
    let mut msg = MaybeUninit::<MSG>::uninit();
    let pm = run_api!(PeekMessageW(msg.as_mut_ptr(), 0 as HWND, 0, 0, PM_REMOVE));
    match pm {
      Ok(pm) => {
        if pm == 0 {
          return true;
        }
      }
      Err(_) => return true, // just continue
    }
    let msg = msg.assume_init();
    if msg.message == winapi::um::winuser::WM_QUIT {
      return false;
    }

    // skip errors
    let _ = run_api!(TranslateMessage(&msg));
    let _ = run_api_z!(DispatchMessageW(&msg));
    true
  }
}

pub fn run(name: &str, models: &mut dyn Iterator<Item = (u32, &Model)>) -> APIResult<HWND> {
  let mut context = Context { stopped: false, gl_data: None };
  let name16: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0).into_iter()).collect();

  unsafe {
    let hinstance = run_api!(GetModuleHandleW(0 as *const u16))?;
    let wnd_class = WNDCLASSW {
      style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
      lpfnWndProc: Some(window_proc),
      hInstance: hinstance,
      lpszClassName: name16.as_ptr(),
      cbClsExtra: 0,
      cbWndExtra: 0,
      hIcon: 0 as HICON,
      hCursor: run_api!(LoadCursorW(0 as HINSTANCE, IDC_ARROW))?,
      hbrBackground: 0 as HBRUSH,
      lpszMenuName: 0 as *const u16,
    };

    run_api!(RegisterClassW(&wnd_class))?;
    let hwnd = run_api!(CreateWindowExW(
      0,                   // dwExStyle
      name16.as_ptr(),     // class we registered
      name16.as_ptr(),     // title
      WS_OVERLAPPEDWINDOW, // dwStyle
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      CW_USEDEFAULT,                     // size and position
      0 as HWND,                         // hWndParent
      0 as HMENU,                        // hMenu
      hinstance,                         // hInstance
      std::mem::transmute(&mut context)  // lpParam
    ))?;

    run_api!(SetWindowLongPtrW(hwnd, GWL_USERDATA, std::mem::transmute(&mut context)))?;

    let gl_data = init_gl(hwnd, models)?;
    context.gl_data = Some(gl_data);

    run_api!(ShowWindow(hwnd, SW_SHOWDEFAULT))?;

    loop {
      if context.stopped {
        break;
      }
      if !handle_message() {
        break;
      }
      if !context.stopped {
        gl_update_and_draw(&mut context);
        if let Ok(dc) = crate::resources::HDC::new(hwnd) {
          SwapBuffers(dc.get_dc());
        }
      }
    }

    Ok(hwnd)
  }
}
