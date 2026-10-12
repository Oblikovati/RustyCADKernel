//! Two genuine destructor levels publish a third real pixmap into the same borrowed field.
use rustycad_tkernel::{standard::*, tcollection::AsciiString};
use rustycad_tkservice::image::{ImageFormat, ImagePixMap};
use rustycad_tkv3d::ais::*;
use std::sync::Arc;
#[derive(Default)]
pub struct TextureChainControl {
    pub nested: Handle<ImagePixMap>,
    pub final_pixels: Handle<ImagePixMap>,
    pub last: WeakHandle<ImagePixMap>,
}
pub struct TextureChainPixels {
    base: ImagePixMap,
    owner: WeakHandle<AisTexturedShape>,
    control: Arc<SharedState<TextureChainControl>>,
    trace: Arc<SharedState<Vec<String>>>,
    stage: usize,
}
rustycad_tkernel::impl_transient!(TextureChainPixels, "TextureChainPixels", ImagePixMap, base);
impl Drop for TextureChainPixels {
    fn drop(&mut self) {
        let owner = self.owner.observe();
        let slot = owner.texture_pix_map();
        let row = slot.with_retiring_pointee(self, |pixels| {
            format!("POINTEE {} {} {} {} {} {}", self.stage,
                i32::from(slot.object_address() == std::ptr::from_ref(self).cast()),
                slot.ref_count(), pixels.width(), pixels.height(), pixels.data().data.data()[0])
        }).unwrap();
        self.trace.write().push(row);
        ais_textured_shape_with_retirement(&**owner, self, |scope| self.run_chain_stage(&scope)).unwrap();
        let count = self.control.read().last.observe().ref_count();
        let file = owner.texture_file().read().unwrap();
        self.trace.write().push(format!("CHAIN {} {} {} {count} {}", self.stage,
            self.base.transient_base().ref_count(), i32::from(slot.is_null()), file.to_string_lossy()).trim_end().to_string());
    }
}
impl TextureChainPixels {
    fn run_chain_stage(&self, scope: &AisTexturedShapeRetirement<'_, AisTexturedShape>) {
        if self.stage == 1 {
            self.release_nested_in_scope(scope);
            scope.set_texture_file_name(&AsciiString::from("inner.raw")).unwrap();
            return;
        }
        if self.stage == 2 {
            self.release_final_in_scope(scope);
        }
    }
    fn release_nested_in_scope(&self, scope: &AisTexturedShapeRetirement<'_, AisTexturedShape>) {
        let reference = HandleFieldRef::new(&self.control, chain_nested_field);
        scope.set_texture_pix_map(&reference).unwrap();
        let owner = std::mem::take(&mut self.control.write().nested);
        drop(owner);
    }
    fn release_final_in_scope(&self, scope: &AisTexturedShapeRetirement<'_, AisTexturedShape>) {
        let reference = HandleFieldRef::new(&self.control, chain_final_field);
        scope.set_texture_pix_map(&reference).unwrap();
        let owner = std::mem::take(&mut self.control.write().final_pixels);
        drop(owner);
    }
}
fn chain_nested_field(control: &TextureChainControl) -> &Handle<ImagePixMap> {
    // Retain the original member's address instead of creating an owning argument snapshot.
    &control.nested
}
fn chain_final_field(control: &TextureChainControl) -> &Handle<ImagePixMap> {
    // This actual member is nullified after native Assign has acquired its reference.
    &control.final_pixels
}
pub fn texture_chain_pixels(owner: &Handle<AisTexturedShape>, control: &Arc<SharedState<TextureChainControl>>, trace: &Arc<SharedState<Vec<String>>>, stage: usize) -> Handle<ImagePixMap> {
    let base = ImagePixMap::default();
    assert!(base.init_trash(ImageFormat::Gray, stage + 2, 2, 0));
    base.change_data().data.change_data()[0] = (73 + stage) as u8;
    Handle::new(TextureChainPixels { base, owner: owner.downgrade(),
        control: control.clone(), trace: trace.clone(), stage }).into_cast()
}
