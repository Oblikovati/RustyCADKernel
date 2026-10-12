//! Named image dependency records native forwarding without filesystem decoding.
use rustycad_tkernel::{
    standard::{Failure, Handle, SharedState, StandardTransient, raise_native_failure},
    tcollection::AsciiString,
};
use rustycad_tkservice::image::*;
pub(super) struct XrImageSourceFake {
    transient: StandardTransient,
    base: ImageTextureBase,
    pub calls: SharedState<i32>,
    pub fail: SharedState<bool>,
    pub same: SharedState<bool>,
    pub expected: Handle<ImageSupportedFormats>,
    pub image: Handle<ImagePixMap>,
}
rustycad_tkernel::impl_transient!(XrImageSourceFake,"Martensite_XrImageSourceFake",dyn ImageTextureSource,transient,interfaces:[dyn ImageTextureSource]);
impl XrImageSourceFake {
    pub fn new(kind: i32) -> Self {
        let mut base = ImageTextureBase::from_file(&AsciiString::default());
        if kind != 0 {
            base.texture_id = AsciiString::from("xr/oracle-source");
        }
        Self {
            transient: StandardTransient::default(),
            base,
            calls: SharedState::new(0),
            fail: SharedState::new(false),
            same: SharedState::new(false),
            expected: Handle::new(ImageSupportedFormats::default()),
            image: Handle::new(ImagePixMap::default()),
        }
    }
}
impl ImageTextureSource for XrImageSourceFake {
    fn texture_base(&self) -> &ImageTextureBase {
        // The wrapper snapshots this ID while retaining the actual image source.
        &self.base
    }
    fn read_image(&self, formats: &Handle<ImageSupportedFormats>) -> Handle<ImagePixMap> {
        *self.calls.write() += 1;
        *self.same.write() = formats == &self.expected;
        if *self.fail.read() {
            raise_native_failure::<()>(Err(Failure::new(
                "XR image fail=true; expected delegated image read",
            )));
        }
        self.image.clone()
    }
}
