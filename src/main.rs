// SPDX-License-Identifier: GPL-3.0-only

use std::{error::Error, ffi::OsString};

use anyhow::Context;
use smithay::{
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, Mode, PostAction},
        wayland_server::Display,
    },
    wayland::socket::ListeningSocketSource,
};
use state::{Data, State};

use crate::log::init_logger;

mod backend;
mod log;
mod state;

fn main() -> Result<(), Box<dyn Error>> {
    let log = init_logger()?;
    slog_scope::info!("Starting up");

    let mut event_loop = EventLoop::try_new_high_precision()
        .with_context(|| "Failed to initialise event loop")
        .unwrap();

    let (display, socket) = init_wayland_display(&mut event_loop)?;

    let mut state = State::new(&display.handle(), socket, log);

    backend::init_backend(&mut event_loop, &mut state)?;

    let mut data = Data { display, state };

    event_loop
        .run(None, &mut data, |data| {
            let _ = data.display.flush_clients();
        })
        .expect("Failed to run Event Loop");

    std::mem::drop(event_loop);
    Ok(())
}

fn init_wayland_display(
    event_loop: &mut EventLoop<Data>,
) -> Result<(Display<State>, OsString), Box<dyn Error>> {
    let mut display = Display::new().unwrap();

    let socket_source = ListeningSocketSource::new_auto(None).expect("Failed to register socket");
    let socket_name = socket_source.socket_name().to_os_string();
    slog_scope::info!("Listening on {:?}", socket_name);

    event_loop
        .handle()
        .insert_source(socket_source, |_stream, _, _data| {
            slog_scope::warn!("Unimplemented");
        })
        .with_context(|| "Failed to initialise socket")?;

    event_loop
        .handle()
        .insert_source(
            Generic::new(display.backend().poll_fd(), Interest::READ, Mode::Level),
            move |_, _, data: &mut Data| match data.display.dispatch_clients(&mut data.state) {
                Ok(_) => Ok(PostAction::Continue),
                Err(e) => {
                    slog_scope::error!("I/O Error on display: {}", e);
                    Err(e)
                }
            },
        )
        .with_context(|| "Failed to initialise event source")?;

    Ok((display, socket_name))
}
