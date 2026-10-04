//! Native differential checks for the input gates of `BRepOffset_MakeOffset`.
use rustycad_tkbrep::{brep::BRepBuilder, topods::Shape};
use rustycad_tkmath::{
    geom_abs::JoinType,
    gp::{Direction3, Plane3, Point3},
};
use rustycad_tkoffset::brep_offset::{BRepOffsetError, BRepOffsetMakeOffset, BRepOffsetMode};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

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
fn disconnected_faces_fail_before_surface_sampling() {
    let first = plane_face(0.0);
    let second = plane_face(5.0);
    let shape = disconnected(&first.clone().into_shape(), &second.clone().into_shape());
    let mut maker = initialized(&shape, 1.0);

    assert!(!maker.check_input_data().unwrap());
    assert_eq!(maker.error(), Some(BRepOffsetError::NotConnectedShell));
    assert!(maker.bad_shape().is_null());
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
