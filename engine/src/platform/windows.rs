use std::ffi::c_void;
use std::ffi::CString;
use crate::*;

use windows::core::PCSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, LARGE_INTEGER};
use windows::Win32::System::LibraryLoader::{HINSTANCE, GetModuleHandleA};
use windows::Win32::System::Console::{WriteConsoleA, GetStdHandle, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE, SetConsoleTextAttribute};
use windows::Win32::System::Diagnostics::Debug::OutputDebugStringA;
use windows::Win32::System::Performance::{QueryPerformanceFrequency, QueryPerformanceCounter};
use windows::Win32::System::Threading::Sleep;
use windows::Win32::UI::WindowsAndMessaging::{
    LoadCursorA,
    LoadIconA,
    RegisterClassA,
    WNDCLASSA,
    IDC_ARROW,
    IDI_APPLICATION,
    CS_DBLCLKS,
    MessageBoxA,
    MB_ICONERROR,
    MB_OK,
    DefWindowProcA,
    PostQuitMessage,
    AdjustWindowRectEx,
    CreateWindowExA,
    ShowWindow,
    DestroyWindow,
    DispatchMessageA,
    MSG,
    PM_REMOVE,
    PeekMessageA,
    TranslateMessage,

    WM_CLOSE,
    WM_DESTROY,
    WM_DPICHANGED,
    WM_ERASEBKGND,
    WM_KEYDOWN,
    WM_KEYUP,
    WM_LBUTTONDOWN,
    WM_LBUTTONUP,
    WM_MBUTTONDOWN,
    WM_MBUTTONUP,
    WM_MOUSEMOVE,
    WM_MOUSEWHEEL,
    WM_RBUTTONDOWN,
    WM_RBUTTONUP,
    WM_SHOWWINDOW,
    WM_SIZE,
    WM_SYSKEYDOWN,
    WM_SYSKEYUP,

    WS_OVERLAPPED,
    WS_SYSMENU,
    WS_MAXIMIZEBOX,
    WS_MINIMIZEBOX,
    WS_EX_APPWINDOW,
    WS_CAPTION,
    WS_THICKFRAME,
};
use windows::Win32::Graphics::Gdi::{GetStockObject, BLACK_BRUSH};

pub struct PlatformState{
    pub internal_state: *mut c_void,
}

struct InternalState {
    h_instance: HINSTANCE,
    hwnd: HWND,
}

static mut CLOCK_FREQUENCY: f64 = 0.0;
static mut START_TIME: LARGE_INTEGER = LARGE_INTEGER::default();


