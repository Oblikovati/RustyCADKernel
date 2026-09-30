//! Behavior tests of BRepBuilderAPI_Sewing after the cases of the upstream suite
//! `BRepBuilderAPI_Sewing_Test.cxx` (TKOffset GTests), which is not mapped: its box
//! comes from BRepPrimAPI_MakeBox (TKPrim), which the port does not provide. The
//! unchanged suite passes natively against the shared build. Here the same box is built
//! with BRep_Builder by the probe's shape reader (the record of
//! `crates/tktopalgo/tests/solid_records`), and each case checks the same property.
#[allow(dead_code)]
#[path = "../examples/tp_probe/face.rs"]
mod face;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;
#[allow(dead_code)]
#[path = "../examples/tp_probe/shape.rs"]
mod shape;
#[allow(dead_code)]
#[path = "../examples/tp_probe/shape_geometry.rs"]
mod shape_geometry;
mod solid_records;

use print::Reader;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPISewing;
use solid_records::BOX10;

/// The faces of the box [0, 10]^3 in explorer order.
fn box_faces() -> Vec<Shape> {
    let solid = shape::tp_shape(&mut Reader::new(&BOX10)).expect("box record");
    Explorer::new(&solid.shape, ShapeEnum::Face, ShapeEnum::Shape)
        .expect("explorer")
        .collect()
}

/// The sewing of the first two faces of the box.
fn sew_two_faces() -> BRepBuilderAPISewing {
    let faces = box_faces();
    let mut sewing = BRepBuilderAPISewing::default();
    sewing.add(&faces[0]).unwrap();
    sewing.add(&faces[1]).unwrap();
    sewing.perform(&ProgressRange::default()).unwrap();
    sewing
}

// BRepBuilderAPI_SewingTest.SewTwoFaces
#[test]
fn sew_two_faces_gives_a_shape() {
    assert!(!sew_two_faces().sewed_shape().is_null());
}

// BRepBuilderAPI_SewingTest.NbFreeEdges
#[test]
fn two_faces_have_free_edges() {
    assert!(sew_two_faces().nb_free_edges() > 0);
}

// BRepBuilderAPI_SewingTest.NbDegeneratedShapes
#[test]
fn planar_faces_have_no_degenerated_shape() {
    assert_eq!(sew_two_faces().nb_degenerated_shapes(), 0);
}

// BRepBuilderAPI_SewingTest.SetTolerance
#[test]
fn set_tolerance_is_read_back() {
    let mut sewing = BRepBuilderAPISewing::default();
    let tol = 0.01;
    sewing.set_tolerance(tol);
    assert!((sewing.tolerance() - tol).abs() <= precision::confusion());
}

// BRepBuilderAPI_SewingTest.SewAllBoxFaces
#[test]
fn all_box_faces_have_no_free_edge() {
    let mut sewing = BRepBuilderAPISewing::default();
    for face in box_faces() {
        sewing.add(&face).unwrap();
    }
    sewing.perform(&ProgressRange::default()).unwrap();
    assert!(!sewing.sewed_shape().is_null());
    assert_eq!(sewing.nb_free_edges(), 0);
}
