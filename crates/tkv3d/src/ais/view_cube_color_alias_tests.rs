//! Exact native reference identity survives material-type changes without reselecting the color field.
use super::{
    view_cube_appearance_fixture::CubeAppearanceScene, view_cube_color_reference::CubeColorLeaf,
    view_cube_interaction_fixture::widget_rows, *,
};
use crate::prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers};
use rustycad_tkernel::quantity::{Color, ColorSpace};
use rustycad_tkservice::graphic3d::*;
const ALIASES: &str = include_str!("../../tests/fixtures/view-cube-appearance-aliases.txt");
#[test]
fn native_color_aliases_bind_fixed_material_interior_and_text_subfields() {
    let scene = CubeAppearanceScene::new();
    let cube = &scene.scene.cube;
    let fill = cube
        .attributes()
        .shading_aspect()
        .aspect()
        .cast::<dyn Graphic3dAspectsSource>();
    let base = cube.view_cube_presentation_base();
    let comparison = AisViewCubeColorRef::field(&fill, CubeColorLeaf::FrontInterior, base);
    let original = cube.box_color_ref();
    assert_color_alias_row(&original, &cube.box_color_ref(), &comparison, 0);
    cube.set_box_color(alias_color(0.2, 0.4, 0.6)).unwrap();
    assert_color_alias_row(&original, &cube.box_color_ref(), &comparison, 1);
    cube.set_material(&Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold))
        .unwrap();
    assert_color_alias_row(&original, &cube.box_color_ref(), &comparison, 2);
    replay_color_material_switch(cube, &fill, &comparison);
    replay_color_back_and_text(cube);
}
fn replay_color_material_switch(
    cube: &AisViewCube,
    fill: &rustycad_tkernel::standard::Handle<dyn Graphic3dAspectsSource>,
    interior: &AisViewCubeColorRef<'_>,
) {
    let retained = cube.box_color_ref();
    let material_field = AisViewCubeColorRef::field(
        fill,
        CubeColorLeaf::FrontMaterial,
        cube.view_cube_presentation_base(),
    );
    let mut aspect = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::UserDefined);
    aspect.set_material_type(Graphic3dTypeOfMaterial::Aspect);
    cube.set_material(&aspect).unwrap();
    assert_color_alias_row(&retained, &cube.box_color_ref(), &material_field, 3);
    cube.set_box_color(alias_color(0.7, 0.2, 0.1)).unwrap();
    assert_color_alias_row(&retained, &cube.box_color_ref(), &material_field, 4);
    let retained_interior = cube.box_color_ref();
    cube.set_material(&Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold))
        .unwrap();
    assert_color_alias_row(&retained_interior, &cube.box_color_ref(), interior, 7);
}
fn replay_color_back_and_text(cube: &AisViewCube) {
    let fill = cube
        .attributes()
        .shading_aspect()
        .aspect()
        .cast::<dyn Graphic3dAspectsSource>();
    let retained = cube.inner_color_ref();
    cube.set_inner_color(alias_color(0.1, 0.8, 0.2)).unwrap();
    let material = AisViewCubeColorRef::field(
        &fill,
        CubeColorLeaf::BackMaterial,
        cube.view_cube_presentation_base(),
    );
    assert_color_alias_row(&retained, &cube.inner_color_ref(), &material, 5);
    replay_cube_text_color(cube);
}
fn replay_cube_text_color(cube: &AisViewCube) {
    let text = cube
        .attributes()
        .text_aspect()
        .aspect()
        .cast::<dyn Graphic3dAspectsSource>();
    let text_color = cube.text_color_ref();
    cube.set_text_color(alias_color(0.8, 0.1, 0.4));
    let field = AisViewCubeColorRef::field(
        &text,
        CubeColorLeaf::FrontInterior,
        cube.view_cube_presentation_base(),
    );
    assert_color_alias_row(&text_color, &cube.text_color_ref(), &field, 6);
}
fn assert_color_alias_row(
    old: &AisViewCubeColorRef<'_>,
    current: &AisViewCubeColorRef<'_>,
    comparison: &AisViewCubeColorRef<'_>,
    step: i32,
) {
    let row = widget_rows(ALIASES, "color")
        .find(|row| row[0].parse::<i32>().unwrap() == step)
        .unwrap();
    assert_eq!(old.is_same_reference(current), row[1] == "1");
    assert_eq!(old.is_same_reference(comparison), row[2] == "1");
    let actual = [old.value(), current.value()]
        .into_iter()
        .flat_map(|color| [color.red(), color.green(), color.blue()])
        .collect::<Vec<_>>();
    for (actual, expected) in actual.iter().zip(&row[3..]) {
        assert!((actual - expected.parse::<f64>().unwrap()).abs() < 1e-6);
    }
    assert_eq!(old.aspect_address(), comparison.aspect_address());
}
fn alias_color(red: f64, green: f64, blue: f64) -> Color {
    Color::new(red, green, blue, ColorSpace::Rgb).unwrap()
}
