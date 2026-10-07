//! `Graphic3d_CubeMapSeparate`: a cube map whose six sides are six image files or six
//! square images of one format.
use super::cube_map::Graphic3dCubeMapBase;
use super::texture_root::graphic3d_convert_to_compatible;
use super::{
    Graphic3dCubeMap, Graphic3dCubeMapOrder, Graphic3dCubeMapSide, Graphic3dTextureMap,
    Graphic3dTextureRoot, Graphic3dTextureRootBase,
};
use crate::image::{
    ImageAlienPixMap, ImageCompressedPixMap, ImageDdsParser, ImageFormat, ImagePixMap,
    ImageSupportedFormats,
};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::osd::{CadPath, File};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// A cube map of six files or images.
///
/// Example: `Graphic3dCubeMapSeparate::from_paths(&paths)?` with six paths.
pub struct Graphic3dCubeMapSeparate {
    transient: StandardTransient,
    base: Graphic3dCubeMapBase,
    paths: [CadPath; 6],
    images: [Handle<ImagePixMap>; 6],
    /// `mySize` and `myFormat`, set by the first side. Native quirk: both start
    /// uninitialised; here they start as 0 and `Gray`.
    first_side: SharedState<(usize, ImageFormat)>,
}
impl Graphic3dTextureRoot for Graphic3dCubeMapSeparate {
    fn texture_root_base(&self) -> &Graphic3dTextureRootBase {
        &self.base.root
    }
    /// True for images, else when all six files exist.
    fn is_done(&self) -> bool {
        self.images[0].is_some()
            || self
                .paths
                .iter()
                .all(|path| File::with_path(path.clone()).exists())
    }
    /// Always null: the sides are read through `value`.
    fn get_image(&self, _supported: &Handle<ImageSupportedFormats>) -> Handle<ImagePixMap> {
        Handle::default()
    }
}
impl Graphic3dTextureMap for Graphic3dCubeMapSeparate {}
rustycad_tkernel::impl_transient!(Graphic3dCubeMapSeparate, "Graphic3d_CubeMapSeparate", dyn Graphic3dCubeMap, transient,
    interfaces: [dyn Graphic3dTextureRoot, dyn Graphic3dTextureMap, dyn Graphic3dCubeMap]);

fn count_error(what: &str, count: usize) -> Failure {
    let message =
        format!("Invalid number of {what} Graphic3d_CubeMapSeparate; got {count}, expected 6");
    Failure::with_type("Standard_Failure", &message, "")
}

/// Whether `image` matches the first image in size, format and row order.
fn same_layout(image: &ImagePixMap, first: &ImagePixMap) -> bool {
    image.width() == first.width()
        && image.height() == first.height()
        && image.format() == first.format()
        && image.is_top_down() == first.is_top_down()
}

impl Graphic3dCubeMapSeparate {
    fn with_sources(
        paths: [CadPath; 6],
        (images, top_down): ([Handle<ImagePixMap>; 6], Option<bool>),
    ) -> Self {
        let base = Graphic3dCubeMapBase::from_pix_map(Handle::default(), false);
        if let Some(is_top_down) = top_down {
            base.root.fields.write().is_top_down = is_top_down;
        }
        let first_side = SharedState::new((0, ImageFormat::Gray));
        Self {
            transient: StandardTransient::default(),
            base,
            paths,
            images,
            first_side,
        }
    }
    /// Six image files; another count raises `Standard_Failure`.
    /// Example: `Graphic3dCubeMapSeparate::from_paths(&paths)?`.
    pub fn from_paths(paths: &[AsciiString]) -> Result<Self, Failure> {
        let paths: [AsciiString; 6] = paths
            .to_vec()
            .try_into()
            .map_err(|_| count_error("paths to load", paths.len()))?;
        Ok(Self::with_sources(
            paths.map(|path| CadPath::new(path.as_bytes())),
            (Default::default(), None),
        ))
    }
    /// Six square images of one size, format and row order; another count raises
    /// `Standard_Failure`, a mismatching set keeps no image. Example: `from_images(&images)?`.
    pub fn from_images(images: &[Handle<ImagePixMap>]) -> Result<Self, Failure> {
        let images: [Handle<ImagePixMap>; 6] = images
            .to_vec()
            .try_into()
            .map_err(|_| count_error("images in", images.len()))?;
        Ok(Self::with_sources(
            Default::default(),
            Self::accepted_images(images),
        ))
    }
    /// The images the native constructor keeps (all six, or none after a mismatch; a null or
    /// non-square first image keeps none) and the row order it records: that of a square
    /// first image, which the native loop stores before it can reset the images.
    fn accepted_images(
        images: [Handle<ImagePixMap>; 6],
    ) -> ([Handle<ImagePixMap>; 6], Option<bool>) {
        let Some(first) = images[0]
            .get()
            .filter(|first| first.width() == first.height())
        else {
            return (Default::default(), None);
        };
        let top_down = Some(first.is_top_down());
        if images[1..]
            .iter()
            .all(|image| image.get().is_some_and(|image| same_layout(image, first)))
        {
            return (images.clone(), top_down);
        }
        (Default::default(), top_down)
    }
    /// The side files (`myPaths`); `cube.paths()`.
    pub fn paths(&self) -> &[CadPath; 6] {
        &self.paths
    }
    /// The side images (`myImages`); `cube.images()`.
    pub fn images(&self) -> &[Handle<ImagePixMap>; 6] {
        &self.images
    }
    /// The first side's size and format (`mySize`, `myFormat`); `cube.first_side_layout()`.
    pub fn first_side_layout(&self) -> (usize, ImageFormat) {
        *self.first_side.read()
    }
    /// The location of the current side in the default order (Y swapped for bottom-up images).
    fn current_location(&self, swap_for_bottom_up: bool) -> usize {
        let mut order = *Graphic3dCubeMapOrder::default_order().order();
        if swap_for_bottom_up && !self.base.root.fields.read().is_top_down {
            order.swap(Graphic3dCubeMapSide::PosY, Graphic3dCubeMapSide::NegY);
        }
        usize::from(order.location_of_side_index(self.base.current_side_index()))
    }
}

