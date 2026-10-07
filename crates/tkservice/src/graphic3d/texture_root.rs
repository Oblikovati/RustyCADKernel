//! `Graphic3d_TextureRoot`: the root of the texture classes (an image given by a file or a
//! pixmap, its parameters, its unique id and revision).
//!
//! The class has virtual members, so it is the trait [`Graphic3dTextureRoot`] (virtuals with
//! their native bodies) over the embedded [`Graphic3dTextureRootBase`]
//! (visualization-design.md section 1). Without an image library (the oracle configuration),
//! loading a file image fails as natively, so only DDS files and pixmaps give images.
use super::{Graphic3dTextureParams, Graphic3dTypeOfTexture};
use crate::image::{
    ImageAlienPixMap, ImageCompressedPixMap, ImageDdsParser, ImageFormat, ImagePixMap,
    ImageSupportedFormats,
};
use rustycad_tkernel::osd::{CadPath, Directory, Environment, File};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, Transient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicI32, Ordering};

/// `THE_TEXTURE_COUNTER`: the source of the `Graphic3d_TextureRoot_<n>` ids.
static TEXTURE_COUNTER: AtomicI32 = AtomicI32::new(0);

/// The mutable protected fields of `Graphic3d_TextureRoot`.
#[derive(Clone)]
pub struct Graphic3dTextureRootFields {
    /// `myTexId`: the unique resource id.
    pub tex_id: AsciiString,
    /// `myPixMap`: the image, when the texture is defined by a pixmap.
    pub pix_map: Handle<ImagePixMap>,
    /// `myPath`: the image file, when the texture is defined by a file.
    pub path: CadPath,
    /// `myRevision`: the image revision.
    pub revision: usize,
    /// `myType`.
    pub kind: Graphic3dTypeOfTexture,
    /// `myIsColorMap`: whether the values are colors.
    pub is_color_map: bool,
    /// `myIsTopDown`: the row order of the last image.
    pub is_top_down: bool,
    /// `myHasMipmaps`: whether mipmaps are generated.
    pub has_mipmaps: bool,
}

/// The embedded state of every texture: the parameters and the mutable fields.
pub struct Graphic3dTextureRootBase {
    /// `myParams`: the texture parameters.
    pub params: Handle<Graphic3dTextureParams>,
    /// The other protected fields.
    pub fields: SharedState<Graphic3dTextureRootFields>,
}

impl Graphic3dTextureRootBase {
    /// The fields both constructors share; `TOT_2D_MIPMAP` becomes a 2D type with mipmaps.
    fn with_source(
        path: CadPath,
        pix_map: Handle<ImagePixMap>,
        kind: Graphic3dTypeOfTexture,
    ) -> Self {
        let mipmaps = kind == Graphic3dTypeOfTexture::Tot2dMipmap;
        let fields = Graphic3dTextureRootFields {
            tex_id: AsciiString::default(),
            pix_map,
            path,
            revision: 0,
            kind: if mipmaps {
                Graphic3dTypeOfTexture::TOT_2D
            } else {
                kind
            },
            is_color_map: true,
            is_top_down: true,
            has_mipmaps: mipmaps,
        };
        let base = Self {
            params: Handle::new(Graphic3dTextureParams::default()),
            fields: SharedState::new(fields),
        };
        base.generate_id();
        base
    }
    /// `Graphic3d_TextureRoot(theFileName, theType)`;
    /// `Graphic3dTextureRootBase::from_file(&"wood.png".into(), Graphic3dTypeOfTexture::TOT_2D)`.
    pub fn from_file(file_name: &AsciiString, kind: Graphic3dTypeOfTexture) -> Self {
        Self::with_source(CadPath::new(file_name.as_bytes()), Handle::default(), kind)
    }
    /// `Graphic3d_TextureRoot(thePixMap, theType)`;
    /// `Graphic3dTextureRootBase::from_pix_map(pixmap, Graphic3dTypeOfTexture::TOT_2D)`.
    pub fn from_pix_map(pix_map: Handle<ImagePixMap>, kind: Graphic3dTypeOfTexture) -> Self {
        Self::with_source(CadPath::default(), pix_map, kind)
    }
    /// `generateId`: a new `Graphic3d_TextureRoot_<n>` id; `base.generate_id()`.
    pub fn generate_id(&self) {
        let number = TEXTURE_COUNTER
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1);
        let mut id = AsciiString::from("Graphic3d_TextureRoot_");
        id.assign_cat(number);
        self.fields.write().tex_id = id;
    }
    /// The image file path as a system name, empty for a pixmap texture.
    fn file_path(&self) -> AsciiString {
        self.fields.read().path.system_name()
    }
}

