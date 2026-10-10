//! Dynamic drawer references follow their original slots across virtual callbacks.
use super::{AisInteractiveContext, AisInteractiveObject};
use crate::{
    prs_mgr::PrsMgrPresentableObjectMembers, prs3d::Prs3dDrawer, select_mgr::*, v3d::V3dViewer,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_UNKNOWN, Graphic3dPresentationAttributesMembers,
};

/// A native drawer slot; reading again observes replacement without retaining a lock.
/// Example: `context.get_hi_style(&object, &owner).read()`.
pub struct AisHighlightStyle<'a> {
    context: &'a AisInteractiveContext,
    object: Handle<dyn AisInteractiveObject>,
    slot: Option<usize>,
}
impl AisHighlightStyle<'_> {
    /// Read the selected slot, without reselecting it; `style.read()`.
    pub fn read(&self) -> Handle<Prs3dDrawer> {
        match self.slot {
            Some(slot) => self.context.fields.read().styles[slot].clone(),
            None => self.object.dynamic_hilight_attributes(),
        }
    }
}
impl AisInteractiveContext {
    /// Select the custom, local or ordinary dynamic drawer slot; `context.get_hi_style(&object,&owner)`.
    pub fn get_hi_style(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ) -> AisHighlightStyle<'_> {
        let local = owner
            .get()
            .is_some_and(|owner| owner.comes_from_decomposition());
        let slot = object
            .dynamic_hilight_attributes()
            .is_null()
            .then_some(if local { 4 } else { 2 });
        AisHighlightStyle {
            context: self,
            object: object.clone(),
            slot,
        }
    }
    /// Select a display mode using the native style/object/default priority; `context.get_hilight_mode(&object,&style,-1)`.
    pub fn get_hilight_mode(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        style: &Handle<Prs3dDrawer>,
        display_mode: i32,
    ) -> i32 {
        self.highlight_mode_from_slot(object, &|| style.clone(), display_mode)
    }
    fn highlight_mode_from_slot(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        style: &impl Fn() -> Handle<Prs3dDrawer>,
        display_mode: i32,
    ) -> i32 {
        let current = style();
        if !current.is_null()
            && current.display_mode() != -1
            && object.accept_display_mode(current.display_mode())
        {
            return style().display_mode();
        }
        if display_mode != -1 {
            return display_mode;
        }
        if object.has_display_mode() {
            return object.display_mode();
        }
        self.default_drawer().display_mode()
    }
    /// Highlight an owner through an immediate presentation; `context.highlight_with_color(&owner,&viewer)?`.
    pub fn highlight_with_color(
        &self,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        viewer: &Handle<V3dViewer>,
    ) -> Result<(), Failure> {
        self.highlight_owner_slot(&|| owner.clone(), viewer)
    }
    pub(super) fn highlight_owner_slot(
        &self,
        owner: &impl Fn() -> Handle<dyn SelectMgrEntityOwnerSource>,
        viewer: &Handle<V3dViewer>,
    ) -> Result<(), Failure> {
        let object: Handle<dyn AisInteractiveObject> = owner().selectable().cast();
        if object.is_null() {
            return Ok(());
        }
        let style = self.get_hi_style(&object, &owner());
        let mode = self.highlight_mode_from_slot(&object, &|| style.read(), -1);
        self.detection_presentations().begin_immediate_draw();
        owner().hilight_with_color(&self.detection_presentations(), &style.read(), mode)?;
        let viewer = if viewer.is_null() {
            self.detection_viewer()
        } else {
            viewer.clone()
        };
        self.detection_presentations().end_immediate_draw(&viewer)
    }
    /// Determine whether dynamic highlighting needs a full redraw; `context.is_slow_hi_style(&owner,&viewer)?`.
    pub fn is_slow_hi_style(
        &self,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        viewer: &Handle<V3dViewer>,
    ) -> Result<bool, Failure> {
        let object: Handle<dyn AisInteractiveObject> = owner.selectable().cast();
        if object.is_null() {
            return Ok(false);
        }
        // OCCT 3d097a0 chooses decomposition from myLastPicked, not the supplied owner.
        let last = self.fields.read().last_picked.clone();
        let style = self.get_hi_style(&object, &last).read();
        Ok(style.z_layer() == GRAPHIC3D_ZLAYER_ID_UNKNOWN
            || !viewer.z_layer_settings(style.z_layer())?.is_immediate())
    }
    /// Clear the last selectable's dynamic presentation; `context.clear_dynamic_highlight()`.
    pub fn clear_dynamic_highlight(&self) {
        let owner = self.fields.read().last_picked.clone();
        if owner.is_null() {
            return;
        }
        owner
            .selectable()
            .clear_dynamic_highlight(&self.detection_presentations());
    }
}
