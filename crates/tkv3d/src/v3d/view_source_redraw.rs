//! Complete inherited V3d_View redraw bodies remain callable from genuine subclass overrides.
use super::V3dView;
use rustycad_tkernel::standard::raise_native_failure;
use rustycad_tkservice::graphic3d::{Graphic3dCViewMembers as _, Graphic3dStructureManager};

/// Execute native base Redraw, including lost-device retry; `v3d_view_redraw(view.view_base())`.
pub fn v3d_view_redraw(view: &V3dView) {
    if !view.view.is_defined() || !view.view.is_active() {
        return;
    }
    view.state.write().is_invalidated_immediate = false;
    // Native MyViewer is a raw pointer; a storage lease must not fabricate an owning reference.
    let viewer = view.state.read().viewer.observe();
    let Some(viewer) = viewer.get() else { return };
    let manager = viewer.structure_manager();
    let Some(manager) = manager.get() else { return };
    for _ in 0..2 {
        if inherited_redraw_attempt(view, manager) {
            return;
        }
    }
}
fn inherited_redraw_attempt(view: &V3dView, manager: &Graphic3dStructureManager) -> bool {
    if manager.is_device_lost() {
        manager.recompute_structures();
    }
    if let Err(failure) = view.auto_z_fit() {
        raise_native_failure::<()>(Err(failure));
    }
    view.view.redraw();
    !manager.is_device_lost()
}
/// Execute native base RedrawImmediate; `v3d_view_redraw_immediate(view.view_base())`.
pub fn v3d_view_redraw_immediate(view: &V3dView) {
    if !view.view.is_defined() || !view.view.is_active() {
        return;
    }
    view.state.write().is_invalidated_immediate = false;
    view.view.redraw_immediate();
}
