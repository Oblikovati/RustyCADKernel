//! Whole-object and owner replacement preserve native fast paths and callback ordering.
use super::*;
use crate::{prs_mgr::PrsMgrPresentableObjectMembers, select_mgr::*};
use rustycad_tkernel::standard::{Failure, Handle};
impl AisInteractiveContext {
    /// Replace the selection with a registered object's global owner; `context.set_selected_object(&object,true)?`.
    pub fn set_selected_object(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update: bool,
    ) -> Result<(), Failure> {
        if object.is_null() || self.object_status(object).is_null() {
            return Ok(());
        }
        let owner = object.global_sel_owner();
        if owner.is_null() {
            return Ok(());
        }
        let style = self.get_sel_style(object, &owner);
        if self.nb_selected() == 1
            && self.object_status(object).is_hilighted()
            && self.automatic_hilight()
        {
            return self.refresh_selected_object_style(object, &style, update, false);
        }
        self.finish_object_replacement(object, &owner, &style, update)
    }
    fn finish_object_replacement(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        owner: &AisSelectedOwner,
        style: &AisHighlightStyle<'_>,
        update: bool,
    ) -> Result<(), Failure> {
        self.replace_object_selection(owner)?;
        if self.automatic_hilight() {
            self.refresh_selected_object_style(object, style, false, true)?;
        }
        if update {
            self.update_current_viewer();
        }
        Ok(())
    }
    fn replace_object_selection(&self, owner: &AisSelectedOwner) -> Result<(), Failure> {
        self.unhighlight_filtered_selection()?;
        let selected: Handle<dyn AisInteractiveObject> = owner.selectable().cast();
        let filters = self.fields.read().filters.cast();
        self.selected_container().clear_and_select(
            owner,
            &filters,
            self.is_detected_object(&selected)?,
        )?;
        Ok(())
    }
    fn refresh_selected_object_style(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        style: &AisHighlightStyle<'_>,
        update: bool,
        highlight_missing: bool,
    ) -> Result<(), Failure> {
        let mut previous = Handle::default();
        if self.object_highlight_style(object, &mut previous) {
            if !previous.is_null() && previous != style.read() {
                self.hilight_object_style_slot(object, &|| style.read(), update)?;
            }
        } else if highlight_missing {
            self.hilight_object_style_slot(object, &|| style.read(), update)?;
        }
        Ok(())
    }
    fn unhighlight_filtered_selection(&self) -> Result<(), Failure> {
        let mut walk = self.selected_walk();
        while let Some(owner) = walk.current() {
            let filter = self.fields.read().filters.clone();
            if filter.is_ok(&owner)? {
                self.unhighlight_filtered_owner(&owner)?;
            }
            walk.next();
        }
        Ok(())
    }
    fn unhighlight_filtered_owner(&self, owner: &AisSelectedOwner) -> Result<(), Failure> {
        let object: Handle<dyn AisInteractiveObject> = owner.selectable().cast();
        if self.automatic_hilight() {
            self.unhilight(&object, false)?;
        }
        if *owner == object.global_sel_owner()
            && let Some(status) = self.object_status(&object).get()
        {
            status.set_hilight_status(false);
        }
        Ok(())
    }
    /// Replace the selection with a filtered owner, respecting forced highlighting; `context.set_selected_owner(&owner,true)?`.
    pub fn set_selected_owner(
        &self,
        owner: &AisSelectedOwner,
        update: bool,
    ) -> Result<(), Failure> {
        if owner.get().is_none_or(|owner| !owner.has_selectable()) {
            return Ok(());
        }
        let filter = self.fields.read().filters.clone();
        if !filter.is_ok(owner)? {
            return Ok(());
        }
        let object: Handle<dyn AisInteractiveObject> = owner.selectable().cast();
        let style = self.get_sel_style(&object, owner);
        if self.nb_selected() == 1 && owner.is_selected() && !owner.is_forced_hilight() {
            return self.refresh_selected_owner_style(owner, &object, &style);
        }
        if self.object_status(&object).is_null() {
            return Ok(());
        }
        self.finish_owner_replacement(owner, &object, &style, update)
    }
    fn finish_owner_replacement(
        &self,
        owner: &AisSelectedOwner,
        object: &Handle<dyn AisInteractiveObject>,
        style: &AisHighlightStyle<'_>,
        update: bool,
    ) -> Result<(), Failure> {
        self.replace_selected_owner(owner, object)?;
        if self.automatic_hilight() && self.owner_style_needs_highlight(owner, style) {
            self.highlight_selected(owner)?;
        }
        if update {
            self.update_current_viewer();
        }
        Ok(())
    }
    fn replace_selected_owner(
        &self,
        owner: &AisSelectedOwner,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<(), Failure> {
        if self.automatic_hilight() {
            self.unhighlight_selected(false)?;
        }
        let filter = self.fields.read().filters.cast();
        self.selected_container()
            .clear_and_select(owner, &filter, self.is_detected_object(object)?)
    }
    fn owner_style_needs_highlight(
        &self,
        owner: &AisSelectedOwner,
        style: &AisHighlightStyle<'_>,
    ) -> bool {
        let mut previous = Handle::default();
        !self.owner_highlight_style(owner, &mut previous)
            || (!previous.is_null() && previous != style.read())
    }
    fn refresh_selected_owner_style(
        &self,
        owner: &AisSelectedOwner,
        object: &Handle<dyn AisInteractiveObject>,
        style: &AisHighlightStyle<'_>,
    ) -> Result<(), Failure> {
        let mut previous = Handle::default();
        if self.automatic_hilight()
            && self.owner_highlight_style(owner, &mut previous)
            && !previous.is_null()
            && previous != style.read()
        {
            let mode = if object.has_hilight_mode() {
                object.hilight_mode()
            } else {
                0
            };
            owner.hilight_with_color(&self.detection_presentations(), &style.read(), mode)?;
        }
        Ok(())
    }
}
