// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use crate::{
    state::{BackendData, Common, Data},
    State,
};
use anyhow::{anyhow, Context};
use smithay::{
    backend::{
        input::{InputEvent, KeyboardKeyEvent},
        renderer::{utils::draw_surface_tree, Frame, Renderer},
        winit::{self, WinitEvent, WinitGraphicsBackend},
    },
    reexports::{
        calloop::{ping, EventLoop},
        wayland_server::{
            protocol::{
                wl_output::{Subpixel, Transform},
                wl_surface::WlSurface,
            },
            DisplayHandle,
        },
    },
    utils::Rectangle,
    wayland::{
        compositor::{with_surface_tree_downward, SurfaceAttributes, TraversalAction},
        output::{Mode, Output, PhysicalProperties, Scale},
        seat::FilterResult,
    },
};

pub struct WinitState {
    pub backend: WinitGraphicsBackend,
    _output: Output,
}

impl WinitState {
    pub fn render_output(&mut self, state: &mut Common) -> Result<(), Box<dyn Error>> {
        self.backend
            .bind()
            .with_context(|| "Failed to bind buffer")?;

        let size = self.backend.window_size().physical_size;
        let damage = Rectangle::from_loc_and_size((0, 0), size);

        let start_time = std::time::Instant::now();

        self.backend.renderer().render(
            size,
            smithay::utils::Transform::Flipped180,
            |renderer, frame| {
                frame.clear([0.1, 0.0, 0.0, 1.0], &[damage]).unwrap();

                state.shell.toplevel_surfaces(|surfaces| {
                    for surface in surfaces {
                        let surface = surface.wl_surface();
                        draw_surface_tree(
                            renderer,
                            frame,
                            surface,
                            1.0,
                            (0.0, 0.0).into(),
                            &[damage],
                            &Self::log(),
                        )
                        .unwrap();

                        Self::send_frames_surface_tree(
                            surface,
                            start_time.elapsed().as_millis() as u32,
                        );
                    }
                });
            },
        )?;

        self.backend.submit(Some(&[damage])).unwrap();
        Ok(())
    }

    fn log() -> ::slog::Logger {
        use slog::Drain;
        ::slog::Logger::root(::slog_stdlog::StdLog.fuse(), slog::o!())
    }

    fn send_frames_surface_tree(surface: &WlSurface, time: u32) {
        with_surface_tree_downward(
            surface,
            (),
            |_, _, &()| TraversalAction::DoChildren(()),
            |_surf, states, &()| {
                // the surface may not have any user_data if it is a subsurface and has not
                // yet been commited
                for callback in states
                    .cached_state
                    .current::<SurfaceAttributes>()
                    .frame_callbacks
                    .drain(..)
                {
                    callback.done(time);
                }
            },
            |_, _, &()| true,
        );
    }
}

pub fn init_backend(event_loop: &mut EventLoop<Data>, state: &mut State) -> Result<(), ()> {
    let (backend, mut winit) = winit::init(None)
        .map_err(|_| anyhow!("Failed to initialise Winit backend"))
        .unwrap();

    let name = format!("WINIT-0");
    let props = PhysicalProperties {
        size: (0, 0).into(),
        subpixel: Subpixel::Unknown,
        make: "ELECTRUM-DEBUG".to_string(),
        model: name.clone(),
    };
    let size = backend.window_size();
    let mode = Mode {
        size: (size.physical_size.w as i32, size.physical_size.h as i32).into(),
        refresh: 60_000,
    };
    let output = Output::new(name, props, None);
    output.add_mode(mode);
    output.set_preferred(mode);
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        Some(Scale::Integer(1)),
        Some((0, 0).into()),
    );

    let (event_ping, event_source) = ping::make_ping()
        .with_context(|| "Failed to init eventloop timer for winit")
        .unwrap();
    let (render_ping, render_source) = ping::make_ping()
        .with_context(|| "Failed to init eventloop timer for winit")
        .unwrap();
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
            .map_err(|_| anyhow::anyhow!("Failed to init eventloop timer for winit"))
            .unwrap(),
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
                        event_loop_handle.remove(token);
                    }
                }
            };
        })
        .map_err(|_| anyhow::anyhow!("Failed to init eventloop timer for winit"))
        .unwrap();
    event_ping.ping();

    state.backend = BackendData::Winit(WinitState {
        backend,
        _output: output.clone(),
    });

    Ok(())
}

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
