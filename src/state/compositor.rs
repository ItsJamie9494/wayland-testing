// SPDX-License-Identifier: GPL-3.0-only

use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor,
    reexports::wayland_server::{protocol::wl_surface::WlSurface, DisplayHandle},
    wayland::compositor::{CompositorHandler, CompositorState},
};

use super::State;

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.common.compositor_state
    }

    fn commit(&mut self, _dh: &DisplayHandle, surface: &WlSurface) {
        // Load buffers
        on_commit_buffer_handler(surface);
    }
}

delegate_compositor!(State);
