//! `Graphic3d_Texture2D` (a 2D texture, also the base of the plane and media textures) and
//! `Graphic3d_Texture2Dplane` (coordinates generated from two object-space planes).
use super::texture_map::{predefined_base, predefined_file, predefined_texture_name};
use super::{
    Graphic3dNameOfTexture2D, Graphic3dNameOfTexturePlane, Graphic3dTextureMap,
    Graphic3dTextureRoot, Graphic3dTextureRootBase, Graphic3dTypeOfTexture,
    Graphic3dTypeOfTextureFilter, Graphic3dTypeOfTextureMode,
};
use crate::image::ImagePixMap;
use rustycad_tkernel::ncollection::{Vec2, Vec4};
use rustycad_tkernel::osd::CadPath;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// `NameOfTexture2d_to_FileName`.
const TEXTURE_2D_FILES: [&str; 22] = [
    "2d_MatraDatavision.rgb",
    "2d_alienskin.rgb",
    "2d_blue_rock.rgb",
    "2d_bluewhite_paper.rgb",
    "2d_brushed.rgb",
    "2d_bubbles.rgb",
    "2d_bumps.rgb",
    "2d_cast.rgb",
    "2d_chipbd.rgb",
    "2d_clouds.rgb",
    "2d_flesh.rgb",
    "2d_floor.rgb",
    "2d_galvnisd.rgb",
    "2d_grass.rgb",
    "2d_aluminum.rgb",
    "2d_rock.rgb",
    "2d_knurl.rgb",
    "2d_maple.rgb",
    "2d_marble.rgb",
    "2d_mottled.rgb",
    "2d_rain.rgb",
    "2d_chess.rgba",
];

/// The embedded state of a 2D texture: the root base and the predefined name.
pub struct Graphic3dTexture2DBase {
    /// The root base.
    pub root: Graphic3dTextureRootBase,
    /// `myName`.
    pub name: SharedState<Graphic3dNameOfTexture2D>,
}

impl Graphic3dTexture2DBase {
    fn with_root(root: Graphic3dTextureRootBase, name: Graphic3dNameOfTexture2D) -> Self {
        Self {
            root,
            name: SharedState::new(name),
        }
    }
    /// `Graphic3d_Texture2D(theFileName, theType)`; `Graphic3dTexture2DBase::from_file(&path, kind)`.
    pub fn from_file(file_name: &AsciiString, kind: Graphic3dTypeOfTexture) -> Self {
        Self::with_root(
            Graphic3dTextureRootBase::from_file(file_name, kind),
            Graphic3dNameOfTexture2D::Unknown,
        )
    }
    /// `Graphic3d_Texture2D(theName, theType)`: the predefined file in the textures folder;
    /// `Graphic3dTexture2DBase::from_name(Graphic3dNameOfTexture2D::Chess, kind)?`.
    pub fn from_name(
        name: Graphic3dNameOfTexture2D,
        kind: Graphic3dTypeOfTexture,
    ) -> Result<Self, Failure> {
        let file = predefined_file(&TEXTURE_2D_FILES, name as i32, "Graphic3d_Texture2D")?;
        Ok(Self::with_root(
            predefined_base(file, "Graphic3d_Texture2D", kind)?,
            name,
        ))
    }
    /// `Graphic3d_Texture2D(thePixMap, theType)`; `Graphic3dTexture2DBase::from_pix_map(pixmap, kind)`.
    pub fn from_pix_map(pix_map: Handle<ImagePixMap>, kind: Graphic3dTypeOfTexture) -> Self {
        Self::with_root(
            Graphic3dTextureRootBase::from_pix_map(pix_map, kind),
            Graphic3dNameOfTexture2D::Unknown,
        )
    }
    /// The body of the public constructors: mipmaps, modulate, repeat and trilinear filtering.
    pub fn with_public_defaults(self) -> Self {
        self.root.fields.write().has_mipmaps = true;
        self.root.params.set_modulate(true);
        self.root.params.set_repeat(true);
        self.root
            .params
            .set_filter(Graphic3dTypeOfTextureFilter::Trilinear);
        self
    }
    /// The number of predefined 2D textures; `Graphic3dTexture2DBase::number_of_textures() == 22`.
    pub fn number_of_textures() -> i32 {
        TEXTURE_2D_FILES.len() as i32
    }
    /// The name of the predefined texture of a rank; `texture_name(22)? == "chess"`.
    pub fn texture_name(rank: i32) -> Result<AsciiString, Failure> {
        predefined_texture_name(&TEXTURE_2D_FILES, rank, 3)
    }
}

