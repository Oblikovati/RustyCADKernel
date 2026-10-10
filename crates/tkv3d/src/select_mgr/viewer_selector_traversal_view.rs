//! Camera matrices carried through native subset order and persistence transforms.
use super::*;
use rustycad_tkernel::{
    ncollection::Mat4,
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{GeneralTransform3, Xyz};
use rustycad_tkservice::graphic3d::*;
/// View arguments shared by object traversal; matrix values follow native subset order.
/// Example: `SelectMgrTraversalView { camera, projection, world_view, size: [800,600] }`.
#[derive(Clone)]
pub struct SelectMgrTraversalView {
    /// The original viewer camera.
    pub camera: Handle<Graphic3dCamera>,
    /// Projection for the current BVH subset.
    pub projection: Mat4<f64>,
    /// World-view matrix for the current BVH subset.
    pub world_view: Mat4<f64>,
    /// Pixel viewport size.
    pub size: [i32; 2],
}
impl SelectMgrTraversalView {
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
        Self {
            camera,
            projection,
            world_view,
            size: [width, height],
        }
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
        let camera = Handle::new(self.camera.copy_members());
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
        camera.copy_mapping_data(&self.camera);
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
