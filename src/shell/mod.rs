//! This Implementation, ideally, will purely be callbacks to an FFI.
//! There should be minimal amounts of code here, any code here is either for debug purposes or in development

use smithay::{
    desktop::{layer_map_for_output, LayerSurface, PopupManager, Window, WindowSurfaceType},
    reexports::wayland_server::{protocol::wl_surface::WlSurface, DisplayHandle},
    utils::{Logical, Point},
    wayland::{
        output::Output,
        seat::{PointerGrabStartData, Seat},
        shell::{
            wlr_layer::WlrLayerShellState,
            xdg::{PopupSurface, PositionerState, XdgShellState},
        },
        Serial,
    },
};

pub mod workspace;

use crate::state::State;

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
    pub fn new(dh: &DisplayHandle) -> Self {
        // TODO Create Workspaces

        Self {
            workspaces: vec![Workspace::new(0)],
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

    pub fn active_workspace(&self) -> &Workspace {
        &self.workspaces.get(0).unwrap()
    }

    pub fn active_workspace_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[0]
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

    pub fn space_for_window_mut(&mut self, surface: &WlSurface) -> Option<&mut Workspace> {
        self.workspaces.iter_mut().find(|workspace| {
            workspace
                .space
                .window_for_surface(surface, WindowSurfaceType::ALL)
                .is_some()
        })
    }

    pub fn map_layer(&mut self, _layer_surface: &LayerSurface, _dh: &DisplayHandle) {
        // DENO FUNCTION
    }

    pub fn map_window(&mut self, _window: &Window, _output: &Output, _dh: &DisplayHandle) {
        // DENO FUNCTION
    }

    pub fn move_request(
        &mut self,
        _window: &Window,
        _seat: &Seat<State>,
        _serial: Serial,
        _start_data: PointerGrabStartData,
    ) {
        // DENO FUNCTION
    }

    pub fn set_focus(
        &mut self,
        _dh: &DisplayHandle,
        _surface: Option<&WlSurface>,
        _active_seat: &Seat<State>,
        _serial: Option<Serial>,
    ) {
        // DENO FUNCTION
    }

    pub fn update_active<'a>(&mut self, _seats: impl Iterator<Item = &'a Seat<State>>) {
        // DENO FUNCTION
    }

    pub fn unconstrain_popup(&self, _surface: &PopupSurface, _positioner: &PositionerState) {
        // DENO FUNCTION
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
    }
}