/// `TexturesFolder` on the environment: `CSF_MDTVTexturesDirectory`, else `CASROOT/src/Textures`.
fn textures_folder_from_environment() -> AsciiString {
    let read = |name: &str| {
        Environment::named(name)
            .map(|mut variable| variable.value())
            .unwrap_or_default()
    };
    let folder = read("CSF_MDTVTexturesDirectory");
    if !folder.is_empty() {
        return folder;
    }
    let mut folder = read("CASROOT");
    if !folder.is_empty() {
        folder.assign_cat("/src/Textures");
    }
    folder
}

/// The check of the first `TexturesFolder` call.
fn check_textures_folder(folder: &AsciiString) -> Result<(), Failure> {
    if folder.is_empty() {
        return Err(Failure::with_type(
            "Standard_Failure",
            "CSF_MDTVTexturesDirectory and CASROOT are undefined; got empty folder",
            "",
        ));
    }
    let mut texture = folder.clone();
    texture.assign_cat("/2d_MatraDatavision.rgb");
    let directory = Directory::with_path(CadPath::new(folder.as_bytes()));
    if !directory.exists() || !File::new(texture.as_bytes()).exists() {
        let message = format!(
            "CSF_MDTVTexturesDirectory or CASROOT not correctly set; got folder '{}'",
            String::from_utf8_lossy(folder.as_bytes())
        );
        return Err(Failure::with_type("Standard_Failure", &message, ""));
    }
    Ok(())
}

/// `Graphic3d_TextureRoot::TexturesFolder`: the folder of the standard textures. The first
/// call raises `Standard_Failure` when no folder is defined or it lacks
/// `2d_MatraDatavision.rgb`. Native quirk: the folder is marked defined before that check, so
/// later calls return it (possibly empty) without raising.
///
/// Example: `graphic3d_textures_folder()?`.
pub fn graphic3d_textures_folder() -> Result<AsciiString, Failure> {
    static FOLDER: OnceLock<AsciiString> = OnceLock::new();
    let mut first_call = false;
    let folder = FOLDER.get_or_init(|| {
        first_call = true;
        textures_folder_from_environment()
    });
    if first_call {
        check_textures_folder(folder)?;
    }
    Ok(folder.clone())
}

/// `convertToCompatible`: swaps BGR(A) images to RGB(A) when the driver lacks their format.
///
/// Example: `graphic3d_convert_to_compatible(&supported, &image)`.
pub fn graphic3d_convert_to_compatible(
    supported: &Handle<ImageSupportedFormats>,
    image: &Handle<ImagePixMap>,
) {
    let (Some(formats), Some(pixmap)) = (supported.get(), image.get()) else {
        return;
    };
    let format = pixmap.format();
    let target = match format {
        ImageFormat::Bgr => ImageFormat::Rgb,
        ImageFormat::Bgra => ImageFormat::Rgba,
        ImageFormat::Bgr32 => ImageFormat::Rgb32,
        _ => return,
    };
    if formats.is_supported(format) {
        return;
    }
    ImagePixMap::swap_rgba_bgra(pixmap);
    // Same-size channel reorder: SetFormat cannot fail between these formats.
    let _ = pixmap.set_format(target);
}

/// A texture; implementers return their embedded base. The provided members are the native
/// virtuals and their bodies.
pub trait Graphic3dTextureRoot: Transient {
    /// The embedded base; `texture.texture_root_base().params`.
    fn texture_root_base(&self) -> &Graphic3dTextureRootBase;
    /// `IsDone`: the pixmap is not empty, or the file exists; `texture.is_done()`.
    fn is_done(&self) -> bool {
        let fields = self.texture_root_base().fields.read();
        if let Some(pixmap) = fields.pix_map.get() {
            return !pixmap.is_empty();
        }
        File::with_path(fields.path.clone()).exists()
    }
    /// `GetCompressedImage`: the DDS image of a `.dds` file texture, else null;
    /// `texture.get_compressed_image(&supported)`.
    fn get_compressed_image(
        &self,
        supported: &Handle<ImageSupportedFormats>,
    ) -> Handle<ImageCompressedPixMap> {
        let base = self.texture_root_base();
        if base.fields.read().pix_map.is_some() {
            return Handle::default();
        }
        let path = base.file_path();
        if path.is_empty() || !path.as_bytes().to_ascii_lowercase().ends_with(b".dds") {
            return Handle::default();
        }
        let image = ImageDdsParser::load(supported, &path, 0, 0);
        if let Some(compressed) = image.get() {
            base.fields.write().is_top_down = compressed.is_top_down();
        }
        image
    }
    /// `GetImage(theSupported)`: the legacy image, else the pixmap, else the decoded file
    /// (always null without an image library); updates the row order.
    /// Example: `texture.get_image(&supported)`.
    fn get_image(&self, supported: &Handle<ImageSupportedFormats>) -> Handle<ImagePixMap> {
        graphic3d_texture_root_get_image(self, supported)
    }
    /// The protected `GetImage() const` of the old API: null by default; `texture.legacy_image()`.
    fn legacy_image(&self) -> Handle<ImagePixMap> {
        Handle::default()
    }
}

