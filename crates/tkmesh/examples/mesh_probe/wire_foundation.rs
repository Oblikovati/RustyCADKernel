//! Native ShapeAnalysis wire-order foundation probes used by model construction.
use super::super::{flag, int, real};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{self, Face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkshhealing::{
    shape_analysis::{ShapeAnalysis, ShapeAnalysisWire, ShapeAnalysisWireOrder},
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};
#[path = "../../../tkshhealing/tests/support/wire_foundation.rs"]
mod fixtures;
const STATUSES: [ShapeExtendStatus; 9] = [
    ShapeExtendStatus::Ok,
    ShapeExtendStatus::Done,
    ShapeExtendStatus::Done1,
    ShapeExtendStatus::Done2,
    ShapeExtendStatus::Done3,
    ShapeExtendStatus::Done4,
    ShapeExtendStatus::Done5,
    ShapeExtendStatus::Fail,
    ShapeExtendStatus::Fail2,
];
pub fn wire_order_case(v: &[i32]) {
    let (_, face, wire) = fixtures::order_fixture(v[0], v[1], v[2]);
    let face = if v[3] != 0 { face } else { Face::default() };
    let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 0.01).unwrap();
    let mut order = ShapeAnalysisWireOrder::new();
    flag(analyzer.is_loaded());
    flag(analyzer.is_ready());
    int(analyzer.nb_edges());
    real(analyzer.precision());
    for _ in 0..=v[7] {
        flag(
            analyzer
                .check_order_with(&mut order, v[6] != 0, v[4] != 0, v[5] != 0)
                .unwrap(),
        );
        int(order.nb_edges());
        int(order.status());
        for i in 1..=order.nb_edges() {
            int(order.ordered(i).unwrap());
        }
        for status in STATUSES {
            flag(analyzer.last_check_status(status));
            flag(analyzer.status_order(status));
        }
    }
    flag(analyzer.check_order(v[6] != 0, v[4] != 0).unwrap());
    for status in STATUSES {
        flag(analyzer.last_check_status(status));
        flag(analyzer.status_order(status));
    }
    analyzer.clear_statuses();
    flag(analyzer.last_check_status(ShapeExtendStatus::Ok));
    flag(analyzer.status_order(ShapeExtendStatus::Ok));
    println!();
}
pub fn outer_wire_case(v: &[i32]) {
    let (first, base, _) = fixtures::order_fixture(v[0], v[1], v[2]);
    let (second, _, _) = fixtures::order_fixture(0, 0, 0);
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &BRepTool::surface(&base).unwrap(), 1e-7)
        .unwrap();
    let first = topods::wire(&first.oriented(if v[3] & 1 != 0 {
        Orientation::Reversed
    } else {
        Orientation::Forward
    }))
    .unwrap();
    let second = topods::wire(&second.oriented(if v[3] & 2 != 0 {
        Orientation::Reversed
    } else {
        Orientation::Forward
    }))
    .unwrap();
    if v[4] > 0 {
        builder.add(&mut face, &first).unwrap();
    }
    if v[4] > 1 {
        builder.add(&mut face, &second).unwrap();
    }
    let ordered = Handle::new(ShapeExtendWireData::from_wire(&first, true, true).unwrap());
    real(ShapeAnalysis::tot_cross_2d(&ordered, &face).unwrap());
    let found = ShapeAnalysis::outer_wire(&face).unwrap();
    int(if found.is_null() {
        0
    } else if found.is_same(&first) {
        1
    } else if found.is_same(&second) {
        2
    } else {
        -1
    });
    println!();
}
