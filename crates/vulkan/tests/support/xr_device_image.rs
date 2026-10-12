//! Named no-I/O source preserves the actual XR texture delegation in rendering fixtures.
use rustycad_tkernel::{
    standard::{Handle, StandardTransient},
    tcollection::AsciiString,
};
use rustycad_tkservice::image::*;
pub struct XrCaptureImage {
    transient: StandardTransient,
    base: ImageTextureBase,
    image: Handle<ImagePixMap>,
}
rustycad_tkernel::impl_transient!(XrCaptureImage,"Martensite_XrCaptureImage",dyn ImageTextureSource,transient,interfaces:[dyn ImageTextureSource]);
impl XrCaptureImage {
    /// Supply deterministic pixels without a decoder; `XrCaptureImage::new()`.
    pub fn new() -> Self {
        let image = Handle::new(ImagePixMap::default());
        assert!(image.init_wrapper(
            ImageFormat::Rgba,
            Some(vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255
            ]),
            2,
            2,
            8
        ));
        image.set_top_down(true);
        Self {
            transient: StandardTransient::default(),
            base: ImageTextureBase::from_file(&AsciiString::from("Martensite-XR-capture")),
            image,
        }
    }
}
impl ImageTextureSource for XrCaptureImage {
    fn texture_base(&self) -> &ImageTextureBase {
        // The constructor snapshots the native source identity and keeps this source alive.
        &self.base
    }
    fn read_image(&self, _formats: &Handle<ImageSupportedFormats>) -> Handle<ImagePixMap> {
        // No external decoding should affect the renderer differential fixture.
        self.image.clone()
    }
}
