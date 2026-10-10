//! Precise sensitive matching and native per-owner criterion accumulation.
use super::*;
use crate::{
    ais::{AisInteractiveObject, AisInteractiveObjectMembers},
    prs_mgr::PrsMgrPresentableObjectMembers,
    select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager},
    select3d::{Select3DSensitiveEntity, Select3DSensitiveEntityMembers},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::GeneralTransform3;
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_DEFAULT, Graphic3dDisplayPriority, Graphic3dStructureMembers,
    Graphic3dTransModeFlags, Graphic3dTransformPersMembers,
};

/// Native private checkOverlap body used by the two-level selector traversal.
/// Example: `select_mgr_check_overlap(&base,&entity,&inverse,&volume)?`.
pub fn select_mgr_check_overlap(
    base: &SelectMgrViewerSelectorBase,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    inverse: &GeneralTransform3,
    volume: &SelectMgrSelectingVolumeManager,
) -> Result<(), Failure> {
    let selectable = entity
        .owner_id()
        .get()
        .map_or_else(Handle::default, SelectMgrEntityOwnerSource::selectable);
    let mut pick = SelectBasicsPickResult::default();
    if !entity.matches(volume, &mut pick)? {
        return Ok(());
    }
    // OCCT keeps a reference to the owner slot, which Matches may replace (3d097a0).
    let owner = entity.owner_id();
    if owner.is_null() {
        return Ok(());
    }
    let criterion = overlap_criterion(base, entity, &owner, &selectable, &pick)?;
    accumulate_overlap(base, criterion, &owner, &pick, inverse, volume)
}
fn overlap_criterion(
    base: &SelectMgrViewerSelectorBase,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    selectable: &Handle<dyn SelectMgrSelectableObject>,
    pick: &SelectBasicsPickResult,
) -> Result<SelectMgrSortCriterion, Failure> {
    let layer = selectable.get().map_or(
        GRAPHIC3D_ZLAYER_ID_DEFAULT,
        PrsMgrPresentableObjectMembers::z_layer,
    );
    let mut criterion = SelectMgrSortCriterion {
        z_layer_position: base
            .fields
            .read()
            .z_layer_order
            .seek(&layer)
            .copied()
            .unwrap_or(0),
        entity: entity.clone(),
        selection_priority: owner.priority(),
        depth: pick.depth(),
        min_dist: pick.dist_to_geom_center(),
        ..SelectMgrSortCriterion::default()
    };
    apply_overlap_display_priority(&mut criterion, selectable)?;
    Ok(criterion)
}
fn apply_overlap_display_priority(
    criterion: &mut SelectMgrSortCriterion,
    selectable: &Handle<dyn SelectMgrSelectableObject>,
) -> Result<(), Failure> {
    let Some(object) = selectable.get() else {
        return Ok(());
    };
    let persistence = object.transform_persistence();
    if object.presentations().is_empty()
        || persistence.is_null()
        || persistence.mode() != Graphic3dTransModeFlags::TMF_2D
    {
        return Ok(());
    }
    criterion.is_prefer_priority = true;
    criterion.display_priority = Graphic3dDisplayPriority::Invalid as i32;
    if let Some(interactive) = selectable.cast::<dyn AisInteractiveObject>().get()
        && let Some(presentation) = interactive.presentation()?.get()
    {
        criterion.display_priority = presentation.display_priority() as i32;
    }
    Ok(())
}
fn previous_owner_criterion(
    base: &SelectMgrViewerSelectorBase,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Option<SelectMgrSortCriterion> {
    let mut fields = base.fields.write();
    let previous = fields.results.stored.change_seek(owner)?;
    previous.nb_owner_matches += 1;
    Some(previous.clone())
}
fn accumulate_overlap(
    base: &SelectMgrViewerSelectorBase,
    mut criterion: SelectMgrSortCriterion,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    pick: &SelectBasicsPickResult,
    inverse: &GeneralTransform3,
    volume: &SelectMgrSelectingVolumeManager,
) -> Result<(), Failure> {
    let previous = previous_owner_criterion(base, owner);
    criterion.nb_owner_matches = previous.as_ref().map_or(1, |value| value.nb_owner_matches);
    if previous.is_some() && volume.get_active_selection_type() == SelectMgrSelectionType::Box {
        return Ok(());
    }
    let entity = criterion.entity.clone();
    select_mgr_update_point3d(base, &mut criterion, pick, &entity, inverse, volume)?;
    if let Some(previous) = previous {
        if criterion.is_closer_depth(&previous)? {
            *base.fields.write().results.stored.change_from_key(owner)? = criterion;
        }
    } else {
        base.fields
            .write()
            .results
            .stored
            .add(entity.owner_id(), criterion);
    }
    Ok(())
}
