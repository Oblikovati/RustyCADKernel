//! OCCT 3d097a0 ViewAnimation returns the mutable handle slot, not a copied handle.
use super::{AisAnimationCamera, AisViewCubeInteractionFields};
use rustycad_tkernel::standard::{Handle, SharedState};
/// Borrow the live animation slot without retaining a lock or animation object.
/// Example: `let alias=cube.view_animation_ref(); cube.set_view_animation(&next); assert_eq!(alias.object_address(),next.object_address());`.
#[derive(Clone, Copy)]
pub struct AisViewCubeAnimationRef<'a> {
    fields: &'a SharedState<AisViewCubeInteractionFields>,
}
impl<'a> AisViewCubeAnimationRef<'a> {
    pub(super) fn new(fields: &'a SharedState<AisViewCubeInteractionFields>) -> Self {
        Self { fields }
    }
    /// Observe current null assignment, including later replacements; `alias.is_null()`.
    pub fn is_null(&self) -> bool {
        self.fields.read().animation.is_null()
    }
    /// Observe current target identity without owning it; `alias.object_address()`.
    pub fn object_address(&self) -> *const () {
        self.fields.read().animation.object_address()
    }
    /// Compare actual borrowed handle slots, including null; `left.is_same_reference(&right)`.
    pub fn is_same_reference(&self, other: &AisViewCubeAnimationRef<'_>) -> bool {
        std::ptr::eq(self.fields, other.fields)
    }
    /// Explicitly retain a snapshot of the current target; `let owning=alias.clone_handle()`.
    pub fn clone_handle(&self) -> Handle<AisAnimationCamera> {
        self.fields.read().animation.clone()
    }
    /// Access the current object outside its field lock; `alias.with_animation(|value| value.is_some())`.
    /// A temporary owning handle prevents concurrent replacement from invalidating the callback borrow.
    pub fn with_animation<R>(&self, inspect: impl FnOnce(Option<&AisAnimationCamera>) -> R) -> R {
        let current = self.clone_handle();
        inspect(current.get())
    }
}
