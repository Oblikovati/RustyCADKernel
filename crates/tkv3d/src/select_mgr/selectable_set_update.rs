//! Camera-dependent invalidation preserves empty trees and native failure ordering.
use super::selectable_set_bounds::SelectableBoundsView;
use super::selectable_set_builder::build_selectable_subset;
use super::{SelectMgrBvhSubset, SelectMgrSelectableObjectSet};
use rustycad_tkernel::{
    ncollection::Mat4,
    standard::{Failure, Handle},
};
use rustycad_tkservice::graphic3d::{
    Graphic3dCamera, Graphic3dCameraProjection, Graphic3dWorldViewProjState,
};

impl SelectMgrSelectableObjectSet {
    /// Update affected trees using the camera and viewport; `set.update_bvh(&camera, [800,600])?`.
    pub fn update_bvh(
        &self,
        camera: &Handle<Graphic3dCamera>,
        size: [i32; 2],
    ) -> Result<(), Failure> {
        self.update_regular_subset()?;
        if !camera.is_null() {
            self.update_camera_subsets(camera, size)?;
        }
        self.view.write().size = size;
        Ok(())
    }
    fn update_regular_subset(&self) -> Result<(), Failure> {
        let mut state = self.subsets[0].write();
        if state.objects.is_empty() || !state.dirty {
            return Ok(());
        }
        build_selectable_subset(&mut state, None)?;
        state.dirty = false;
        Ok(())
    }
    fn update_camera_subsets(
        &self,
        camera: &Handle<Graphic3dCamera>,
        size: [i32; 2],
    ) -> Result<(), Failure> {
        // Read both matrices before the state, matching native lazy camera updates.
        let view = SelectableBoundsView::new(camera, size);
        let current = camera.world_view_proj_state();
        for subset in SelectMgrBvhSubset::ALL.into_iter().skip(1) {
            if self.needs_camera_subset(subset, current, size) {
                self.rebuild_camera_subset(subset, view.clone())?;
            }
        }
        for subset in self.subsets.iter().skip(1) {
            subset.write().dirty = false;
        }
        self.view.write().camera = current;
        Ok(())
    }
    fn needs_camera_subset(
        &self,
        subset: SelectMgrBvhSubset,
        current: Graphic3dWorldViewProjState,
        size: [i32; 2],
    ) -> bool {
        let state = self.subsets[subset as usize].read();
        if state.objects.is_empty() {
            return false;
        }
        let previous = self.view.read();
        let changed = if subset.is_two_dimensional() {
            previous.camera.is_projection_changed(&current)
        } else {
            previous.camera.is_changed(&current)
        };
        state.dirty || changed || previous.size != size
    }
    fn rebuild_camera_subset(
        &self,
        subset: SelectMgrBvhSubset,
        mut view: SelectableBoundsView,
    ) -> Result<(), Failure> {
        let orthographic;
        if subset.is_orthographic() {
            orthographic = Handle::new(view.camera.copy_members());
            orthographic.set_projection_type(Graphic3dCameraProjection::Orthographic);
            view.camera = orthographic.clone();
            view.projection = orthographic.projection_matrix();
            view.world_view = orthographic.orientation_matrix();
        }
        if subset.is_two_dimensional() {
            view.world_view = Mat4::default();
        }
        build_selectable_subset(&mut self.subsets[subset as usize].write(), Some(&view))
    }
}
