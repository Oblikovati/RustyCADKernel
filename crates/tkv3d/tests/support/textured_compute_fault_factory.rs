//! Named I/O adapter distinguishes checked failures, native transport, and unrelated panics.
#[path = "textured_texture_factory_fixture.rs"]
mod texture_factory;
use rustycad_tkernel::standard::{Failure, Handle, raise_native_failure};
use rustycad_tkservice::graphic3d::Graphic3dTexture2DSource;
use rustycad_tkv3d::ais::*;
#[derive(Clone, Copy, PartialEq)]
pub enum TexturedComputeFault {
    CheckedConstructor,
    NativeConstructor,
    NativeAvailability,
    UnrelatedAvailability,
}
pub struct TexturedComputeFaultFactory {
    fault: TexturedComputeFault,
    delegate: texture_factory::TexturedTextureFactoryFixture,
}
impl TexturedComputeFaultFactory {
    pub fn new(fault: TexturedComputeFault) -> Self {
        let delegate = texture_factory::TexturedTextureFactoryFixture::new(
            fault == TexturedComputeFault::CheckedConstructor,
            true,
        );
        Self { fault, delegate }
    }
}
impl AisTexturedTextureFactory for TexturedComputeFaultFactory {
    fn create(
        &self,
        source: AisTexturedTextureSource,
    ) -> Result<Handle<dyn Graphic3dTexture2DSource>, Failure> {
        if self.fault == TexturedComputeFault::NativeConstructor {
            return raise_native_failure(Err(Failure::new(
                "injected native constructor failure; expected an available texture source",
            )));
        }
        self.delegate.create(source)
    }
    fn is_done(&self, texture: &Handle<dyn Graphic3dTexture2DSource>) -> bool {
        if self.fault == TexturedComputeFault::UnrelatedAvailability {
            std::panic::panic_any(73_u32);
        }
        if self.fault == TexturedComputeFault::NativeAvailability {
            return raise_native_failure(Err(Failure::new(
                "injected native availability failure; expected an available texture source",
            )));
        }
        self.delegate.is_done(texture)
    }
}
