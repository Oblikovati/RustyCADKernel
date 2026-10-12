//! Owned native constructor/filesystem boundary; source assignment never invents image codecs.
use rustycad_tkernel::{
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tkservice::{graphic3d::*, image::ImagePixMap};

/// Native priority-selected source; `AisTexturedTextureSource::PixMap(pixels.clone())` retains identity.
pub enum AisTexturedTextureSource {
    /// Non-null custom image wins over predefined/file sources.
    PixMap(Handle<ImagePixMap>),
    /// A native predefined name, resolved by Graphic3d_Texture2D.
    Predefined(Graphic3dNameOfTexture2D),
    /// Native path, including the empty failed-source case.
    File(AsciiString),
}
/// Inject actual texture construction and IsDone I/O; `factory.create(source)?` can raise native failure.
pub trait AisTexturedTextureFactory: Send + Sync {
    /// Build a fresh real texture; `factory.create(AisTexturedTextureSource::File(path))?`.
    fn create(
        &self,
        source: AisTexturedTextureSource,
    ) -> Result<Handle<dyn Graphic3dTexture2DSource>, Failure>;
    /// Test native pixmap/file availability without decoding; `factory.is_done(&texture)`.
    fn is_done(&self, texture: &Handle<dyn Graphic3dTexture2DSource>) -> bool;
}
/// Production adapter for existing Graphic3d constructors; `AisNativeTexturedTextureFactory.create(source)?`.
pub struct AisNativeTexturedTextureFactory;
impl AisTexturedTextureFactory for AisNativeTexturedTextureFactory {
    fn create(
        &self,
        source: AisTexturedTextureSource,
    ) -> Result<Handle<dyn Graphic3dTexture2DSource>, Failure> {
        let texture = match source {
            AisTexturedTextureSource::PixMap(pixels) => Graphic3dTexture2D::from_pix_map(pixels),
            AisTexturedTextureSource::Predefined(name) => Graphic3dTexture2D::from_name(name)?,
            AisTexturedTextureSource::File(path) => Graphic3dTexture2D::from_file(&path),
        };
        Ok(Handle::new(texture).cast())
    }
    fn is_done(&self, texture: &Handle<dyn Graphic3dTexture2DSource>) -> bool {
        let Some(texture) = texture.get() else {
            return false;
        };
        texture.is_done()
    }
}
