//! Shading handle references bind the actual slot owner, including drawer inheritance at lookup.
use crate::prs3d::{Prs3dDrawer, Prs3dShadingAspect};
use rustycad_tkernel::standard::{Handle, HandleObserver, SharedState};
use std::marker::PhantomData;
enum CubeShadingSlot<'a> {
    Local(&'a SharedState<Handle<Prs3dShadingAspect>>),
    Drawer(
        HandleObserver<Prs3dDrawer>,
        PhantomData<&'a super::AisViewCubePresentationBase>,
    ),
}
/// Native borrowed shading handle slot, resolved without keeping a lock.
/// Example: `let alias=cube.box_edge_style_ref(); alias.with_shading(|style| style.unwrap().set_color(red, facing))`.
pub struct AisViewCubeShadingRef<'a> {
    slot: CubeShadingSlot<'a>,
}
impl<'a> AisViewCubeShadingRef<'a> {
    pub(super) fn local(slot: &'a SharedState<Handle<Prs3dShadingAspect>>) -> Self {
        Self {
            slot: CubeShadingSlot::Local(slot),
        }
    }
    pub(super) fn drawer(
        drawer: &Handle<Prs3dDrawer>,
        _: &'a super::AisViewCubePresentationBase,
    ) -> Self {
        Self {
            slot: CubeShadingSlot::Drawer(cube_shading_slot_owner(drawer), PhantomData),
        }
    }
    /// Read current null assignment; `alias.is_null()`.
    pub fn is_null(&self) -> bool {
        self.current().is_null()
    }
    /// Observe current object identity; `alias.object_address()`.
    pub fn object_address(&self) -> *const () {
        self.current().object_address()
    }
    /// Explicit owning snapshot, unaffected by later slot assignment; `alias.clone_handle()`.
    pub fn clone_handle(&self) -> Handle<Prs3dShadingAspect> {
        self.current().clone()
    }
    /// Compare actual slots, not merely equal current targets; `left.is_same_reference(&right)`.
    pub fn is_same_reference(&self, other: &AisViewCubeShadingRef<'_>) -> bool {
        match (&self.slot, &other.slot) {
            (CubeShadingSlot::Local(left), CubeShadingSlot::Local(right)) => {
                std::ptr::eq(*left, *right)
            }
            (CubeShadingSlot::Drawer(left, _), CubeShadingSlot::Drawer(right, _)) => {
                left.object_address() == right.object_address()
            }
            _ => false,
        }
    }
    /// Invoke user code outside the slot lock, preserving storage without a native reference increment.
    /// Example: `alias.with_shading(|style| style.is_some())`.
    pub fn with_shading<R>(&self, inspect: impl FnOnce(Option<&Prs3dShadingAspect>) -> R) -> R {
        let current = self.current();
        inspect(current.get())
    }
    fn current(&self) -> HandleObserver<Prs3dShadingAspect> {
        match &self.slot {
            CubeShadingSlot::Local(slot) => slot.read().observe(),
            CubeShadingSlot::Drawer(drawer, _) => drawer.fields.read().shading_aspect.observe(),
        }
    }
}
fn cube_shading_slot_owner(drawer: &Handle<Prs3dDrawer>) -> HandleObserver<Prs3dDrawer> {
    let fields = drawer.fields.read();
    if !fields.shading_aspect.is_null() || fields.link.is_null() {
        return drawer.observe();
    }
    let link = fields.link.observe();
    drop(fields);
    cube_shading_slot_owner(&link)
}
