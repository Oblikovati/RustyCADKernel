//! State and native ordering foundation; no claim of the other wire checks.
#[path = "support/wire_foundation.rs"]
mod fixtures;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkshhealing::{
    shape_analysis::{ShapeAnalysis, ShapeAnalysisWire, ShapeAnalysisWireOrder},
    shape_extend::ShapeExtendStatus,
};
#[test]
fn wire_order_statuses_and_shared_initialization_follow_native() {
    let (wire, face, ordered) = fixtures::order_fixture(1, 0, 0);
    let analyzer = ShapeAnalysisWire::from_wire_data(&ordered, &face, 0.01).unwrap();
    assert!(analyzer.is_loaded() && analyzer.is_ready());
    assert_eq!(analyzer.nb_edges(), 4);
    assert!(analyzer.face().is_same(&face));
    assert_eq!(analyzer.precision(), 0.01);
    let mut order = ShapeAnalysisWireOrder::new();
    assert!(
        analyzer
            .check_order_with(&mut order, true, false, false)
            .unwrap()
    );
    assert!(analyzer.last_check_status(ShapeExtendStatus::Done));
    assert!(analyzer.status_order(ShapeExtendStatus::Ok));
    assert!(analyzer.check_order(true, false).unwrap());
    assert!(analyzer.status_order(ShapeExtendStatus::Done));
    let surface = analyzer.surface();
    analyzer.set_face(&Face::default()).unwrap();
    assert!(!analyzer.is_ready());
    assert!(analyzer.surface() == surface);
    analyzer.load_wire(&wire).unwrap();
    assert!(analyzer.last_check_status(ShapeExtendStatus::Ok));
    analyzer.set_face_with_analysis(&face, &surface);
    analyzer.set_surface_analysis(&surface);
    analyzer.set_precision(-1.);
    assert_eq!(analyzer.precision(), -1.);
    assert_eq!(
        ShapeAnalysisWire::static_type().name(),
        "ShapeAnalysis_Wire"
    );
}
#[test]
fn mixed_mode_missing_pcurve_clears_accumulated_order_as_native_does() {
    let (_, face, ordered) = fixtures::order_fixture(0, 0, 1);
    let analyzer = ShapeAnalysisWire::from_wire_data(&ordered, &face, 1e-7).unwrap();
    let mut order = ShapeAnalysisWireOrder::new();
    assert!(
        !analyzer
            .check_order_with(&mut order, true, true, true)
            .unwrap()
    );
    assert_eq!(order.nb_edges(), 0);
    assert!(analyzer.last_check_status(ShapeExtendStatus::Ok));
    assert!(
        !analyzer
            .check_order_with(&mut order, true, false, false)
            .unwrap()
    );
    assert!(analyzer.last_check_status(ShapeExtendStatus::Fail2));
    analyzer.clear_statuses();
    assert!(analyzer.last_check_status(ShapeExtendStatus::Ok));
}
#[test]
fn outer_wire_uses_signed_native_curve_sampling() {
    let (wire, face, ordered) = fixtures::order_fixture(0, 0, 0);
    assert_eq!(ShapeAnalysis::tot_cross_2d(&ordered, &face).unwrap(), 6.);
    assert!(ShapeAnalysis::outer_wire(&face).unwrap().is_same(&wire));
    let analyzer = ShapeAnalysisWire::from_wire(&wire, &face, 1e-7).unwrap();
    assert!(analyzer.is_ready());
    analyzer.init_wire(&wire, &face, 0.5).unwrap();
    analyzer.init_wire_data(&ordered, &face, 0.75).unwrap();
    let geometry = rustycad_tkbrep::brep::BRepTool::surface(&face).unwrap();
    analyzer.set_surface(&geometry).unwrap();
    analyzer
        .set_located_surface(&geometry, &Default::default())
        .unwrap();
    assert!(!analyzer.surface().is_null());
    analyzer.load_wire_data(&Handle::default());
    assert!(!analyzer.is_loaded());
    assert_eq!(analyzer.nb_edges(), 0);
}
