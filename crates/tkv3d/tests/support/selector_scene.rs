//! Shared named fixtures for selector ownership and native state traces.
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
use rustycad_tkv3d::{
    select_mgr::*,
    select3d::{Select3DSensitiveEntityMembers, Select3DSensitivePoint},
};

pub fn seeded_pick(
    selector: &impl SelectMgrViewerSelectorSource,
    object: &Handle<dyn SelectMgrSelectableObject>,
    depth: f64,
    priority: i32,
) -> Handle<dyn SelectMgrEntityOwnerSource> {
    let owner = SelectMgrEntityOwner::with_selectable(object, priority).cast();
    let entity = Handle::new(Select3DSensitivePoint::new(
        &owner,
        Point3::new(depth, 0., 0.),
    ))
    .cast();
    let criterion = SelectMgrSortCriterion {
        entity,
        point: Point3::new(depth, 1., 2.),
        depth,
        selection_priority: priority,
        ..SelectMgrSortCriterion::new()
    };
    let mut fields = selector.viewer_selector_base().fields.write();
    fields.results.stored.add(owner.clone(), criterion);
    fields.results.is_sorted = false;
    owner
}
pub fn mode_selection(
    object: &Handle<dyn SelectMgrSelectableObject>,
    mode: i32,
) -> Handle<SelectMgrSelection> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    let owner = SelectMgrEntityOwner::with_selectable(object, 1).cast();
    selection
        .add(&Handle::new(Select3DSensitivePoint::new(&owner, Point3::default())).cast())
        .unwrap();
    object.add_selection(&selection, mode).unwrap();
    selection
}
pub fn mode_owner(selection: &SelectMgrSelection) -> Handle<dyn SelectMgrEntityOwnerSource> {
    selection.entities()[0].base_sensitive().owner_id()
}
