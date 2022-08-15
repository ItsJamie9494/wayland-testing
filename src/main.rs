// SPDX-License-Identifier: GPL-3.0-only

use std::{
    error::Error,
    ffi::OsString,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use smithay::{
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, Mode, PostAction},
        wayland_server::Display,
    },
    wayland::socket::ListeningSocketSource,
};
use state::{Data, LoopData, State};
use std::ops::DerefMut;

use crate::log::init_logger;

mod backend;
mod log;
mod runtime;
mod shell;
mod state;

fn main() -> Result<(), Box<dyn Error>> {
    let log = init_logger()?;
    slog_scope::info!("Starting up");

    let mut event_loop =
        EventLoop::try_new_high_precision().with_context(|| "Failed to initialise event loop")?;

    let (display, socket) = init_wayland_display(&mut event_loop)?;

    let mut state = State::new(
        &display.handle(),
        socket,
        event_loop.handle(),
        event_loop.get_signal(),
        log,
    );

    backend::init_backend(&display.handle(), &mut event_loop, &mut state)?;

    let mut data = Arc::new(Mutex::new(Data { display, state }));

    let runtime = runtime::Runtime::new(data.clone());
    runtime.run_with_calloop(event_loop.handle());

    event_loop
        .run(None, &mut data, |data| {
            let mut data = data.lock().unwrap();

            // Shut down
            if data.state.common.shell.outputs().next().is_none() || data.state.common.should_stop {
                slog_scope::info!("Shutting down");
                data.state.common.event_loop_signal.stop();
                data.state.common.event_loop_signal.wakeup();
                return;
            }

            let handle = &data.display.handle();
            data.state.common.shell.refresh(handle);
            data.state.common.refresh_focus(handle);

            // Send events to Clients
            let _ = data.display.flush_clients();
        })
        .expect("Failed to run Event Loop");

//     let _log = data.state.destroy_with_log();
    std::mem::drop(event_loop);
    Ok(())
}

fn init_wayland_display(
    event_loop: &mut EventLoop<LoopData>,
) -> Result<(Display<State>, OsString), Box<dyn Error>> {
    let mut display = Display::new().unwrap();

    let socket_source = ListeningSocketSource::new_auto(None).expect("Failed to register socket");
    let socket_name = socket_source.socket_name().to_os_string();
    slog_scope::info!("Listening on {:?}", socket_name);

    event_loop
        .handle()
        .insert_source(socket_source, |stream, _, data| {
            let data = data.lock().unwrap();
            if let Err(err) = data.display.handle().insert_client(
                stream,
                Arc::new(if cfg!(debug_assertions) {
                    // TODO privledges?
                    data.state.new_client_state()
                } else {
                    data.state.new_client_state()
                }),
            ) {
                slog_scope::warn!("Error adding wayland client: {}", err);
            };
        })
        .with_context(|| "Failed to initialise Wayland socket")?;

    event_loop
        .handle()
        .insert_source(
            Generic::new(display.backend().poll_fd(), Interest::READ, Mode::Level),
            move |_, _, data: &mut LoopData| {
                let mut data = data.lock().unwrap();
                let data = data.deref_mut();
                let state = &mut data.state;
                match data.display.dispatch_clients(state) {
                    Ok(_) => Ok(PostAction::Continue),
                    Err(e) => {
                        slog_scope::error!("I/O Error on display: {}", e);
                        data.state.common.should_stop = true;
                        Err(e)
                    }
                }
            },
        )
        .with_context(|| "Failed to initialise Wayland event source")?;

    Ok((display, socket_name))
}
