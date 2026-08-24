use std::ffi::c_void;
use std::time::{Duration, Instant};
use std::sync::OnceLock;
use std::io::{self, Write};
use std::os::fd::AsFd;

use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_compositor::{self, WlCompositor},
        wl_registry,
        wl_surface::{self, WlSurface},
        wl_shm::{self, WlShm},
        wl_buffer::{self, WlBuffer},
        wl_shm_pool::{self, WlShmPool},
    },
    Connection, Dispatch, EventQueue, QueueHandle,
};

use wayland_protocols::xdg::shell::client::{
    xdg_surface::{self, XdgSurface},
    xdg_toplevel::{self, XdgToplevel},
    xdg_wm_base::{self, XdgWmBase},
};

use crate::*;


pub struct PlatformState {
    pub internal_state: *mut c_void,
}


struct InternalState {
    connection: Connection,
    event_queue: EventQueue<WaylandState>,

    compositor: WlCompositor,
    surface: WlSurface,
    shm: WlShm,
    buffer: WlBuffer,

    xdg_wm_base: XdgWmBase,
    xdg_surface: XdgSurface,
    xdg_toplevel: XdgToplevel,

    width: i32,
    height: i32,

    running: bool,

    start_time: Instant,
}


struct WaylandState {
    running: bool,
}

static PLATFORM_START_TIME: OnceLock<Instant> = OnceLock::new();


// ---------------------------------------------------------
// Wayland event handlers
// ---------------------------------------------------------

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}


impl Dispatch<WlCompositor, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlCompositor,
        _event: wl_compositor::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}


impl Dispatch<WlSurface, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSurface,
        _event: wl_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}


impl Dispatch<XdgWmBase, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        proxy: &XdgWmBase,
        event: xdg_wm_base::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            xdg_wm_base::Event::Ping { serial } => {
                proxy.pong(serial);
            }

            _ => {}
        }
    }
}


impl Dispatch<XdgSurface, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        proxy: &XdgSurface,
        event: xdg_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            xdg_surface::Event::Configure { serial } => {
                proxy.ack_configure(serial);
            }

            _ => {}
        }
    }
}


impl Dispatch<XdgToplevel, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &XdgToplevel,
        event: xdg_toplevel::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            xdg_toplevel::Event::Close => {
                state.running = false;
            }

            xdg_toplevel::Event::Configure {
                width,
                height,
                ..
            } => {
                // TODO:
                // Handle window resizing here.
                let _ = (width, height);
            }

            _ => {}
        }
    }
}

impl Dispatch<WlShm, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlShm,
        _event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlShmPool, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlShmPool,
        _event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlBuffer, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlBuffer,
        _event: wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

// ---------------------------------------------------------
// Startup
// ---------------------------------------------------------