/// A 2D texture class (`Graphic3d_Texture2D` and the classes derived from it).
pub trait Graphic3dTexture2DSource: Graphic3dTextureMap {
    /// The embedded 2D base; `texture.texture_2d_base()`.
    fn texture_2d_base(&self) -> &Graphic3dTexture2DBase;
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dTexture2DSource,
    "Graphic3d_Texture2D",
    dyn Graphic3dTextureMap
);

/// The non-virtual members of `Graphic3d_Texture2D`.
pub trait Graphic3dTexture2DMembers: Graphic3dTexture2DSource {
    /// The predefined name, `Unknown` otherwise; `texture.name()`.
    fn name(&self) -> Graphic3dNameOfTexture2D {
        *self.texture_2d_base().name.read()
    }
    /// Replaces the image by a pixmap and forgets the file and the name; `texture.set_image(pixmap)`.
    fn set_image(&self, pix_map: Handle<ImagePixMap>) {
        let base = self.texture_2d_base();
        let mut fields = base.root.fields.write();
        fields.pix_map = pix_map;
        fields.path = CadPath::default();
        *base.name.write() = Graphic3dNameOfTexture2D::Unknown;
    }
}
impl<T: Graphic3dTexture2DSource + ?Sized> Graphic3dTexture2DMembers for T {}

/// Implements the texture traits of a 2D texture struct with a `base: Graphic3dTexture2DBase`.
macro_rules! texture_2d_traits {
    ($ty:ty, $name:literal) => {
        impl Graphic3dTextureRoot for $ty {
            fn texture_root_base(&self) -> &Graphic3dTextureRootBase {
                &self.base.root
            }
        }
        impl Graphic3dTextureMap for $ty {}
        impl Graphic3dTexture2DSource for $ty {
            fn texture_2d_base(&self) -> &Graphic3dTexture2DBase {
                &self.base
            }
        }
        rustycad_tkernel::impl_transient!($ty, $name, dyn Graphic3dTexture2DSource, transient,
            interfaces: [dyn Graphic3dTextureRoot, dyn Graphic3dTextureMap, dyn Graphic3dTexture2DSource]);
    };
}

/// The native `Graphic3d_Texture2D`; `Graphic3dTexture2D::from_file(&"wood.png".into())`.
pub struct Graphic3dTexture2D {
    transient: StandardTransient,
    base: Graphic3dTexture2DBase,
}
texture_2d_traits!(Graphic3dTexture2D, "Graphic3d_Texture2D");

impl Graphic3dTexture2D {
    /// Wraps a base built by a constructor; `Graphic3dTexture2D::with_base(base)`.
    pub fn with_base(base: Graphic3dTexture2DBase) -> Self {
        Self {
            transient: StandardTransient::default(),
            base,
        }
    }
    /// A file texture with mipmaps, modulation, repeat and trilinear filtering.
    /// Example: `Graphic3dTexture2D::from_file(&"wood.png".into())`.
    pub fn from_file(file_name: &AsciiString) -> Self {
        Self::with_base(
            Graphic3dTexture2DBase::from_file(file_name, Graphic3dTypeOfTexture::TOT_2D)
                .with_public_defaults(),
        )
    }
    /// A predefined texture with the public defaults; `Graphic3dTexture2D::from_name(Graphic3dNameOfTexture2D::Chess)?`.
    pub fn from_name(name: Graphic3dNameOfTexture2D) -> Result<Self, Failure> {
        Ok(Self::with_base(
            Graphic3dTexture2DBase::from_name(name, Graphic3dTypeOfTexture::TOT_2D)?
                .with_public_defaults(),
        ))
    }
    /// A pixmap texture with the public defaults; `Graphic3dTexture2D::from_pix_map(pixmap)`.
    pub fn from_pix_map(pix_map: Handle<ImagePixMap>) -> Self {
        Self::with_base(
            Graphic3dTexture2DBase::from_pix_map(pix_map, Graphic3dTypeOfTexture::TOT_2D)
                .with_public_defaults(),
        )
    }
}

