// SPDX-License-Identifier: GPL-2.0-only
//! Mutating source handles after capture cannot mutate owned result coefficients or evidence.
use super::{tests::*, *};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge},
};
use rustycad_tkg3d::geom::GeomBezierCurve;
use rustycad_tkmath::gp::Point3;

fn change_cubic_pole(source: &Shape) {
    let curve = BRepTool::curve_with_location(&edge(source).unwrap()).unwrap();
    curve
        .curve
        .cast::<GeomBezierCurve>()
        .set_pole(2, Point3::new(6., 0., 0.))
        .unwrap();
}
fn copied_edge(receipt: &super::super::PrismReceipt) -> Shape {
    receipt
        .copied
        .as_ref()
        .unwrap()
        .relations
        .iter()
        .find(|r| r.output.shape_type().ok() == Some(rustycad_tkg3d::top_abs::ShapeEnum::Edge))
        .unwrap()
        .output
        .clone()
}
#[test]
fn source_and_body_mutation_cannot_change_already_frozen_output() {
    let (source, receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    let projection = captured(freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress));
    let bytes = projection.replay_bytes();
    change_cubic_pole(&source);
    assert_eq!(
        captured(freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress)).replay_bytes(),
        bytes
    );
    change_cubic_pole(&copied_edge(&receipt));
    assert_eq!(projection.replay_bytes(), bytes);
    assert_stale(freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress));
}
#[test]
fn stale_source_coefficients_cannot_reuse_retained_chart_evidence() {
    let (source, receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    change_cubic_pole(&copied_edge(&receipt));
    assert_stale(freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress));
}
fn assert_stale(outcome: PrismFreezeOutcome) {
    assert!(matches!(
        outcome,
        PrismFreezeOutcome::Incomplete(PrismFreezeFailure {
            cause: PrismFreezeCause::StaleCapture { .. },
            ..
        })
    ));
}
