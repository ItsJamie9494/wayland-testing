// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::OsString;

use smithay::{
    reexports::wayland_server::{Display, DisplayHandle},
    wayland::{
        seat::{Seat, SeatState},
        shell::xdg::XdgShellState,
    },
};

use crate::{backend::winit::WinitState, log::LogState};

mod seat;
mod wayland;

pub enum BackendData {
    Winit(WinitState),
    Unset,
}

impl BackendData {
    pub fn winit(&mut self) -> &mut WinitState {
        match self {
            BackendData::Winit(ref mut winit_state) => winit_state,
            _ => unreachable!("Called winit() in non-winit backend"),
        }
    }
}

pub struct Data {
    pub display: Display<State>,
    pub state: State,
}

pub struct State {
    pub backend: BackendData,
    pub common: Common,
}

pub struct Common {
    pub socket: OsString,

    pub log: LogState,

    pub shell: XdgShellState,
    pub seat_state: SeatState<State>,
    pub seat: Seat<State>,
}

impl State {
    pub fn new(display_handle: &DisplayHandle, socket: OsString, log: LogState) -> State {
        State {
            backend: BackendData::Unset,
            common: Common {
                socket: socket,

                log: log,

                shell: XdgShellState::new::<Self, _>(&display_handle, None),
                seat_state: SeatState::<Self>::new(),
                seat: Seat::<Self>::new(&display_handle, "seat-0", None),
            },
        }
    }
}
