//! Genuine constructors with an injected availability fake avoid native file lookup in replay.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::ais::*;
use std::sync::atomic::{AtomicBool, Ordering};
#[derive(Default)]
pub struct TexturedAllocationFactory {
    done: AtomicBool,
}
impl AisTexturedTextureFactory for TexturedAllocationFactory {
    fn create(
        &self,
        source: AisTexturedTextureSource,
    ) -> Result<Handle<dyn Graphic3dTexture2DSource>, Failure> {
        let done =
            matches!(&source, AisTexturedTextureSource::PixMap(pixels) if !pixels.is_empty());
        self.done.store(done, Ordering::Relaxed);
        AisNativeTexturedTextureFactory.create(source)
    }
    fn is_done(&self, _: &Handle<dyn Graphic3dTexture2DSource>) -> bool {
        self.done.load(Ordering::Relaxed)
    }
}
