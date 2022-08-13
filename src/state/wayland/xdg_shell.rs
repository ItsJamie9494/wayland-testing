// SPDX-License-Identifier: GPL-3.0-only

use smithay::{
    delegate_xdg_shell,
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{protocol::wl_seat::WlSeat, DisplayHandle},
    },
    wayland::{
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
        },
        Serial,
    },
};

use crate::state::State;

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.common.shell
    }

    fn new_toplevel(&mut self, _dh: &DisplayHandle, surface: ToplevelSurface) {
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
        });
        surface.send_configure();
    }

    fn new_popup(
        &mut self,
        _dh: &DisplayHandle,
        _surface: PopupSurface,
        _positioner: PositionerState,
    ) {
        todo!()
    }

    fn grab(
        &mut self,
        _dh: &DisplayHandle,
        _surface: PopupSurface,
        _seat: WlSeat,
        _serial: Serial,
    ) {
        todo!()
    }
}

delegate_xdg_shell!(State);