/// A 2D texture whose coordinates come from two object-space planes (XY by default).
///
/// Example: `let t = Graphic3dTexture2Dplane::from_file(&"wood.png".into()); t.set_plane(Graphic3dNameOfTexturePlane::Yz);`.
pub struct Graphic3dTexture2Dplane {
    transient: StandardTransient,
    base: Graphic3dTexture2DBase,
    plane_name: SharedState<Graphic3dNameOfTexturePlane>,
}
texture_2d_traits!(Graphic3dTexture2Dplane, "Graphic3d_Texture2Dplane");

/// The S and T planes of a predefined plane.
fn predefined_planes(plane: Graphic3dNameOfTexturePlane) -> Option<(Vec4<f32>, Vec4<f32>)> {
    let (x, y, z) = (
        Vec4::new(1.0, 0.0, 0.0, 0.0),
        Vec4::new(0.0, 1.0, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 1.0, 0.0),
    );
    match plane {
        Graphic3dNameOfTexturePlane::Xy => Some((x, y)),
        Graphic3dNameOfTexturePlane::Yz => Some((y, z)),
        Graphic3dNameOfTexturePlane::Zx => Some((z, x)),
        Graphic3dNameOfTexturePlane::Unknown => None,
    }
}

impl Graphic3dTexture2Dplane {
    /// Object generation on the XY planes over a public 2D base. Native quirk: the plane name
    /// is left uninitialised by the constructors; here it starts as `Unknown`.
    fn wrap(base: Graphic3dTexture2DBase) -> Self {
        let texture = Self {
            transient: StandardTransient::default(),
            base,
            plane_name: SharedState::new(Graphic3dNameOfTexturePlane::Unknown),
        };
        texture.set_planes(Vec4::new(1.0, 0.0, 0.0, 0.0), Vec4::new(0.0, 1.0, 0.0, 0.0));
        texture
    }
    /// A file texture; `Graphic3dTexture2Dplane::from_file(&"wood.png".into())`.
    pub fn from_file(file_name: &AsciiString) -> Self {
        Self::wrap(
            Graphic3dTexture2DBase::from_file(file_name, Graphic3dTypeOfTexture::TOT_2D)
                .with_public_defaults(),
        )
    }
    /// A predefined texture; `Graphic3dTexture2Dplane::from_name(Graphic3dNameOfTexture2D::Chess)?`.
    pub fn from_name(name: Graphic3dNameOfTexture2D) -> Result<Self, Failure> {
        Ok(Self::wrap(
            Graphic3dTexture2DBase::from_name(name, Graphic3dTypeOfTexture::TOT_2D)?
                .with_public_defaults(),
        ))
    }
    /// A pixmap texture; `Graphic3dTexture2Dplane::from_pix_map(pixmap)`.
    pub fn from_pix_map(pix_map: Handle<ImagePixMap>) -> Self {
        Self::wrap(
            Graphic3dTexture2DBase::from_pix_map(pix_map, Graphic3dTypeOfTexture::TOT_2D)
                .with_public_defaults(),
        )
    }
    fn set_planes(&self, plane_s: Vec4<f32>, plane_t: Vec4<f32>) {
        self.base
            .root
            .params
            .set_gen_mode(Graphic3dTypeOfTextureMode::Object, plane_s, plane_t);
    }
    /// Sets the plane S `a x + b y + c z + d`; the plane name becomes `Unknown`. `t.set_plane_s(1.0, 0.0, 0.0, 0.0)`.
    pub fn set_plane_s(&self, a: f32, b: f32, c: f32, d: f32) {
        self.set_planes(Vec4::new(a, b, c, d), self.base.root.params.gen_plane_t());
        *self.plane_name.write() = Graphic3dNameOfTexturePlane::Unknown;
    }
    /// Sets the plane T; the plane name becomes `Unknown`. `t.set_plane_t(0.0, 1.0, 0.0, 0.0)`.
    pub fn set_plane_t(&self, a: f32, b: f32, c: f32, d: f32) {
        self.set_planes(self.base.root.params.gen_plane_s(), Vec4::new(a, b, c, d));
        *self.plane_name.write() = Graphic3dNameOfTexturePlane::Unknown;
    }
    /// Sets both planes from a predefined plane (`Unknown` keeps them); `t.set_plane(Graphic3dNameOfTexturePlane::Zx)`.
    pub fn set_plane(&self, plane: Graphic3dNameOfTexturePlane) {
        if let Some((plane_s, plane_t)) = predefined_planes(plane) {
            self.set_planes(plane_s, plane_t);
        }
        *self.plane_name.write() = plane;
    }
    fn update_scale(&self, update: impl FnOnce(&mut Vec2<f32>)) {
        let mut scale = self.base.root.params.scale();
        update(&mut scale);
        self.base.root.params.set_scale(scale);
    }
    fn update_translation(&self, update: impl FnOnce(&mut Vec2<f32>)) {
        let mut translation = self.base.root.params.translation();
        update(&mut translation);
        self.base.root.params.set_translation(translation);
    }
    /// Sets the S scale; `t.set_scale_s(2.0)`.
    pub fn set_scale_s(&self, value: f32) {
        self.update_scale(|scale| scale.x = value);
    }
    /// Sets the T scale; `t.set_scale_t(2.0)`.
    pub fn set_scale_t(&self, value: f32) {
        self.update_scale(|scale| scale.y = value);
    }
    /// Sets the S translation; `t.set_translate_s(0.5)`.
    pub fn set_translate_s(&self, value: f32) {
        self.update_translation(|translation| translation.x = value);
    }
    /// Sets the T translation; `t.set_translate_t(0.5)`.
    pub fn set_translate_t(&self, value: f32) {
        self.update_translation(|translation| translation.y = value);
    }
    /// Sets the rotation in degrees; `t.set_rotation(30.0)`.
    pub fn set_rotation(&self, angle_degrees: f32) {
        self.base.root.params.set_rotation(angle_degrees);
    }
}

