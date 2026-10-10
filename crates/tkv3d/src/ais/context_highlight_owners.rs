//! Selected highlighting groups nonautomatic owners in native map order.
use super::{
    context_owner_walk::{AisOwnerList, AisOwnerWalk},
    *,
};
use crate::{prs3d::Prs3dDrawer, select_mgr::*};
use rustycad_tkernel::{
    ncollection::{DataMap, List},
    standard::{Failure, Handle},
};
type SelectedGroups = DataMap<Handle<dyn AisInteractiveObject>, Vec<AisSelectedOwner>>;
impl AisInteractiveContext {
    /// Highlight a supplied owner list with an optional explicit style; `context.highlight_owners(&owners,&style)?`.
    pub fn highlight_owners(
        &self,
        owners: &List<AisSelectedOwner>,
        style: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        self.highlight_owner_walk(AisOwnerWalk::new(AisOwnerList::Borrowed(owners)), style)
    }
    pub(super) fn highlight_owner_walk(
        &self,
        owners: AisOwnerWalk<'_>,
        style: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        self.highlight_owner_walk_with_style(owners, &|| style.clone())
    }
    pub(super) fn highlight_owner_walk_with_style(
        &self,
        mut owners: AisOwnerWalk<'_>,
        style: &impl Fn() -> Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        let mut groups = SelectedGroups::default();
        while let Some(owner) = owners.current() {
            self.highlight_selected_entry(&owner, style, &mut groups)?;
            owners.next();
        }
        for (object, owners) in groups.items() {
            object.hilight_selected(&self.detection_presentations(), owners)?;
        }
        Ok(())
    }
    fn highlight_selected_entry(
        &self,
        owner: &AisSelectedOwner,
        style: &impl Fn() -> Handle<Prs3dDrawer>,
        groups: &mut SelectedGroups,
    ) -> Result<(), Failure> {
        let object: Handle<dyn AisInteractiveObject> = owner.selectable().cast();
        if object.is_null() {
            return Ok(());
        }
        let selected = self.get_sel_style(&object, owner);
        let explicit = !style().is_null();
        let read_style = || if explicit { style() } else { selected.read() };
        let status = self.object_status(&object);
        if status.is_null() {
            return Ok(());
        }
        self.highlight_registered_owner(owner, &object, &read_style, groups)
    }
    fn highlight_registered_owner(
        &self,
        owner: &AisSelectedOwner,
        object: &Handle<dyn AisInteractiveObject>,
        read_style: &impl Fn() -> Handle<Prs3dDrawer>,
        groups: &mut SelectedGroups,
    ) -> Result<(), Failure> {
        if *owner == object.global_sel_owner() {
            self.object_status(object).set_hilight_status(true);
            self.object_status(object).set_hilight_style(&read_style());
        }
        if !owner.is_auto_hilight() {
            append_selected_group(groups, object, owner);
            return Ok(());
        }
        let mode = self.highlight_mode_from_slot(
            object,
            read_style,
            self.object_status(object).display_mode(),
        );
        owner.hilight_with_color(&self.detection_presentations(), &read_style(), mode)
    }
    /// Highlight current selection after clearing dynamic highlighting; `context.hilight_selected(true)?`.
    pub fn hilight_selected(&self, update: bool) -> Result<(), Failure> {
        self.clear_dynamic_highlight();
        self.highlight_owner_walk(self.selected_walk(), &Handle::default())?;
        if update {
            self.update_current_viewer();
        }
        Ok(())
    }
    /// Highlight one owner or all selected owners of its nonautomatic object; `context.highlight_selected(&owner)?`.
    pub fn highlight_selected(&self, owner: &AisSelectedOwner) -> Result<(), Failure> {
        let object: Handle<dyn AisInteractiveObject> = owner.selectable().cast();
        if object.is_null() {
            return Ok(());
        }
        let owners = self.highlight_members_of(&object, owner);
        self.highlight_owners(&owners, &Handle::default())
    }
    pub(super) fn highlight_members_of(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        owner: &AisSelectedOwner,
    ) -> List<AisSelectedOwner> {
        let mut owners = List::default();
        if owner.is_auto_hilight() {
            owners.append(owner.clone());
            return owners;
        }
        let mut walk = self.selected_walk();
        while let Some(selected) = walk.current() {
            if selected.is_same_selectable(&object.cast()) {
                owners.append(selected);
            }
            walk.next();
        }
        owners
    }
}
fn append_selected_group(
    groups: &mut SelectedGroups,
    object: &Handle<dyn AisInteractiveObject>,
    owner: &AisSelectedOwner,
) {
    if let Some(owners) = groups.change_seek(object) {
        owners.push(owner.clone());
        return;
    }
    groups.bind(object.clone(), vec![owner.clone()]);
}
