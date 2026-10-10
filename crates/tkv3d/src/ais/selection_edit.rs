//! Native mutation boundaries release state locks before virtual owner/filter callbacks.
use super::{AisSelectStatus, AisSelectedOwner, AisSelectionBase, AisSelectionScheme};
use crate::select_mgr::{SelectMgrEntityOwnerMembers, SelectMgrFilter};
use rustycad_tkernel::{
    ncollection::{Array1, ListCursor},
    standard::{Failure, Handle, exceptions::null_object},
};
/// Native Clear body, callable from overrides; `ais_selection_clear(&base)?`.
pub fn ais_selection_clear(base: &AisSelectionBase) -> Result<(), Failure> {
    let owners: Vec<_> = base.fields.read().owners.iter().cloned().collect();
    for owner in &owners {
        owner
            .get()
            .ok_or_else(|| null_object("selected owner is null; expected a non-null owner"))?
            .set_selected(false);
    }
    let mut fields = base.fields.write();
    fields.owners.clear();
    fields.positions.clear(true);
    fields.cursor = ListCursor::default();
    drop(fields);
    drop(owners);
    Ok(())
}
/// Native AddSelect body, callable from overrides; `ais_selection_add_select(&base,&owner)?`.
pub fn ais_selection_add_select(
    base: &AisSelectionBase,
    owner: &AisSelectedOwner,
) -> Result<AisSelectStatus, Failure> {
    if owner.get().is_none_or(|owner| !owner.has_selectable())
        || base.fields.read().positions.is_bound(owner)
    {
        return Ok(AisSelectStatus::NotDone);
    }
    append_selected_owner(base, owner);
    Ok(AisSelectStatus::Added)
}
/// Native Select body with observable forced-highlight cursor timing.
/// Example: `ais_selection_select(&base,&owner,&filter,AisSelectionScheme::Add,true)?`.
pub fn ais_selection_select(
    base: &AisSelectionBase,
    owner: &AisSelectedOwner,
    filter: &Handle<dyn SelectMgrFilter>,
    scheme: AisSelectionScheme,
    detected: bool,
) -> Result<AisSelectStatus, Failure> {
    let Some(_) = owner.get().filter(|owner| owner.has_selectable()) else {
        return Ok(AisSelectStatus::NotDone);
    };
    let detected = detected
        && match filter.get() {
            Some(filter) => filter.is_ok(owner)?,
            None => true,
        };
    select_eligible_owner(base, owner, scheme, detected)
}
fn select_eligible_owner(
    base: &AisSelectionBase,
    owner: &AisSelectedOwner,
    scheme: AisSelectionScheme,
    detected: bool,
) -> Result<AisSelectStatus, Failure> {
    let was_selected = owner.is_selected();
    let to_select = owner.select(scheme, detected);
    if was_selected && base.fields.read().positions.is_bound(owner) {
        return deselect_bound_owner(base, owner);
    }
    if !to_select {
        return Ok(AisSelectStatus::NotDone);
    }
    append_selected_owner(base, owner);
    Ok(AisSelectStatus::Added)
}
fn deselect_bound_owner(
    base: &AisSelectionBase,
    owner: &AisSelectedOwner,
) -> Result<AisSelectStatus, Failure> {
    let position = advance_selection_cursor(base, owner)?;
    // OCCT 3d097a0 advances iteration even when forced highlighting keeps the owner.
    if owner.is_forced_hilight() {
        return Ok(AisSelectStatus::Added);
    }
    remove_selected_owner(base, owner, position)?;
    Ok(AisSelectStatus::Removed)
}
fn append_selected_owner(base: &AisSelectionBase, owner: &AisSelectedOwner) {
    let mut fields = base.fields.write();
    let mut position = ListCursor::default();
    fields.owners.append_at(owner.clone(), &mut position);
    fields.positions.bind(owner.clone(), position);
    drop(fields);
    owner.set_selected(true);
}
fn advance_selection_cursor(
    base: &AisSelectionBase,
    owner: &AisSelectedOwner,
) -> Result<ListCursor, Failure> {
    let mut fields = base.fields.write();
    let position = *fields.positions.find(owner)?;
    if fields.cursor == position {
        let super::AisSelectionFields { owners, cursor, .. } = &mut *fields;
        if cursor.more() {
            cursor.next(owners);
        } else {
            *cursor = ListCursor::default();
        }
    }
    Ok(position)
}
fn remove_selected_owner(
    base: &AisSelectionBase,
    owner: &AisSelectedOwner,
    mut position: ListCursor,
) -> Result<(), Failure> {
    {
        let mut fields = base.fields.write();
        fields.owners.remove(&mut position)?;
        fields.positions.unbind(owner);
    }
    owner.set_selected(false);
    let mut fields = base.fields.write();
    if let Some(next) = position.value(&fields.owners).cloned() {
        fields.positions.bind(next, position);
    }
    Ok(())
}
pub(super) fn ais_replace_extra_matches(
    base: &AisSelectionBase,
    picked: &Array1<'_, AisSelectedOwner>,
    scheme: AisSelectionScheme,
) -> bool {
    if scheme != AisSelectionScheme::ReplaceExtra {
        return false;
    }
    let fields = base.fields.read();
    picked.len() == fields.owners.len()
        && picked
            .as_slice()
            .iter()
            .all(|owner| fields.positions.is_bound(owner))
}
