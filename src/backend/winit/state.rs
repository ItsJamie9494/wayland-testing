// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use anyhow::Context;
use smithay::{
    backend::{
        renderer::{utils::draw_surface_tree, Frame, Renderer},
        winit::WinitGraphicsBackend,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::Rectangle,
    wayland::{
        compositor::{with_surface_tree_downward, SurfaceAttributes, TraversalAction},
        output::Output,
    },
};

use crate::state::CommonState;

pub struct WinitState {
    pub backend: WinitGraphicsBackend,
    pub _output: Output,
}

impl WinitState {
    pub fn render_output(&mut self, state: &mut CommonState) -> Result<(), Box<dyn Error>> {
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
