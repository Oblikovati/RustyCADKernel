//! `Graphic3d_TextureEnv`: an environment (sphere-mapped) texture of a view.
use super::texture_map::{predefined_base, predefined_file, predefined_texture_name};
use super::{
    Graphic3dNameOfTextureEnv, Graphic3dTextureRoot, Graphic3dTextureRootBase,
    Graphic3dTypeOfTexture, Graphic3dTypeOfTextureFilter, Graphic3dTypeOfTextureMode,
};
use crate::image::ImagePixMap;
use rustycad_tkernel::ncollection::Vec4;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// `NameOfTextureEnv_to_FileName`.
const TEXTURE_ENV_FILES: [&str; 8] = [
    "env_clouds.rgb",
    "env_cv.rgb",
    "env_medit.rgb",
    "env_pearl.rgb",
    "env_sky1.rgb",
    "env_sky2.rgb",
    "env_lines.rgb",
    "env_road.rgb",
];

/// An environment texture with mipmaps, trilinear filtering and sphere coordinates.
///
/// Example: `Graphic3dTextureEnv::from_file(&"sky.png".into())`.
pub struct Graphic3dTextureEnv {
    transient: StandardTransient,
    base: Graphic3dTextureRootBase,
    name: Graphic3dNameOfTextureEnv,
}
impl Graphic3dTextureRoot for Graphic3dTextureEnv {
    fn texture_root_base(&self) -> &Graphic3dTextureRootBase {
        &self.base
    }
}
rustycad_tkernel::impl_transient!(Graphic3dTextureEnv, "Graphic3d_TextureEnv", dyn Graphic3dTextureRoot, transient,
    interfaces: [dyn Graphic3dTextureRoot]);

impl Graphic3dTextureEnv {
    /// Mipmaps, trilinear filtering and sphere generation (the constructors' body).
    fn wrap(base: Graphic3dTextureRootBase, name: Graphic3dNameOfTextureEnv) -> Self {
        base.fields.write().has_mipmaps = true;
        base.params
            .set_filter(Graphic3dTypeOfTextureFilter::Trilinear);
        let (plane_s, plane_t) = (Vec4::new(1.0, 0.0, 0.0, 0.0), Vec4::new(0.0, 1.0, 0.0, 0.0));
        base.params
            .set_gen_mode(Graphic3dTypeOfTextureMode::Sphere, plane_s, plane_t);
        Self {
            transient: StandardTransient::default(),
            base,
            name,
        }
    }
    /// A file texture; `Graphic3dTextureEnv::from_file(&"sky.png".into())`.
    pub fn from_file(file_name: &AsciiString) -> Self {
        let base = Graphic3dTextureRootBase::from_file(file_name, Graphic3dTypeOfTexture::TOT_2D);
        Self::wrap(base, Graphic3dNameOfTextureEnv::Unknown)
    }
    /// A predefined texture of the textures folder; `Graphic3dTextureEnv::from_name(Graphic3dNameOfTextureEnv::Sky1)?`.
    pub fn from_name(name: Graphic3dNameOfTextureEnv) -> Result<Self, Failure> {
        let file = predefined_file(&TEXTURE_ENV_FILES, name as i32, "Graphic3d_TextureEnv")?;
        let base = predefined_base(file, "Graphic3d_TextureEnv", Graphic3dTypeOfTexture::TOT_2D)?;
        Ok(Self::wrap(base, name))
    }
    /// A pixmap texture; `Graphic3dTextureEnv::from_pix_map(pixmap)`.
    pub fn from_pix_map(pix_map: Handle<ImagePixMap>) -> Self {
        let base = Graphic3dTextureRootBase::from_pix_map(pix_map, Graphic3dTypeOfTexture::TOT_2D);
        Self::wrap(base, Graphic3dNameOfTextureEnv::Unknown)
    }
    /// The predefined name, `Unknown` otherwise; `texture.name()`.
    pub fn name(&self) -> Graphic3dNameOfTextureEnv {
        self.name
    }
    /// The number of predefined environment textures; `Graphic3dTextureEnv::number_of_textures() == 8`.
    pub fn number_of_textures() -> i32 {
        TEXTURE_ENV_FILES.len() as i32
    }
    /// The name of the predefined texture of a rank; `Graphic3dTextureEnv::texture_name(1)? == "clouds"`.
    pub fn texture_name(rank: i32) -> Result<AsciiString, Failure> {
        predefined_texture_name(&TEXTURE_ENV_FILES, rank, 4)
    }
}
