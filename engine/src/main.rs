mod define;
mod core;
mod platform;
use crate::platform::*;

fn main(){



    let mut platform_state = PlatformState {
    internal_state: std::ptr::null_mut(),
    };

    if !platform_startup(
        &mut platform_state,
        "Vanta",
        100,
        100,
        1280,
        720,
    ){
        log_fatal!("Failed to start paltform!");
        return;
    }


    while platform_pump_messages(&mut platform_state){

    }

    platform_shutdown(&mut platform_state);

}
