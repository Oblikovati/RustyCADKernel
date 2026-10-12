//! Viewer activation rereads the caller's genuine view slot after native virtual callbacks.
use super::{V3dViewSource, V3dViewer};
use rustycad_tkernel::standard::{Failure, HandleReference};
use rustycad_tkservice::aspect::AspectGridMembers as _;

impl V3dViewer {
    /// Activate the currently referenced view without pinning stale targets; `viewer.set_view_on_one(&slot)?`.
    pub fn set_view_on_one(
        &self,
        view: &dyn HandleReference<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let current = view.observe();
        let Some(source) = current.get() else {
            return Ok(());
        };
        let implementation = source.view();
        drop(current);
        if !implementation.is_defined() || self.contains_active_view(view.object_address()) {
            return Ok(());
        }
        self.append_active_view_reference(view);
        implementation.activate();
        apply_active_view_properties(self, view)
    }
    /// Deactivate the referenced native node with the original driver-view copy; `viewer.set_view_off_one(&slot)`.
    pub fn set_view_off_one(&self, view: &dyn HandleReference<dyn V3dViewSource>) {
        let current = view.observe();
        let Some(source) = current.get() else { return };
        let implementation = source.view();
        drop(current);
        if !implementation.is_defined() || !self.contains_active_view(view.object_address()) {
            return;
        }
        super::viewer::remove_view_from(
            &mut self.state.write().active_views,
            view.object_address(),
        );
        implementation.deactivate();
    }
    fn append_active_view_reference(&self, view: &dyn HandleReference<dyn V3dViewSource>) {
        // Method receivers evaluate first; read a same-viewer node alias before locking for append.
        let activated = view.clone_handle();
        self.state.write().active_views.append(activated);
    }
    fn contains_active_view(&self, identity: *const ()) -> bool {
        self.state
            .read()
            .active_views
            .iter()
            .any(|stored| stored.object_address() == identity)
    }
}
fn apply_active_view_properties(
    viewer: &V3dViewer,
    view: &dyn HandleReference<dyn V3dViewSource>,
) -> Result<(), Failure> {
    let lights = viewer.state.read().active_lights.clone();
    for light in &lights {
        view.observe().set_light_on_one(light)?;
    }
    // Native light iteration ends before Redraw can inspect light intrusive counts.
    drop(lights);
    apply_active_view_grid(viewer, view)?;
    if view.observe().set_immediate_update(false) {
        view.observe().redraw();
        view.observe().set_immediate_update(true);
    }
    Ok(())
}
fn apply_active_view_grid(
    viewer: &V3dViewer,
    view: &dyn HandleReference<dyn V3dViewSource>,
) -> Result<(), Failure> {
    let grid = viewer.grid(false)?;
    if let Some(grid_ref) = grid.get() {
        view.observe().set_grid(&viewer.privileged_plane(), &grid);
        view.observe().set_grid_activity(grid_ref.is_active());
    }
    Ok(())
}
