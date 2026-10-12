//! Context fields remain real native aliases while virtual callbacks can mutate the context.
use super::{AisInteractiveContext, AisInteractiveContextFields};
use crate::{prs_mgr::PrsMgrPresentationManager, v3d::V3dViewer};
use rustycad_tkernel::standard::{Handle, HandleFieldRef};
impl AisInteractiveContext {
    /// Observe the actual viewer slot without retaining a context lock; `context.current_viewer_ref().observe()`.
    pub fn current_viewer_ref(&self) -> HandleFieldRef<'_, AisInteractiveContextFields, V3dViewer> {
        // Native CurrentViewer aliases myMainVwr, even across unrelated context callbacks.
        HandleFieldRef::new(&self.fields, select_context_frame_viewer)
    }
    /// Observe the actual presentation-manager slot without blocking callbacks; `context.main_prs_mgr_ref().observe()`.
    pub fn main_prs_mgr_ref(
        &self,
    ) -> HandleFieldRef<'_, AisInteractiveContextFields, PrsMgrPresentationManager> {
        // A whole-context read guard prevents native-valid updates from virtual owner callbacks.
        HandleFieldRef::new(&self.fields, select_context_frame_presentation_manager)
    }
}
fn select_context_frame_viewer(fields: &AisInteractiveContextFields) -> &Handle<V3dViewer> {
    // Reference identity is the actual field, rather than a temporary copied target.
    &fields.viewer
}
fn select_context_frame_presentation_manager(
    fields: &AisInteractiveContextFields,
) -> &Handle<PrsMgrPresentationManager> {
    // This field selection remains fixed if callers later select a different context object.
    &fields.presentation_manager
}
