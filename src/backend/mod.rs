// SPDX-License-Identifier: GPL-3.0-only

use std::{env, error::Error};

use smithay::reexports::{calloop::EventLoop, wayland_server::DisplayHandle};

use crate::state::{Data, State};

// TODO Support Wayland-only backend
pub mod renderer;
pub mod winit;

pub fn init_backend(
    dh: &DisplayHandle,
    event_loop: &mut EventLoop<'static, Data>,
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    let res = match env::var("ELECTRUM_BACKEND") {
        Ok(x) if x == "winit" => winit::init_backend(dh, event_loop, state),
        Ok(x) => unimplemented!("Backend {} does not exist", x),
        // TODO create gpu backend
        Err(_) => {
            slog_scope::warn!(
                "Backend does not exist or not identified, falling back to winit backend."
            );
            winit::init_backend(dh, event_loop, state)
        }
    };

    if res.is_ok() {
        // TODO: Handle seats
    }
    res
}
