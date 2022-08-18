//! This Implementation, ideally, will purely be callbacks to an FFI.
//! There should be minimal amounts of code here, any code here is either for debug purposes or in development

use calloop::channel::Sender;
use smithay::{
    desktop::{layer_map_for_output, LayerSurface, PopupManager, Window, WindowSurfaceType},
    reexports::wayland_server::{protocol::wl_surface::WlSurface, DisplayHandle},
    utils::{Logical, Point},
    wayland::{
        compositor::with_states,
        output::Output,
        seat::{PointerGrabStartData, Seat},
        shell::{
            wlr_layer::{
                KeyboardInteractivity, Layer, LayerSurfaceCachedState, WlrLayerShellState,
            },
            xdg::{PopupSurface, PositionerState, XdgShellState},
        },
        Serial,
    },
};

pub mod workspace;

use crate::{runtime::messages::RuntimeMessage, state::State};

use self::workspace::Workspace;

pub struct Shell {
    pub workspaces: Vec<Workspace>,
    pub outputs: Vec<Output>,
    pub popups: PopupManager,

    pub pending_windows: Vec<(Window, Seat<State>)>,
    pub pending_layers: Vec<(LayerSurface, Output, Seat<State>)>,

    // Wayland State
    pub layer_shell_state: WlrLayerShellState,
    pub xdg_shell_state: XdgShellState,
}

impl Shell {
    pub fn new(dh: &DisplayHandle, rs: Sender<RuntimeMessage>) -> Self {
        Self {
            // TODO: Make a way to create new Workspaces
            workspaces: vec![Workspace::new(0, rs)],
            outputs: Vec::new(),
            popups: PopupManager::new(slog_scope::logger()),

            pending_windows: Vec::new(),
            pending_layers: Vec::new(),

            layer_shell_state: WlrLayerShellState::new::<State, _>(dh, slog_scope::logger()),
            xdg_shell_state: XdgShellState::new::<State, _>(dh, slog_scope::logger()),
        }
    }

    pub fn outputs(&self) -> impl Iterator<Item = &Output> {
        self.outputs.iter()
    }

    pub fn active_workspace(&self) -> &Workspace {
        &self.workspaces.get(0).unwrap()
    }

    pub fn active_workspace_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[0]
    }

    pub fn space_for_window_mut(&mut self, surface: &WlSurface) -> Option<&mut Workspace> {
        self.workspaces.iter_mut().find(|workspace| {
            workspace
                .space
                .window_for_surface(surface, WindowSurfaceType::ALL)
                .is_some()
        })
    }

    pub fn add_output(&mut self, output: &Output) {
        self.outputs.push(output.clone());
        remap_output(
            output,
            &mut self.workspaces,
            None,
            0,
            output.current_location(),
        );
    }

    pub fn remove_output(&mut self, output: &Output) {
        self.outputs.retain(|o| o != output);
        remap_output(output, &mut self.workspaces, None, None, None);
    }

    pub fn refresh_outputs(&mut self) {
        let workspace = &mut self.workspaces[0];
        for output in self.outputs.iter() {
            workspace
                .space
                .map_output(output, output.current_location());
        }
    }

    pub fn refresh(&mut self, dh: &DisplayHandle) {
        let workspace = &mut self.workspaces[0];
        workspace.refresh(dh);

        for output in &self.outputs {
            let mut map = layer_map_for_output(output);
            map.cleanup(dh);
        }
    }

    pub fn map_layer(&mut self, layer_surface: &LayerSurface, dh: &DisplayHandle) {
        let pos = self
            .pending_layers
            .iter()
            .position(|(l, _, _)| l == layer_surface)
            .unwrap();
        let (layer_surface, output, seat) = self.pending_layers.remove(pos);

        let surface = layer_surface.wl_surface();
        let wants_focus = {
            with_states(surface, |states| {
                let state = states.cached_state.current::<LayerSurfaceCachedState>();
                matches!(state.layer, Layer::Top | Layer::Overlay)
                    && state.keyboard_interactivity != KeyboardInteractivity::None
            })
        };

        let mut map = layer_map_for_output(&output);
        map.map_layer(dh, &layer_surface).unwrap();

        if wants_focus {
            self.set_focus(dh, Some(surface), &seat, None)
        }
    }

    pub fn map_window(&mut self, window: &Window, _output: &Output, _dh: &DisplayHandle) {
        let workspace = self.active_workspace_mut();

        workspace
            .space
            .map_window(window, Point::from((0, 0)), 0, false);
    }

    /// Deno Function
    pub fn move_request(
        &mut self,
        window: &Window,
        seat: &Seat<State>,
        serial: Serial,
        start_data: PointerGrabStartData,
    ) {
        if let Some(_pointer) = seat.get_pointer() {
            let workspace = self
                .space_for_window_mut(window.toplevel().wl_surface())
                .unwrap();
            if workspace.fullscreen.values().any(|w| w == window) {
                return;
            }

            workspace
                .runtime_sender
                .send(RuntimeMessage::MoveRequest {
                    window: window.clone(),
                    seat: seat.clone(),
                    serial,
                    start_data,
                })
                .unwrap();
        }
    }

    pub fn set_focus(
        &mut self,
        _dh: &DisplayHandle,
        surface: Option<&WlSurface>,
        active_seat: &Seat<State>,
        _serial: Option<Serial>,
    ) {
        // update FocusStack and notify layouts about new focus (if any window)
        if let Some(surface) = surface {
            if let Some(workspace) = self.space_for_window_mut(surface) {
                if let Some(window) = workspace
                    .space
                    .window_for_surface(surface, WindowSurfaceType::ALL)
                {
                    let mut focus_stack = workspace.focus_stack_mut(active_seat);
                    if Some(window) != focus_stack.last().as_ref() {
                        slog_scope::debug!("Focusing window: {:?}", window);
                        focus_stack.append(window);
                        // TODO popups
                    }
                }
            }
        }
    }

    pub fn update_active<'a>(&mut self, seats: impl Iterator<Item = &'a Seat<State>>) {
        let focused_windows = seats
            .flat_map(|seat| {
                self.outputs
                    .iter()
                    .flat_map(|_| self.active_workspace().focus_stack(seat).last().clone())
            })
            .collect::<Vec<_>>();

        for _ in self.outputs.iter() {
            let workspace = &mut self.workspaces[0];
            for focused in focused_windows.iter() {
                workspace.space.raise_window(focused, true);
            }
            for window in workspace.space.windows() {
                window.set_activated(focused_windows.contains(window));
                window.configure();
            }
        }
    }

    /// Deno Function
    pub fn unconstrain_popup(&self, _surface: &PopupSurface, _positioner: &PositionerState) {
        // TODO: Popups
    }
}

fn remap_output(
    output: &Output,
    spaces: &mut [Workspace],
    old: impl Into<Option<usize>>,
    new: impl Into<Option<usize>>,
    pos: impl Into<Option<Point<i32, Logical>>>,
) {
    if let Some(old) = old.into() {
        let old_space = &mut spaces[old].space;
        old_space.unmap_output(output);
    }
    if let Some(new) = new.into() {
        let new_space = &mut spaces[new].space;
        new_space.map_output(output, pos.into().expect("Position required"));
    }
}
