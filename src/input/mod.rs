// SPDX-License-Identifier: GPL-3.0-only

use smithay::backend::input::{Device, DeviceCapability, InputBackend, InputEvent};
use smithay::reexports::wayland_server::DisplayHandle;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::state::State;

#[derive(Default)]
pub struct Devices(RefCell<HashMap<String, Vec<DeviceCapability>>>);

impl Devices {
    pub fn has_device<D: Device>(&self, device: &D) -> bool {
        self.0.borrow().contains_key(&device.id())
    }
}

impl State {
    pub fn process_input_event<B: InputBackend>(
        &mut self,
        dh: &DisplayHandle,
        event: InputEvent<B>,
    ) {
        match event {
            InputEvent::DeviceAdded { device } => todo!(),
            InputEvent::DeviceRemoved { device } => todo!(),
            InputEvent::Keyboard { event } => todo!(),
            InputEvent::PointerMotion { event } => todo!(),
            InputEvent::PointerMotionAbsolute { event } => todo!(),
            InputEvent::PointerButton { event } => todo!(),
            InputEvent::PointerAxis { event } => todo!(),
            InputEvent::TouchDown { event } => todo!(),
            InputEvent::TouchMotion { event } => todo!(),
            InputEvent::TouchUp { event } => todo!(),
            InputEvent::TouchCancel { event } => todo!(),
            InputEvent::TouchFrame { event } => todo!(),
            InputEvent::TabletToolAxis { event } => todo!(),
            InputEvent::TabletToolProximity { event } => todo!(),
            InputEvent::TabletToolTip { event } => todo!(),
            InputEvent::TabletToolButton { event } => todo!(),
            InputEvent::Special(_) => todo!(),
        }
    }
}
