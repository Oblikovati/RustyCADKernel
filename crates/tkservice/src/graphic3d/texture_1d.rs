//! `Graphic3d_Texture1D` and its classes `Graphic3d_Texture1Dmanual` (manual coordinates)
//! and `Graphic3d_Texture1Dsegment` (coordinates generated along a segment).
use super::texture_map::{predefined_base, predefined_file, predefined_texture_name};
use super::{
    Graphic3dNameOfTexture1D, Graphic3dTextureMap, Graphic3dTextureRoot, Graphic3dTextureRootBase,
    Graphic3dTypeOfTexture, Graphic3dTypeOfTextureMode,
};
use crate::image::ImagePixMap;
use rustycad_tkernel::ncollection::Vec4;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// `NameOfTexture1d_to_FileName`.
const TEXTURE_1D_FILES: [&str; 1] = ["1d_elevation.rgb"];

/// The embedded state of a 1D texture: the root base and the predefined name.
pub struct Graphic3dTexture1DBase {
    /// The root base.
    pub root: Graphic3dTextureRootBase,
    name: Graphic3dNameOfTexture1D,
}

impl Graphic3dTexture1DBase {
    /// `Graphic3d_Texture1D(theFileName, theType)`; `Graphic3dTexture1DBase::from_file(&path, kind)`.
    pub fn from_file(file_name: &AsciiString, kind: Graphic3dTypeOfTexture) -> Self {
        Self {
            root: Graphic3dTextureRootBase::from_file(file_name, kind),
            name: Graphic3dNameOfTexture1D::Unknown,
        }
    }
    /// `Graphic3d_Texture1D(theName, theType)`: the predefined file in the textures folder;
    /// `Graphic3dTexture1DBase::from_name(Graphic3dNameOfTexture1D::Elevation, kind)?`.
    pub fn from_name(
        name: Graphic3dNameOfTexture1D,
        kind: Graphic3dTypeOfTexture,
    ) -> Result<Self, Failure> {
        let file = predefined_file(&TEXTURE_1D_FILES, name as i32, "Graphic3d_Texture1D")?;
        Ok(Self {
            root: predefined_base(file, "Graphic3d_Texture1D", kind)?,
            name,
        })
    }
    /// `Graphic3d_Texture1D(thePixMap, theType)`; `Graphic3dTexture1DBase::from_pix_map(pixmap, kind)`.
    pub fn from_pix_map(pix_map: Handle<ImagePixMap>, kind: Graphic3dTypeOfTexture) -> Self {
        Self {
            root: Graphic3dTextureRootBase::from_pix_map(pix_map, kind),
            name: Graphic3dNameOfTexture1D::Unknown,
        }
    }
    /// The number of predefined 1D textures; `Graphic3dTexture1DBase::number_of_textures() == 1`.
    pub fn number_of_textures() -> i32 {
        TEXTURE_1D_FILES.len() as i32
    }
    /// The name of the predefined texture of a rank; `texture_name(1)? == "elevation"`.
    pub fn texture_name(rank: i32) -> Result<AsciiString, Failure> {
        predefined_texture_name(&TEXTURE_1D_FILES, rank, 3)
    }
}

/// A 1D texture (`Graphic3d_Texture1D`, protected constructors).
pub trait Graphic3dTexture1D: Graphic3dTextureMap {
    /// The embedded 1D base; `texture.texture_1d_base()`.
    fn texture_1d_base(&self) -> &Graphic3dTexture1DBase;
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dTexture1D,
    "Graphic3d_Texture1D",
    dyn Graphic3dTextureMap
);

/// The non-virtual members of `Graphic3d_Texture1D`.
pub trait Graphic3dTexture1DMembers: Graphic3dTexture1D {
    /// The predefined name, `Unknown` for a file or pixmap texture; `texture.name()`.
    fn name(&self) -> Graphic3dNameOfTexture1D {
        self.texture_1d_base().name
    }
}
impl<T: Graphic3dTexture1D + ?Sized> Graphic3dTexture1DMembers for T {}

/// Implements the texture traits of a 1D texture struct with a `base: Graphic3dTexture1DBase`.
macro_rules! texture_1d_traits {
    ($ty:ty, $name:literal) => {
        impl Graphic3dTextureRoot for $ty {
            fn texture_root_base(&self) -> &Graphic3dTextureRootBase {
                &self.base.root
            }
        }
        impl Graphic3dTextureMap for $ty {}
        impl Graphic3dTexture1D for $ty {
            fn texture_1d_base(&self) -> &Graphic3dTexture1DBase {
                &self.base
            }
        }
        rustycad_tkernel::impl_transient!($ty, $name, dyn Graphic3dTexture1D, transient,
            interfaces: [dyn Graphic3dTextureRoot, dyn Graphic3dTextureMap, dyn Graphic3dTexture1D]);
    };
}

