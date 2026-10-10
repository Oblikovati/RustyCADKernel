//! Global-owner and sub-intensity highlighting share native presentation paths.
use super::*;
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle},
};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_TOPMOST, Graphic3dPresentationAttributesMembers,
    Graphic3dPresentationAttributesSource,
};
impl AisInteractiveContext {
    /// Highlight a whole object through its global owner or its presentation; `context.highlight_global(&object,&style,-1)?`.
    pub fn highlight_global(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        style: &Handle<Prs3dDrawer>,
        display_mode: i32,
    ) -> Result<(), Failure> {
        self.highlight_global_style_slot(object, &|| style.clone(), display_mode)
    }
    pub(super) fn highlight_global_style_slot(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        style: &impl Fn() -> Handle<Prs3dDrawer>,
        display_mode: i32,
    ) -> Result<(), Failure> {
        if object.is_null() {
            return Ok(());
        }
        let mode = self.highlight_mode_from_slot(object, style, display_mode);
        let owner = object.global_sel_owner();
        if owner.is_null() {
            return self.color_global_presentation(object, &style(), mode);
        }
        let owners = self.highlight_members_of(object, &owner);
        self.highlight_owner_walk_with_style(
            super::context_owner_walk::AisOwnerWalk::new(
                super::context_owner_walk::AisOwnerList::Borrowed(&owners),
            ),
            style,
        )
    }
    /// Unhighlight a whole object through its global owner; `context.unhighlight_global(&object)?`.
    pub fn unhighlight_global(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<(), Failure> {
        if object.is_null() {
            return Ok(());
        }
        let owner = object.global_sel_owner();
        if owner.is_null() {
            self.detection_presentations().unhighlight(&object.cast());
            return Ok(());
        }
        let mut owners = List::default();
        owners.append(owner);
        self.unhighlight_owners(&owners, false)
    }
    pub(super) fn color_global_presentation(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        style: &Handle<Prs3dDrawer>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.detection_presentations().color(
            &object.cast(),
            style,
            mode,
            &Handle::default(),
            GRAPHIC3D_ZLAYER_ID_TOPMOST,
        )
    }
    fn synchronize_subintensity_transparency(&self) {
        let selected = self.fields.read().styles[1].clone();
        let sub = self.fields.read().styles[5].clone();
        sub.set_transparency(selected.transparency());
    }
    /// Apply sub-intensity with selected transparency to an object; `context.highlight_object_with_subintensity(&object,0)?`.
    pub fn highlight_object_with_subintensity(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.synchronize_subintensity_transparency();
        let style = self.fields.read().styles[5].clone();
        self.color_global_presentation(object, &style, mode)
    }
    /// Apply sub-intensity with selected transparency to an owner; `context.highlight_owner_with_subintensity(&owner,0)?`.
    pub fn highlight_owner_with_subintensity(
        &self,
        owner: &AisSelectedOwner,
        mode: i32,
    ) -> Result<(), Failure> {
        self.synchronize_subintensity_transparency();
        let style = self.fields.read().styles[5].clone();
        owner.hilight_with_color(&self.detection_presentations(), &style, mode)
    }
}
