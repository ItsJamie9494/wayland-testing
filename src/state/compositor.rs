// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Mutex;

use smithay::{
    backend::renderer::utils::{on_commit_buffer_handler, with_renderer_surface_state},
    delegate_compositor,
    desktop::{layer_map_for_output, Kind, LayerSurface, PopupKind, WindowSurfaceType},
    reexports::wayland_server::{protocol::wl_surface::WlSurface, DisplayHandle},
    wayland::{
        compositor::{with_states, CompositorHandler, CompositorState},
        shell::{
            wlr_layer::LayerSurfaceAttributes,
            xdg::{
                ToplevelSurface, XdgPopupSurfaceRoleAttributes, XdgToplevelSurfaceRoleAttributes,
            },
        },
    },
};

use super::{output::active_output, State};

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.common.compositor_state
    }

    fn commit(&mut self, dh: &DisplayHandle, surface: &WlSurface) {
        // Load buffers
        on_commit_buffer_handler(surface);

        // Map Windows, Layers, Popups
        if let Some((window, seat)) = self
            .common
            .shell
            .pending_windows
            .iter()
            .find(|(window, _)| window.toplevel().wl_surface() == surface)
            .cloned()
        {
            match window.toplevel() {
                Kind::Xdg(toplevel) => {
                    if self.toplevel_ensure_initial_configure(&toplevel)
                        && with_renderer_surface_state(&surface, |state| {
                            state.wl_buffer().is_some()
                        })
                    {
                        // TODO: Active Output
                        let output = active_output(&seat, &self.common);
                        self.common.shell.map_window(&window, &output, dh);
                    } else {
                        return;
                    }
                }
            }
        }

        if let Some((layer_surface, _, _)) = self
            .common
            .shell
            .pending_layers
            .iter()
            .find(|(layer_surface, _, _)| layer_surface.wl_surface() == surface)
            .cloned()
        {
            if !self.layer_surface_ensure_inital_configure(&layer_surface, dh) {
                return;
            }
        };

        if let Some(popup) = self.common.shell.popups.find_popup(surface) {
            self.xdg_popup_ensure_initial_configure(&popup);
        }

        // Handle weird special shit, probably in Deno?

        // Commit and finalise mappings
        self.common.shell.popups.commit(surface);
        for workspace in &self.common.shell.workspaces {
            workspace.space.commit(surface);
        }

        if let Some(output) = self.common.shell.outputs().find(|o| {
            let map = layer_map_for_output(o);
            map.layer_for_surface(surface, WindowSurfaceType::ALL)
                .is_some()
        }) {
            layer_map_for_output(output).arrange(dh);
        }
    }
}

impl State {
    fn toplevel_ensure_initial_configure(&mut self, toplevel: &ToplevelSurface) -> bool {
        let initial_configure_sent = with_states(toplevel.wl_surface(), |states| {
            states
                .data_map
                .get::<Mutex<XdgToplevelSurfaceRoleAttributes>>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        });
        if !initial_configure_sent {
            toplevel.with_pending_state(|states| states.size = None);
            toplevel.send_configure();
        }
        initial_configure_sent
    }

    fn layer_surface_ensure_inital_configure(
        &mut self,
        surface: &LayerSurface,
        dh: &DisplayHandle,
    ) -> bool {
        let initial_configure_sent = with_states(surface.wl_surface(), |states| {
            states
                .data_map
                .get::<Mutex<LayerSurfaceAttributes>>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        });
        if !initial_configure_sent {
            self.common.shell.map_layer(&surface, dh);
        }
        initial_configure_sent
    }

    fn xdg_popup_ensure_initial_configure(&mut self, popup: &PopupKind) {
        let PopupKind::Xdg(ref popup) = popup;
        let initial_configure_sent = with_states(popup.wl_surface(), |states| {
            states
                .data_map
                .get::<Mutex<XdgPopupSurfaceRoleAttributes>>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        });
        if !initial_configure_sent {
            popup.send_configure().expect("initial configure failed");
        }
    }
}

delegate_compositor!(State);