/// A 1D texture with manual coordinates; `Graphic3dTexture1Dmanual::from_file(&"ramp.png".into())`.
pub struct Graphic3dTexture1Dmanual {
    transient: StandardTransient,
    base: Graphic3dTexture1DBase,
}
texture_1d_traits!(Graphic3dTexture1Dmanual, "Graphic3d_Texture1Dmanual");

impl Graphic3dTexture1Dmanual {
    fn wrap(base: Graphic3dTexture1DBase) -> Self {
        Self {
            transient: StandardTransient::default(),
            base,
        }
    }
    /// A file texture; `Graphic3dTexture1Dmanual::from_file(&"ramp.png".into())`.
    pub fn from_file(file_name: &AsciiString) -> Self {
        Self::wrap(Graphic3dTexture1DBase::from_file(
            file_name,
            Graphic3dTypeOfTexture::TOT_1D,
        ))
    }
    /// A predefined texture; `Graphic3dTexture1Dmanual::from_name(Graphic3dNameOfTexture1D::Elevation)?`.
    pub fn from_name(name: Graphic3dNameOfTexture1D) -> Result<Self, Failure> {
        Ok(Self::wrap(Graphic3dTexture1DBase::from_name(
            name,
            Graphic3dTypeOfTexture::TOT_1D,
        )?))
    }
    /// A pixmap texture; `Graphic3dTexture1Dmanual::from_pix_map(pixmap)`.
    pub fn from_pix_map(pix_map: Handle<ImagePixMap>) -> Self {
        Self::wrap(Graphic3dTexture1DBase::from_pix_map(
            pix_map,
            Graphic3dTypeOfTexture::TOT_1D,
        ))
    }
}

/// A 1D texture whose coordinate runs along a segment (object-space generation).
///
/// Example: `let t = Graphic3dTexture1Dsegment::from_file(&"ramp.png".into()); t.set_segment(0.0, 0.0, 0.0, 0.0, 0.0, 10.0);`.
pub struct Graphic3dTexture1Dsegment {
    transient: StandardTransient,
    base: Graphic3dTexture1DBase,
    segment: SharedState<[f32; 6]>,
}
texture_1d_traits!(Graphic3dTexture1Dsegment, "Graphic3d_Texture1Dsegment");

impl Graphic3dTexture1Dsegment {
    /// Repeat on, object generation along Z, a null segment (the three constructors' body).
    fn wrap(base: Graphic3dTexture1DBase) -> Self {
        base.root.params.set_repeat(true);
        let (plane_s, plane_t) = (Vec4::new(0.0, 0.0, 1.0, 0.0), Vec4::new(0.0, 0.0, 0.0, 0.0));
        base.root
            .params
            .set_gen_mode(Graphic3dTypeOfTextureMode::Object, plane_s, plane_t);
        Self {
            transient: StandardTransient::default(),
            base,
            segment: SharedState::new([0.0; 6]),
        }
    }
    /// A file texture; `Graphic3dTexture1Dsegment::from_file(&"ramp.png".into())`.
    pub fn from_file(file_name: &AsciiString) -> Self {
        Self::wrap(Graphic3dTexture1DBase::from_file(
            file_name,
            Graphic3dTypeOfTexture::TOT_1D,
        ))
    }
    /// A predefined texture; `Graphic3dTexture1Dsegment::from_name(Graphic3dNameOfTexture1D::Elevation)?`.
    pub fn from_name(name: Graphic3dNameOfTexture1D) -> Result<Self, Failure> {
        Ok(Self::wrap(Graphic3dTexture1DBase::from_name(
            name,
            Graphic3dTypeOfTexture::TOT_1D,
        )?))
    }
    /// A pixmap texture; `Graphic3dTexture1Dsegment::from_pix_map(pixmap)`.
    pub fn from_pix_map(pix_map: Handle<ImagePixMap>) -> Self {
        Self::wrap(Graphic3dTexture1DBase::from_pix_map(
            pix_map,
            Graphic3dTypeOfTexture::TOT_1D,
        ))
    }
    /// Sets the segment and the generation plane S `(d / |d|^2, -d.p1 / |d|^2)` in float, as
    /// natively (a null segment divides by zero). Example: `t.set_segment(0.0, 0.0, 0.0, 0.0, 0.0, 10.0)`.
    pub fn set_segment(&self, x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32) {
        *self.segment.write() = [x1, y1, z1, x2, y2, z2];
        let (mut x, mut y, mut z) = (x2 - x1, y2 - y1, z2 - z1);
        let square_norm = x * x + y * y + z * z;
        (x, y, z) = (x / square_norm, y / square_norm, z / square_norm);
        let w = -x * x1 - y * y1 - z * z1;
        let params = &self.base.root.params;
        params.set_gen_mode(
            Graphic3dTypeOfTextureMode::Object,
            Vec4::new(x, y, z, w),
            Vec4::new(0.0, 0.0, 0.0, 0.0),
        );
    }
    /// `Segment`: the two end points `[x1, y1, z1, x2, y2, z2]`; `t.segment()`.
    pub fn segment(&self) -> [f32; 6] {
        *self.segment.read()
    }
}
