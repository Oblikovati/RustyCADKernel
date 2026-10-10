//! Native context location order, parent placement, selection and dynamic highlight state.
#[path = "../../tests/support/location_owner.rs"]
mod location_owner;
use super::{
    ais_object::context_object::ContextObject,
    common::{Reader, end, flag, int, xyz},
};
use location_owner::LocationOwner;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    gp::{Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, v3d::*};
/// Compare one independent native placement transition; `location_case(&[9.,2.,0.,4.])`.
pub fn location_case(args: &[f64]) {
    let mut reader = Reader::new(args);
    let flags = reader.integer();
    let operation = reader.integer();
    let (context, object, _parent, owner) = location_scene(flags, reader.real());
    let final_location = probe_translation(reader.real());
    object.events.write().clear();
    match operation {
        0 => context.set_location(&object.cast(), &Location::default()),
        1 => context.set_location(&object.cast(), &Location::new(Transform3::default())),
        2 => context.set_location(&object.cast(), &final_location),
        _ => context.reset_location(&object.cast()),
    }
    .unwrap();
    location_state(&context, &object, &owner);
    end();
}
fn probe_translation(x: f64) -> Location {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(x, 0., 0.));
    Location::new(transform)
}
type LocationScene = (
    Handle<AisInteractiveContext>,
    Handle<ContextObject>,
    Handle<ContextObject>,
    Handle<LocationOwner>,
);
fn location_scene(flags: i32, initial: f64) -> LocationScene {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    let object = ContextObject::new();
    let parent = ContextObject::new();
    seed_location_object(&context, &object, &parent, flags, initial);
    let owner = LocationOwner::new(&object.cast(), &context);
    if flags & 8 != 0 {
        object.set_hilight_mode(7);
        context.fields.write().last_picked = owner.cast();
    }
    (context, object, parent, owner)
}
fn seed_location_object(
    context: &AisInteractiveContext,
    object: &Handle<ContextObject>,
    parent: &Handle<ContextObject>,
    flags: i32,
    initial: f64,
) {
    if initial != 0. || flags & 4 != 0 {
        object
            .set_local_transformation(&probe_translation(initial).transformation())
            .unwrap();
    }
    if flags & 2 != 0 {
        parent
            .set_local_transformation(&probe_translation(10.).transformation())
            .unwrap();
        parent.add_child(&object.cast()).unwrap();
    }
    if flags & 1 != 0 {
        context.display(&object.cast(), false).unwrap();
    }
    if flags & 16 != 0 {
        context.erase(&object.cast(), false).unwrap();
    }
}
fn location_state(
    context: &AisInteractiveContext,
    object: &Handle<ContextObject>,
    owner: &LocationOwner,
) {
    flag(context.has_location(&object.cast()));
    flag(context.location(&object.cast()).unwrap().is_identity());
    flag(object.local_transformation_geom().is_null());
    xyz(object.local_transformation().translation_part());
    xyz(context
        .location(&object.cast())
        .unwrap()
        .transformation()
        .translation_part());
    int(owner.modes.read().len() as i32);
    for mode in owner.modes.read().iter() {
        int(*mode);
    }
    int(object.selections().len() as i32);
    for selection in object.selections() {
        int(selection.update_status() as i32);
        int(selection.bvh_update_status() as i32);
    }
    int(object.events.read().len() as i32);
    for event in object.events.read().iter() {
        int(*event);
    }
}
