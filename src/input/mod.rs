// SPDX-License-Identifier: GPL-3.0-only

use smithay::backend::input::{Device, DeviceCapability, InputBackend, InputEvent};

use smithay::reexports::wayland_server::DisplayHandle;
use smithay::wayland::seat::{CursorImageStatus, Seat};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::state::State;

#[derive(Default)]
pub struct SupressedKeys(RefCell<Vec<u32>>);
#[derive(Default)]
pub struct Devices(RefCell<HashMap<String, Vec<DeviceCapability>>>);

impl Devices {
    fn add_device<D: Device>(&self, device: &D) -> Vec<DeviceCapability> {
        let id = device.id();
        let mut map = self.0.borrow_mut();
        let caps = [DeviceCapability::Keyboard, DeviceCapability::Pointer]
            .iter()
            .cloned()
            .filter(|c| device.has_capability(*c))
            .collect::<Vec<_>>();
        let new_caps = caps
            .iter()
            .cloned()
            .filter(|c| map.values().flatten().all(|has| *c != *has))
            .collect::<Vec<_>>();
        map.insert(id, caps);
        new_caps
    }

    fn remove_device<D: Device>(&self, device: &D) -> Vec<DeviceCapability> {
        let id = device.id();
        let mut map = self.0.borrow_mut();
        map.remove(&id)
            .unwrap_or(Vec::new())
            .into_iter()
            .filter(|c| map.values().flatten().all(|has| *c != *has))
            .collect()
    }

    pub fn has_device<D: Device>(&self, device: &D) -> bool {
        self.0.borrow().contains_key(&device.id())
    }
}

pub fn add_seat(dh: &DisplayHandle, name: String) -> Seat<State> {
    let mut seat = Seat::<State>::new(dh, name, None);
    let userdata = seat.user_data();
    // userdata.insert_if_missing(SeatId::default);
    userdata.insert_if_missing(Devices::default);
    userdata.insert_if_missing(SupressedKeys::default);
    userdata.insert_if_missing(|| RefCell::new(CursorImageStatus::Default));

    let owned_seat = seat.clone();
    seat.add_pointer(move |status| {
        *owned_seat
            .user_data()
            .get::<RefCell<CursorImageStatus>>()
            .unwrap()
            .borrow_mut() = status;
    });

    seat
}

impl State {
    pub fn process_input_event<B: InputBackend>(
        &mut self,
        _dh: &DisplayHandle,
        event: InputEvent<B>,
    ) {
        match event {
            InputEvent::DeviceAdded { device } => {
                let seat = &mut self.common.last_active_seat;
                let userdata = seat.user_data();
                let devices = userdata.get::<Devices>().unwrap();
                for cap in devices.add_device(&device) {
                    match cap {
                        // TODO: Handle touch, tablet
                        _ => {}
                    }
                }
            }
            InputEvent::DeviceRemoved { device } => {
                for seat in &mut self.common.seats {
                    let userdata = seat.user_data();
                    let devices = userdata.get::<Devices>().unwrap();
                    if devices.has_device(&device) {
                        for cap in devices.remove_device(&device) {
                            match cap {
                                // TODO: Handle touch, tablet
                                _ => {}
                            }
                        }
                        break;
                    }
                }
            }
            InputEvent::Keyboard { event: _ } => {}
            InputEvent::PointerMotion { event: _ } => {}
            InputEvent::PointerMotionAbsolute { event: _ } => {}
            InputEvent::PointerButton { event: _ } => {}
            InputEvent::PointerAxis { event: _ } => {}
            InputEvent::TouchDown { event: _ } => {}
            InputEvent::TouchMotion { event: _ } => {}
            InputEvent::TouchUp { event: _ } => {}
            InputEvent::TouchCancel { event: _ } => {}
            InputEvent::TouchFrame { event: _ } => {}
            InputEvent::TabletToolAxis { event: _ } => {}
            InputEvent::TabletToolProximity { event: _ } => {}
            InputEvent::TabletToolTip { event: _ } => {}
            InputEvent::TabletToolButton { event: _ } => {}
            InputEvent::Special(_) => {}
        }
    }
}
