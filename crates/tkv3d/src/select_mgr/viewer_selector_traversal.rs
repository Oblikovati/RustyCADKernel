//! Native first-level traversal across the five object BVH subsets.
use super::viewer_selector_walk::*;
use super::*;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::standard::Failure;
use rustycad_tkservice::graphic3d::Graphic3dCamera;
/// Traverse actual object/entity BVHs and dispatch virtual final sorting.
/// Example: `selector.traverse_sensitives(-1)?`.
pub trait SelectMgrViewerSelectorTraversalMembers: SelectMgrViewerSelectorSource {
    /// Clear first, preserve completed matches on failure and sort only on success.
    /// Example: `selector.traverse_sensitives(view_id)?`.
    fn traverse_sensitives(&self, view_id: i32) -> Result<(), Failure> {
        let base = self.viewer_selector_base();
        let pool = base.fields.read().bvh_thread_pool.clone();
        let _sentry = SelectMgrBvhThreadSentry::new(pool);
        {
            let mut fields = base.fields.write();
            fields.results.stored.clear(true);
            fields.results.is_sorted = false;
        }
        let global = base.selecting_volume.read().clone();
        let mut view = SelectMgrTraversalView::from_volume(&global);
        if let Some(camera) = view.camera().get() {
            set_selector_camera_frame(base, camera);
        }
        base.selectable_objects
            .update_bvh(&view.camera(), view.size)?;
        for subset in SelectMgrBvhSubset::ALL {
            if base.selectable_objects.is_subset_empty(subset)
                || (view.camera().is_null() && subset != SelectMgrBvhSubset::ThreeD)
            {
                continue;
            }
            let subset_global = base.selecting_volume.read().clone();
            let volume = view.subset_volume(subset, &subset_global)?;
            set_selector_camera_scale(base, &volume, view.size);
            traverse_object_subset(base, subset, &volume, &view, view_id)?;
        }
        self.sort_result()
    }
}
impl<T: SelectMgrViewerSelectorSource + ?Sized> SelectMgrViewerSelectorTraversalMembers for T {}
fn set_selector_camera_frame(base: &SelectMgrViewerSelectorBase, camera: &Graphic3dCamera) {
    let mut fields = base.fields.write();
    fields.settings.camera_eye = camera.eye();
    fields.settings.camera_direction = camera.direction();
}
fn set_selector_camera_scale(
    base: &SelectMgrViewerSelectorBase,
    volume: &SelectMgrSelectingVolumeManager,
    size: [i32; 2],
) {
    let camera = volume.camera();
    let Some(camera) = camera.get() else {
        return;
    };
    let pixels = if size[0] > 0 && size[1] > 0 {
        (1. / f64::from(size[0])).max(1. / f64::from(size[1]))
    } else {
        1.
    };
    let scale = if camera.is_orthographic() {
        camera.scale()
    } else {
        2. * (camera.fov_y() * std::f64::consts::PI / 360.).tan()
    };
    base.fields.write().settings.camera_scale = scale * pixels;
}
fn traverse_object_subset(
    base: &SelectMgrViewerSelectorBase,
    subset: SelectMgrBvhSubset,
    volume: &SelectMgrSelectingVolumeManager,
    view: &SelectMgrTraversalView,
    view_id: i32,
) -> Result<(), Failure> {
    let tree = base.selectable_objects.bvh(subset);
    if !selector_node_overlaps(&tree, 0, volume)? {
        return Ok(());
    }
    let mut stack = vec![0];
    while let Some(node) = next_selector_leaf(&tree, &mut stack, |node| {
        selector_node_overlaps(&tree, node, volume)
    })? {
        let current = tree.read();
        let range = current.beg_primitive(node)?..=current.end_primitive(node)?;
        drop(current);
        for index in range {
            let object = base.selectable_objects.get_object_by_id(subset, index)?;
            if view_id == -1 || object.view_affinity().is_visible(view_id) {
                select_mgr_traverse_object(base, &object, volume, view)?;
            }
        }
    }
    Ok(())
}