unsafe extern "system" fn win32_process_message(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT{

    match msg{
        WM_ERASEBKGND => {
            LRESULT(1)
        },

        WM_CLOSE => {
            //TODO: Call event for quit app
            LRESULT(0)
        }

        WM_DESTROY => {
            unsafe { PostQuitMessage(0); }
            LRESULT(0)
        },

        WM_SIZE => {
            //TODO: Get Updated size and call event for window
            LRESULT(1);
        },

        WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP => {
            // TODO handle input
        },

        WM_MOUSEMOVE => {
            //TODO handle mouse move
        },

        WM_MOUSEWHEEL => {
            //TODO handle mouse wheel
        },

        WM_LBUTTONUP | WM_MBUTTONUP | WM_RBUTTONUP |
            WM_LBUTTONDOWN | WM_MBUTTONDOWN | WM_RBUTTONDOWN => {
            // TODO handle mouse button
            }

        _ => unsafe {
            DefWindowProcA(hwnd, msg, w_param, l_param,)
        },
    }

}


pub fn platform_startup(platform_state: &mut PlatformState, platform_name: &str, x: i32, y: i32, width: i32, height: i32) -> bool{

    let state = setup_internal_state(platform_state);
    if !setup_window_class(state) { return false; }
    adjust_window_styling(x,y,width,height, state);

    let should_activate : bool = true;
    let show_window_command_flags = if should_activate {SW_SHOW} else { SW_SHOWNOTACTIVE  };
    ShowWindow(state.hwnd, show_window_command_flags);

    let mut frequency = LARGE_INTEGER::default();
    unsafe {
        QueryPerformanceFrequency(&mut frequency);
    }
    CLOCK_FREQUENCY = 1.0 / frequency.QuadPart() as f64;

    unsafe{
        QueryPerformanceCounter(&mut START_TIME);
    }

    true
}

pub fn platform_shutdown(platform_state: &mut PlatformState){
    let state = unsafe{
        &mut *(platform_state.internal_state as *mut InternalState)
    };

    if state.hwnd.0 != 0 {
        unsafe{
            DestroyWindow(state.hwnd);
        }
        state.hwnd = HWND::default();
    }
}

pub fn platform_pump_messages() -> bool {
    let mut message = MSG::default();

    unsafe {
        while PeekMessageA(
            &mut message,
            None,
            0,
            0,
            PM_REMOVE,
        ).as_bool(){
            TranslateMessage(&message);
            DispatchMessageA(&message);
        }
    }
    true
}
pub fn platform_console_write(msg: &str, color: u8){

    let levels : [u16; 6] = [64,4,6,2,1,0];

    let handle = unsafe{
        GetStdHandle(STD_OUTPUT_HANDLE);
    };

    unsafe{
        SetConsoleTextAttribute(
            handle,
            levels[color as usize],
        );
    }

    let c_msg = match CString::new(msg){
        Ok(msg) => msg,
        Err(_) => return,
    };

    unsafe {
        OutputDebugStringA(
            PCSTR(c_msg.as_ptr() as * const u8)
        );
    }


    let mut written: u32 = 0;

    unsafe{
        WriteConsoleA(
            handle,
            bytes.as_ptr() as *const _,
            bytes.len() as u32,
            &mut written,
            None,
        );
    }
}

pub fn platform_console_write_err(msg: &str, color: u8){

    let levels : [u16; 6] = [64,4,6,2,1,0];

    let handle = unsafe{
        GetStdHandle(STD_ERROR_HANDLE);
    };

    unsafe{
        SetConsoleTextAttribute(
            handle,
            levels[color as usize],
        );
    }

    let c_msg = match CString::new(msg){
        Ok(msg) => msg,
        Err(_) => return,
    };

    unsafe {
        OutputDebugStringA(
            PCSTR(c_msg.as_ptr() as * const u8)
        );
    }


    let mut written: u32 = 0;

    unsafe{
        WriteConsoleA(
            handle,
            bytes.as_ptr() as *const _,
            bytes.len() as u32,
            &mut written,
            None,
        );
    }
}

pub fn platform_get_absolute_time() -> f64{
    let mut now_time : LARGE_INTEGER = LARGE_INTEGER::default();

    unsafe {
        QueryPerformanceCounter(&mut now_time);
        now_time.QuadPart() as f64 * CLOCK_FREQUENCY;
    }
}

pub fn platform_sleep(milliseconds: u64){
    unsafe{
        Sleep(milliseconds);
    }
}

fn setup_internal_state(platform_state: &mut PlatformState) -> &mut InternalState{



    let h_instance = unsafe {
        match GetModuleHandleA(None){
            Ok(handle) => handle,
            Err(_) => {
                panic!("Failed to get module handle");
            },
        }
    };

    let state= Box::new(InternalState {
        h_instance,
        hwnd: HWND::default(),
    });


    let state_ptr = Box::into_raw(state);

    platform_state.internal_state = state_ptr as *mut c_void;

    unsafe {
        &mut *state_ptr
    }

}

fn setup_window_class(state: &mut InternalState) -> bool{
    let icon = unsafe{
        LoadIconA(state.h_instance, IDI_APPLICATION)
    };
    let cursor = unsafe{
        LoadCursorA(None, IDC_ARROW)
            .expect("Failed to load cursor")
    };

    let background = unsafe{
        GetStockObject(BLACK_BRUSH)
            .expect("Failed to load black brush")
    };

    let wc = WNDCLASSA{
        style: CS_DBLCLKS,
        lpfnWndProc: Some(win32_process_message),

        cbClsExtra:0,
        cbWndExtra:0,

        hInstance: state.h_instance,
        hIcon: icon,
        hCursor: cursor,

        hbrBackground: background.into(),

        lpszMenuName: PCSTR::null(),

        lpszClassName: PCSTR(b"VantaWindowClass\0".as_ptr()),

    };

    let result = unsafe{
        RegisterClassA(&wc)
    };
    if result == 0 {
        unsafe{
            MessageBoxA(
                None,
                PCSTR(b"Failed to register window class!\0".as_ptr()),
                PCSTR(b"Vanta Error\0".as_ptr()),
                MB_ICONERROR | MB_OK,
            );
        }
        panic!("Failed to register Vanta window Class");
        return false;
    }

    true
}

fn adjust_window_styling(x: i32, y: i32, w: i32, h: i32, state: &mut InternalState){

    let client_x = x;
    let client_y = y;
    let client_w = w;
    let client_h = h;

    let mut window_x = x;
    let mut window_y = y;
    let mut window_w = w;
    let mut window_h = h;


    let mut window_style : u32 = WS_OVERLAPPED | WS_SYSMENU | WS_CAPTION;
    let mut window_ex_style : u32 = WS_EX_APPWINDOW;

    window_style |= WS_MAXIMIZEBOX;
    window_style |= WS_MINIMIZEBOX;
    window_style |= WS_THICKFRAME;

    let mut border_rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    unsafe {
        AdjustWindowRectEx(
            &mut border_rect,
            window_style,
            false,
            window_ex_style,
        );
    }

    window_x += border_rect.left;
    window_y += border_rect.top;

    window_w += border_rect.right - border_rect.left;
    window_h += border_rect.bottom - border_rect.top;

    let handle = unsafe {
        CreateWindowExA(
            window_ex_style,
            PCSTR(b"VantaWindowClass\0".as_ptr()),
            PCSTR(b"Vanta\0".as_ptr()),
            window_style,
            window_x,
            window_y,
            window_w,
            window_h,
            false,
            false,
            state.h_instance,
            false
        )
    };

    if (handle == 0 || handle == null){
        MessageBoxA(NULL, "Window creation failed!", "Error!", MB_ICONERROR | MB_OK);

        log_fatal!("Window Creation Failed!");
    } else{
        state.hwnd = handle;
    }

}


