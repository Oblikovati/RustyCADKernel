//! Native V3d_View subclasses own the actual view base and override its four virtual operations.
use super::V3dView;
use rustycad_tkernel::standard::{HandleReference, RuntimeType, RuntimeTyped, Transient};

/// Embed the genuine view fields; `derived.view_base().viewer()`.
pub trait V3dViewSource: Transient {
    /// Borrow the actual owned native base, without retaining another view; `view.view_base()`.
    fn view_base(&self) -> &V3dView;
    /// Native virtual redraw; `view.redraw()`.
    fn redraw(&self) {
        super::view_source_redraw::v3d_view_redraw(self.view_base());
    }
    /// Native virtual immediate redraw; `view.redraw_immediate()`.
    fn redraw_immediate(&self) {
        super::view_source_redraw::v3d_view_redraw_immediate(self.view_base());
    }
    /// Native virtual insertion borrows the caller's live plane slot; `view.add_clip_plane(&plane)`.
    fn add_clip_plane(
        &self,
        plane: &dyn HandleReference<dyn rustycad_tkservice::graphic3d::Graphic3dClipPlaneSource>,
    ) {
        super::view_source_clipping::v3d_view_add_clip_plane(self.view_base(), plane);
    }
    /// Native virtual removal borrows the caller's live plane slot; `view.remove_clip_plane(&plane)`.
    fn remove_clip_plane(
        &self,
        plane: &dyn HandleReference<dyn rustycad_tkservice::graphic3d::Graphic3dClipPlaneSource>,
    ) {
        super::view_source_clipping::v3d_view_remove_clip_plane(self.view_base(), plane);
    }
}
impl RuntimeTyped for dyn V3dViewSource {
    const TYPE_NAME: &'static str = "V3d_View";
    fn static_type() -> &'static RuntimeType {
        // Native base and interface handles share descriptor pointer identity.
        V3dView::static_type()
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        rustycad_tkernel::standard::project_interface::<Self>(object)
    }
}
impl std::ops::Deref for dyn V3dViewSource {
    type Target = V3dView;
    fn deref(&self) -> &Self::Target {
        // Nonvirtual members execute on the embedded fields, never a copied core view.
        self.view_base()
    }
}
impl V3dViewSource for V3dView {
    fn view_base(&self) -> &V3dView {
        self
    }
}
