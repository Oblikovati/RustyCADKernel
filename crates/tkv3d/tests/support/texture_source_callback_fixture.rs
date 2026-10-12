//! The actual descendant's destructor inspects an actual source slot, without owning either source.
use rustycad_tkernel::{standard::*, tcollection::AsciiString};
use rustycad_tkservice::{graphic3d::Graphic3dNameOfTexture2D, image::ImagePixMap};
use rustycad_tkv3d::ais::*;
use std::sync::Arc;
pub struct TextureSlotCallbackPixels {
    pub base: ImagePixMap,
    pub owner: WeakHandle<AisTexturedShape>,
    pub incoming: WeakHandle<ImagePixMap>,
    pub label: String,
    pub trace: Arc<SharedState<Vec<String>>>,
    pub callback: Option<Box<dyn Fn() + Send + Sync>>,
}
rustycad_tkernel::impl_transient!(
    TextureSlotCallbackPixels,
    "NativeTextureSlotPixels",
    ImagePixMap,
    base
);
impl Drop for TextureSlotCallbackPixels {
    fn drop(&mut self) {
        if let Some(callback) = &self.callback {
            callback();
        }
        let owner = self.owner.observe();
        let slot = owner.texture_pix_map();
        let same = slot.object_address() == std::ptr::from_ref(self).cast();
        let count = self.base.transient_base().ref_count();
        let incoming = incoming_texture_count(&self.incoming);
        let prefix = format!(
            "MARTENSITE_TEXTURE_SLOT {} {} {} {count} {incoming}",
            self.label,
            i32::from(same),
            i32::from(slot.is_null())
        );
        self.trace
            .write()
            .push(texture_state_suffix(&owner, prefix));
    }
}
fn incoming_texture_count(incoming: &WeakHandle<ImagePixMap>) -> i32 {
    if incoming.is_null() {
        return -1;
    }
    incoming.observe().ref_count()
}
pub fn texture_state_suffix(owner: &AisTexturedShape, prefix: String) -> String {
    let predefined = owner.textured_shape_base().fields.read().predefined_texture as i32;
    let file = owner.texture_file().read().unwrap();
    format!("{prefix} {predefined} {}", file.to_string_lossy())
        .trim_end()
        .to_string()
}
pub fn seed_callback_texture_file(owner: &AisTexturedShape) {
    let mut fields = owner.textured_shape_base().fields.write();
    fields.texture_file = AsciiString::from("prior.raw");
    fields.predefined_texture = Graphic3dNameOfTexture2D::Matra;
}
pub fn callback_texture_pixels(
    owner: &Handle<AisTexturedShape>,
    incoming: &Handle<ImagePixMap>,
    label: &str,
    trace: &Arc<SharedState<Vec<String>>>,
) -> Handle<ImagePixMap> {
    Handle::new(TextureSlotCallbackPixels {
        base: ImagePixMap::default(),
        owner: owner.downgrade(),
        incoming: incoming.downgrade(),
        label: label.to_string(),
        trace: trace.clone(),
        callback: None,
    })
    .into_cast()
}
