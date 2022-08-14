// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use anyhow::Context;
use smithay::{backend::winit::WinitGraphicsBackend, wayland::output::Output};

use crate::{backend::renderer, state::CommonState};

pub struct WinitState {
    pub backend: WinitGraphicsBackend,
    pub output: Output,
    pub age_reset: u8,
}

impl WinitState {
    pub fn render_output(&mut self, state: &mut CommonState) -> Result<(), Box<dyn Error>> {
        if renderer::needs_buffer_reset(&self.output, state) {
            self.reset_buffers();
        }

        self.backend
            .bind()
            .with_context(|| "Failed to bind buffer")?;

        let age = if self.age_reset > 0 {
            self.age_reset -= 1;
            0
        } else {
            self.backend.buffer_age().unwrap_or(0)
        };

        match renderer::render_output(
            self.backend.renderer(),
            age as u8,
            state,
            &self.output,
            true,
        ) {
            Ok(damage) => {
                state
                    .shell
                    .active_workspace_mut()
                    .space
                    .send_frames(state.start_time.elapsed().as_millis() as u32);
                self.backend
                    .submit(damage.as_ref().map(|x| &**x))
                    .with_context(|| "Failed to submit buffer for display")?;
            }
            Err(err) => {
                // TODO handle errors better
                slog_scope::error!("Rendering failed {}", err);
                //anyhow::bail!("Rendering failed: {}", err);
            }
        };

        Ok(())
    }

    pub fn reset_buffers(&mut self) {
        self.age_reset = 3;
    }
}
