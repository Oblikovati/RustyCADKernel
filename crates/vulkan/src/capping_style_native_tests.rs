//! Actual OpenGl_CappingPlaneResource aspect snapshots replay without an OCCT dependency.
use super::*;
use rustycad_tkernel::quantity::ColorRgba;
#[test]
fn native_front_only_material_and_texture_selection_preserves_cap_alpha_policy() {
    let plane = native_cap_aspect_fixture();
    let group = native_object_aspect_fixture();
    for line in include_str!("../tests/fixtures/texture-texgen-capping/native-cap-contract.txt")
        .lines()
        .filter(|line| line.starts_with("ASPECT"))
    {
        let row: Vec<f32> = line
            .split_whitespace()
            .skip(1)
            .map(|value| value.parse().unwrap())
            .collect();
        let flags = row[0] as u32;
        plane.set_use_object_material(flags & 1 != 0);
        plane.set_use_object_texture(flags & 2 != 0);
        assert_native_cap_aspect(&cap_aspect(&plane, &group.cast()), &group, &row);
    }
}
fn native_cap_aspect_fixture() -> Graphic3dClipPlane {
    let plane = Graphic3dClipPlane::default();
    let aspect = plane.capping_aspect();
    let mut front = aspect.front_material();
    front.set_transparency(0.1).unwrap();
    let mut back = aspect.back_material();
    back.set_transparency(0.2).unwrap();
    aspect.set_front_material(front);
    aspect.set_back_material(back);
    aspect.set_interior_color_rgba(ColorRgba::new(0.2, 0.3, 0.4, 0.5).unwrap());
    aspect.set_alpha_mode(Graphic3dAlphaMode::MaskBlend, 0.25);
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Phong);
    aspect.set_texture_map(native_contract_texture().cast());
    aspect.set_texture_map_on(true);
    plane
}
fn native_object_aspect_fixture() -> Handle<Graphic3dAspects> {
    let aspect = Handle::new(Graphic3dAspects::default());
    let mut front = aspect.front_material();
    front.set_transparency(0.4).unwrap();
    let mut back = aspect.back_material();
    back.set_transparency(0.7).unwrap();
    aspect.set_front_material(front);
    aspect.set_back_material(back);
    aspect.set_interior_color_rgba(ColorRgba::new(0.6, 0.7, 0.8, 0.9).unwrap());
    aspect.set_texture_map(native_contract_texture().cast());
    aspect.set_texture_map_on(true);
    aspect
}
fn assert_native_cap_aspect(
    aspect: &Handle<dyn Graphic3dAspectsSource>,
    object: &Handle<Graphic3dAspects>,
    row: &[f32],
) {
    let rgba = aspect.interior_color_rgba();
    let actual = [
        aspect.front_material().transparency(),
        aspect.back_material().transparency(),
        aspect.alpha_mode() as i32 as f32,
        aspect.alpha_cutoff(),
        aspect.shading_model() as i32 as f32,
        f32::from(u8::from(aspect.texture_map() == object.texture_map())),
        rgba.rgb().red() as f32,
        rgba.rgb().green() as f32,
        rgba.rgb().blue() as f32,
        rgba.alpha(),
    ];
    assert_eq!(actual.as_slice(), &row[1..], "native flags={}", row[0]);
}

fn native_contract_texture() -> Handle<Graphic3dTexture2D> {
    let image = Handle::new(rustycad_tkservice::image::ImagePixMap::default());
    assert!(image.init_zero(rustycad_tkservice::image::ImageFormat::Rgba, 1, 1, 0, 0));
    Handle::new(Graphic3dTexture2D::from_pix_map(image))
}
