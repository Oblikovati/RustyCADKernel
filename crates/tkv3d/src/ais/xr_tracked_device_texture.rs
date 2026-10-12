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
    configure_xr_shading_material(&shading)?;
    if !texture.is_null() {
        shading
            .aspect()
            .set_texture_map(Handle::new(XrDeviceTexture::new(texture)).cast());
        shading.aspect().set_texture_map_on(true);
    }
    Ok(())
}
fn configure_xr_shading_material(shading: &Handle<Prs3dShadingAspect>) -> Result<(), Failure> {
    shading.set_material(
        &Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Default),
        AspectTypeOfFacingModel::BothSide,
    );
    shading.set_color(
        Color::named(NamedColor::WHITE),
        AspectTypeOfFacingModel::BothSide,
    )?;
    Ok(())
}
struct XrDeviceTexture {
    core: Graphic3dTexture2D,
    image: Handle<dyn ImageTextureSource>,
}
impl XrDeviceTexture {
    fn new(image: &Handle<dyn ImageTextureSource>) -> Self {
        let core = Graphic3dTexture2D::from_file(&AsciiString::default());
        let identity = image.texture_id();
        if !identity.is_empty() {
            core.texture_2d_base().root.fields.write().tex_id = identity;
        }
        core.texture_2d_base()
            .root
            .params
            .set_texture_unit(Graphic3dTextureUnit::BASE_COLOR);
        Self {
            core,
            image: image.clone(),
        }
    }
}
// Native private XRTexture inherits Texture2D's descriptor; Rust caster metadata is separate.
impl rustycad_tkernel::standard::RuntimeTyped for XrDeviceTexture {
    const TYPE_NAME: &'static str = "Martensite_XrDeviceTexture";
    fn static_type() -> &'static rustycad_tkernel::standard::RuntimeType {
        static CASTERS: std::sync::OnceLock<Handle<rustycad_tkernel::standard::RuntimeType>> =
            std::sync::OnceLock::new();
        CASTERS
            .get_or_init(register_xr_texture_interfaces)
            .get()
            .unwrap()
    }
    fn project_from(object: &dyn rustycad_tkernel::standard::Transient) -> Option<&Self> {
        // A cast recovers this actual holder to forward the retained image source.
        rustycad_tkernel::standard::project_concrete::<Self>(object)
    }
}
impl rustycad_tkernel::standard::Transient for XrDeviceTexture {
    fn dynamic_type(&self) -> &'static rustycad_tkernel::standard::RuntimeType {
        // Native XRTexture does not declare RTTI and inherits its Texture2D descriptor.
        <Graphic3dTexture2D as rustycad_tkernel::standard::RuntimeTyped>::static_type()
    }
    fn rust_interface_descriptor(&self) -> &rustycad_tkernel::standard::RuntimeType {
        // The caster metadata retains the holder's actual virtual image override.
        <Self as rustycad_tkernel::standard::RuntimeTyped>::static_type()
    }
    fn transient_base(&self) -> &StandardTransient {
        // All texture views retain the same intrusive ownership subobject.
        rustycad_tkernel::standard::Transient::transient_base(&self.core)
    }
    fn project(&self, requested: std::any::TypeId) -> Option<&dyn std::any::Any> {
        if requested == std::any::TypeId::of::<Self>() {
            return Some(self);
        }
        rustycad_tkernel::standard::Transient::project(&self.core, requested)
    }
}
fn register_xr_texture_interfaces() -> Handle<rustycad_tkernel::standard::RuntimeType> {
    use rustycad_tkernel::standard::{RuntimeType, RuntimeTyped};
    RuntimeType::register_owned_with_interfaces::<XrDeviceTexture>(
        XrDeviceTexture::TYPE_NAME,
        std::mem::size_of::<XrDeviceTexture>(),
        Some(Graphic3dTexture2D::static_type_handle()),
        xr_texture_interfaces(),
    )
}
fn xr_texture_interfaces() -> Vec<rustycad_tkernel::standard::InterfaceEntry> {
    use rustycad_tkernel::standard::{InterfaceCaster, InterfaceEntry, project_concrete};
    vec![
        InterfaceEntry::new(InterfaceCaster::<dyn Graphic3dTextureRoot>::new(|value| {
            Some(project_concrete::<XrDeviceTexture>(value)?)
        })),
        InterfaceEntry::new(InterfaceCaster::<dyn Graphic3dTextureMap>::new(|value| {
            Some(project_concrete::<XrDeviceTexture>(value)?)
        })),
        InterfaceEntry::new(InterfaceCaster::<dyn Graphic3dTexture2DSource>::new(
            |value| Some(project_concrete::<XrDeviceTexture>(value)?),
        )),
    ]
}
impl Graphic3dTexture2DSource for XrDeviceTexture {
    fn texture_2d_base(&self) -> &Graphic3dTexture2DBase {
        // Preserve public texture parent state and immutable source identity.
        self.core.texture_2d_base()
    }
}
impl Graphic3dTextureMap for XrDeviceTexture {}
impl Graphic3dTextureRoot for XrDeviceTexture {
    fn texture_root_base(&self) -> &Graphic3dTextureRootBase {
        // Texture reads use the retained image source while resource parameters remain native.
        self.core.texture_root_base()
    }
    fn get_image(&self, supported: &Handle<ImageSupportedFormats>) -> Handle<ImagePixMap> {
        // OCCT's XRTexture bypasses the root file/image path and forwards the exact handle.
        self.image.read_image(supported)
    }
}
