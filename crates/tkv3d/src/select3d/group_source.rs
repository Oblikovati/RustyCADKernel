//! Martensite application groups compose native state while retaining virtual selection overrides.
use super::{Select3DSensitiveGroup, Select3DSensitiveSet, select3d_set_matches};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrSelectionType;
use rustycad_tkernel::standard::Failure;

/// The native Group `base_type` alias, retaining the sensitive-set virtual interface.
/// Example: `let base: Handle<Select3DSensitiveGroupBase> = group.clone().cast();`.
pub type Select3DSensitiveGroupBase = dyn Select3DSensitiveSet;

/// A composite group with overridable sensitive-set hooks and one shared group state.
/// Example: `select3d_group_matches(&application_group, &volume, &mut pick)?;`.
pub trait Select3DSensitiveGroupSource: Select3DSensitiveSet {
    /// Borrows the native group subobject; `let members = source.sensitive_group().entities_ref();`.
    fn sensitive_group(&self) -> &Select3DSensitiveGroup;
}
impl Select3DSensitiveGroupSource for Select3DSensitiveGroup {
    fn sensitive_group(&self) -> &Select3DSensitiveGroup {
        // The concrete group and application wrappers use the same state protocol.
        self
    }
}
/// Runs native group selection with the source's virtual traversal/distance hooks.
/// Example: `select3d_group_matches(&application_group, &volume, &mut pick)?;`.
pub fn select3d_group_matches<S: Select3DSensitiveGroupSource + ?Sized>(
    source: &S,
    volume: &dyn SelectBasicsSelectingVolumeManager,
    pick: &mut SelectBasicsPickResult,
) -> Result<bool, Failure> {
    super::triangulation_failure::catch_triangulation_failure(|| {
        match_group_source(source, volume, pick)
    })
}
fn match_group_source<S: Select3DSensitiveGroupSource + ?Sized>(
    source: &S,
    volume: &dyn SelectBasicsSelectingVolumeManager,
    pick: &mut SelectBasicsPickResult,
) -> Result<bool, Failure> {
    let group = source.sensitive_group();
    // OCCT 3d097a0 invokes selection type twice; each callback may change the subsequent policy.
    let must = volume.get_active_selection_type() != SelectMgrSelectionType::Point
        && group.must_match_all();
    let check = volume.get_active_selection_type() != SelectMgrSelectionType::Point
        && group.to_check_overlap_all();
    if !must && !check {
        return select3d_set_matches(source, volume, pick, false);
    }
    match_source_children(source, volume, pick, (must, check))
}
fn match_source_children<S: Select3DSensitiveGroupSource + ?Sized>(
    source: &S,
    volume: &dyn SelectBasicsSelectingVolumeManager,
    pick: &mut SelectBasicsPickResult,
    policy: (bool, bool),
) -> Result<bool, Failure> {
    // Direct all/check-all matching also dispatches the application's virtual distanceToCOG.
    if source
        .sensitive_group()
        .check_group_children(volume, pick, policy)?
    {
        return Ok(false);
    }
    pick.set_dist_to_geom_center(source.distance_to_cog(volume)?);
    Ok(true)
}

/// Runs the native qualified Group overlap hook on its stored child.
/// Example: `select3d_group_overlaps_element(&source,&mut pick,&volume,0,false)?;`.
pub fn select3d_group_overlaps_element<S: Select3DSensitiveGroupSource + ?Sized>(
    source: &S,
    pick: &mut SelectBasicsPickResult,
    volume: &dyn SelectBasicsSelectingVolumeManager,
    index: i32,
    _inside: bool,
) -> Result<bool, Failure> {
    // OCCT 3d097a0 ignores full-inside policy here and always dispatches the child Matches call.
    source
        .sensitive_group()
        .primitive_member(index)
        .get()
        .expect("Group overlap null member; expected sensitive entity")
        .matches(volume, pick)
}
/// Runs native inclusion through the source's virtual overlap override with a fresh dummy pick.
/// Example: `select3d_group_element_is_inside(&source,&volume,0,false)?;`.
pub fn select3d_group_element_is_inside<S: Select3DSensitiveGroupSource + ?Sized>(
    source: &S,
    volume: &dyn SelectBasicsSelectingVolumeManager,
    index: i32,
    inside: bool,
) -> Result<bool, Failure> {
    // A qualified Group::elementIsInside still calls virtual overlapsElement on the subclass.
    let mut dummy = SelectBasicsPickResult::default();
    source.overlaps_element(&mut dummy, volume, index, inside)
}
/// Computes native group distance through the source's virtual geometry center.
/// Example: `select3d_group_distance_to_cog(&source,&volume)?;`.
pub fn select3d_group_distance_to_cog<S: Select3DSensitiveGroupSource + ?Sized>(
    source: &S,
    volume: &dyn SelectBasicsSelectingVolumeManager,
) -> Result<f64, Failure> {
    // Calling the core's center directly would bypass a defined application override.
    volume.dist_to_geometry_center(&source.center_of_geometry())
}

rustycad_tkernel::impl_transient_interface!(
    dyn Select3DSensitiveGroupSource,
    "Martensite_SensitiveGroupSource",
    dyn Select3DSensitiveSet
);

/// Downcasts a native Group entity into a view retaining its application's virtual hooks.
/// Example: `let group = select3d_group_downcast(&entity); group.matches(&volume,&mut pick)?;`.
pub fn select3d_group_downcast(
    entity: &rustycad_tkernel::standard::Handle<dyn super::Select3DSensitiveEntity>,
) -> rustycad_tkernel::standard::Handle<dyn Select3DSensitiveGroupSource> {
    use rustycad_tkernel::standard::RuntimeTyped;
    if !entity
        .get()
        .is_some_and(|value| value.is_kind(Select3DSensitiveGroup::static_type()))
    {
        return rustycad_tkernel::standard::Handle::default();
    }
    // A concrete base-subobject projection would bypass the application's overridden virtual methods.
    entity.clone().cast()
}