pub fn platform_startup(
    platform_state: &mut PlatformState,
    platform_name: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> bool {

    let _ = (platform_name, x, y);

    let connection = match Connection::connect_to_env() {
        Ok(connection) => {
            log_trace!("Connected to Wayland");
            connection
        },

        Err(_) => {
            log_fatal!("Failed to connect to Wayland compositor!");
            return false;
        }
    };



    let (globals, mut event_queue) =
        match registry_queue_init::<WaylandState>(&connection) {
            Ok(result) => {
                log_trace!("Wayland Registry initialized");
                result
            },

            Err(_) => {
                log_fatal!("Failed to initialize Wayland registry!");
                return false;
            }
        };

    let qh = event_queue.handle();

    let compositor: WlCompositor =
        match globals.bind::<WlCompositor, _, _>(&qh, 4..=6, ()) {
            Ok(compositor) => {
                log_trace!("Wayland compositor created");
                compositor
            },

            Err(_) => {
                log_fatal!("Wayland compositor is not available!");
                return false;
            }
        };




    let shm: WlShm =
        match globals.bind::<WlShm, _, _>(&qh, 1..=1, ()) {
            Ok(shm) => {
                log_trace!("Wayland SHM created");
                shm
            },

            Err(_) => {
                log_fatal!("Wayland SHM is not available!");
                return false;
            }
        };

    let xdg_wm_base: XdgWmBase =
        match globals.bind::<XdgWmBase, _, _>(&qh, 1..=1, ()) {
            Ok(xdg_wm_base) => {
                log_trace!("xdg_wm_base created");
                xdg_wm_base
            },

            Err(_) => {
                log_fatal!("xdg_wm_base is not available!");
                return false;
            }
        };


    // Create the Wayland surface.
    let surface = compositor.create_surface(&qh, ());


    // Create the XDG surface.
    let xdg_surface = xdg_wm_base.get_xdg_surface(&surface, &qh, ());


    // Create the top-level window.
    let xdg_toplevel =
        xdg_surface.get_toplevel(&qh, ());


    // Set the window title.
    xdg_toplevel.set_title(platform_name.to_string());


    // Set an application identifier.
    xdg_toplevel.set_app_id("vanta".to_string());

    let buffer = match create_test_buffer(
        &shm,
        &qh,
        width,
        height,
    ) {
        Some(buffer) => buffer,

        None => {
            return false;
        }
    };

surface.attach(Some(&buffer), 0, 0);
surface.damage_buffer(0, 0, width, height);

    // Tell Wayland that we want to show this surface.
    surface.commit();




    // Process the initial configure event.
    if event_queue.roundtrip(&mut WaylandState {
        running: true,
    }).is_err() {
        log_fatal!("Failed to process initial Wayland events!");
        return false;
    }


    let state = Box::new(InternalState {
        connection,
        event_queue,

        compositor,
        surface,
        shm,
        buffer,

        xdg_wm_base,
        xdg_surface,
        xdg_toplevel,

        width,
        height,

        running: true,

        start_time: Instant::now(),
    });


    let state_ptr = Box::into_raw(state);

    platform_state.internal_state =
        state_ptr as *mut c_void;

    PLATFORM_START_TIME.get_or_init(Instant::now);


    true
}


// ---------------------------------------------------------
// Shutdown
// ---------------------------------------------------------

pub fn platform_shutdown(
    platform_state: &mut PlatformState,
) {
    if platform_state.internal_state.is_null() {
        return;
    }

    let state = unsafe {
        Box::from_raw(
            platform_state.internal_state as *mut InternalState
        )
    };

    state.xdg_toplevel.destroy();
    state.xdg_surface.destroy();
    state.surface.destroy();

    platform_state.internal_state =
        std::ptr::null_mut();
}


// ---------------------------------------------------------
// Message pumping
// ---------------------------------------------------------

pub fn platform_pump_messages(
    platform_state: &mut PlatformState,
) -> bool {

    if platform_state.internal_state.is_null() {
        return false;
    }

    let state = unsafe {
        &mut *(platform_state.internal_state as *mut InternalState)
    };


    if state
        .connection
        .flush()
        .is_err()
    {
        log_error!("Failed to flush Wayland connection!");
        return false;
    }


    let mut wayland_state = WaylandState {
        running: state.running,
    };


    if state
        .event_queue
        .dispatch_pending(&mut wayland_state)
        .is_err()
    {
        log_error!("Failed to dispatch Wayland events!");
        return false;
    }


    state.running = wayland_state.running;

    state.running
}


// ---------------------------------------------------------
// Console
// ---------------------------------------------------------

pub fn platform_console_write(msg: &str, color: u8) {

    let colors = [
         "\x1b[0;41m", // FATAL   - red background
        "\x1b[0;31m", // ERROR   - red
        "\x1b[0;33m", // WARNING - yellow
        "\x1b[0;32m", // INFO    - green
        "\x1b[0;34m", // DEBUG   - blue
        "\x1b[0;39m", // TRACE   - default
   ];



    if let Some(color_code) =  colors.get(color as usize) {
        print!("\x1b[49m\x1b[0m{}{}\x1b[0m", color_code, msg);
    } else {
        print!("\x1b[49m\x1b[0m{}", msg);
    }

    io::stdout().flush().unwrap();
}


pub fn platform_console_write_err(msg: &str, color: u8) {
    let colors = [
        "\x1b[7m\x1b[91m", // FATAL
        "\x1b[31m", // ERROR
        "\x1b[0;33m", // WARNING
        "\x1b[0;32m", // INFO
        "\x1b[0;34m", // DEBUG
        "\x1b[0;39m", // TRACE
    ];

    if let Some(color_code) = colors.get(color as usize) {
        eprint!("\x1b[49m\x1b[0m{}{}\x1b[0m\x1b[49m", color_code, msg);
    } else {
        eprint!("\x1b[49m\x1b[0m{}", msg);
    }

    io::stdout().flush().unwrap();
}

// ---------------------------------------------------------
// Time
// ---------------------------------------------------------

pub fn platform_get_absolute_time() -> f64 {
    match PLATFORM_START_TIME.get(){
        Some(start) => start.elapsed().as_secs_f64(),
        None => 0.0,
    }
}


pub fn platform_sleep(
    milliseconds: u64,
) {
    std::thread::sleep(
        Duration::from_millis(milliseconds)
    );
}




// ---------------------------------------------------------
// Test Buffer
// ---------------------------------------------------------

fn create_test_buffer(
    shm: &WlShm,
    qh: &QueueHandle<WaylandState>,
    width: i32,
    height: i32,
) -> Option<WlBuffer> {
    let stride = width * 4;
    let size = stride * height;

    let memfd = match memfd::MemfdOptions::default()
        .allow_sealing(true)
        .create("vanta-buffer")
    {
        Ok(memfd) => memfd,

        Err(_) => {
            log_error!("Failed to create Wayland shared memory");
            return None;
        }
    };

    let file = memfd.as_file();

    if file.set_len(size as u64).is_err() {
        log_error!("Failed to resize Wayland shared memory");
        return None;
    }

    let pool = unsafe {
        shm.create_pool(
            file.as_fd(),
            size,
            qh,
            (),
        )
    };

    let buffer = pool.create_buffer(
        0,
        width,
        height,
        stride,
        wl_shm::Format::Argb8888,
        qh,
        (),
    );

    pool.destroy();

    Some(buffer)
}
