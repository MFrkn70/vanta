use std::ffi::c_void;
use std::time::Instant;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    Atom,
    AtomEnum,
    ConnectionExt as _,
    CreateWindowAux,
    EventMask,
    PropMode,
    Screen,
    Window,
    WindowClass,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt;

use crate::*;

pub struct PlatformState {
    pub internal_state: *mut c_void,
}

struct InternalState {
    connection: RustConnection,
    window: Window,
    screen: Screen,
    wm_protocols: Atom,
    wm_delete_win: Atom,
    start_time: Instant,
}

pub fn platform_startup(
    platform_state: &mut PlatformState,
    application_name: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> bool {

    // Connect to X server.
    let (connection, screen_num) = match RustConnection::connect(None) {
        Ok(result) => result,

        Err(_) => {
            log_fatal!("Failed to connect to X server!");
            return false;
        }
    };

    // Get screen information.
    let screen = connection.setup().roots[screen_num].clone();

    // Generate a window ID.
    let window = match connection.generate_id() {
        Ok(id) => id,

        Err(_) => {
            log_fatal!("Failed to generate X11 window ID!");
            return false;
        }
    };

    let event_mask =
        EventMask::BUTTON_PRESS
        | EventMask::BUTTON_RELEASE
        | EventMask::KEY_PRESS
        | EventMask::KEY_RELEASE
        | EventMask::EXPOSURE
        | EventMask::POINTER_MOTION
        | EventMask::STRUCTURE_NOTIFY;

    let window_aux = CreateWindowAux::new()
        .background_pixel(screen.black_pixel)
        .event_mask(event_mask);

    // Create the window.
    if connection
        .create_window(
            screen.root_depth,
            window,
            screen.root,
            x as i16,
            y as i16,
            width as u16,
            height as u16,
            0,
            WindowClass::INPUT_OUTPUT,
            screen.root_visual,
            &window_aux,
        )
        .is_err()
    {
        log_fatal!("Failed to create X11 window!");
        return false;
    }

    // Set the window title.
    if connection
        .change_property8(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            application_name.as_bytes(),
        )
        .is_err()
    {
        log_warning!("Failed to set X11 window title.");
    }

    // Create WM_DELETE_WINDOW atom.
    let wm_delete_win = match connection
        .intern_atom(false, b"WM_DELETE_WINDOW")
    {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply.atom,
            Err(_) => {
                log_fatal!("Failed to create WM_DELETE_WINDOW atom!");
                return false;
            }
        },

        Err(_) => {
            log_fatal!("Failed to intern WM_DELETE_WINDOW!");
            return false;
        }
    };

    // Create WM_PROTOCOLS atom.
    let wm_protocols = match connection
        .intern_atom(false, b"WM_PROTOCOLS")
    {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply.atom,
            Err(_) => {
                log_fatal!("Failed to create WM_PROTOCOLS atom!");
                return false;
            }
        },

        Err(_) => {
            log_fatal!("Failed to intern WM_PROTOCOLS!");
            return false;
        }
    };

    // Tell the window manager that we support WM_DELETE_WINDOW.
    if connection
        .change_property32(
            PropMode::REPLACE,
            window,
            wm_protocols,
            AtomEnum::ATOM,
            &[wm_delete_win],
        )
        .is_err()
    {
        log_fatal!("Failed to set WM_PROTOCOLS!");
        return false;
    }

    // Make the window visible.
    if connection.map_window(window).is_err() {
        log_fatal!("Failed to map X11 window!");
        return false;
    }

    // Flush all pending X11 commands.
    if connection.flush().is_err() {
        log_fatal!("Failed to flush X11 connection!");
        return false;
    }

    // Create our internal state.
    let state = Box::new(InternalState {
        connection,
        window,
        screen,
        wm_protocols,
        wm_delete_win,
        start_time: Instant::now(),
    });

    let state_ptr = Box::into_raw(state);

    platform_state.internal_state =
        state_ptr as *mut c_void;

    true
}

pub fn platform_shutdown(
    platform_state: &mut PlatformState,
) {
    if platform_state.internal_state.is_null() {
        return;
    }

    let state = unsafe {
        &mut *(platform_state.internal_state as *mut InternalState)
    };

    let _ = state.connection.destroy_window(state.window);
    let _ = state.connection.flush();

    unsafe {
        drop(Box::from_raw(
            platform_state.internal_state as *mut InternalState
        ));
    }

    platform_state.internal_state =
        std::ptr::null_mut();
}

pub fn platform_pump_messages(
    platform_state: &mut PlatformState,
) -> bool {

    let state = unsafe {
        &mut *(platform_state.internal_state as *mut InternalState)
    };

    loop {
        let event = match state.connection.poll_for_event() {
            Ok(Some(event)) => event,

            Ok(None) => {
                break;
            }

            Err(_) => {
                log_error!("Failed to poll X11 event.");
                return false;
            }
        };

        match event {
            Event::KeyPress(_) => {
                // TODO: Key press
            }

            Event::KeyRelease(_) => {
                // TODO: Key release
            }

            Event::ButtonPress(_) => {
                // TODO: Mouse button press
            }

            Event::ButtonRelease(_) => {
                // TODO: Mouse button release
            }

            Event::MotionNotify(_) => {
                // TODO: Mouse movement
            }

            Event::ConfigureNotify(_) => {
                // TODO: Window resize
            }

            Event::ClientMessage(event) => {
                if event.data.as_data32()[0]
                    == state.wm_delete_win
                {
                    return false;
                }
            }

            _ => {
                // Ignore other events for now.
            }
        }
    }

    true
}

pub fn platform_console_write(
    message: &str,
    colour: u8,
) {
    let colour_strings = [
        "0;41",
        "1;31",
        "1;33",
        "1;32",
        "1;34",
        "1;30",
    ];

    print!(
        "\x1b[{}m{}\x1b[0m",
        colour_strings[colour as usize],
        message
    );
}

pub fn platform_console_write_err(
    message: &str,
    colour: u8,
) {

    let colour_strings = [
        "0;7;91",
        "1;31",
        "1;33",
        "1;32",
        "1;34",
        "1;30",
    ];

    eprint!(
        "\x1b[{}m{}\x1b[0m",
        colour_strings[colour as usize],
        message
    );
}

pub fn platform_get_absolute_time(
    platform_state: &mut PlatformState,
) -> f64 {

    let state = unsafe {
        &*(platform_state.internal_state as *mut InternalState)
    };

    state.start_time.elapsed().as_secs_f64()
}
