
use std::ffi::c_void;

pub struct PlatformState{
    pub internal_state: *mut c_void,
}


pub fn platform_startup(platform_state: &mut PlatformState, platform_name: &str, x: i32, y: i32, width: i32, height: i32) -> bool{

    true
}

pub fn platform_shutdown(platform_state: &mut PlatformState){

}

pub fn platform_pump_msg(platform_state: &mut PlatformState) -> bool{

}

pub fn platform_console_write(msg: &str, color: u8){

}

pub fn platform_console_write_err(msg: &str, color: u8){

}

pub fn platform_get_absolute_time() -> f64{

}

pub fn platform_sleep(milliseconds: u64){

}
