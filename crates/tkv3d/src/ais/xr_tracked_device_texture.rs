//! Native private XR texture holder delegates image reads while preserving resource identity.
use super::AisXrTrackedDeviceSource;
use crate::{prs_mgr::PrsMgrPresentableObjectMembers, prs3d::Prs3dShadingAspect};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, StandardTransient},
    tcollection::AsciiString,
};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*, image::*};
pub(super) fn initialize_xr_shading<O: AisXrTrackedDeviceSource + ?Sized>(
    object: &O,
    texture: &Handle<dyn ImageTextureSource>,
) -> Result<(), Failure> {
    let shading = Handle::new(Prs3dShadingAspect::new()?);
    object.attributes().set_shading_aspect(shading.clone());
    shading.set_material(
        &Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Default),
        AspectTypeOfFacingModel::BothSide,
    );
    shading.set_color(
        Color::named(NamedColor::WHITE),
        AspectTypeOfFacingModel::BothSide,
    )?;
    if !texture.is_null() {
        shading
            .aspect()
            .set_texture_map(Handle::new(XrDeviceTexture::new(texture)).cast());
        shading.aspect().set_texture_map_on(true);
    }
    Ok(())
}
struct XrDeviceTexture {
    transient: StandardTransient,
    base: Graphic3dTexture2DBase,
    image: Handle<dyn ImageTextureSource>,
}
impl XrDeviceTexture {
    fn new(image: &Handle<dyn ImageTextureSource>) -> Self {
        let base = Graphic3dTexture2DBase::from_file(
            &AsciiString::default(),
            Graphic3dTypeOfTexture::TOT_2D,
        )
        .with_public_defaults();
        let identity = image.texture_id();
        if !identity.is_empty() {
            base.root.fields.write().tex_id = identity;
        }
        base.root
            .params
            .set_texture_unit(Graphic3dTextureUnit::BASE_COLOR);
        Self {
            transient: StandardTransient::default(),
            base,
            image: image.clone(),
        }
    }
}
// Native private XRTexture inherits Texture2D's descriptor; this holder grants no private API credit.
rustycad_tkernel::impl_transient!(XrDeviceTexture,"Graphic3d_Texture2D",dyn Graphic3dTextureMap,transient,
    interfaces:[dyn Graphic3dTextureRoot,dyn Graphic3dTextureMap,dyn Graphic3dTexture2DSource]);
impl Graphic3dTexture2DSource for XrDeviceTexture {
    fn texture_2d_base(&self) -> &Graphic3dTexture2DBase {
        // Preserve public texture parent state and immutable source identity.
        &self.base
    }
}
impl Graphic3dTextureMap for XrDeviceTexture {}
impl Graphic3dTextureRoot for XrDeviceTexture {
    fn texture_root_base(&self) -> &Graphic3dTextureRootBase {
        // Texture reads use the retained image source while resource parameters remain native.
        &self.base.root
    }
    fn get_image(&self, supported: &Handle<ImageSupportedFormats>) -> Handle<ImagePixMap> {
        // OCCT's XRTexture bypasses the root file/image path and forwards the exact handle.
        self.image.read_image(supported)
    }
}
