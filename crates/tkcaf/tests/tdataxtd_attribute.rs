//! 1:1 translation of src/ApplicationFramework/TKCAF/GTests/TDataXtd_Attribute_Test.cxx.
#![allow(non_snake_case)]
mod support;

use rustycad_tkcaf::tdata_xtd::{
    TDataXtdAxis, TDataXtdGeometry, TDataXtdPlane, TDataXtdPoint, TDataXtdShape,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use rustycad_tkmath::gp::{Axis1, Direction3, Line3, Plane3, Point3};

/// `NewDocument()`: a `BinOcaf` document with an undo limit of 10.
fn new_document() -> Handle<TDocStdDocument> {
    let document = support::new_document();
    document.set_undo_limit(10).unwrap();
    document
}

/// `NewGeometryLabel(theDocument)`.
fn new_geometry_label(document: &Handle<TDocStdDocument>) -> TdfLabel {
    support::main_child(document, 2)
}

fn expect_point(actual: &Point3, expected: &Point3) {
    assert!(actual.is_equal(expected, 1.0e-12));
}

fn expect_axis(actual: Axis1, expected: Axis1) {
    assert!(actual.is_coaxial(expected, 1.0e-12, 1.0e-12));
}

fn expect_plane(actual: &Plane3, expected: &Plane3) {
    let actual = actual.coefficients().unwrap();
    let expected = expected.coefficients().unwrap();
    for (actual, expected) in actual.iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= 1.0e-12,
            "{actual} != {expected}"
        );
    }
}

fn line(location: [f64; 3], direction: [f64; 3]) -> Line3 {
    let [x, y, z] = direction;
    Line3::new(
        Point3::new(location[0], location[1], location[2]),
        Direction3::new(x, y, z).unwrap(),
    )
}

fn plane(location: [f64; 3], normal: [f64; 3]) -> Plane3 {
    let [x, y, z] = normal;
    Plane3::new(
        Point3::new(location[0], location[1], location[2]),
        Direction3::new(x, y, z).unwrap(),
    )
    .unwrap()
}

// caf/basic/G1: point geometry is stored as a construction shape.
#[test]
fn TDataXtd_Attribute_Test__CafBasic_G1_PointSetGet() {
    let document = new_document();
    let label = new_geometry_label(&document);
    let expected = Point3::new(10.0, 20.0, 30.0);
    assert!(
        TDataXtdPoint::set_point(&label, expected)
            .unwrap()
            .is_some()
    );

    let mut actual = Point3::default();
    assert!(TDataXtdGeometry::point_of_label(&label, &mut actual).unwrap());
    expect_point(&actual, &expected);
}

// caf/basic/G2: setting a new point replaces the construction geometry.
#[test]
fn TDataXtd_Attribute_Test__CafBasic_G2_PointUndo() {
    let document = new_document();
    let label = new_geometry_label(&document);
    let expected = Point3::new(10.0, 20.0, 30.0);
    let replacement = Point3::new(-10.0, -20.0, -30.0);
    TDataXtdPoint::set_point(&label, expected).unwrap();
    document.new_command().unwrap();
    TDataXtdPoint::set_point(&label, replacement).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();

    let mut actual = Point3::default();
    assert!(TDataXtdGeometry::point_of_label(&label, &mut actual).unwrap());
    expect_point(&actual, &expected);
}

// caf/basic/H1: axis construction geometry is retrievable from its label.
#[test]
fn TDataXtd_Attribute_Test__CafBasic_H1_AxisSetGet() {
    let document = new_document();
    let label = new_geometry_label(&document);
    let expected = line([10.0, 20.0, 30.0], [100.0, 200.0, 300.0]);
    assert!(TDataXtdAxis::set_line(&label, &expected).unwrap().is_some());

    let mut actual = Axis1::default();
    assert!(TDataXtdGeometry::axis_of_label(&label, &mut actual).unwrap());
    expect_axis(actual, expected.position());
}

#[test]
fn TDataXtd_Attribute_Test__CafBasic_H2_AxisUndo() {
    let document = new_document();
    let label = new_geometry_label(&document);
    let expected = line([10.0, 20.0, 30.0], [100.0, 200.0, 300.0]);
    let replacement = line([-10.0, -20.0, -30.0], [-100.0, 200.0, 300.0]);
    TDataXtdAxis::set_line(&label, &expected).unwrap();
    document.new_command().unwrap();
    TDataXtdAxis::set_line(&label, &replacement).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();

    let mut actual = Axis1::default();
    assert!(TDataXtdGeometry::axis_of_label(&label, &mut actual).unwrap());
    expect_axis(actual, expected.position());
}

// caf/basic/I1-I2: plane construction geometry is retrievable and undoable.
#[test]
fn TDataXtd_Attribute_Test__CafBasic_I1_PlaneSetGet() {
    let document = new_document();
    let label = new_geometry_label(&document);
    let expected = plane([10.0, 20.0, 30.0], [-1.0, 0.0, 0.0]);
    assert!(
        TDataXtdPlane::set_plane(&label, &expected)
            .unwrap()
            .is_some()
    );

    let mut actual = Plane3::default();
    assert!(TDataXtdGeometry::plane_of_label(&label, &mut actual).unwrap());
    expect_plane(&actual, &expected);
}

#[test]
fn TDataXtd_Attribute_Test__CafBasic_I2_PlaneUndo() {
    let document = new_document();
    let label = new_geometry_label(&document);
    let expected = plane([10.0, 20.0, 30.0], [-1.0, 0.0, 0.0]);
    let replacement = plane([-10.0, -20.0, -30.0], [0.0, -1.0, 0.0]);
    TDataXtdPlane::set_plane(&label, &expected).unwrap();
    document.new_command().unwrap();
    TDataXtdPlane::set_plane(&label, &replacement).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();

    let mut actual = Plane3::default();
    assert!(TDataXtdGeometry::plane_of_label(&label, &mut actual).unwrap());
    expect_plane(&actual, &expected);
}

// caf/named_shape/B1: NewShape stores and restores an associated shape.
#[test]
fn TDataXtd_Attribute_Test__CafNamedShape_B1_NewShapeUndoRedo() {
    let document = new_document();
    let mut label = new_geometry_label(&document);
    let shape = support::make_box(100.0, 200.0, 300.0);

    document.new_command().unwrap();
    assert!(TDataXtdShape::set(&label, &shape).unwrap().is_some());
    document.new_command().unwrap();
    document.undo().unwrap();
    let mut attribute: Handle<TDataXtdShape> = Handle::default();
    label = document.main().unwrap().find_child(2, false).unwrap();
    assert!(!TDataXtdShape::find(&label, &mut attribute).unwrap());

    document.redo().unwrap();
    label = document.main().unwrap().find_child(2, false).unwrap();
    assert!(TDataXtdShape::find(&label, &mut attribute).unwrap());
    assert!(TDataXtdShape::get(&label).unwrap().is_same(&shape));
    document.clear_undos();
    document.clear_redos();
}
