//! Camera matrices carried through native subset order and persistence transforms.
use super::*;
use rustycad_tkernel::{
    ncollection::Mat4,
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{GeneralTransform3, Xyz};
use rustycad_tkservice::graphic3d::*;
/// View arguments shared by object traversal; matrix values follow native subset order.
/// Example: `SelectMgrTraversalView::new(camera, projection, world_view, [800,600])`.
#[derive(Clone)]
pub struct SelectMgrTraversalView {
    camera: Handle<Graphic3dCamera>,
    camera_source: Handle<dyn SelectMgrBaseIntersector>,
    /// Projection for the current BVH subset.
    pub projection: Mat4<f64>,
    /// World-view matrix for the current BVH subset.
    pub world_view: Mat4<f64>,
    /// Pixel viewport size.
    pub size: [i32; 2],
}
impl SelectMgrTraversalView {
    /// Use a caller-owned camera value, as in a direct native traverseObject call.
    /// Example: `SelectMgrTraversalView::new(camera, projection, world_view, [800,600])`.
    pub fn new(
        camera: Handle<Graphic3dCamera>,
        projection: Mat4<f64>,
        world_view: Mat4<f64>,
        size: [i32; 2],
    ) -> Self {
        Self {
            camera,
            camera_source: Handle::default(),
            projection,
            world_view,
            size,
        }
    }
    /// Read the retained original camera slot without holding a guard over callbacks.
    /// Example: `view.camera().is_null()`.
    pub fn camera(&self) -> Handle<Graphic3dCamera> {
        self.camera_source
            .get()
            .map_or_else(|| self.camera.clone(), |source| source.camera())
    }
    pub(super) fn from_volume(volume: &SelectMgrSelectingVolumeManager) -> Self {
        let camera = volume.camera();
        let (width, height) = volume.window_size().unwrap_or((0, 0));
        let projection = camera
            .get()
            .map_or_else(Mat4::identity, Graphic3dCamera::projection_matrix);
        let world_view = camera
            .get()
            .map_or_else(Mat4::identity, Graphic3dCamera::orientation_matrix);
        if let Some(camera) = camera.get() {
            let _ = camera.world_view_proj_state();
        }
        let mut view = Self::new(Handle::default(), projection, world_view, [width, height]);
        // OCCT 3d097a0 TraverseSensitives binds Camera() by reference to the original
        // intersector slot: callbacks can replace its handle during later traversal.
        view.camera_source = volume.active_volume().clone();
        view
    }
    pub(super) fn subset_volume(
        &mut self,
        subset: SelectMgrBvhSubset,
        global: &SelectMgrSelectingVolumeManager,
    ) -> Result<SelectMgrSelectingVolumeManager, Failure> {
        if subset.is_two_dimensional() {
            return self.screen_subset_volume(subset, global);
        }
        if subset != SelectMgrBvhSubset::OrthoThreeDPersistent {
            return Ok(global.clone());
        }
        let camera = Handle::new(self.camera().copy_members());
        camera.set_projection_type(Graphic3dCameraProjection::Orthographic);
        self.world_view = camera.orientation_matrix();
        self.projection = camera.projection_matrix();
        global.copy_with_builder(&self.subset_builder(&camera))
    }
    fn screen_subset_volume(
        &mut self,
        subset: SelectMgrBvhSubset,
        global: &SelectMgrSelectingVolumeManager,
    ) -> Result<SelectMgrSelectingVolumeManager, Failure> {
        let inverse = screen_selection_transform(&self.world_view)?;
        let camera = Handle::new(Graphic3dCamera::default());
        camera.copy_mapping_data(&self.camera());
        camera.set_identity_orientation()?;
        if subset == SelectMgrBvhSubset::OrthoTwoDPersistent {
            camera.set_projection_type(Graphic3dCameraProjection::Orthographic);
        }
        self.world_view = camera.orientation_matrix();
        self.projection = camera.projection_matrix();
        global.scale_and_transform(1, &inverse, &self.subset_builder(&camera))
    }
    fn subset_builder(&self, camera: &Handle<Graphic3dCamera>) -> Handle<SelectMgrFrustumBuilder> {
        let builder = Handle::new(SelectMgrFrustumBuilder::new());
        builder.set_camera(camera);
        builder.set_window_size(self.size[0], self.size[1]);
        builder
    }
}
fn screen_selection_transform(world: &Mat4<f64>) -> Result<GeneralTransform3, Failure> {
    let mut transform = GeneralTransform3::default();
    for row in 0..3 {
        for col in 0..3 {
            transform.set_value(
                row + 1,
                col + 1,
                world.get_value(row as usize, col as usize),
            )?;
        }
    }
    transform.set_translation_part(Xyz::new(
        world.get_value(0, 3),
        world.get_value(1, 3),
        world.get_value(2, 3),
    ));
    Ok(transform)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkmath::gp::{Axis1, Point2};

    #[test]
    fn direct_object_frame_keeps_its_camera_value_and_matrices() {
        let camera = Handle::new(Graphic3dCamera::default());
        let projection = camera.projection_matrix();
        let world = camera.orientation_matrix();
        let view = SelectMgrTraversalView::new(camera.clone(), projection, world, [80, 60]);
        assert_eq!(view.camera(), camera);
        assert_eq!(view.projection, projection);
        assert_eq!(view.world_view, world);
        assert_eq!(view.size, [80, 60]);
    }

    #[test]
    fn root_frame_does_not_keep_a_replaced_camera_alive() {
        let mut manager = SelectMgrSelectingVolumeManager::new();
        manager.init_point_selecting_volume(Point2::default());
        let camera = Handle::new(Graphic3dCamera::default());
        let weak = camera.downgrade();
        manager.set_camera(&camera).unwrap();
        drop(camera);
        let view = SelectMgrTraversalView::from_volume(&manager);
        manager
            .set_camera(&Handle::new(Graphic3dCamera::default()))
            .unwrap();
        assert!(weak.upgrade().is_null());
        assert_eq!(view.camera(), manager.camera());
    }

    #[test]
    fn root_frame_observes_original_camera_slot_after_manager_replacement() {
        let mut manager = SelectMgrSelectingVolumeManager::new();
        manager.init_point_selecting_volume(Point2::default());
        let original = manager.active_volume().clone();
        let view = SelectMgrTraversalView::from_volume(&manager);
        manager.init_axis_selecting_volume(Axis1::default());
        let camera = Handle::new(Graphic3dCamera::default());
        original.set_camera(&camera);
        assert_eq!(view.camera(), camera);
        assert!(manager.camera().is_null());
        original.set_camera(&Handle::default());
        assert!(view.camera().is_null());
    }
}
