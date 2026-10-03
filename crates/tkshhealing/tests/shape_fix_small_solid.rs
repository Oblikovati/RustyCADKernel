//! Native differential checks for `ShapeFix_FixSmallSolid`.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Shape, ShapeIterator, Solid, Vertex, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_fix::ShapeFixFixSmallSolid;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

fn expected(name: &str) -> (i32, i32, i32, f64) {
    let row = include_str!("../../../tools/oracle/shape_fix_small_solid.expected")
        .lines()
        .find(|row| row.starts_with(name))
        .unwrap();
    let mut values = row.split_whitespace().skip(1);
    (
        values.next().unwrap().parse().unwrap(),
        values.next().unwrap().parse().unwrap(),
        values.next().unwrap().parse().unwrap(),
        values.next().unwrap().parse().unwrap(),
    )
}

fn box_shape(x: f64, width: f64) -> Shape {
    box_dimensions(x, width, width, width)
}

fn box_dimensions(x: f64, width: f64, height: f64, depth: f64) -> Shape {
    BRepPrimAPIMakeBox::at_point(
        rustycad_tkmath::gp::Point3::new(x, 0.0, 0.0),
        width,
        height,
        depth,
    )
    .unwrap()
    .shape()
    .unwrap()
    .clone()
}

fn compound(first: &Shape, second: &Shape) -> Shape {
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    let mut shape = compound.into_shape();
    BRepBuilder::default().add(&mut shape, first).unwrap();
    BRepBuilder::default().add(&mut shape, second).unwrap();
    shape
}

fn shape_counts(shape: &Shape) -> (i32, i32, i32, f64) {
    let solids = count(shape, ShapeEnum::Solid);
    let shells = count(shape, ShapeEnum::Shell);
    let faces = count(shape, ShapeEnum::Face);
    let mut properties = GProps::new();
    BRepGProp::volume_properties(shape, &mut properties, false, false, false).unwrap();
    (solids, shells, faces, properties.mass())
}

fn count(shape: &Shape, kind: ShapeEnum) -> i32 {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut total = 0;
    while explorer.more() {
        total += 1;
        explorer.next().unwrap();
    }
    total
}

#[test]
fn remove_is_disabled_by_default() {
    let input = removal_input();
    let fixer = ShapeFixFixSmallSolid::new();
    assert_row("disabled", &fixer.remove(&input, &context()).unwrap());
    assert!(
        fixer
            .remove(&input, &Handle::default())
            .unwrap()
            .is_same(&input)
    );
}

#[test]
fn remove_uses_volume_only_mode() {
    let input = removal_input();
    let fixer = ShapeFixFixSmallSolid::new();
    fixer.set_fix_mode(2);
    fixer.set_volume_threshold(1.0);
    assert_row("volume", &fixer.remove(&input, &context()).unwrap());
}

#[test]
fn remove_uses_width_only_mode() {
    let input = removal_input();
    let fixer = ShapeFixFixSmallSolid::new();
    fixer.set_fix_mode(1);
    fixer.set_width_factor_threshold(1.0);
    assert_row("width", &fixer.remove(&input, &context()).unwrap());
}

#[test]
fn remove_resets_invalid_mode_to_both_criteria() {
    let input = removal_input();
    let fixer = ShapeFixFixSmallSolid::new();
    fixer.set_fix_mode(99);
    fixer.set_volume_threshold(0.0);
    fixer.set_width_factor_threshold(1.0);
    assert_row("invalid_mode", &fixer.remove(&input, &context()).unwrap());
}

#[test]
fn remove_in_mode_zero_requires_both_criteria_to_pass() {
    let input = removal_input();
    let fixer = ShapeFixFixSmallSolid::new();
    fixer.set_volume_threshold(1.1);
    fixer.set_width_factor_threshold(0.3);
    assert_row(
        "both_require_both",
        &fixer.remove(&input, &context()).unwrap(),
    );
}

#[test]
fn negative_and_nan_thresholds_disable_their_criteria() {
    let input = removal_input();
    let fixer = ShapeFixFixSmallSolid::new();
    fixer.set_volume_threshold(-1.0);
    fixer.set_width_factor_threshold(f64::NAN);
    assert_row(
        "negative_nan_disable",
        &fixer.remove(&input, &context()).unwrap(),
    );
}

fn removal_input() -> Shape {
    compound(&box_shape(0.0, 1.0), &box_shape(2.0, 10.0))
}

#[test]
fn invalid_shape_and_null_shape_are_returned_unchanged() {
    let fixer = ShapeFixFixSmallSolid::new();
    fixer.set_volume_threshold(100.0);
    let mut wire = Wire::default();
    BRepBuilder::default().make_wire(&mut wire);
    let wire = wire.into_shape();
    let null = Shape::default();
    assert!(fixer.remove(&null, &context()).unwrap().is_null());
    assert!(fixer.merge(&null, &context()).unwrap().is_null());
    assert!(fixer.remove(&wire, &context()).unwrap().is_same(&wire));
    assert!(fixer.merge(&wire, &context()).unwrap().is_same(&wire));
}

#[test]
fn merge_moves_shared_shell_faces_into_the_larger_solid() {
    let large = box_shape(0.0, 10.0);
    let small = box_dimensions(10.0, 1.0, 10.0, 10.0);
    let shared_small = share_contact_face(&large, &small);
    let input = compound(&large, &shared_small);
    let fixer = ShapeFixFixSmallSolid::new();
    fixer.set_fix_mode(2);
    fixer.set_volume_threshold(101.0);
    let merged = fixer.merge(&input, &context()).unwrap();
    assert_row("merge", &merged);
}

