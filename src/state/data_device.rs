// SPDX-License-Identifier: GPL-3.0-only

use smithay::{wayland::data_device::{DataDeviceHandler, DataDeviceState, ServerDndGrabHandler, ClientDndGrabHandler}, delegate_data_device};

use super::State;

impl ClientDndGrabHandler for State {

}
impl ServerDndGrabHandler for State {}
impl DataDeviceHandler for State {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.common.data_device_state
    }
}

delegate_data_device!(State);