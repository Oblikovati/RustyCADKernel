//! Per-view visibility only applies to objects already registered in the context.
use super::{AisInteractiveContext, AisInteractiveObject};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::v3d::V3dViewSource;
use rustycad_tkernel::standard::{Failure, Handle, exceptions::null_object};
use rustycad_tkservice::graphic3d::Graphic3dCViewMembers;
impl AisInteractiveContext {
    /// Change one view's visibility without redrawing or altering other affinity bits.
    /// Example: `context.set_view_affinity(&object, &view, false)?`.
    pub fn set_view_affinity(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        view: &Handle<dyn V3dViewSource>,
        visible: bool,
    ) -> Result<(), Failure> {
        if object.is_null() || self.object_status(object).is_null() {
            return Ok(());
        }
        let view = view
            .get()
            .ok_or_else(|| null_object("affinity view is null; expected a live V3d view"))?;
        object
            .view_affinity()
            .set_view_visible(view.view().identification(), visible);
        Ok(())
    }
}
