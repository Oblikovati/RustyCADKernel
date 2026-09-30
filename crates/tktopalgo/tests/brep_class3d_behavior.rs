//! Supplemental BRepClass3d coverage for the explorer, selectors, intersector and
//! OuterShell. The five legacy cases now use the TKPrim builders in
//! brep_class3d_solid_classifier.rs; these additional topology fixtures remain here.
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
use rustycad_tkbrep::topods;
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeomalgo::int_curve_surface::TransitionOnCurve;
use rustycad_tkmath::gp::{Direction3, Line3, Point3};
use rustycad_tktopalgo::brep_class3d::{
    BRepClass3d, BRepClass3dBndBoxTreeSelectorLine, BRepClass3dBndBoxTreeSelectorPoint,
    BRepClass3dIntersector3d, BRepClass3dSClassifier, BRepClass3dSolidClassifier,
    BRepClass3dSolidExplorer, FacePointSample,
};
use shape::{TpShape, tp_shape};
use solid_records::{BOX10, HOLLOW10};

fn shape(record: &[f64]) -> TpShape {
    tp_shape(&mut Reader::new(record)).expect("shape record")
}

fn classify(record: &[f64], point: Point3) -> State {
    let solid = shape(record);
    let mut classifier = BRepClass3dSolidClassifier::with_shape(&solid.shape).unwrap();
    classifier.perform(&point, precision::confusion()).unwrap();
    classifier.state()
}

#[test]
fn hollow_box_cavity_is_out_and_outer_shell_is_found() {
    assert_eq!(classify(&HOLLOW10, Point3::new(5.0, 5.0, 5.0)), State::Out);
    assert_eq!(classify(&HOLLOW10, Point3::new(1.0, 1.0, 1.0)), State::In);
    assert_eq!(classify(&HOLLOW10, Point3::new(3.0, 5.0, 5.0)), State::On);
    let solid = shape(&HOLLOW10);
    let outer = BRepClass3d::outer_shell(&topods::solid(&solid.shape).unwrap()).unwrap();
    assert!(outer.is_same(&solid.shells[0]));
}

#[test]
fn sclassifier_on_a_shared_explorer_and_the_forced_states() {
    let solid = shape(&BOX10);
    let mut explorer = BRepClass3dSolidExplorer::with_shape(&solid.shape).unwrap();
    let inside =
        BRepClass3dSClassifier::with_point(&mut explorer, &Point3::new(2.0, 3.0, 4.0), 1e-7)
            .unwrap();
    assert_eq!(inside.state(), State::In);
    assert!(!inside.rejected() && !inside.is_on_a_face());
    let mut classifier = BRepClass3dSClassifier::new();
    assert_eq!(classifier.state(), State::Out);
    classifier.force_in();
    assert_eq!(classifier.state(), State::In);
    classifier.force_out();
    assert_eq!(classifier.state(), State::Out);
}

#[test]
fn explorer_maps_the_boundary_and_finds_face_points() {
    let solid = shape(&BOX10);
    let mut explorer = BRepClass3dSolidExplorer::with_shape(&solid.shape).unwrap();
    assert!(!explorer.reject(&Point3::default()));
    assert_eq!(explorer.get_map_ev().extent(), 20); // 12 edges and 8 vertices
    let mut line = Line3::default();
    let mut par = 0.0;
    assert_eq!(
        explorer
            .segment(&Point3::new(5.0, 5.0, 5.0), &mut line, &mut par)
            .unwrap(),
        0
    );
    assert_eq!(explorer.get_face_segment_index(), 1);
    assert!(par > 0.0);
    let mut sample = FacePointSample::default();
    let found = BRepClass3dSolidExplorer::find_a_point_in_the_face_d1(
        &solid.faces[0],
        &mut sample,
        &mut 0.5,
    )
    .unwrap();
    assert!(found);
    assert!(sample.point.x().abs() <= 1e-12); // the face x = 0
    explorer.destroy();
    assert!(explorer.change_intersector(&solid.faces[0]).is_err());
}

#[test]
fn selectors_find_the_edges_and_vertices_near_a_point_or_a_line() {
    let solid = shape(&BOX10);
    let explorer = BRepClass3dSolidExplorer::with_shape(&solid.shape).unwrap();
    let mut point = BRepClass3dBndBoxTreeSelectorPoint::new(explorer.get_map_ev());
    point.set_current_point(Point3::new(0.0, 0.0, 5.0));
    assert_eq!(explorer.get_tree().select(&mut point), 1);
    let mut line = BRepClass3dBndBoxTreeSelectorLine::new(explorer.get_map_ev());
    let diagonal = Line3::new(
        Point3::new(-1.0, -1.0, -1.0),
        Direction3::new(1.0, 1.0, 1.0).unwrap(),
    );
    line.set_current_line(diagonal, 20.0).unwrap();
    assert!(explorer.get_tree().select(&mut line) > 0);
    assert_eq!(line.get_nb_vert_param(), 2); // the corners (0, 0, 0) and (10, 10, 10)
    assert!(line.get_vert_param(0).is_err() && line.get_vert_param(3).is_err());
    line.clear_results();
    assert_eq!(line.get_nb_vert_param(), 0);
}

#[test]
fn intersector3d_keeps_the_nearest_crossing() {
    let solid = shape(&BOX10);
    let mut intersector = BRepClass3dIntersector3d::new();
    let up = Line3::new(
        Point3::new(5.0, 5.0, -5.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    );
    // Face 4 is the bottom face z = 0 (outward normal -z).
    intersector
        .perform(&up, 1.0, 1e-7, &solid.faces[4])
        .unwrap();
    assert!(intersector.is_done() && intersector.has_a_point());
    assert_eq!(intersector.w_parameter(), 5.0);
    assert_eq!(intersector.state(), State::In);
    assert_eq!(intersector.transition(), TransitionOnCurve::In);
}