/// The body of `Graphic3d_TextureRoot::GetImage`, for the classes that override it and call
/// the base version; `graphic3d_texture_root_get_image(&texture, &supported)`.
pub fn graphic3d_texture_root_get_image<T: Graphic3dTextureRoot + ?Sized>(
    texture: &T,
    supported: &Handle<ImageSupportedFormats>,
) -> Handle<ImagePixMap> {
    let base = texture.texture_root_base();
    // Copy the pixmap handle first: a guard in the `for` header would live through the
    // loop body and deadlock the write below.
    let pix_map = base.fields.read().pix_map.clone();
    for image in [texture.legacy_image(), pix_map] {
        if let Some(pixmap) = image.get() {
            base.fields.write().is_top_down = pixmap.is_top_down();
            return image;
        }
    }
    load_file_image(base, supported)
}

/// The file branch of `GetImage`.
fn load_file_image(
    base: &Graphic3dTextureRootBase,
    supported: &Handle<ImageSupportedFormats>,
) -> Handle<ImagePixMap> {
    let path = base.file_path();
    if path.is_empty() {
        return Handle::default();
    }
    let image = Handle::new(ImageAlienPixMap::default());
    if !image.load(&path) {
        return Handle::default();
    }
    let pixmap: Handle<ImagePixMap> = image.cast();
    base.fields.write().is_top_down = pixmap.get().is_some_and(ImagePixMap::is_top_down);
    graphic3d_convert_to_compatible(supported, &pixmap);
    pixmap
}

/// The non-virtual members of `Graphic3d_TextureRoot`.
pub trait Graphic3dTextureRootMembers: Graphic3dTextureRoot {
    /// The image file path; `texture.path()`.
    fn path(&self) -> CadPath {
        self.texture_root_base().fields.read().path.clone()
    }
    /// The texture type; `texture.texture_type()`.
    fn texture_type(&self) -> Graphic3dTypeOfTexture {
        self.texture_root_base().fields.read().kind
    }
    /// The unique id; `texture.get_id()`.
    fn get_id(&self) -> AsciiString {
        self.texture_root_base().fields.read().tex_id.clone()
    }
    /// The image revision; `texture.revision()`.
    fn revision(&self) -> usize {
        self.texture_root_base().fields.read().revision
    }
    /// Signals a change of the image; `texture.update_revision()`.
    fn update_revision(&self) {
        let mut fields = self.texture_root_base().fields.write();
        fields.revision = fields.revision.wrapping_add(1);
    }
    /// The texture parameters; `texture.get_params().set_repeat(true)`.
    fn get_params(&self) -> Handle<Graphic3dTextureParams> {
        self.texture_root_base().params.clone()
    }
    /// Whether the values are colors; `texture.is_color_map()`.
    fn is_color_map(&self) -> bool {
        self.texture_root_base().fields.read().is_color_map
    }
    /// Sets the color nature of the values; `texture.set_color_map(false)`.
    fn set_color_map(&self, is_color: bool) {
        self.texture_root_base().fields.write().is_color_map = is_color;
    }
    /// Whether mipmaps are generated; `texture.has_mipmaps()`.
    fn has_mipmaps(&self) -> bool {
        self.texture_root_base().fields.read().has_mipmaps
    }
    /// Sets mipmap generation; `texture.set_mipmaps_generation(true)`.
    fn set_mipmaps_generation(&self, to_generate: bool) {
        self.texture_root_base().fields.write().has_mipmaps = to_generate;
    }
    /// The row order of the last image; `texture.is_top_down()`.
    fn is_top_down(&self) -> bool {
        self.texture_root_base().fields.read().is_top_down
    }
}
impl<T: Graphic3dTextureRoot + ?Sized> Graphic3dTextureRootMembers for T {}

rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dTextureRoot,
    "Graphic3d_TextureRoot",
    rustycad_tkernel::standard::StandardTransient
);
