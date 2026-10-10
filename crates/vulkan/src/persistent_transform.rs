//! Mirror OpenGl_Structure::applyPersistence at OCCT 3d097a0, including local anchors.
use crate::{Graphic3dDrawGroup, RenderError, error::rejected};
use rustycad_tkernel::{
    ncollection::{Mat4, Vec4},
    standard::Handle,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;

pub(crate) fn group_clip_transform(
    group: &Graphic3dDrawGroup,
    camera: &Graphic3dCamera,
    extent: Option<[u32; 2]>,
    structure: &Graphic3dCStructureFields,
) -> Result<([f32; 16], bool), RenderError> {
    let parent = &structure.trsf_pers;
    let local = group.transform_persistence();
    let model = model_transform(group, structure);
    if parent.is_null() && local.is_null() {
        return Ok((ordinary_clip(camera, model), false));
    }
    let transform = persistent_view(camera, model, parent, &local, extent)?;
    let clamp = !parent.is_null() && parent.mode() == Graphic3dTransModeFlags::CAMERA_PERS;
    Ok((transform.clip_matrix(), clamp))
}
fn persistent_view(
    camera: &Graphic3dCamera,
    model: Mat4<f64>,
    parent: &Handle<dyn Graphic3dTransformPersSource>,
    local: &Handle<dyn Graphic3dTransformPersSource>,
    extent: Option<[u32; 2]>,
) -> Result<PersistentView, RenderError> {
    let mut transform = PersistentView::new(camera, model);
    let extent = viewport_size(extent)?;
    transform.apply(parent, extent, None)?;
    let anchor = local_anchor(local, &mut transform.model)?;
    transform.apply(local, extent, anchor.as_ref())?;
    Ok(transform)
}
pub(crate) fn text_matrices(
    group: &Graphic3dDrawGroup,
    camera: &Graphic3dCamera,
    extent: [u32; 2],
    structure: &Graphic3dCStructureFields,
) -> Result<(Mat4<f64>, Mat4<f64>), RenderError> {
    let model = model_transform(group, structure);
    let parent = &structure.trsf_pers;
    let local = group.transform_persistence();
    if parent.is_null() && local.is_null() {
        return Ok((
            camera.projection_matrix(),
            camera.orientation_matrix() * model,
        ));
    }
    let transform = persistent_view(camera, model, parent, &local, Some(extent))?;
    Ok((
        Mat4::from_other(&transform.projection),
        Mat4::from_other(&(transform.world_view * transform.model)),
    ))
}
struct PersistentView {
    camera: Handle<Graphic3dCamera>,
    model: Mat4<f32>,
    projection: Mat4<f32>,
    world_view: Mat4<f32>,
}
impl PersistentView {
    fn new(camera: &Graphic3dCamera, model: Mat4<f64>) -> Self {
        Self {
            camera: Handle::new(camera.copy_members()),
            model: Mat4::from_other(&model),
            projection: Mat4::from_other(&camera.projection_matrix()),
            world_view: Mat4::from_other(&camera.orientation_matrix()),
        }
    }
    fn apply(
        &mut self,
        persistence: &Handle<dyn Graphic3dTransformPersSource>,
        extent: [i32; 2],
        anchor: Option<&Point3>,
    ) -> Result<(), RenderError> {
        if persistence.is_null() {
            return Ok(());
        }
        persistence
            .apply_world_view(
                &self.camera,
                &self.projection,
                &mut self.world_view,
                extent,
                anchor,
                true,
            )
            .map_err(|error| rejected(error, "a valid persistent world-view transform"))
    }
    fn clip_matrix(&self) -> [f32; 16] {
        (self.projection * self.world_view * self.model).data()
    }
}

fn model_transform(group: &Graphic3dDrawGroup, structure: &Graphic3dCStructureFields) -> Mat4<f64> {
    let mut model = Mat4::<f64>::identity();
    let mut local = Mat4::<f64>::identity();
    if let Some(datum) = structure.trsf.get() {
        datum.transformation().get_mat4(&mut model);
    }
    group.transformation().get_mat4(&mut local);
    model * local
}
fn viewport_size(extent: Option<[u32; 2]>) -> Result<[i32; 2], RenderError> {
    let Some([width, height]) = extent else {
        return Err(rejected(
            extent,
            "an actual viewport for transform persistence",
        ));
    };
    if width == 0 || height == 0 || width > i32::MAX as u32 || height > i32::MAX as u32 {
        return Err(rejected(
            [width, height],
            "positive viewport dimensions fitting i32",
        ));
    }
    Ok([width as i32, height as i32])
}
fn local_anchor(
    persistence: &Handle<dyn Graphic3dTransformPersSource>,
    model: &mut Mat4<f32>,
) -> Result<Option<Point3>, RenderError> {
    if persistence.is_null() || !persistence.is_zoom_or_rotate() {
        return Ok(None);
    }
    let point = persistence
        .anchor_point()
        .map_err(|e| rejected(e, "a local persistence anchor"))?;
    let transformed = *model * Vec4::new(point.x() as f32, point.y() as f32, point.z() as f32, 1.);
    // OpenGL moves the local anchor through the full model, then clears translation exactly once.
    model.set_column(3, Vec4::new(0., 0., 0., 1.));
    Ok(Some(Point3::new(
        transformed.x() as f64,
        transformed.y() as f64,
        transformed.z() as f64,
    )))
}

fn ordinary_clip(camera: &Graphic3dCamera, model: Mat4<f64>) -> [f32; 16] {
    (camera.projection_matrix() * camera.orientation_matrix() * model)
        .data()
        .map(|value| value as f32)
}
