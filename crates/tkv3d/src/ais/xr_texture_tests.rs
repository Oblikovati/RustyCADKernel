//! Native bytes prove private image delegation, constructor material and actual descriptor identity.
use super::xr_texture_source_fake::XrImageSourceFake;
use super::*;
use crate::prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers};
use rustycad_tkernel::standard::{Handle, RuntimeTyped, catch_native_failure};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*, image::*};
const XR_TEXTURE_NATIVE: &str = include_str!("../../tests/fixtures/xr-device-texture.txt");
fn xr_texture_replay(kind: i32, expected: &mut std::str::Lines<'_>) {
    let source = Handle::new(XrImageSourceFake::new(kind));
    let mesh = super::xr_state_fixture::xr_state_model(2);
    let device = AisXrTrackedDevice::with_render_model(&mesh, &source.clone().cast()).unwrap();
    assert_eq!(xr_shading_row(&device, kind), expected.next().unwrap());
    let texture = device.attributes().shading_aspect().aspect().texture_map();
    for step in 0..4 {
        replay_xr_texture_read(&source, &texture, kind, step, expected);
    }
}
fn replay_xr_texture_read(
    source: &Handle<XrImageSourceFake>,
    texture: &Handle<dyn Graphic3dTextureMap>,
    kind: i32,
    step: i32,
    expected: &mut std::str::Lines<'_>,
) {
    *source.fail.write() = step == 2;
    let formats = if step == 1 {
        Handle::default()
    } else {
        source.expected.clone()
    };
    let output = catch_native_failure(|| Ok(texture.get_image(&formats)));
    let row = xr_texture_read_outcome(source, texture, (kind, step), output);
    assert_eq!(row, expected.next().unwrap());
}
fn xr_texture_read_outcome(
    source: &Handle<XrImageSourceFake>,
    texture: &Handle<dyn Graphic3dTextureMap>,
    ordinal: (i32, i32),
    output: Result<Handle<ImagePixMap>, rustycad_tkernel::standard::Failure>,
) -> String {
    let failure = output.is_err();
    let image = output.unwrap_or_default();
    xr_texture_row(source, texture, ordinal.0, ordinal.1, failure, image)
}
fn xr_shading_row(device: &Handle<AisXrTrackedDevice>, kind: i32) -> String {
    let drawer = device.attributes();
    let shading = drawer.shading_aspect();
    let side = AspectTypeOfFacingModel::BothSide;
    let [r, g, b] = shading.color(side).rgb();
    format!(
        "shading {kind} {} {} {r} {g} {b} {}",
        i32::from(drawer.has_own_shading_aspect()),
        i32::from(shading.aspect().to_map_texture()),
        shading.material(side).name() as i32
    )
}
fn xr_texture_row(
    source: &Handle<XrImageSourceFake>,
    texture: &Handle<dyn Graphic3dTextureMap>,
    kind: i32,
    step: i32,
    failure: bool,
    output: Handle<ImagePixMap>,
) -> String {
    let prefix = xr_texture_identity_row(source, texture, kind, step, failure, output);
    format!(
        "{prefix} {} {}",
        xr_texture_parameter_row(texture),
        xr_texture_descriptor_row(texture)
    )
}
fn xr_texture_identity_row(
    source: &Handle<XrImageSourceFake>,
    texture: &Handle<dyn Graphic3dTextureMap>,
    kind: i32,
    step: i32,
    failure: bool,
    output: Handle<ImagePixMap>,
) -> String {
    format!(
        "{kind} {step} {} {} {} {} {} {} {}",
        i32::from(failure),
        *source.calls.read(),
        i32::from(*source.same.read()),
        i32::from(output == source.image),
        i32::from(texture.get_id() == source.texture_id()),
        i32::from(!texture.get_id().is_empty()),
        texture.texture_type() as i32
    )
}
fn xr_texture_parameter_row(texture: &Handle<dyn Graphic3dTextureMap>) -> String {
    let params = texture.get_params();
    format!(
        "{} {} {} {} {} {}",
        i32::from(texture.is_color_map()),
        i32::from(texture.has_mipmaps()),
        i32::from(params.is_modulate()),
        i32::from(params.is_repeat()),
        params.filter() as i32,
        params.texture_unit() as i32
    )
}
fn xr_texture_descriptor_row(texture: &Handle<dyn Graphic3dTextureMap>) -> String {
    let descriptor = texture.dynamic_type().unwrap();
    format!(
        "{} {} {} {}",
        descriptor.name(),
        descriptor.parent().unwrap().name(),
        i32::from(std::ptr::eq(descriptor, Graphic3dTexture2D::static_type())),
        i32::from(texture.is_kind(<dyn Graphic3dTextureMap>::static_type()))
    )
}
fn xr_rtti_row() -> String {
    let device = AisXrTrackedDevice::new().unwrap();
    let interactive = device.clone().cast::<dyn AisInteractiveObject>();
    let source = interactive.clone().cast::<dyn AisXrTrackedDeviceSource>();
    let descriptor = AisXrTrackedDevice::static_type();
    assert!(std::ptr::eq(
        <dyn AisXrTrackedDeviceSource>::static_type(),
        descriptor
    ));
    assert_eq!(source.object_address(), interactive.object_address());
    let flags = xr_descriptor_flags(&device, &interactive);
    let mut row = xr_rtti_identity_header();
    for flag in flags {
        row.push_str(&format!(" {}", i32::from(flag)));
    }
    for mode in -1..3 {
        row.push_str(&format!(" {}", i32::from(device.accept_display_mode(mode))));
    }
    row
}
fn xr_rtti_identity_header() -> String {
    let descriptor = AisXrTrackedDevice::static_type();
    format!(
        "rtti {} {} {}",
        AisXrTrackedDevice::TYPE_NAME,
        descriptor.name(),
        descriptor.parent().unwrap().name()
    )
}
fn xr_descriptor_flags(
    device: &Handle<AisXrTrackedDevice>,
    interactive: &Handle<dyn AisInteractiveObject>,
) -> [bool; 6] {
    let descriptor = AisXrTrackedDevice::static_type();
    let projected = interactive.clone().cast::<AisXrTrackedDevice>();
    [
        std::ptr::eq(descriptor, AisXrTrackedDevice::static_type()),
        std::ptr::eq(interactive.dynamic_type().unwrap(), descriptor),
        interactive.is_instance(descriptor),
        interactive.is_kind(<dyn AisInteractiveObject>::static_type()),
        projected.object_address() == device.object_address(),
        std::ptr::eq(
            <AisXrTrackedDeviceParent>::static_type(),
            descriptor.parent().unwrap(),
        ),
    ]
}
fn xr_public_texture_rtti_row() -> String {
    let plain = Handle::new(Graphic3dTexture2D::from_file(&"".into()));
    let plane = Handle::new(Graphic3dTexture2Dplane::from_file(&"".into()));
    assert!(std::ptr::eq(
        <dyn Graphic3dTexture2DSource>::static_type(),
        Graphic3dTexture2D::static_type()
    ));
    format!(
        "texture-rtti {} {} {} {} {} {}",
        plain.dynamic_type().unwrap().name(),
        plain.dynamic_type().unwrap().parent().unwrap().name(),
        plane.dynamic_type().unwrap().name(),
        plane.dynamic_type().unwrap().parent().unwrap().name(),
        i32::from(plain.is_kind(<dyn Graphic3dTextureMap>::static_type())),
        i32::from(plane.is_kind(Graphic3dTexture2D::static_type()))
    )
}
#[test]
fn native_xr_texture_forwarding_shading_and_descriptor_ancestry() {
    let mut expected = XR_TEXTURE_NATIVE.lines();
    assert_eq!(xr_rtti_row(), expected.next().unwrap());
    assert_eq!(xr_public_texture_rtti_row(), expected.next().unwrap());
    for kind in 0..2 {
        xr_texture_replay(kind, &mut expected);
    }
    assert!(expected.next().is_none());
}
