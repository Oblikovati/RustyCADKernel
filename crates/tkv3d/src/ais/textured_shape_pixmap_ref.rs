//! Native field aliases never retain a lock or acquire pixmap ownership.
use rustycad_tkernel::standard::{Failure, Handle, HandleObserver, HandleReference, IntrusiveReferenceSlot, Transient};
use rustycad_tkservice::image::ImagePixMap;
/// Borrowed actual pointer slot; `object.texture_pix_map().object_address()`.
#[derive(Clone, Copy)]
pub struct AisTexturedShapePixMapRef<'a> {
    pixels: &'a IntrusiveReferenceSlot<ImagePixMap>,
}
impl<'a> AisTexturedShapePixMapRef<'a> {
    pub(super) fn new(pixels: &'a IntrusiveReferenceSlot<ImagePixMap>) -> Self {
        // Keep the original slot reference rather than a pointer-value snapshot.
        Self { pixels }
    }
    /// Compatibility inspection holds no guard; `object.texture_pix_map().read().is_null()`.
    pub fn read(&self) -> Self {
        // The returned alias continues observing the same actual field after callbacks.
        *self
    }
    /// Current pointer nullness; `object.texture_pix_map().is_null()`.
    pub fn is_null(&self) -> bool {
        let pixels = self.pixels;
        pixels.is_null()
    }
    /// Current pointer identity, including old-target destruction; `alias.object_address()`.
    pub fn object_address(&self) -> *const () {
        let pixels = self.pixels;
        pixels.object_address()
    }
    /// Current native count without retaining the target; `alias.ref_count()`.
    pub fn ref_count(&self) -> i32 {
        let pixels = self.pixels;
        pixels.ref_count()
    }
    /// Inspect the real destructor borrow; `alias.with_retiring_pointee(self, |p| p.width())?`.
    /// The original field identity is checked and no native owning count is acquired.
    pub fn with_retiring_pointee<R>(
        &self,
        retiring: &dyn Transient,
        inspect: impl FnOnce(&ImagePixMap) -> R,
    ) -> Result<R, Failure> {
        // Rust Drop already owns the valid reborrow; an expired Arc must not be resurrected.
        self.pixels.with_retiring_pointee(retiring, inspect)
    }
    /// Acquire ownership explicitly, independently of the native const field alias; `alias.clone_handle()`.
    pub fn clone_handle(&self) -> Handle<ImagePixMap> {
        let observed = self.pixels.observe();
        observed.clone()
    }
}
impl HandleReference<ImagePixMap> for AisTexturedShapePixMapRef<'_> {
    fn observe(&self) -> HandleObserver<ImagePixMap> {
        let pixels = self.pixels;
        pixels.observe()
    }
    fn reference_address(&self) -> *const () {
        let pixels = self.pixels;
        pixels.reference_address()
    }
    fn object_address(&self) -> *const () {
        let pixels = self.pixels;
        pixels.object_address()
    }
    fn is_null(&self) -> bool {
        let pixels = self.pixels;
        pixels.is_null()
    }
}
