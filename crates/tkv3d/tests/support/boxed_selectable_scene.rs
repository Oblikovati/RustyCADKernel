//! Shared scene fixture for selection-set group persistence and flipping.
use super::boxed_selectable::BoxedSelectable;
use rustycad_tkernel::{
    ncollection::Mat4,
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3, Transform3, Vector3};
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use rustycad_tkv3d::prs_mgr::*;

/// Create a headless presentation manager; `boxed_scene_manager()`.
pub fn boxed_scene_manager() -> Handle<PrsMgrPresentationManager> {
    let structures =
        Handle::new(Graphic3dStructureManager::new(&NullGraphicDriver::new_handle()).unwrap());
    PrsMgrPresentationManager::new(&structures)
}
/// Select ordinary or injected failing persistence; `boxed_persistence(1)?`.
pub fn boxed_persistence(code: i32) -> Result<Handle<dyn Graphic3dTransformPersSource>, Failure> {
    if code == 10 {
        return Ok(Handle::new(FailingBoxPersistence {
            transient: StandardTransient::default(),
            base: Graphic3dTransformPersBase::new(Graphic3dTransModeFlags::ZOOM_PERS)?,
        })
        .cast());
    }
    let modes = [0, 2, 64, 258, 320, 8, 10, 32, 128, 514];
    let mode = modes[code as usize % modes.len()];
    if mode == 0 {
        return Ok(Handle::default());
    }
    Ok(Handle::new(Graphic3dTransformPers::new(Graphic3dTransModeFlags(mode))?).cast())
}
/// Add valid, ignored and void group bounds; `boxed_scene_groups(&manager, &object, 3)?`.
pub fn boxed_scene_groups(
    manager: &Handle<PrsMgrPresentationManager>,
    object: &Handle<BoxedSelectable>,
    flags: i32,
) -> Result<(), Failure> {
    let prs = manager.presentation(&object.clone().cast(), 0, true, &Handle::default())?;
    let group = prs.new_group()?;
    group.set_min_max_values([-2., -3., -4., 5., 6., 7.]);
    boxed_group_flipping(&*group, flags & 1 != 0);
    group.set_transform_persistence(&boxed_persistence(if flags & 2 != 0 { 1 } else { 0 })?);
    let ignored = prs.new_group()?;
    ignored.set_min_max_values([-100., -100., -100., 100., 100., 100.]);
    let void = prs.new_group()?;
    void.set_transform_persistence(&boxed_persistence(1)?);
    // The invalid persistent group must never contribute its default extreme corners.
    Ok(())
}

struct FailingBoxPersistence {
    transient: StandardTransient,
    base: Graphic3dTransformPersBase,
}
rustycad_tkernel::impl_transient!(FailingBoxPersistence,"FailingBoxPersistence",dyn Graphic3dTransformPersSource,transient,
    interfaces:[dyn Graphic3dTransformPersSource]);
impl Graphic3dTransformPersSource for FailingBoxPersistence {
    fn transform_pers_base(&self) -> &Graphic3dTransformPersBase {
        &self.base
    }
    fn compute_apply(
        &self,
        _: &mut Handle<Graphic3dCamera>,
        _: i32,
        _: i32,
        _: Option<&Point3>,
    ) -> Result<Mat4<f64>, Failure> {
        Err(Failure::with_message(
            "Standard_ProgramError",
            "injected persistence failure; expected successful transform computation",
        ))
    }
}

/// Rotate and translate one object; `boxed_object_transform(4)?`.
pub fn boxed_object_transform(value: i32) -> Result<Transform3, Failure> {
    let mut transform = Transform3::default();
    transform.set_rotation(
        Axis1::new(Point3::default(), Direction3::Z),
        f64::from(value) / 8.,
    )?;
    transform.set_translation_part(Vector3::new(f64::from(value) * 0.5, 1., -2.));
    Ok(transform)
}
/// Change group-local transforms without changing their stored bounds; `boxed_group_transform(&object, 4)?`.
pub fn boxed_group_transform(object: &BoxedSelectable, value: i32) -> Result<(), Failure> {
    let transform = boxed_object_transform(value)?;
    for presentation in object.presentations() {
        for group in presentation.groups() {
            group.set_transformation(&transform);
        }
    }
    Ok(())
}

fn boxed_group_flipping(group: &dyn Graphic3dGroup, enabled: bool) {
    // The null driver omits render brackets; supply OpenGl_Group's selection metadata.
    if !enabled {
        return;
    }
    group.group_base().fields.write().flipper =
        Handle::new(Graphic3dFlipper::new(Axis2::default()));
    group.structure().cstructure().set_group_flipping(true);
}
