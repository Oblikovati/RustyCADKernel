//! Per-object context state, retaining native ordered selection modes and style identity.
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::{
    ncollection::List,
    standard::{Handle, SharedState, StandardTransient},
};
use std::sync::RwLockReadGuard;
#[derive(Default)]
struct GlobalDisplayState {
    mode: i32,
    highlighted: bool,
    sub_intensity: bool,
}
/// Context status for one interactive object; `let status = AisGlobalStatus::new()`.
pub struct AisGlobalStatus {
    transient: StandardTransient,
    display: SharedState<GlobalDisplayState>,
    style: SharedState<Handle<Prs3dDrawer>>,
    modes: SharedState<List<i32>>,
}
rustycad_tkernel::impl_transient!(
    AisGlobalStatus,
    "AIS_GlobalStatus",
    StandardTransient,
    transient
);
impl AisGlobalStatus {
    /// Start with mode zero and no highlighting or selection modes; `AisGlobalStatus::new()`.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            display: SharedState::default(),
            style: SharedState::default(),
            modes: SharedState::default(),
        }
    }
    /// Read the retained display mode; `status.display_mode()`.
    pub fn display_mode(&self) -> i32 {
        self.display.read().mode
    }
    /// Store any native display-mode integer; `status.set_display_mode(-3)`.
    pub fn set_display_mode(&self, mode: i32) {
        self.display.write().mode = mode;
    }
    /// Read the highlighted flag; `status.is_hilighted()`.
    pub fn is_hilighted(&self) -> bool {
        self.display.read().highlighted
    }
    /// Store highlighting independently of the style; `status.set_hilight_status(true)`.
    pub fn set_hilight_status(&self, highlighted: bool) {
        self.display.write().highlighted = highlighted;
    }
    /// Retain the supplied style handle, including null; `status.set_hilight_style(&style)`.
    pub fn set_hilight_style(&self, style: &Handle<Prs3dDrawer>) {
        *self.style.write() = style.clone();
    }
    /// Borrow the actual retained style handle; release before mutation; `status.hilight_style().is_null()`.
    pub fn hilight_style(&self) -> RwLockReadGuard<'_, Handle<Prs3dDrawer>> {
        self.style.read()
    }
    /// Borrow the native ordered list; release before mutation; `status.selection_modes().length()`.
    pub fn selection_modes(&self) -> RwLockReadGuard<'_, List<i32>> {
        self.modes.read()
    }
    /// Test membership without reordering; `status.is_s_mode_in(2)`.
    pub fn is_s_mode_in(&self, mode: i32) -> bool {
        self.modes.read().contains(&mode)
    }
    /// Append a missing mode, preserving insertion order; `status.add_selection_mode(2)`.
    pub fn add_selection_mode(&self, mode: i32) -> bool {
        let mut modes = self.modes.write();
        if modes.contains(&mode) {
            return false;
        }
        modes.append(mode);
        true
    }
    /// Remove the mode if present; `status.remove_selection_mode(2)`.
    pub fn remove_selection_mode(&self, mode: i32) -> bool {
        self.modes.write().remove_value(&mode)
    }
    /// Forget all modes without changing display/highlight state; `status.clear_selection_modes()`.
    pub fn clear_selection_modes(&self) {
        self.modes.write().clear();
    }
    /// Read the sub-intensity flag; `status.is_sub_intensity_on()`.
    pub fn is_sub_intensity_on(&self) -> bool {
        self.display.read().sub_intensity
    }
    /// Store sub-intensity independently of highlight state; `status.set_sub_intensity(true)`.
    pub fn set_sub_intensity(&self, enabled: bool) {
        self.display.write().sub_intensity = enabled;
    }
}
impl Default for AisGlobalStatus {
    fn default() -> Self {
        Self::new()
    }
}