/// `"'<path>' inconsistent image format or dimension in Graphic3d_CubeMapSeparate"`.
fn warn_inconsistent(path: &AsciiString) {
    let text = format!(
        "'{}' inconsistent image format or dimension in Graphic3d_CubeMapSeparate",
        String::from_utf8_lossy(path.as_bytes())
    );
    Message::send_warning_text(&AsciiString::from(text.as_str()));
}

impl Graphic3dCubeMapSeparate {
    /// The file branch of `Value`: the decoded side (never without an image library).
    fn load_side(
        &self,
        supported: &Handle<ImageSupportedFormats>,
        path: &AsciiString,
    ) -> Handle<ImagePixMap> {
        let image = Handle::new(ImageAlienPixMap::default());
        if !image.load(path) {
            let text = format!(
                "Unable to load '{}' image of Graphic3d_CubeMapSeparate",
                String::from_utf8_lossy(path.as_bytes())
            );
            Message::send_warning_text(&AsciiString::from(text.as_str()));
            return Handle::default();
        }
        let pixmap: Handle<ImagePixMap> = image.cast();
        graphic3d_convert_to_compatible(supported, &pixmap);
        let Some(side) = pixmap.get().filter(|side| side.width() == side.height()) else {
            return Handle::default();
        };
        if self.accept_side(side, path) {
            return pixmap.clone();
        }
        Handle::default()
    }
    /// Records the first side's layout, or checks a later side against it.
    fn accept_side(&self, side: &ImagePixMap, path: &AsciiString) -> bool {
        if self.base.current_side_index() == 0 {
            *self.first_side.write() = (side.width(), side.format());
            self.base.root.fields.write().is_top_down = side.is_top_down();
            return true;
        }
        let (size, format) = *self.first_side.read();
        if side.format() == format
            && side.width() == size
            && side.is_top_down() == self.base.root.fields.read().is_top_down
        {
            return true;
        }
        warn_inconsistent(path);
        false
    }
}

impl Graphic3dCubeMap for Graphic3dCubeMapSeparate {
    fn cube_map_base(&self) -> &Graphic3dCubeMapBase {
        &self.base
    }
    /// The square DDS file of the current side, consistent with the first side, or null.
    fn compressed_value(
        &self,
        supported: &Handle<ImageSupportedFormats>,
    ) -> Handle<ImageCompressedPixMap> {
        if self.images[0].is_some() {
            return Handle::default();
        }
        let path = self.paths[self.current_location(false)].system_name();
        if path.is_empty() {
            return Handle::default();
        }
        let image = ImageDdsParser::load(supported, &path, 0, 0);
        let Some(compressed) = image.get().filter(|c| c.size_x() == c.size_y()) else {
            return Handle::default();
        };
        if self.base.current_side_index() == 0 {
            *self.first_side.write() = (compressed.size_x() as usize, compressed.base_format());
            self.base.root.fields.write().is_top_down = compressed.is_top_down();
            return image;
        }
        let (size, format) = *self.first_side.read();
        if compressed.base_format() == format && compressed.size_x() == size as i32 {
            return image;
        }
        warn_inconsistent(&path);
        Handle::default()
    }
    /// The image of the current side, else its decoded file, else null with a warning.
    fn value(&self, supported: &Handle<ImageSupportedFormats>) -> Handle<ImagePixMap> {
        let location = self.current_location(true);
        if self.images[location].is_some() {
            return self.images[location].clone();
        }
        let path = self.paths[location].system_name();
        if path.is_empty() {
            let text = format!(
                "[{}] path of Graphic3d_CubeMapSeparate is invalid",
                self.base.current_side_index()
            );
            Message::send_warning_text(&AsciiString::from(text.as_str()));
            return Handle::default();
        }
        self.load_side(supported, &path)
    }
}
