//! Actual native protected emission replays topology buffers and nonrestored deflection failure state.
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
#[allow(dead_code)]
#[path = "../src/ais/colored_shape_expansion_inputs.rs"]
mod geometry;
use rustycad_tkbrep::{topexp::Explorer, topods::Shape};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkservice::aspect::AspectTypeOfFacingModel;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::prs3d::Prs3dShadingAspect;
use rustycad_tkv3d::{
    ais::*,
    prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers},
    v3d::V3dViewer,
};
#[test]
fn actual_custom_emission_replays_native_buffers_hidden_infinite_and_failure_state() {
    let native = include_str!("fixtures/colored-shape-compute/native-emission.txt");
    for row in native.lines() {
        replay_emission_row(row);
    }
}
fn replay_emission_row(row: &str) {
    let fields: Vec<_> = row.split_whitespace().collect();
    let scenario = fields[1].parse::<usize>().unwrap();
    let mode = fields[2].parse::<i32>().unwrap();
    let fail = fields[3] == "1";
    let scene = ColoredEmissionFixture::new(scenario);
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(i32::from(fail));
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let result = ais_colored_shape_add_custom_props(
        &*scene.object,
        &prs.cast(),
        &scene.opened,
        &scene.closed,
        mode,
    );
    check_emission_state(&scene, &prs, &result, &fields[4..10]);
    compare_emission_buffers(&trace.read().events, &fields[10..]);
}
struct ColoredEmissionFixture {
    object: Handle<AisColoredShape>,
    base: Handle<AisColoredDrawer>,
    custom: Handle<AisColoredDrawer>,
    opened: AisColoredOpenedCompounds,
    closed: AisColoredDrawerCompounds,
}
impl ColoredEmissionFixture {
    fn new(scenario: usize) -> Self {
        let input = geometry::ColoredExpansionInputs::new();
        let shape = emission_input_shape(&input, scenario);
        let object = AisColoredShape::new(&shape);
        object.attributes().set_auto_triangulation(false);
        object.attributes().setup_own_defaults().unwrap();
        let base = Handle::new(AisColoredDrawer::new(&object.attributes()));
        let custom = Handle::new(AisColoredDrawer::new(&object.attributes()));
        let styles = emission_styles(&input, &shape, &custom, scenario);
        configure_emission_flags(&object, scenario);
        let scene = Self {
            object,
            base,
            custom,
            opened: std::array::from_fn(|_| Default::default()),
            closed: Default::default(),
        };
        dispatch_emission_fixture(&scene, &shape, &styles);
        scene
    }
}
fn configure_emission_flags(object: &AisColoredShape, scenario: usize) {
    if scenario == 3 {
        object.set_infinite_state(true);
    }
    if scenario >= 7 {
        object.attributes().set_face_boundary_draw(true);
    }
}
fn emission_input_shape(input: &geometry::ColoredExpansionInputs, scenario: usize) -> Shape {
    if scenario < 4 {
        return input.base.clone();
    }
    let mut builder = BRepPrimAPIMakeBox::from_sizes(2., 3., 4.).unwrap();
    let shape = builder.solid().unwrap().into_shape();
    BRepMeshIncrementalMesh::with_shape(&shape, 0.1, false, 0.5, false).unwrap();
    shape
}
fn initialize_emission_custom_shading(custom: &AisColoredDrawer) {
    let shading = Handle::new(Prs3dShadingAspect::new().unwrap());
    custom.set_shading_aspect(shading.clone());
    shading
        .set_color(
            Color::named(NamedColor::BLUE),
            AspectTypeOfFacingModel::BothSide,
        )
        .unwrap();
}
fn emission_styles(
    input: &geometry::ColoredExpansionInputs,
    shape: &Shape,
    custom: &Handle<AisColoredDrawer>,
    scenario: usize,
) -> AisShapeDrawerMap {
    let styles = AisShapeDrawerMap::default();
    let face = emission_custom_face(input, shape, scenario);
    if scenario >= 1 && scenario != 4 {
        styles.bind(&face, custom).unwrap();
    }
    if scenario == 2 || scenario == 8 {
        custom.set_hidden(true);
    }
    if scenario >= 5 {
        initialize_emission_custom_shading(custom);
    }
    styles
}
fn dispatch_emission_fixture(
    scene: &ColoredEmissionFixture,
    shape: &rustycad_tkbrep::topods::Shape,
    styles: &AisShapeDrawerMap,
) {
    ais_colored_shape_dispatch_colors(
        &scene.base,
        shape,
        styles,
        ShapeEnum::Compound,
        false,
        &scene.opened,
        &scene.closed,
    )
    .unwrap();
}
fn check_emission_state(
    scene: &ColoredEmissionFixture,
    prs: &Graphic3dStructure,
    result: &Result<(), Failure>,
    fields: &[&str],
) {
    assert_eq!(result.is_err(), fields[0] == "1");
    if let Err(failure) = result {
        assert_eq!(failure.exception_type(), "Standard_ProgramError");
    }
    assert_eq!(prs.groups().len(), fields[1].parse::<usize>().unwrap());
    check_emission_drawer(&scene.base, &fields[2..4]);
    check_emission_drawer(&scene.custom, &fields[4..6]);
}
fn check_emission_drawer(drawer: &AisColoredDrawer, fields: &[&str]) {
    assert_eq!(
        drawer.type_of_deflection() as i32,
        fields[0].parse::<i32>().unwrap()
    );
    assert_eq!(drawer.has_own_type_of_deflection(), fields[1] == "1");
}

