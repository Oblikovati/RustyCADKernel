//! Read-only live handle-field lens preserves native TexturePixMap const-reference aliases.
use rustycad_tkernel::standard::{Handle, SharedState};
use rustycad_tkservice::image::ImagePixMap;
use std::sync::RwLockReadGuard;
/// A borrowed native handle field, with no assignment capability; `object.texture_pix_map().read().is_null()`.
#[derive(Clone, Copy)]
pub struct AisTexturedShapePixMapRef<'a> {
    pixels: &'a SharedState<Handle<ImagePixMap>>,
}
impl<'a> AisTexturedShapePixMapRef<'a> {
    pub(super) fn new(pixels: &'a SharedState<Handle<ImagePixMap>>) -> Self {
        // Borrow the field so an earlier alias observes subsequent handle assignments.
        Self { pixels }
    }
    /// Read the current actual handle after source assignments; `let alias=object.texture_pix_map(); alias.read().object_address()`.
    pub fn read(&self) -> RwLockReadGuard<'a, Handle<ImagePixMap>> {
        // Hold native handle storage stable while the caller inspects its current identity.
        self.pixels.read()
    }
}
