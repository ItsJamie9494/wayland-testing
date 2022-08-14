// SPDX-License-Identifier: GPL-3.0-only

use smithay::{delegate_primary_selection, wayland::primary_selection::PrimarySelectionHandler};

use super::State;

impl PrimarySelectionHandler for State {
    fn primary_selection_state(
        &self,
    ) -> &smithay::wayland::primary_selection::PrimarySelectionState {
        &self.common.primary_selection_state
    }
}

delegate_primary_selection!(State);
