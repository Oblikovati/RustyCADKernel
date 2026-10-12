//! Real owned object replay: partial failure, inherited virtual dispatch, custom flags and NaN repair.
use super::{
    colored_shape_appearance_snapshot::*, colored_shape_expansion_inputs::ColoredExpansionInputs, *,
};
use crate::prs_mgr::*;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkservice::graphic3d::*;
fn appearance_object(input: &ColoredExpansionInputs) -> Handle<AisColoredShape> {
    let object = AisColoredShape::new(&input.base);
    object.attributes().setup_own_defaults().unwrap();
    object
}
fn custom_appearance_cases(rows: &mut String, input: &ColoredExpansionInputs) {
    let object = appearance_object(input);
    null_custom_cases(rows, &object);
    object
        .set_custom_color(&input.edges[0], Color::named(NamedColor::RED))
        .unwrap();
    let drawer = object.custom_aspects(&input.edges[0]).unwrap();
    appearance_drawer(rows, "custom-color", &drawer);
    appearance_failure(
        rows,
        "custom-width-failure",
        object.set_custom_width(&input.edges[0], -2.0),
    );
    appearance_drawer(rows, "after-width-failure", &drawer);
    custom_transparency_cases(rows, input, &object, &drawer);
}
fn custom_transparency_cases(
    rows: &mut String,
    input: &ColoredExpansionInputs,
    object: &AisColoredShape,
    drawer: &AisColoredDrawer,
) {
    object.set_custom_width(&input.edges[0], 2.5).unwrap();
    appearance_failure(
        rows,
        "custom-transparency-failure",
        object.set_custom_transparency(&input.edges[0], 2.0),
    );
    appearance_drawer(rows, "after-transparency-failure", drawer);
    object
        .set_custom_transparency(&input.edges[0], 0.4)
        .unwrap();
    appearance_drawer(rows, "custom-repaired", drawer);
    appearance_parent(rows, "custom-parent", object);
}
fn global_appearance_cases(rows: &mut String, input: &ColoredExpansionInputs) {
    let object = appearance_object(input);
    let shared = object.custom_aspects(&input.edges[0]).unwrap();
    let explicit = object.custom_aspects(&input.edges[1]).unwrap();
    shared.setup_own_shading_aspect(&Handle::default()).unwrap();
    shared.set_own_line_aspects(&Handle::default()).unwrap();
    shared.setup_own_point_aspect(&Handle::default()).unwrap();
    object
        .set_custom_color(&input.edges[1], Color::named(NamedColor::RED))
        .unwrap();
    object.set_custom_width(&input.edges[1], 2.5).unwrap();
    object
        .set_custom_transparency(&input.edges[1], 0.4)
        .unwrap();
    apply_global_appearance(&object);
    appearance_drawer(rows, "global-shared", &shared);
    appearance_drawer(rows, "global-explicit", &explicit);
    appearance_parent(rows, "global-parent", &object);
    unset_global_appearance(rows, &object, &shared, &explicit);
}
fn apply_global_appearance(object: &AisColoredShape) {
    object.set_color(Color::named(NamedColor::GREEN)).unwrap();
    object.set_width(4.0).unwrap();
    object.set_transparency(0.7).unwrap();
    object
        .set_material(&Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold))
        .unwrap();
}
fn unset_global_appearance(
    rows: &mut String,
    object: &AisColoredShape,
    shared: &AisColoredDrawer,
    explicit: &AisColoredDrawer,
) {
    object.unset_width().unwrap();
    object.unset_transparency().unwrap();
    appearance_drawer(rows, "unset-shared", shared);
    appearance_drawer(rows, "unset-explicit", explicit);
    appearance_parent(rows, "unset-parent", object);
    material_owned_cases(rows, object, shared, explicit);
}
fn global_failure_cases(rows: &mut String, input: &ColoredExpansionInputs) {
    let object = appearance_object(input);
    let child = object.custom_aspects(&input.edges[0]).unwrap();
    child.set_own_line_aspects(&Handle::default()).unwrap();
    appearance_failure(rows, "global-width-failure", object.set_width(-2.0));
    appearance_parent(rows, "before-parent-width", &object);
    appearance_drawer(rows, "width-child", &child);
    object.set_width(3.0).unwrap();
    child.setup_own_shading_aspect(&Handle::default()).unwrap();
    appearance_failure(
        rows,
        "global-transparency-failure",
        object.set_transparency(2.0),
    );
    appearance_parent(rows, "before-parent-transparency", &object);
    nan_transparency_repair(rows, &object, &child);
}
#[test]
fn actual_colored_shape_custom_and_global_appearance_replays_native_failures_and_repairs() {
    let input = ColoredExpansionInputs::new();
    let mut rows = String::new();
    custom_appearance_cases(&mut rows, &input);
    global_appearance_cases(&mut rows, &input);
    global_failure_cases(&mut rows, &input);
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-expansion/native-appearance.txt")
    );
}

fn null_custom_cases(rows: &mut String, object: &AisColoredShape) {
    let empty = Shape::default();
    object
        .set_custom_color(&empty, Color::named(NamedColor::RED))
        .unwrap();
    object.set_custom_width(&empty, -2.0).unwrap();
    object.set_custom_transparency(&empty, 2.0).unwrap();
    appearance_parent(rows, "null", object);
}

fn nan_transparency_repair(rows: &mut String, object: &AisColoredShape, child: &AisColoredDrawer) {
    object.set_transparency(f64::NAN).unwrap();
    appearance_drawer(rows, "nan-child", child);
    appearance_parent(rows, "nan-parent", object);
    object.set_transparency(0.25).unwrap();
    appearance_drawer(rows, "global-repaired", child);
}

fn material_owned_cases(
    rows: &mut String,
    object: &AisColoredShape,
    shared: &AisColoredDrawer,
    explicit: &AisColoredDrawer,
) {
    explicit.set_own_material();
    object
        .set_material(&Graphic3dMaterialAspect::new(
            Graphic3dNameOfMaterial::Silver,
        ))
        .unwrap();
    appearance_drawer(rows, "material-shared", shared);
    appearance_drawer(rows, "material-owned", explicit);
}
