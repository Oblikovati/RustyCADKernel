//! Viewer activation rereads the caller's genuine view slot after native virtual callbacks.
use super::{V3dViewSource, V3dViewer};
use rustycad_tkernel::standard::{Failure, Handle, HandleReference, catch_native_failure};
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
        // Method-call receivers evaluate first: a native node alias must be read before this write guard.
        let activated = view.clone_handle();
        self.state.write().active_views.append(activated);
        activate_driver_view_properties(self, view, &implementation)
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
    fn contains_active_view(&self, identity: *const ()) -> bool {
        self.state
            .read()
            .active_views
            .iter()
            .any(|stored| stored.object_address() == identity)
    }
}
fn activate_driver_view_properties(
    viewer: &V3dViewer,
    view: &dyn HandleReference<dyn V3dViewSource>,
    implementation: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dCView>,
) -> Result<(), Failure> {
    catch_native_failure(|| {
        implementation.activate();
        apply_active_view_properties(viewer, view)
    })
}
fn apply_active_view_properties(
    viewer: &V3dViewer,
    view: &dyn HandleReference<dyn V3dViewSource>,
) -> Result<(), Failure> {
    let mut cursor = viewer.active_light_iterator();
    while cursor.more() {
        view.observe().set_light_on_one(&cursor.value_ref())?;
        cursor.next();
    }
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
