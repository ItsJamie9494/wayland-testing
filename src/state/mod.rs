// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::OsString;

use smithay::{
    backend::drm::DrmNode,
    reexports::{
        calloop::{LoopHandle, LoopSignal},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            Display, DisplayHandle,
        },
    },
    wayland::{
        compositor::CompositorState,
        dmabuf::DmabufState,
        output::OutputManagerState,
        seat::{Seat, SeatState},
        shell::xdg::XdgShellState,
        shm::ShmState,
    },
};

use crate::{backend::winit::state::WinitState, log::LogState};

mod buffer;
mod compositor;
mod dmabuf;
mod output;
mod seat;
mod shm;
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

pub struct ClientState {
    pub drm_node: Option<DrmNode>,
    pub privileged: bool,
}
impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

pub struct Data {
    pub display: Display<State>,
    pub state: State,
}

pub struct State {
    pub backend: BackendData,
    pub common: CommonState,
}

pub struct CommonState {
    pub socket: OsString,
    pub event_loop_handle: LoopHandle<'static, Data>,
    pub event_loop_signal: LoopSignal,

    pub should_stop: bool,

    pub log: LogState,

    pub seat: Seat<State>,
    pub shell: XdgShellState,

    pub compositor_state: CompositorState,
    pub dmabuf_state: DmabufState,
    pub output_state: OutputManagerState,
    pub seat_state: SeatState<State>,
    pub shm_state: ShmState,
}

impl State {
    pub fn new(
        dh: &DisplayHandle,
        socket: OsString,
        handle: LoopHandle<'static, Data>,
        signal: LoopSignal,
        log: LogState,
    ) -> State {
        State {
            backend: BackendData::Unset,
            common: CommonState {
                socket: socket,
                event_loop_handle: handle,
                event_loop_signal: signal,

                should_stop: false,

                log: log,

                seat: Seat::<Self>::new(&dh, "seat-0", None),
                shell: XdgShellState::new::<Self, _>(&dh, None),

                compositor_state: CompositorState::new::<Self, _>(dh, None),
                dmabuf_state: DmabufState::new(),
                output_state: OutputManagerState::new_with_xdg_output::<Self>(dh),
                seat_state: SeatState::<Self>::new(),
                shm_state: ShmState::new::<Self, _>(dh, vec![], None),
            },
        }
    }

    pub fn new_client_state(&self) -> ClientState {
        ClientState {
            drm_node: match &self.backend {
                _ => None,
            },
            privileged: false,
        }
    }

    pub fn new_privileged_client_state(&self) -> ClientState {
        ClientState {
            drm_node: match &self.backend {
                _ => None,
            },
            privileged: true,
        }
    }

    pub fn destroy_with_log(self) -> LogState {
        self.common.log
    }
}
