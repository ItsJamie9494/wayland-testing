// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use crate::{
    state::{BackendData, Data},
    State,
};
use anyhow::{anyhow, Context};
use smithay::{
    backend::{
        input::{InputEvent, KeyboardKeyEvent},
        renderer::ImportDma,
        winit::{self, WinitEvent, WinitGraphicsBackend},
    },
    reexports::{
        calloop::{ping, EventLoop},
        wayland_server::{
            protocol::wl_output::{Subpixel, Transform},
            DisplayHandle,
        },
    },
    wayland::{
        output::{Mode, Output, PhysicalProperties, Scale},
        seat::FilterResult,
    },
};

use self::state::WinitState;

pub mod state;

pub fn init_backend(
    dh: &DisplayHandle,
    event_loop: &mut EventLoop<Data>,
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    let (backend, mut winit) =
        winit::init(None).map_err(|_| anyhow!("Failed to initialise Winit backend"))?;

    init_egl(dh, state, &mut backend);

    let name = format!("WINIT-0");
    let props = PhysicalProperties {
        size: (0, 0).into(),
        subpixel: Subpixel::Unknown,
        make: String::from("ELECTRUM"),
        model: name.clone(),
    };
    let size = backend.window_size();
    let mode = Mode {
        size: (size.physical_size.w as i32, size.physical_size.h as i32).into(),
        refresh: 60_000,
    };
    let output = Output::new(name, props, None);
    let _global = output.create_global::<State>(dh);
    output.add_mode(mode);
    output.set_preferred(mode);
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        Some(Scale::Integer(1)),
        Some((0, 0).into()),
    );

    let (event_ping, event_source) =
        ping::make_ping().with_context(|| "Failed to init eventloop timer for winit")?;
    let (render_ping, render_source) =
        ping::make_ping().with_context(|| "Failed to init eventloop timer for winit")?;
    let event_ping_handle = event_ping.clone();
    let render_ping_handle = render_ping.clone();
    let mut token = Some(
        event_loop
            .handle()
            .insert_source(render_source, move |_, _, data| {
                if let Err(err) = data
                    .state
                    .backend
                    .winit()
                    .render_output(&mut data.state.common)
                {
                    slog_scope::error!("Failed to render frame: {}", err);
                    render_ping.ping();
                }
            })
            .map_err(|_| anyhow::anyhow!("Failed to init eventloop timer for winit"))?,
    );
    let event_loop_handle = event_loop.handle();
    event_loop
        .handle()
        .insert_source(event_source, move |_, _, data| {
            match winit.dispatch_new_events(|event| {
                data.state
                    .process_winit_event(&data.display.handle(), event, &render_ping_handle)
            }) {
                Ok(_) => {
                    event_ping_handle.ping();
                    render_ping_handle.ping();
                }
                Err(winit::WinitError::WindowClosed) => {
                    if let Some(token) = token.take() {
                        // TODO remove output from ElectrumShell
                        event_loop_handle.remove(token);
                    }
                }
            };
        })
        .map_err(|_| anyhow::anyhow!("Failed to init eventloop timer for winit"))?;
    event_ping.ping();

    state.backend = BackendData::Winit(WinitState {
        backend,
        output: output.clone(),
        age_reset: 0,
    });

    Ok(())
}

fn init_egl(
    dh: &DisplayHandle,
    state: &mut State,
    renderer: &mut WinitGraphicsBackend,
) -> Result<(), Box<dyn Error>> {
    slog_scope::info!("EGL Hardware Acceleration should be enabled");
    let formats = renderer
        .renderer()
        .dmabuf_formats()
        .cloned()
        .collect::<Vec<_>>();
    state
        .common
        .dmabuf_state
        .create_global::<State, _>(dh, formats, slog_scope::logger());

    Ok(())
}

// TODO: Forward all events to input handlers
impl State {
    pub fn process_winit_event(
        &mut self,
        dh: &DisplayHandle,
        event: WinitEvent,
        render_ping: &ping::Ping,
    ) {
        let keyboard = self
            .common
            .seat
            .add_keyboard(Default::default(), 200, 200, |_, _| {})
            .unwrap();

        match event {
            WinitEvent::Resized { .. } => {}
            WinitEvent::Focus(_) => {}
            WinitEvent::Input(event) => match event {
                InputEvent::Keyboard { event } => {
                    keyboard.input::<(), _>(
                        dh,
                        event.key_code(),
                        event.state(),
                        0.into(),
                        0,
                        |_, _| {
                            //
                            FilterResult::Forward
                        },
                    );
                }
                InputEvent::PointerMotionAbsolute { .. } => {
                    self.common.shell.toplevel_surfaces(|surfaces| {
                        if let Some(surface) = surfaces.iter().next() {
                            let surface = surface.wl_surface();
                            keyboard.set_focus(dh, Some(surface), 0.into());
                        }
                    });
                }
                _ => {}
            },
            WinitEvent::Refresh => render_ping.ping(),
        }
    }
}
