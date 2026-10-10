//! Unhighlight registered owners before clearing grouped nonautomatic objects.
use super::{
    context_owner_walk::{AisOwnerList, AisOwnerWalk},
    *,
};

use rustycad_tkernel::{
    ncollection::{IndexedMap, List},
    standard::{Failure, Handle},
};
impl AisInteractiveContext {
    /// Unhighlight supplied owners, optionally restoring sub-intensity; `context.unhighlight_owners(&owners,false)?`.
    pub fn unhighlight_owners(
        &self,
        owners: &List<AisSelectedOwner>,
        sub_intensity: bool,
    ) -> Result<(), Failure> {
        self.unhighlight_owner_walk(
            AisOwnerWalk::new(AisOwnerList::Borrowed(owners)),
            sub_intensity,
        )
    }
    fn unhighlight_owner_walk(
        &self,
        mut owners: AisOwnerWalk<'_>,
        sub_intensity: bool,
    ) -> Result<(), Failure> {
        let mut objects = IndexedMap::default();
        while let Some(owner) = owners.current() {
            self.unhighlight_selected_entry(&owner, sub_intensity, &mut objects)?;
            owners.next();
        }
        for object in &objects {
            self.detection_presentations().unhighlight(&object.cast());
            object.clear_selected();
        }
        Ok(())
    }
    fn unhighlight_selected_entry(
        &self,
        owner: &AisSelectedOwner,
        sub_intensity: bool,
        objects: &mut IndexedMap<Handle<dyn AisInteractiveObject>>,
    ) -> Result<(), Failure> {
        let object: Handle<dyn AisInteractiveObject> = owner.selectable().cast();
        if self.object_status(&object).is_null() {
            return Ok(());
        }
        if owner.is_auto_hilight() {
            self.unhighlight_automatic_owner(&object, owner, sub_intensity)?;
        } else {
            objects.add(object.clone());
        }
        if *owner == object.global_sel_owner() {
            self.object_status(&object).set_hilight_status(false);
        }
        self.object_status(&object)
            .set_hilight_style(&Handle::default());
        Ok(())
    }
    fn unhighlight_automatic_owner(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        owner: &AisSelectedOwner,
        sub_intensity: bool,
    ) -> Result<(), Failure> {
        owner.unhilight(&self.detection_presentations(), 0);
        if sub_intensity && self.object_status(object).is_sub_intensity_on() {
            let read_style = || self.object_status(object).hilight_style().clone();
            let mode = self.highlight_mode_from_slot(
                object,
                &read_style,
                self.object_status(object).display_mode(),
            );
            self.highlight_owner_with_subintensity(owner, mode)?;
        }
        Ok(())
    }
    /// Clear highlighting of current selected owners; `context.unhighlight_selected(false)?`.
    pub fn unhighlight_selected(&self, sub_intensity: bool) -> Result<(), Failure> {
        self.unhighlight_owner_walk(self.selected_walk(), sub_intensity)
    }
    /// Clear selected highlighting and optionally redraw; `context.unhilight_selected(true)?`.
    pub fn unhilight_selected(&self, update: bool) -> Result<(), Failure> {
        self.unhighlight_selected(false)?;
        if update {
            self.update_current_viewer();
        }
        Ok(())
    }
    /// Clear selection with native callback and automatic-highlight ordering; `context.clear_selected(true)?`.
    pub fn clear_selected(&self, update: bool) -> Result<(), Failure> {
        if self.nb_selected() == 0 {
            return Ok(());
        }
        let automatic = self.fields.read().automatic_hilight;
        if automatic {
            self.unhighlight_selected(false)?;
        }
        self.selected_container().clear()?;
        let automatic = self.fields.read().automatic_hilight;
        if automatic {
            self.clear_dynamic_highlight();
        }
        if update {
            self.update_current_viewer();
        }
        Ok(())
    }
}
