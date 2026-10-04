//! Differential tests for the directly supported no-cap ThickSolid branch.
use rustycad_tkbrep::{topexp::Explorer, topods::Shape};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::{geom_abs::JoinType, gp::Plane3};
use rustycad_tkoffset::brep_offset::{BRepOffsetError, BRepOffsetMakeOffset, BRepOffsetMode};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

#[test]
fn no_cap_thick_solid_preserves_native_offset_orientation_and_failure_state() {
    assert_thick_solid(2.0, true, Orientation::Reversed);
    assert_thick_solid(-2.0, true, Orientation::Forward);
    assert_thick_solid(0.0, false, Orientation::External);
}

#[test]
fn unsupported_closing_face_reports_partial_port_without_mutating_shape() {
    let face = test_face();
    let mut maker = initialized(&face.clone().into_shape(), 2.0);
    maker.add_face(&face);

    let error = maker.make_thick_solid().unwrap_err();

    assert_eq!(error.exception_type(), "Standard_NotImplemented");
    assert!(maker.shape().is_null());
    assert!(!maker.is_done());
}

fn assert_thick_solid(offset: f64, expected_done: bool, expected_orientation: Orientation) {
    let source = test_face().into_shape();
    let mut maker = initialized(&source, offset);

    maker.make_thick_solid().unwrap();

    assert_eq!(maker.is_done(), expected_done);
    if !expected_done {
        assert_eq!(maker.error(), Some(BRepOffsetError::NullOffset));
        assert!(maker.shape().is_null());
        return;
    }
    assert_eq!(maker.error(), Some(BRepOffsetError::NoError));
    assert_eq!(maker.shape().orientation(), expected_orientation);
    assert_eq!(result_face_orientation(maker.shape()), expected_orientation);
}

fn result_face_orientation(shape: &Shape) -> Orientation {
    let faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    assert!(faces.more());
    faces.current().orientation()
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

fn test_face() -> rustycad_tkbrep::topods::Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.0, 1.0, 0.0, 1.0))
        .unwrap()
        .face()
        .unwrap()
}
