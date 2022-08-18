// SPDX-License-Identifier: GPL-3.0-only

use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use calloop::channel::Sender;
use indexmap::IndexSet;
use smithay::{
    desktop::{Kind, Space, Window},
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel::{self, ResizeEdge},
        wayland_server::{protocol::wl_surface::WlSurface, DisplayHandle},
    },
    utils::IsAlive,
    wayland::{
        output::Output,
        seat::{PointerGrabStartData, Seat},
        Serial,
    },
};

use crate::{runtime::messages::RuntimeMessage, state::State};

pub struct FocusStack<'a>(Ref<'a, IndexSet<Window>>);
pub struct FocusStackMut<'a>(RefMut<'a, IndexSet<Window>>);

impl<'a> FocusStack<'a> {
    pub fn last(&self) -> Option<Window> {
        self.0.iter().rev().find(|w| w.toplevel().alive()).cloned()
    }
}

impl<'a> FocusStackMut<'a> {
    pub fn append(&mut self, window: &Window) {
        self.0.retain(|w| w.toplevel().alive());
        self.0.shift_remove(window);
        self.0.insert(window.clone());
    }

    pub fn last(&self) -> Option<Window> {
        self.0.iter().rev().find(|w| w.toplevel().alive()).cloned()
    }
}

type FocusStackData = RefCell<(HashMap<u8, IndexSet<Window>>, IndexSet<Window>)>;

pub struct ActiveFocus(RefCell<Option<WlSurface>>);

impl ActiveFocus {
    pub fn get(seat: &Seat<State>) -> Option<WlSurface> {
        seat.user_data()
            .get::<ActiveFocus>()
            .and_then(|a| a.0.borrow().clone())
    }
}

pub struct Workspace {
    pub idx: u8,
    pub space: Space,
    pub fullscreen: HashMap<String, Window>,
    pub runtime_sender: Sender<RuntimeMessage>,
}

impl Workspace {
    pub fn new(idx: u8, rs: Sender<RuntimeMessage>) -> Self {
        Self {
            idx,
            space: Space::new(slog_scope::logger()),
            fullscreen: HashMap::new(),
            runtime_sender: rs,
        }
    }

    pub fn refresh(&mut self, dh: &DisplayHandle) {
        let outputs = self.space.outputs().collect::<Vec<_>>();
        let dead_windows = self
            .fullscreen
            .iter()
            .filter(|(name, _)| !outputs.iter().any(|o| o.name() == **name))
            .map(|(_, w)| w)
            .cloned()
            .collect::<Vec<_>>();
        for window in dead_windows {
            self.unfullscreen_request(&window);
        }
        self.fullscreen.retain(|_, w| w.alive());
        self.space.refresh(dh);
    }

    /// Deno Function
    pub fn maximize_request(&mut self, window: &Window, output: &Output) {
        if self.fullscreen.values().any(|w| w == window) {
            return;
        }

        self.runtime_sender
            .send(RuntimeMessage::MaximizeRequest {
                window: window.clone(),
                output: output.clone(),
            })
            .unwrap();
    }

    /// Deno Function
    pub fn unmaximize_request(&mut self, window: &Window) {
        if self.fullscreen.values().any(|w| w == window) {
            return self.unfullscreen_request(window);
        }

        self.runtime_sender
            .send(RuntimeMessage::UnmaximizeRequest {
                window: window.clone(),
            })
            .unwrap();
    }

    /// Deno Function
    pub fn resize_request(
        &mut self,
        window: &Window,
        seat: &Seat<State>,
        serial: Serial,
        start_data: PointerGrabStartData,
        edges: ResizeEdge,
    ) {
        if self.fullscreen.values().any(|w| w == window) {
            return;
        }

        self.runtime_sender
            .send(RuntimeMessage::ResizeRequest {
                window: window.clone(),
                seat: seat.clone(),
                serial,
                start_data,
                edges,
            })
            .unwrap();
    }

    pub fn fullscreen_request(&mut self, window: &Window, output: &Output) {
        if self.fullscreen.contains_key(&output.name()) {
            return;
        }

        #[allow(irrefutable_let_patterns)]
        if let Kind::Xdg(xdg) = &window.toplevel() {
            xdg.with_pending_state(|state| {
                state.states.set(xdg_toplevel::State::Fullscreen);
                state.size = Some(
                    output
                        .current_mode()
                        .map(|m| m.size)
                        .unwrap_or((0, 0).into())
                        .to_f64()
                        .to_logical(output.current_scale().fractional_scale())
                        .to_i32_round(),
                );
            });

            xdg.send_configure();
            self.fullscreen.insert(output.name(), window.clone());
        }
    }

    /// Deno Function
    pub fn unfullscreen_request(&mut self, window: &Window) {
        if self.fullscreen.values().any(|w| w == window) {
            #[allow(irrefutable_let_patterns)]
            if let Kind::Xdg(xdg) = &window.toplevel() {
                xdg.with_pending_state(|state| {
                    state.states.unset(xdg_toplevel::State::Fullscreen);
                    state.size = None;
                });
                xdg.send_configure();
            }

            self.runtime_sender
                .send(RuntimeMessage::UnfullscreenRequest {
                    window: window.clone(),
                })
                .unwrap();

            self.fullscreen.retain(|_, w| w != window);
        }
    }

    pub fn get_fullscreen(&self, output: &Output) -> Option<&Window> {
        if !self.space.outputs().any(|o| o == output) {
            return None;
        }

        self.fullscreen.get(&output.name()).filter(|w| w.alive())
    }

    pub fn focus_stack<'a, 'b>(&'b self, seat: &'a Seat<State>) -> FocusStack<'a> {
        seat.user_data()
            .insert_if_missing(|| FocusStackData::new((HashMap::new(), IndexSet::new())));
        let idx = self.idx;
        FocusStack(Ref::map(
            seat.user_data().get::<FocusStackData>().unwrap().borrow(),
            |map| map.0.get(&idx).unwrap_or(&map.1),
        ))
    }

    pub fn focus_stack_mut<'a, 'b>(&'b self, seat: &'a Seat<State>) -> FocusStackMut<'a> {
        seat.user_data()
            .insert_if_missing(|| FocusStackData::new((HashMap::new(), IndexSet::new())));
        let idx = self.idx;
        FocusStackMut(RefMut::map(
            seat.user_data()
                .get::<FocusStackData>()
                .unwrap()
                .borrow_mut(),
            |map| map.0.entry(idx).or_insert_with(|| IndexSet::new()),
        ))
    }
}
