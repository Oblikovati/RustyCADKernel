//! `Graphic3d_CubeMap`: the abstract cube map texture, iterated side by side
//! (`More`/`Next`/`CurrentSide`), whose classes give the image of the current side.
use super::{
    Graphic3dCubeMapSide, Graphic3dTextureMap, Graphic3dTextureRootBase, Graphic3dTypeOfTexture,
};
use crate::image::{ImageCompressedPixMap, ImagePixMap, ImageSupportedFormats};
use rustycad_tkernel::standard::{Handle, SharedState};
use rustycad_tkernel::tcollection::AsciiString;

/// The iteration state of a cube map.
#[derive(Clone, Copy, Debug)]
pub struct Graphic3dCubeMapIteration {
    /// `myCurrentSide` as its integer value: `Next` after the end goes past `NEG_Z` natively.
    pub current_side: i32,
    /// `myEndIsReached`.
    pub end_is_reached: bool,
    /// `myZIsInverted`.
    pub z_is_inverted: bool,
}

/// The embedded state of a cube map: the root base and the iteration.
pub struct Graphic3dCubeMapBase {
    /// The root base.
    pub root: Graphic3dTextureRootBase,
    /// The iteration state.
    pub iteration: SharedState<Graphic3dCubeMapIteration>,
}

impl Graphic3dCubeMapBase {
    fn with_root(root: Graphic3dTextureRootBase, to_generate_mipmaps: bool) -> Self {
        root.fields.write().has_mipmaps = to_generate_mipmaps;
        let iteration = Graphic3dCubeMapIteration {
            current_side: 0,
            end_is_reached: false,
            z_is_inverted: false,
        };
        Self {
            root,
            iteration: SharedState::new(iteration),
        }
    }
    /// `Graphic3d_CubeMap(theFileName, theToGenerateMipmaps)`;
    /// `Graphic3dCubeMapBase::from_file(&"sky.dds".into(), false)`.
    pub fn from_file(file_name: &AsciiString, to_generate_mipmaps: bool) -> Self {
        Self::with_root(
            Graphic3dTextureRootBase::from_file(file_name, Graphic3dTypeOfTexture::Cubemap),
            to_generate_mipmaps,
        )
    }
    /// `Graphic3d_CubeMap(thePixmap, theToGenerateMipmaps)`;
    /// `Graphic3dCubeMapBase::from_pix_map(Handle::default(), false)`.
    pub fn from_pix_map(pix_map: Handle<ImagePixMap>, to_generate_mipmaps: bool) -> Self {
        Self::with_root(
            Graphic3dTextureRootBase::from_pix_map(pix_map, Graphic3dTypeOfTexture::Cubemap),
            to_generate_mipmaps,
        )
    }
    /// The current side's integer value; `base.current_side_index()`.
    pub fn current_side_index(&self) -> i32 {
        self.iteration.read().current_side
    }
}

/// A cube map texture (abstract `Graphic3d_CubeMap`).
pub trait Graphic3dCubeMap: Graphic3dTextureMap {
    /// The embedded cube map base; `cube_map.cube_map_base()`.
    fn cube_map_base(&self) -> &Graphic3dCubeMapBase;
    /// The compressed image of the current side, or null; `cube_map.compressed_value(&supported)`.
    fn compressed_value(
        &self,
        supported: &Handle<ImageSupportedFormats>,
    ) -> Handle<ImageCompressedPixMap>;
    /// The image of the current side, or null; `cube_map.value(&supported)`.
    fn value(&self, supported: &Handle<ImageSupportedFormats>) -> Handle<ImagePixMap>;
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dCubeMap,
    "Graphic3d_CubeMap",
    dyn Graphic3dTextureMap
);

/// The side of an integer value, `None` past `NEG_Z`.
pub(crate) fn cube_map_side(value: i32) -> Option<Graphic3dCubeMapSide> {
    use Graphic3dCubeMapSide as S;
    [S::PosX, S::NegX, S::PosY, S::NegY, S::PosZ, S::NegZ]
        .into_iter()
        .find(|side| *side as i32 == value)
}

/// The non-virtual members of `Graphic3d_CubeMap`.
pub trait Graphic3dCubeMapMembers: Graphic3dCubeMap {
    /// Whether the iteration has not ended; `cube_map.more()`.
    fn more(&self) -> bool {
        !self.cube_map_base().iteration.read().end_is_reached
    }
    /// The current side; `None` when `Next` went past `NEG_Z` (natively an out-of-range enum).
    /// Example: `cube_map.current_side() == Some(Graphic3dCubeMapSide::PosX)`.
    fn current_side(&self) -> Option<Graphic3dCubeMapSide> {
        cube_map_side(self.cube_map_base().current_side_index())
    }
    /// Moves to the next side; at `NEG_Z` it marks the end instead; `cube_map.next()`.
    fn next(&self) {
        let mut iteration = self.cube_map_base().iteration.write();
        if !iteration.end_is_reached && iteration.current_side == Graphic3dCubeMapSide::NegZ as i32
        {
            iteration.end_is_reached = true;
        } else {
            iteration.current_side = iteration.current_side.wrapping_add(1);
        }
    }
    /// Sets the Z inversion flag; `cube_map.set_z_inversion(true)`.
    fn set_z_inversion(&self, is_inverted: bool) {
        self.cube_map_base().iteration.write().z_is_inverted = is_inverted;
    }
    /// Whether Z is inverted; `cube_map.z_is_inverted()`.
    fn z_is_inverted(&self) -> bool {
        self.cube_map_base().iteration.read().z_is_inverted
    }
    /// Restarts the iteration at `POS_X`; `cube_map.reset()`.
    fn reset(&self) {
        let mut iteration = self.cube_map_base().iteration.write();
        (iteration.current_side, iteration.end_is_reached) = (0, false);
    }
}
impl<T: Graphic3dCubeMap + ?Sized> Graphic3dCubeMapMembers for T {}
