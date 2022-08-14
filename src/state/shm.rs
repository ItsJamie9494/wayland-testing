// SPDX-License-Identifier: GPL-3.0-only

use smithay::{
    delegate_shm,
    wayland::shm::{ShmHandler, ShmState},
};

use super::State;

impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.common.shm_state
    }
}

delegate_shm!(State);
