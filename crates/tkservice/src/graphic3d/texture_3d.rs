//! `Graphic3d_Texture3D`: a volume texture given by one file, a 3D pixmap or one file per
//! slice.
use super::texture_root::{graphic3d_convert_to_compatible, graphic3d_texture_root_get_image};
use super::{
    Graphic3dTextureMap, Graphic3dTextureRoot, Graphic3dTextureRootBase, Graphic3dTypeOfTexture,
    Graphic3dTypeOfTextureFilter,
};
use crate::image::{ImageAlienPixMap, ImagePixMap, ImageSupportedFormats};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::osd::CadPath;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// A 3D texture with trilinear filtering.
///
/// Example: `Graphic3dTexture3D::from_slices(&["s0.png".into(), "s1.png".into()])`.
pub struct Graphic3dTexture3D {
    transient: StandardTransient,
    base: Graphic3dTextureRootBase,
    /// `myPaths`: the slice files.
    paths: SharedState<Vec<AsciiString>>,
}
impl Graphic3dTextureMap for Graphic3dTexture3D {}
rustycad_tkernel::impl_transient!(Graphic3dTexture3D, "Graphic3d_Texture3D", dyn Graphic3dTextureMap, transient,
    interfaces: [dyn Graphic3dTextureRoot, dyn Graphic3dTextureMap]);

impl Graphic3dTexture3D {
    fn wrap(base: Graphic3dTextureRootBase, paths: Vec<AsciiString>) -> Self {
        base.params
            .set_filter(Graphic3dTypeOfTextureFilter::Trilinear);
        Self {
            transient: StandardTransient::default(),
            base,
            paths: SharedState::new(paths),
        }
    }
    /// A volume file; `Graphic3dTexture3D::from_file(&"volume.dds".into())`.
    pub fn from_file(file_name: &AsciiString) -> Self {
        Self::wrap(
            Graphic3dTextureRootBase::from_file(file_name, Graphic3dTypeOfTexture::Texture3d),
            Vec::new(),
        )
    }
    /// A 3D pixmap; `Graphic3dTexture3D::from_pix_map(pixmap)`.
    pub fn from_pix_map(pix_map: Handle<ImagePixMap>) -> Self {
        Self::wrap(
            Graphic3dTextureRootBase::from_pix_map(pix_map, Graphic3dTypeOfTexture::Texture3d),
            Vec::new(),
        )
    }
    /// One file per slice; `Graphic3dTexture3D::from_slices(&["s0.png".into()])`.
    pub fn from_slices(files: &[AsciiString]) -> Self {
        Self::wrap(
            Graphic3dTextureRootBase::from_file(
                &AsciiString::default(),
                Graphic3dTypeOfTexture::Texture3d,
            ),
            files.to_vec(),
        )
    }
    /// Replaces the image by a pixmap and forgets the file and the slices; `texture.set_image(pixmap)`.
    pub fn set_image(&self, pix_map: Handle<ImagePixMap>) {
        let mut fields = self.base.fields.write();
        fields.pix_map = pix_map;
        fields.path = CadPath::default();
        self.paths.write().clear();
    }
    /// The slice files; `texture.slice_paths()`.
    pub fn slice_paths(&self) -> Vec<AsciiString> {
        self.paths.read().clone()
    }
}

fn fail(text: String) -> Handle<ImagePixMap> {
    Message::send_fail_text(&AsciiString::from(text.as_str()));
    Handle::default()
}

impl Graphic3dTexture3D {
    /// The 3D image of the first slice's layout, with that slice's row order.
    fn allocate_volume(
        &self,
        slice: &ImagePixMap,
        nb_slices: usize,
    ) -> Option<Handle<ImagePixMap>> {
        let is_top_down = slice.is_top_down();
        self.base.fields.write().is_top_down = is_top_down;
        let volume = ImagePixMap::default();
        volume.set_top_down(is_top_down);
        let size = Vec3::new(slice.width(), slice.height(), nb_slices);
        volume
            .init_trash_3d(slice.format(), size, slice.size_row_bytes())
            .then(|| Handle::new(volume))
    }
    /// Loads slice `index` of `count`, allocates the volume on the first slice and copies the
    /// slice into it; false (after the native failure message) when any step fails.
    fn stack_slice(
        &self,
        volume: &mut Option<Handle<ImagePixMap>>,
        (index, count): (usize, usize),
        path: &AsciiString,
        supported: &Handle<ImageSupportedFormats>,
    ) -> bool {
        let image = Handle::new(ImageAlienPixMap::default());
        if !image.load(path) {
            fail(format!(
                "Graphic3d_Texture3D::GetImage() failed to load slice {index} from '{}'",
                String::from_utf8_lossy(path.as_bytes())
            ));
            return false;
        }
        let slice: Handle<ImagePixMap> = image.cast();
        graphic3d_convert_to_compatible(supported, &slice);
        let Some(pixels) = slice.get() else {
            return false;
        };
        if volume.is_none() {
            *volume = self.allocate_volume(pixels, count);
            if volume.is_none() {
                fail(format!(
                    "Graphic3d_Texture3D::GetImage() failed to allocate 3D image {}x{}x{count}",
                    pixels.width() as i32,
                    pixels.height() as i32
                ));
                return false;
            }
        }
        volume
            .as_ref()
            .and_then(Handle::get)
            .is_some_and(|target| Self::copy_slice(target, pixels, index, path))
    }
    /// Copies one slice into the volume, or reports a slice of another layout.
    fn copy_slice(
        volume: &ImagePixMap,
        slice: &ImagePixMap,
        index: usize,
        path: &AsciiString,
    ) -> bool {
        if slice.format() != volume.format()
            || slice.width() != volume.width()
            || slice.height() != volume.height()
            || slice.size_row_bytes() != volume.size_row_bytes()
        {
            fail(format!(
                "Graphic3d_Texture3D::GetImage() slice {index} from '{}' have different dimensions",
                String::from_utf8_lossy(path.as_bytes())
            ));
            return false;
        }
        let source = slice.data().data.data().to_vec();
        let mut state = volume.change_data();
        let count = source.len().min(slice.size_bytes());
        if let Some(target) = state.data.change_slice(index) {
            let count = count.min(target.len());
            target[..count].copy_from_slice(&source[..count]);
        }
        true
    }
}

impl Graphic3dTextureRoot for Graphic3dTexture3D {
    fn texture_root_base(&self) -> &Graphic3dTextureRootBase {
        &self.base
    }
    /// The base image for a pixmap or single file, else the slices stacked into one 3D image
    /// (never without an image library: the first slice fails to load and is reported).
    fn get_image(&self, supported: &Handle<ImageSupportedFormats>) -> Handle<ImagePixMap> {
        let paths = self.slice_paths();
        if paths.is_empty() || self.base.fields.read().pix_map.is_some() {
            return graphic3d_texture_root_get_image(self, supported);
        }
        let mut volume: Option<Handle<ImagePixMap>> = None;
        for (index, path) in paths.iter().enumerate() {
            if !self.stack_slice(&mut volume, (index, paths.len()), path, supported) {
                return Handle::default();
            }
        }
        volume.unwrap_or_default()
    }
}
