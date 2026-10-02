//! Pinned OCCT ShapeFix_Wire reorder and gap-pass observations.
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{self, ShapeIterator},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
    shape_fix::ShapeFixWire,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

#[test]
fn empty_check_order_and_fix_reorder_keep_native_empty_state() {
    let (analyzer, mut order) = empty_wire_order();
    assert_empty_order(&analyzer, &mut order);
    assert_empty_fixer(&analyzer, &order);
}

fn empty_wire_order() -> (
    Handle<ShapeAnalysisWire>,
    rustycad_tkshhealing::shape_analysis::ShapeAnalysisWireOrder,
) {
    let empty_data = Handle::new(ShapeExtendWireData::new());
    let analyzer = Handle::new(
        ShapeAnalysisWire::from_wire_data(&empty_data, &Default::default(), 1e-7).unwrap(),
    );
    let order =
        rustycad_tkshhealing::shape_analysis::ShapeAnalysisWireOrder::with_mode(true, 1e-7, false);
    (analyzer, order)
}

fn assert_empty_order(
    analyzer: &ShapeAnalysisWire,
    order: &mut rustycad_tkshhealing::shape_analysis::ShapeAnalysisWireOrder,
) {
    assert!(!analyzer.check_order_with(order, true, true, false).unwrap());
    assert_eq!(order.nb_edges(), 0);
    assert_eq!(order.status(), 0);
    assert!(analyzer.status_order(ShapeExtendStatus::Ok));
    assert!(analyzer.last_check_status(ShapeExtendStatus::Ok));
}

fn assert_empty_fixer(
    analyzer: &Handle<ShapeAnalysisWire>,
    order: &rustycad_tkshhealing::shape_analysis::ShapeAnalysisWireOrder,
) {
    let fixer = ShapeFixWire::new();
    fixer.init(analyzer);
    assert!(!fixer.fix_reorder_with(order).unwrap());
    assert_eq!(analyzer.nb_edges(), 0);
    assert!(!fixer.status_reorder(ShapeExtendStatus::Done));
    assert!(!fixer.status_reorder(ShapeExtendStatus::Fail));
}

#[test]
fn valid_box_wires_match_native_reorder_and_zero_gap_statuses() {
    assert_eq!(check_all_box_wires(), 6);
}

fn check_all_box_wires() -> i32 {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(100.0, 200.0, 300.0).unwrap();
    let shape = maker.shape().unwrap().clone();
    let mut face_explorer = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut wires_seen = 0;
    while face_explorer.more() {
        let face = topods::face(face_explorer.current()).unwrap();
        let mut children = ShapeIterator::new(&face.clone().into_shape(), true, true);
        while children.more() {
            let child = children.value().unwrap().clone();
            children.next();
            if child.shape_type().unwrap() != ShapeEnum::Wire {
                continue;
            }
            let wire = topods::wire(&child).unwrap();
            assert_box_wire_fixes(&wire, &face);
            wires_seen += 1;
        }
        face_explorer.next().unwrap();
    }
    wires_seen
}

fn assert_box_wire_fixes(
    wire: &rustycad_tkbrep::topods::Wire,
    face: &rustycad_tkbrep::topods::Face,
) {
    let analyzer =
        Handle::new(ShapeAnalysisWire::from_wire(wire, face, 1e-7).expect("native analyzer load"));
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    analyzer.set_face(face).unwrap();
    assert!(fixer.fix_reorder().unwrap());
    assert!(fixer.status_reorder(ShapeExtendStatus::Done1));
    assert!(fixer.status_reorder(ShapeExtendStatus::Done5));
    assert!(!fixer.status_reorder(ShapeExtendStatus::Fail));
    assert_zero_gap_analysis(&analyzer);
    assert_zero_gap_fixes(&fixer);
}

fn assert_zero_gap_analysis(analyzer: &ShapeAnalysisWire) {
    assert!(!analyzer.check_gaps_3d().unwrap());
    assert!(!analyzer.status_gaps_3d(ShapeExtendStatus::Done));
    assert!(!analyzer.status_gaps_3d(ShapeExtendStatus::Fail));
    assert_eq!(analyzer.max_distance_3d(), 0.0);
    assert!(!analyzer.check_gaps_2d().unwrap());
    assert!(!analyzer.status_gaps_2d(ShapeExtendStatus::Done));
    assert!(!analyzer.status_gaps_2d(ShapeExtendStatus::Fail));
    assert_eq!(analyzer.max_distance_2d(), 0.0);
}

fn assert_zero_gap_fixes(fixer: &ShapeFixWire) {
    assert!(!fixer.fix_gaps_3d().unwrap());
    assert!(!fixer.status_gaps_3d(ShapeExtendStatus::Done));
    assert!(!fixer.status_gaps_3d(ShapeExtendStatus::Fail));
    assert!(!fixer.fix_gaps_2d().unwrap());
    assert!(!fixer.status_gaps_2d(ShapeExtendStatus::Done));
    assert!(!fixer.status_gaps_2d(ShapeExtendStatus::Fail));
}
