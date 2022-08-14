// SPDX-License-Identifier: GPL-3.0-only

use std::{ffi::OsString, time::Instant};

use smithay::{
    reexports::{
        calloop::{LoopHandle, LoopSignal},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            Display, DisplayHandle,
        },
    },
    wayland::{
        compositor::CompositorState,
        data_device::DataDeviceState,
        dmabuf::DmabufState,
        output::OutputManagerState,
        seat::{Seat, SeatState},
        shm::ShmState,
        viewporter::ViewporterState,
    },
};

use crate::{backend::winit::state::WinitState, log::LogState};

mod buffer;
mod compositor;
mod data_device;
mod dmabuf;
mod output;
mod seat;
mod shm;
mod viewporter;

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

pub struct ClientState {}
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

    // pub shell: Shell,
    pub seats: Vec<Seat<State>>,

    pub start_time: Instant,
    pub should_stop: bool,
    pub log: LogState,

    // Wayland State
    pub compositor_state: CompositorState,
    pub data_device_state: DataDeviceState,
    pub dmabuf_state: DmabufState,
    pub output_state: OutputManagerState,
    pub seat_state: SeatState<State>,
    pub shm_state: ShmState,
    pub viewporter_state: ViewporterState,
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

                // TODO: Have input managers handle this
                seats: vec![Seat::<Self>::new(&dh, "seat-0", None)],

                start_time: Instant::now(),
                should_stop: false,
                log: log,

                compositor_state: CompositorState::new::<Self, _>(dh, slog_scope::logger()),
                data_device_state: DataDeviceState::new::<Self, _>(dh, slog_scope::logger()),
                dmabuf_state: DmabufState::new(),
                output_state: OutputManagerState::new_with_xdg_output::<Self>(dh),
                seat_state: SeatState::<Self>::new(),
                shm_state: ShmState::new::<Self, _>(dh, vec![], slog_scope::logger()),
                viewporter_state: ViewporterState::new::<Self, _>(dh, slog_scope::logger()),
            },
        }
    }

    pub fn new_client_state(&self) -> ClientState {
        ClientState {}
    }

    pub fn destroy_with_log(self) -> LogState {
        self.common.log
    }
}