#[test]
fn merge_repeats_for_newly_adjacent_solids() {
    let large = box_shape(0.0, 10.0);
    let middle = share_contact_face(&large, &box_dimensions(10.0, 1.0, 10.0, 10.0));
    let end = share_face_at_x(&middle, &box_dimensions(11.0, 1.0, 10.0, 10.0), 11.0);
    let chain = compound(&large, &compound(&middle, &end));
    let fixer = ShapeFixFixSmallSolid::new();
    fixer.set_fix_mode(2);
    fixer.set_volume_threshold(101.0);
    assert_row("merge_chain", &fixer.merge(&chain, &context()).unwrap());
}

#[test]
fn merge_keeps_solids_without_shared_faces() {
    let large = box_shape(0.0, 10.0);
    let isolated = compound(&large, &box_shape(100.0, 1.0));
    let fixer = ShapeFixFixSmallSolid::new();
    fixer.set_fix_mode(2);
    fixer.set_volume_threshold(101.0);
    assert_row(
        "merge_isolated",
        &fixer.merge(&isolated, &context()).unwrap(),
    );
}

#[test]
fn merge_preserves_unattached_shells_from_the_small_solid() {
    let large = box_shape(0.0, 10.0);
    let small = share_contact_face(&large, &box_dimensions(10.0, 1.0, 10.0, 10.0));
    let distant = box_shape(100.0, 1.0);
    let multi_shell = two_shell_solid(&small, &distant);
    let input = compound(&large, &multi_shell);
    let fixer = ShapeFixFixSmallSolid::new();
    fixer.set_fix_mode(2);
    fixer.set_volume_threshold(102.0);
    let context = context();
    assert_row("merge_other_shell", &fixer.merge(&input, &context).unwrap());
    let replacement = context.value(&first_shell(&large)).unwrap();
    assert_row("merge_shell_replacement", &replacement);
}

#[test]
fn merge_reports_native_alias_failure_for_trailing_non_shell_child() {
    let large = box_shape(0.0, 10.0);
    let small_box = box_dimensions(10.0, 1.0, 10.0, 10.0);
    let shared_small = share_contact_face(&large, &small_box);
    let malformed = solid_with_trailing_vertex(&shared_small);
    let input = compound(&large, &malformed);
    let fixer = ShapeFixFixSmallSolid::new();
    fixer.set_fix_mode(2);
    fixer.set_volume_threshold(102.0);
    let failure = fixer.merge(&input, &context()).unwrap_err();
    assert!(failure.message().contains("expected a listed shell"));
}

fn solid_with_trailing_vertex(shell_source: &Shape) -> Shape {
    let mut malformed = Solid::default();
    BRepBuilder::default().make_solid(&mut malformed);
    let mut malformed = malformed.into_shape();
    BRepBuilder::default()
        .add(&mut malformed, &first_shell(shell_source))
        .unwrap();
    let mut trailing_vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut trailing_vertex, Point3::default(), 0.1)
        .unwrap();
    BRepBuilder::default()
        .add(&mut malformed, &trailing_vertex.into_shape())
        .unwrap();
    malformed
}

fn two_shell_solid(first: &Shape, second: &Shape) -> Shape {
    let mut solid = Solid::default();
    BRepBuilder::default().make_solid(&mut solid);
    let mut result = solid.into_shape();
    BRepBuilder::default()
        .add(&mut result, &first_shell(first))
        .unwrap();
    BRepBuilder::default()
        .add(&mut result, &first_shell(second))
        .unwrap();
    result
}

fn assert_row(name: &str, shape: &Shape) {
    let actual = shape_counts(shape);
    let expected = expected(name);
    assert_eq!(actual.0, expected.0, "solid count: {name}");
    assert_eq!(actual.1, expected.1, "shell count: {name}");
    assert_eq!(actual.2, expected.2, "face count: {name}");
    assert!(
        (actual.3 - expected.3).abs() < 1e-9,
        "volume: {name}: {actual:?}"
    );
}

fn context() -> Handle<ShapeBuildReShape> {
    Handle::new(ShapeBuildReShape::new())
}

fn share_contact_face(large: &Shape, small: &Shape) -> Shape {
    share_face_at_x(large, small, 10.0)
}

fn share_face_at_x(large: &Shape, small: &Shape, x: f64) -> Shape {
    let large_face = face_at_x(large, x);
    let small_face = face_at_x(small, x);
    let reshaper = ShapeBuildReShape::new();
    reshaper
        .replace(&small_face, &large_face.reversed())
        .unwrap();
    reshaper.apply(small, ShapeEnum::Shape).unwrap()
}

fn face_at_x(shape: &Shape, x: f64) -> Shape {
    let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while faces.more() {
        let face = topods::face(faces.current()).unwrap();
        let mut properties = GProps::new();
        BRepGProp::surface_properties(&face.clone().into_shape(), &mut properties, false, false)
            .unwrap();
        if (properties.centre_of_mass().x() - x).abs() < 1e-10 {
            return face.into_shape();
        }
        faces.next().unwrap();
    }
    panic!("no planar contact face at x={x}");
}

fn first_shell(shape: &Shape) -> Shape {
    let mut children = ShapeIterator::new(shape, true, true);
    while children.more() {
        let child = children.value().unwrap();
        if child.shape_type().unwrap() == ShapeEnum::Shell {
            return child.clone();
        }
        children.next();
    }
    panic!("shape has no direct shell: {:?}", shape.shape_type());
}
