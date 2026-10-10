//! Persistent subset bounds follow presentation-group and object transform order.
use super::SelectMgrSelectableObject;
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrPresentation};
use rustycad_tkernel::{
    ncollection::Mat4,
    standard::{Failure, Handle},
};
use rustycad_tkmath::{
    bnd::BoundingBox3,
    bvh::{BvhBox, BvhVec3d},
};
use rustycad_tkservice::graphic3d::{
    Graphic3dBndBox4f, Graphic3dCStructureMembers, Graphic3dCamera, Graphic3dGroup,
    Graphic3dGroupMembers, Graphic3dStructureMembers, Graphic3dTransformPersApply,
    Graphic3dTransformPersSource, transform_utils,
};

#[derive(Clone)]
pub(super) struct SelectableBoundsView {
    pub camera: Handle<Graphic3dCamera>,
    pub projection: Mat4<f64>,
    pub world_view: Mat4<f64>,
    pub size: [i32; 2],
}
pub(super) fn selectable_regular_bounds(
    object: &dyn SelectMgrSelectableObject,
) -> Result<BvhBox<BvhVec3d>, Failure> {
    let mut bounds = BoundingBox3::default();
    object.bounding_box(&mut bounds);
    selectable_bvh_bounds(&bounds)
}
fn selectable_bvh_bounds(bounds: &BoundingBox3) -> Result<BvhBox<BvhVec3d>, Failure> {
    if bounds.is_void() {
        return Ok(BvhBox::default());
    }
    let minimum = bounds.corner_min()?;
    let maximum = bounds.corner_max()?;
    Ok(BvhBox::from_corners(
        BvhVec3d::new(minimum.x(), minimum.y(), minimum.z()),
        BvhVec3d::new(maximum.x(), maximum.y(), maximum.z()),
    ))
}
pub(super) fn selectable_persistent_bounds(
    object: &dyn SelectMgrSelectableObject,
    view: &SelectableBoundsView,
) -> Result<BvhBox<BvhVec3d>, Failure> {
    let mut bounds = BoundingBox3::default();
    object.bounding_box(&mut bounds);
    let flip_view = selectable_flip_matrix(object, &view.world_view);
    for presentation in object.presentations() {
        add_selectable_presentation_bounds(&mut bounds, &presentation, view, &flip_view)?;
    }
    let persistence = object.transform_persistence();
    if !bounds.is_void() {
        apply_selectable_persistence(&mut bounds, &persistence, view)?;
    }
    selectable_bvh_bounds(&bounds)
}
fn selectable_flip_matrix(
    object: &dyn SelectMgrSelectableObject,
    world_view: &Mat4<f64>,
) -> Mat4<f64> {
    if !object.has_transformation() {
        return *world_view;
    }
    let mut model = Mat4::default();
    transform_utils::convert(&object.transformation(), &mut model);
    *world_view * model
}
fn add_selectable_group_bounds(
    bounds: &mut BoundingBox3,
    group: &dyn Graphic3dGroup,
    view: &SelectableBoundsView,
    flip_view: &Mat4<f64>,
) -> Result<(), Failure> {
    let raw = group.bounding_box();
    let (flipper, persistence) = (group.flipper(), group.transform_persistence());
    if !raw.is_valid() || (flipper.is_null() && persistence.is_null()) {
        return Ok(());
    }
    let mut group_box = selectable_float_box(&raw);
    if !flipper.is_null() {
        flipper.apply_bnd(flip_view, &mut group_box)?;
    }
    apply_selectable_persistence(&mut group_box, &persistence, view)?;
    bounds.add_box(&group_box);
    Ok(())
}

fn add_selectable_presentation_bounds(
    bounds: &mut BoundingBox3,
    presentation: &PrsMgrPresentation,
    view: &SelectableBoundsView,
    flip_view: &Mat4<f64>,
) -> Result<(), Failure> {
    let structure = presentation.cstructure();
    let Some(structure) = structure.get() else {
        return Ok(());
    };
    if !structure.has_group_transform_persistence() && !structure.has_group_flipping() {
        return Ok(());
    }
    for group in presentation.groups() {
        add_selectable_group_bounds(bounds, &*group, view, flip_view)?;
    }
    Ok(())
}
fn apply_selectable_persistence(
    bounds: &mut BoundingBox3,
    persistence: &Handle<dyn Graphic3dTransformPersSource>,
    view: &SelectableBoundsView,
) -> Result<(), Failure> {
    if persistence.is_null() {
        return Ok(());
    }
    persistence.apply_bnd(
        &view.camera,
        &view.projection,
        &view.world_view,
        view.size,
        bounds,
    )
}

fn selectable_float_box(raw: &Graphic3dBndBox4f) -> BoundingBox3 {
    let min = raw.corner_min();
    let max = raw.corner_max();
    let mut bounds = BoundingBox3::default();
    bounds.update_bounds(
        [min.x.into(), min.y.into(), min.z.into()],
        [max.x.into(), max.y.into(), max.z.into()],
    );
    bounds
}
impl SelectableBoundsView {
    pub(super) fn new(camera: &Handle<Graphic3dCamera>, size: [i32; 2]) -> Self {
        Self {
            camera: camera.clone(),
            projection: camera.projection_matrix(),
            world_view: camera.orientation_matrix(),
            size,
        }
    }
}