/// The plane getters of `Graphic3d_Texture2Dplane` (the native out-parameters are returned).
impl Graphic3dTexture2Dplane {
    /// `PlaneS`: `[a, b, c, d]`; `t.plane_s()`.
    pub fn plane_s(&self) -> [f32; 4] {
        let plane = self.base.root.params.gen_plane_s();
        [plane.x, plane.y, plane.z, plane.w]
    }
    /// `PlaneT`: `[a, b, c, d]`; `t.plane_t()`.
    pub fn plane_t(&self) -> [f32; 4] {
        let plane = self.base.root.params.gen_plane_t();
        [plane.x, plane.y, plane.z, plane.w]
    }
    /// The S translation; `t.translate_s()`.
    pub fn translate_s(&self) -> f32 {
        self.base.root.params.translation().x
    }
    /// The T translation; `t.translate_t()`.
    pub fn translate_t(&self) -> f32 {
        self.base.root.params.translation().y
    }
    /// The rotation in degrees; `t.rotation()`.
    pub fn rotation(&self) -> f32 {
        self.base.root.params.rotation()
    }
    /// The predefined plane, `Unknown` after a custom plane; `t.plane()`.
    pub fn plane(&self) -> Graphic3dNameOfTexturePlane {
        *self.plane_name.read()
    }
    /// The S scale; `t.scale_s()`.
    pub fn scale_s(&self) -> f32 {
        self.base.root.params.scale().x
    }
    /// The T scale; `t.scale_t()`.
    pub fn scale_t(&self) -> f32 {
        self.base.root.params.scale().y
    }
}
