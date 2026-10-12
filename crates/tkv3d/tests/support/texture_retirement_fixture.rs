//! Genuine ImagePixMap Drop reborrows, not copied pixmap state or owning handle snapshots.
use rustycad_tkernel::{standard::*, tcollection::AsciiString};
use rustycad_tkservice::image::{ImageFormat, ImagePixMap};
use rustycad_tkv3d::ais::*;
use std::sync::Arc;
pub struct RetirementProjectionPixels {
    pub base: ImagePixMap,
    pub owner: WeakHandle<AisTexturedShape>,
    pub nested: WeakHandle<ImagePixMap>,
    pub mode: &'static str,
    pub trace: Arc<SharedState<Vec<String>>>,
}
rustycad_tkernel::impl_transient!(RetirementProjectionPixels, "RetirementProjectionPixels", ImagePixMap, base);
impl Drop for RetirementProjectionPixels {
    fn drop(&mut self) {
        let owner = self.owner.observe();
        let slot = owner.texture_pix_map();
        let row = slot.with_retiring_pointee(self, |pixels| {
            format!("POINTEE {} {} {} {} {} {}", self.mode,
                i32::from(slot.object_address() == std::ptr::from_ref(self).cast()),
                slot.ref_count(), pixels.width(), pixels.height(), pixels.data().data.data()[0])
        }).unwrap();
        self.trace.write().push(row);
        ais_textured_shape_with_retirement(&**owner, self, |scope| run_nested_retirement(&scope, self)).unwrap().unwrap();
        self.trace.write().push(retirement_nested_row(&owner, self));
    }
}
fn run_nested_retirement(
    scope: &AisTexturedShapeRetirement<'_, AisTexturedShape>,
    pixels: &RetirementProjectionPixels,
) -> Result<(), Failure> {
    match pixels.mode {
        "same" => scope.set_texture_pix_map(&pixels.owner.observe().texture_pix_map()),
        "file" => scope.set_texture_file_name(&AsciiString::from("inner.raw")),
        "pixmap" => scope.set_texture_pix_map(&*pixels.nested.observe()),
        "none" => Ok(()),
        offending => panic!("mode={offending}; expected none/same/file/pixmap"),
    }
}
fn retirement_nested_row(owner: &AisTexturedShape, pixels: &RetirementProjectionPixels) -> String {
    let slot = owner.texture_pix_map();
    let file = owner.texture_file().read().unwrap();
    format!("NESTED {} {} {} {} {} {}", pixels.mode,
        pixels.base.transient_base().ref_count(),
        i32::from(slot.object_address() == std::ptr::from_ref(pixels).cast()),
        i32::from(slot.is_null()), pixels.nested.observe().ref_count(), file.to_string_lossy())
        .trim_end().to_string()
}
pub fn retirement_projection_pixels(
    owner: &Handle<AisTexturedShape>,
    nested: &Handle<ImagePixMap>,
    mode: &'static str,
    trace: &Arc<SharedState<Vec<String>>>,
) -> Handle<ImagePixMap> {
    let base = ImagePixMap::default();
    assert!(base.init_trash(ImageFormat::Gray, 3, 2, 0));
    base.change_data().data.change_data()[0] = 73;
    Handle::new(RetirementProjectionPixels { base, owner: owner.downgrade(),
        nested: nested.downgrade(), mode, trace: trace.clone() }).into_cast()
}
