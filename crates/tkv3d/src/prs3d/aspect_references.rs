//! Borrow actual Graphic3d aspect fields while allowing native setter replacement and driver callbacks.
use super::{Prs3dLineAspect, Prs3dPointAspect, Prs3dShadingAspect};
use rustycad_tkernel::standard::{Handle, HandleFieldRef, RuntimeTyped};
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectFillArea3d, Graphic3dAspectLine3d, Graphic3dAspectMarker3d,
};
impl Prs3dLineAspect {
    /// Borrow the real const aspect slot without ownership; `line.aspect_ref().observe()`.
    pub fn aspect_ref(
        &self,
    ) -> HandleFieldRef<'_, Handle<Graphic3dAspectLine3d>, Graphic3dAspectLine3d> {
        let field = &self.aspect;
        HandleFieldRef::new(field, borrow_graphic_aspect_slot)
    }
}
impl Prs3dPointAspect {
    /// Borrow the actual marker field without incrementing its count; `point.aspect_ref().observe()`.
    pub fn aspect_ref(
        &self,
    ) -> HandleFieldRef<'_, Handle<Graphic3dAspectMarker3d>, Graphic3dAspectMarker3d> {
        let field = &self.aspect;
        HandleFieldRef::new(field, borrow_graphic_aspect_slot)
    }
}
impl Prs3dShadingAspect {
    /// Borrow the native fill-area field, retaining later setter visibility; `shading.aspect_ref().observe()`.
    pub fn aspect_ref(
        &self,
    ) -> HandleFieldRef<'_, Handle<Graphic3dAspectFillArea3d>, Graphic3dAspectFillArea3d> {
        let field = &self.aspect;
        HandleFieldRef::new(field, borrow_graphic_aspect_slot)
    }
}
fn borrow_graphic_aspect_slot<T: RuntimeTyped>(slot: &Handle<T>) -> &Handle<T> {
    // A selector points at the actual member; it never fabricates a copied const-reference result.
    slot
}
