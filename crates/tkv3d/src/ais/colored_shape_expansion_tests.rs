//! Actual native aggregate/explicit precedence, append behavior, orientation and visibility matrix.
use super::colored_shape_expansion_inputs::ColoredExpansionInputs;
use super::*;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use std::fmt::Write;
fn expansion_style(
    map: &AisShapeDrawerMap,
    key: &Shape,
    first: &Handle<AisColoredDrawer>,
    second: &Handle<AisColoredDrawer>,
) -> i32 {
    if !map.is_bound(key) {
        return 0;
    }
    let address = map.node_ref(key).unwrap().drawer_address().unwrap();
    if address == first.object_address() {
        return 1;
    }
    if address == second.object_address() {
        return 2;
    }
    3
}
fn expansion_row(
    rows: &mut String,
    label: &str,
    map: &AisShapeDrawerMap,
    input: &ColoredExpansionInputs,
    first: &Handle<AisColoredDrawer>,
    second: &Handle<AisColoredDrawer>,
) {
    write!(rows, "{label} {}", map.len()).unwrap();
    for key in input.report_keys() {
        write!(rows, " {}", expansion_style(map, key, first, second)).unwrap();
    }
    writeln!(rows).unwrap();
}
fn aggregate_cases(rows: &mut String, input: &ColoredExpansionInputs) {
    let object = AisColoredShape::new(&input.base);
    let first = object.custom_aspects(&input.own).unwrap();
    let second = object.custom_aspects(&input.external).unwrap();
    for (label, key) in [
        ("wire", &input.wire),
        ("shell", &input.shell),
        ("solid", &input.solid),
        ("compsolid", &input.compsolid),
    ] {
        let output = AisShapeDrawerMap::default();
        object.bind_sub_shapes(&output, key, &first).unwrap();
        expansion_row(rows, label, &output, input, &first, &second);
    }
}
fn explicit_cases(rows: &mut String, input: &ColoredExpansionInputs) {
    let object = AisColoredShape::new(&input.base);
    let first = object.custom_aspects(&input.own).unwrap();
    let second = object.custom_aspects(&input.external).unwrap();
    let output = AisShapeDrawerMap::default();
    output.bind(&input.faces[0], &second).unwrap();
    object
        .bind_sub_shapes(&output, &input.solid, &first)
        .unwrap();
    expansion_row(rows, "preserved-leaf", &output, input, &first, &second);
    object
        .bind_sub_shapes(&output, &input.faces[0], &first)
        .unwrap();
    expansion_row(rows, "explicit-leaf", &output, input, &first, &second);
    append_explicit_vertices(rows, input, &object, &output, &first, &second);
}
fn external_cases(rows: &mut String, input: &ColoredExpansionInputs) {
    let object = AisColoredShape::new(&input.base);
    let first = object.custom_aspects(&input.external).unwrap();
    let second = object.custom_aspects(&input.edges[0]).unwrap();
    let output = AisShapeDrawerMap::default();
    object.fill_subshape_drawer_map(&output).unwrap();
    expansion_row(rows, "external-explicit", &output, input, &first, &second);
    object.clear_custom_aspects().unwrap();
    object
        .change_custom_aspects_map()
        .bind(&input.own, &first)
        .unwrap();
    object.fill_subshape_drawer_map(&output).unwrap();
    expansion_row(rows, "own-compound-append", &output, input, &first, &second);
}
fn orientation_cases(rows: &mut String, input: &ColoredExpansionInputs) {
    let object = AisColoredShape::new(&input.base);
    let first = object.custom_aspects(&input.base).unwrap();
    let output = AisShapeDrawerMap::default();
    object.fill_subshape_drawer_map(&output).unwrap();
    let second = Handle::default();
    expansion_row(rows, "same-base", &output, input, &first, &second);
    object.clear_custom_aspects().unwrap();
    object
        .change_custom_aspects_map()
        .bind(&input.base.reversed(), &first)
        .unwrap();
    object.fill_subshape_drawer_map(&output).unwrap();
    expansion_row(rows, "reversed-base", &output, input, &first, &second);
}
fn visibility_cases(rows: &mut String, input: &ColoredExpansionInputs) {
    let object = AisColoredShape::new(&input.base);
    visibility_row(rows, "empty-visible", &object);
    let first = object.custom_aspects(&input.external).unwrap();
    visibility_row(rows, "outside-visible", &object);
    first.set_hidden(true);
    visibility_row(rows, "outside-hidden", &object);
    object.unset_custom_aspects(&input.external, false).unwrap();
    visibility_row(rows, "replacement-visible", &object);
}
fn visibility_row(rows: &mut String, label: &str, object: &AisColoredShape) {
    let visible = i32::from(object.is_shape_entirely_visible().unwrap());
    writeln!(rows, "{label} {visible}").unwrap();
}
#[test]
fn protected_colored_shape_expansion_replays_actual_native_topology_matrix() {
    let input = ColoredExpansionInputs::new();
    let mut rows = String::new();
    aggregate_cases(&mut rows, &input);
    explicit_cases(&mut rows, &input);
    external_cases(&mut rows, &input);
    orientation_cases(&mut rows, &input);
    visibility_cases(&mut rows, &input);
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-expansion/native-expansion.txt")
    );
}

fn append_explicit_vertices(
    rows: &mut String,
    input: &ColoredExpansionInputs,
    object: &AisColoredShape,
    output: &AisShapeDrawerMap,
    first: &Handle<AisColoredDrawer>,
    second: &Handle<AisColoredDrawer>,
) {
    object.bind_sub_shapes(output, &input.own, second).unwrap();
    object
        .bind_sub_shapes(output, &input.vertex, first)
        .unwrap();
    expansion_row(rows, "compound-vertex", output, input, first, second);
}
