// Rust port of DataExchange/TKXCAF/XCAFPrs/XCAFPrs_Texture.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFPrs_Texture`: a 2D texture whose image comes from the `Image_Texture` of a
//! visualization material, bound to a texture unit.
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::graphic3d::{
    Graphic3dTexture2DBase, Graphic3dTexture2DSource, Graphic3dTextureMap, Graphic3dTextureRoot,
    Graphic3dTextureRootBase, Graphic3dTextureUnit, Graphic3dTypeOfTexture,
    graphic3d_convert_to_compatible,
};
use rustycad_tkservice::image::{
    ImageCompressedPixMap, ImagePixMap, ImageSupportedFormats, ImageTextureMembers,
    ImageTextureSource,
};

/// `XCAFPrs_Texture`.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkservice::graphic3d::{Graphic3dTextureRoot, Graphic3dTextureUnit};
/// use rustycad_tkxcaf::xcaf_prs::XCAFPrsTexture;
/// let texture = XCAFPrsTexture::new(Handle::default(), Graphic3dTextureUnit::BASE_COLOR);
/// assert!(texture.texture_root_base().fields.read().is_color_map);
/// ```
pub struct XCAFPrsTexture {
    transient: StandardTransient,
    base: Graphic3dTexture2DBase,
    image_source: Handle<dyn ImageTextureSource>,
}
rustycad_tkernel::impl_transient!(XCAFPrsTexture, "XCAFPrs_Texture", dyn Graphic3dTexture2DSource,
    transient, interfaces: [dyn Graphic3dTextureRoot, dyn Graphic3dTextureMap, dyn Graphic3dTexture2DSource]);

impl Graphic3dTextureRoot for XCAFPrsTexture {
    fn texture_root_base(&self) -> &Graphic3dTextureRootBase {
        &self.base.root
    }
    /// `GetCompressedImage`: the compressed image of the source, null without one.
    fn get_compressed_image(
        &self,
        supported: &Handle<ImageSupportedFormats>,
    ) -> Handle<ImageCompressedPixMap> {
        self.image_source
            .get()
            .map_or_else(Handle::default, |source| {
                source.read_compressed_image(supported)
            })
    }
    /// `GetImage`: the image of the source converted to a supported format, null without one.
    fn get_image(&self, supported: &Handle<ImageSupportedFormats>) -> Handle<ImagePixMap> {
        let Some(source) = self.image_source.get() else {
            return Handle::default();
        };
        let image = source.read_image(supported);
        graphic3d_convert_to_compatible(supported, &image);
        image
    }
}
impl Graphic3dTextureMap for XCAFPrsTexture {}
impl Graphic3dTexture2DSource for XCAFPrsTexture {
    fn texture_2d_base(&self) -> &Graphic3dTexture2DBase {
        &self.base
    }
}

impl XCAFPrsTexture {
    /// `XCAFPrs_Texture(theImageSource, theUnit)`: an empty file texture (with the public
    /// defaults) identified as its source, on the unit, a color map for base color and
    /// emission.
    pub fn new(source: Handle<dyn ImageTextureSource>, unit: Graphic3dTextureUnit) -> Self {
        let base = Graphic3dTexture2DBase::from_file(
            &AsciiString::default(),
            Graphic3dTypeOfTexture::TOT_2D,
        )
        .with_public_defaults();
        if let Some(id) = source
            .get()
            .map(|s| s.texture_id())
            .filter(|id| !id.is_empty())
        {
            base.root.fields.write().tex_id = id;
        }
        live_params(&base).set_texture_unit(unit);
        base.root.fields.write().is_color_map =
            unit == Graphic3dTextureUnit::BASE_COLOR || unit == Graphic3dTextureUnit::EMISSIVE;
        Self {
            transient: StandardTransient::default(),
            base,
            image_source: source,
        }
    }
    /// `GetImageSource`.
    pub fn get_image_source(&self) -> &Handle<dyn ImageTextureSource> {
        &self.image_source
    }
}

/// The texture parameters, always set by the base constructors.
fn live_params(
    base: &Graphic3dTexture2DBase,
) -> &rustycad_tkservice::graphic3d::Graphic3dTextureParams {
    base.root
        .params
        .get()
        .expect("the texture constructors create the parameters")
}
