//! Native differential checks for the input gates of `BRepOffset_MakeOffset`.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::Explorer,
    topods::Shape,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::{
    geom_abs::JoinType,
    gp::{Direction3, Plane3, Point3},
};
use rustycad_tkoffset::brep_offset::{BRepOffsetError, BRepOffsetMakeOffset, BRepOffsetMode};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::{
    brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape},
    brep_check::BRepCheckAnalyzer,
    brep_gprop::BRepGProp,
};

#[test]
fn null_global_offset_fails_and_clears_bad_shape() {
    let face = plane_face(0.0);
    let mut maker = initialized(&face.clone().into_shape(), 0.0);

    assert!(!maker.check_input_data().unwrap());
    assert_eq!(maker.error(), Some(BRepOffsetError::NullOffset));
    assert!(maker.shape().is_null());
    assert!(maker.bad_shape().is_null());
}

#[test]
fn nonzero_face_override_allows_zero_global_offset() {
    let face = plane_face(0.0);
    let mut maker = initialized(&face.clone().into_shape(), 0.0);
    maker.set_offset_on_face(&face, 1.0);

    assert!(maker.check_input_data().unwrap());
    assert_eq!(maker.error(), Some(BRepOffsetError::NoError));
}

#[test]
fn disconnected_faces_fail_during_standard_offset_precheck() {
    let first = plane_face(0.0);
    let second = plane_face(5.0);
    let shape = disconnected(&first.clone().into_shape(), &second.clone().into_shape());
    let mut maker = initialized(&shape, 1.0);

    maker.make_offset_shape().unwrap();
    assert!(!maker.is_done());
    assert_eq!(maker.error(), Some(BRepOffsetError::NotConnectedShell));
    assert!(maker.bad_shape().is_null());
}

#[test]
fn one_plane_face_offset_builds_the_native_open_shell() {
    assert_single_plane_offset(plane_face(0.0), 2.0, 2.0);
}

#[test]
fn reversed_face_and_negative_offset_use_oriented_plane_normal() {
    let face = plane_face(0.0);
    let reversed = rustycad_tkbrep::topods::Face::unchecked(face.reversed());
    assert_single_plane_offset(reversed, 2.0, -2.0);
    assert_single_plane_offset(plane_face(0.0), -2.0, -2.0);
}

#[test]
fn unsupported_join_mode_returns_a_named_partial_port_error() {
    let face = plane_face(0.0);
    let mut maker = BRepOffsetMakeOffset::new();
    maker.initialize(
        &face.clone().into_shape(),
        2.0,
        1.0e-7,
        BRepOffsetMode::Skin,
        false,
        false,
        JoinType::Intersection,
        false,
        false,
    );

    let error = maker.make_offset_shape().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NotImplemented");
    assert!(!maker.is_done());
    assert!(maker.shape().is_null());
}

#[test]
fn multiple_faces_stop_at_the_explicit_single_face_boundary() {
    let mut source = BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0).unwrap();
    let shape = source.shape().unwrap().clone();
    let mut maker = initialized(&shape, 0.5);

    let error = maker.make_offset_shape().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NotImplemented");
    assert!(error.message().contains("exactly one source face"));
    assert!(!maker.is_done());
    assert!(maker.shape().is_null());
}

fn assert_single_plane_offset(face: rustycad_tkbrep::topods::Face, offset: f64, z: f64) {
    let mut maker = initialized(&face.into_shape(), offset);
    maker.make_offset_shape().unwrap();
    assert!(maker.is_done());
    assert_eq!(maker.error(), Some(BRepOffsetError::NoError));
    assert_shell_topology(maker.shape());
    let offset_face = first_face(maker.shape());
    assert_offset_sample(&offset_face, z);
    assert_shell_geometry(maker.shape(), &offset_face);
}

fn assert_shell_topology(shape: &Shape) {
    assert_eq!(shape.shape_type().unwrap(), ShapeEnum::Shell);
    assert_eq!(count(shape, ShapeEnum::Face), 1);
    assert_eq!(count(shape, ShapeEnum::Shell), 1);
}

fn first_face(shape: &Shape) -> rustycad_tkbrep::topods::Face {
    let faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    rustycad_tkbrep::topods::Face::unchecked(faces.current().clone())
}

fn assert_offset_sample(face: &rustycad_tkbrep::topods::Face, expected_z: f64) {
    let sample = BRepTool::surface(face)
        .unwrap()
        .get()
        .unwrap()
        .d0(0.5, 0.5)
        .unwrap();
    assert!((sample.z() - expected_z).abs() < 1.0e-12);
}

fn assert_shell_geometry(shape: &Shape, face: &rustycad_tkbrep::topods::Face) {
    let mut properties = GProps::new();
    BRepGProp::surface_properties(shape, &mut properties, false, false).unwrap();
    assert!((properties.mass() - 1.0).abs() < 1.0e-12);
    assert!(
        BRepCheckAnalyzer::new(&face.clone().into_shape(), true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    assert!(
        BRepCheckAnalyzer::new(shape, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}

fn initialized(shape: &Shape, offset: f64) -> BRepOffsetMakeOffset {
    let mut maker = BRepOffsetMakeOffset::new();
    maker.initialize(
        shape,
        offset,
        1.0e-7,
        BRepOffsetMode::Skin,
        false,
        false,
        JoinType::Arc,
        false,
        false,
    );
    maker
}

fn plane_face(z: f64) -> rustycad_tkbrep::topods::Face {
    let plane = Plane3::new(
        Point3::new(0.0, 0.0, z),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    BRepBuilderAPIMakeFace::from_plane_bounds(plane, (0.0, 1.0, 0.0, 1.0))
        .unwrap()
        .face()
        .unwrap()
}

fn disconnected(first: &Shape, second: &Shape) -> Shape {
    let mut compound = rustycad_tkbrep::topods::Compound::default();
    let builder = BRepBuilder::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, first).unwrap();
    builder.add(&mut compound, second).unwrap();
    compound.into_shape()
}

fn count(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while explorer.more() {
        count += 1;
        explorer.next().unwrap();
    }
    count
}