fn compare_emission_buffers(events: &[f64], fields: &[&str]) {
    assert_eq!(fields[0].parse::<usize>().unwrap(), events.len());
    let expected: Vec<f64> = fields[1..]
        .iter()
        .map(|field| field.parse().unwrap())
        .collect();
    assert_eq!(expected.len(), events.len());
    for (index, (actual, native)) in events.iter().zip(expected).enumerate() {
        assert!(
            (actual - native).abs() < 1e-7,
            "event={index} actual={actual}; expected {native}"
        );
    }
}

#[test]
fn actual_colored_compute_replays_native_modes_bounds_null_and_failures() {
    let native = include_str!("fixtures/colored-shape-compute/native-compute.txt");
    for row in native.lines() {
        replay_compute_row(row);
    }
}
fn replay_compute_row(row: &str) {
    let fields: Vec<_> = row.split_whitespace().collect();
    let scenario = fields[1].parse::<usize>().unwrap();
    let mode = fields[2].parse::<i32>().unwrap();
    let fail = fields[3] == "1";
    let object = compute_fixture_object(scenario);
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(i32::from(fail));
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let result = object.compute(&Handle::default(), &prs.cast(), mode);
    assert_eq!(result.is_err(), fields[4] == "1", "{row}");
    assert_eq!(prs.groups().len(), fields[5].parse::<usize>().unwrap());
    assert_eq!(prs.is_infinite(), fields[6] == "1");
    assert_eq!(
        prs.display_priority() as i32,
        fields[7].parse::<i32>().unwrap()
    );
    compare_emission_buffers(&trace.read().events, &fields[8..]);
}
fn compute_fixture_object(scenario: usize) -> Handle<AisColoredShape> {
    let input = geometry::ColoredExpansionInputs::new();
    let shape = compute_fixture_shape(&input, scenario);
    let object = AisColoredShape::new(&shape);
    object.attributes().set_auto_triangulation(false);
    object.attributes().setup_own_defaults().unwrap();
    if scenario == 3 {
        object.set_infinite_state(true);
    }
    if (7..=8).contains(&scenario) {
        object.attributes().set_face_boundary_draw(true);
    }
    configure_compute_custom(&object, &input, &shape, scenario);
    object
}
fn compute_fixture_shape(input: &geometry::ColoredExpansionInputs, scenario: usize) -> Shape {
    if scenario == 9 {
        return Shape::default();
    }
    if scenario == 10 {
        let mut empty = rustycad_tkbrep::topods::Compound::default();
        rustycad_tkbrep::topods::Builder.make_compound(&mut empty);
        return empty.into_shape();
    }
    emission_input_shape(input, scenario)
}
fn configure_compute_custom(
    object: &AisColoredShape,
    input: &geometry::ColoredExpansionInputs,
    shape: &Shape,
    scenario: usize,
) {
    if scenario < 1 || scenario == 4 || scenario >= 9 {
        return;
    }
    let face = emission_custom_face(input, shape, scenario);
    let custom = object.custom_aspects(&face).unwrap();
    if scenario == 2 || scenario == 8 {
        custom.set_hidden(true);
    }
    if scenario >= 5 {
        initialize_emission_custom_shading(&custom);
    }
}
fn emission_custom_face(
    input: &geometry::ColoredExpansionInputs,
    shape: &Shape,
    scenario: usize,
) -> Shape {
    if scenario < 4 {
        return input.faces[0].clone();
    }
    Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone()
}

#[test]
fn native_driver_observed_graphics_ownership_matches_actual_const_handle_arguments() {
    use std::fmt::Write;
    let mut rows = String::new();
    for mode in [0, 1] {
        writeln!(rows, "mode {mode}").unwrap();
        record_native_driver_counts(mode, &mut rows);
    }
    assert_eq!(
        rows,
        include_str!("fixtures/colored-shape-compute/native-graphics-counts.txt")
    );
}

fn record_native_driver_counts(mode: i32, rows: &mut String) {
    let object = compute_fixture_object(4);
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    object
        .compute(&Handle::default(), &prs.cast(), mode)
        .unwrap();
    for count in &trace.read().ownership_counts {
        record_graphic_owner_count(rows, count);
    }
}
fn record_graphic_owner_count(rows: &mut String, count: &[i32; 5]) {
    use std::fmt::Write;
    writeln!(
        rows,
        "count {} {} {} {} {}",
        count[0], count[1], count[2], count[3], count[4]
    )
    .unwrap();
}
