// TODO Support Wayland-only backend

use std::error::Error;

use smithay::reexports::calloop::EventLoop;

use crate::state::{Data, State};

pub mod winit;

// TODO allow backend switching, for debug reasons
pub fn init_backend(event_loop: &mut EventLoop<'static, Data>, state: &mut State) -> Result<(), Box<dyn Error>> {
    winit::init_backend(event_loop, state).unwrap();

    Ok(())
}
