//! Existing mode3 refresh and intentionally stale UnsetColor copies are retained behavior.
#[allow(dead_code)] // Shared named fixture also serves source-state-only replay modules.
#[path = "support/textured_shape_state_fixture.rs"]
mod state_fixture;
use rustycad_tkernel::{
    quantity::{Color, ColorSpace, NamedColor},
    standard::Handle,
};
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::Prs3dDrawer, v3d::V3dViewer};
use state_fixture::TexturedStateFixture;

#[test]
fn textured_color_refreshes_mode3_then_unset_color_keeps_the_copied_aspect() {
    let object = TexturedStateFixture::new();
    prepare_appearance_drawer(&object);
    object.set_texture_map_off();
    let (_, mode3) = appearance_presentations(&object);
    let color = Color::named(NamedColor::RED);
    ais_textured_shape_set_color(&*object, color).unwrap();
    let saved = object.base.fields.read().aspect.clone();
    compare_native_appearance(&object, &saved, "red");
    assert_eq!(
        mode3.groups()[0].aspects(),
        saved.cast::<dyn Graphic3dAspectsSource>()
    );
    ais_textured_shape_unset_color(&*object).unwrap();
    compare_native_appearance(&object, &saved, "unset");
    assert_eq!(object.base.fields.read().aspect, saved);
    assert_eq!(saved.interior_color(), color);
}
fn prepare_appearance_drawer(object: &TexturedStateFixture) {
    let linked = Handle::new(Prs3dDrawer::new());
    linked.setup_own_defaults().unwrap();
    object.attributes().link_to(&linked);
}
#[test]
fn textured_material_reset_retains_native_ownership_color_and_refreshes_mode3() {
    let object = TexturedStateFixture::new();
    prepare_appearance_drawer(&object);
    object.set_texture_map_off();
    appearance_presentations(&object);
    ais_textured_shape_set_color(&*object, Color::named(NamedColor::RED)).unwrap();
    let gold = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold);
    ais_textured_shape_set_material(&*object, &gold).unwrap();
    let previous = object.base.fields.read().aspect.clone();
    compare_native_appearance(&object, &previous, "material");
    ais_textured_shape_unset_material(&*object).unwrap();
    let current = object.base.fields.read().aspect.clone();
    compare_native_appearance(&object, &current, "unsetMaterial");
    assert_ne!(current, previous);
}
fn appearance_presentations(
    object: &Handle<TexturedStateFixture>,
) -> (Handle<PrsMgrPresentation>, Handle<PrsMgrPresentation>) {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let mode0 = PrsMgrPresentation::new(&manager, &object.cast(), 0).unwrap();
    let mode3 = PrsMgrPresentation::new(&manager, &object.cast(), 3).unwrap();
    mode0.new_group().unwrap();
    mode3.new_group().unwrap();
    object.presentable_base().fields.write().presentations = vec![mode0.clone(), mode3.clone()];
    (mode0, mode3)
}

fn compare_native_appearance(
    object: &TexturedStateFixture,
    aspect: &Graphic3dAspectFillArea3d,
    label: &str,
) {
    let expected = native_appearance_values(label);
    let rgb = aspect.interior_color().values(ColorSpace::Rgb);
    let actual = [
        f64::from(i32::from(object.has_color())),
        f64::from(i32::from(object.has_material())),
        rgb[0],
        rgb[1],
        rgb[2],
    ];
    assert_eq!(actual.as_slice(), expected, "native appearance {label}");
}

fn native_appearance_values(label: &str) -> Vec<f64> {
    let native = include_str!("fixtures/ais_textured_shape/native-appearance.txt");
    let prefix = format!("MARTENSITE_APPEARANCE {label} ");
    native
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap()
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect()
}
