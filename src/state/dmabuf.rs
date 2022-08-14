// SPDX-License-Identifier: GPL-3.0-only

use smithay::{
    backend::{allocator::dmabuf::Dmabuf, renderer::ImportDma},
    delegate_dmabuf,
    reexports::wayland_server::DisplayHandle,
    wayland::dmabuf::{DmabufGlobal, DmabufHandler, ImportError},
};

use super::State;

impl DmabufHandler for State {
    fn dmabuf_state(&mut self) -> &mut smithay::wayland::dmabuf::DmabufState {
        &mut self.common.dmabuf_state
    }

    fn dmabuf_imported(
        &mut self,
        _dh: &DisplayHandle,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
    ) -> Result<(), ImportError> {
        match &mut self.backend {
            super::BackendData::Winit(ref mut state) => state
                .backend
                .renderer()
                .import_dmabuf(&dmabuf, None)
                .map(|_| ())
                .map_err(|_| ImportError::Failed),
            super::BackendData::Unset => unreachable!("Tried to import dmabuf without a backend"),
        }
    }
}

delegate_dmabuf!(State);
