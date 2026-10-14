// SPDX-License-Identifier: GPL-2.0-only
use super::super::{tests::*, *};
use super::{tests::*, *};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, edge},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomPlane, GeomSurfaceOfLinearExtrusion, GeomSweptSurface,
};
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};

fn side_surface(receipt: &PrismReceipt) -> Handle<GeomSurfaceOfLinearExtrusion> {
    let face = live_face(receipt, PrismImageRole::Generated);
    let (surface, _) = BRepTool::surface_with_location(&face).unwrap();
    surface.cast()
}
fn live_side_trim(receipt: &PrismReceipt, role: PrismTrimRole) -> Edge {
    let chart = entry_by_role(receipt, PrismImageRole::Generated)
        .captured
        .as_ref()
        .unwrap();
    let trim = chart.trims.iter().find(|trim| trim.role == role).unwrap();
    let binding = receipt
        .diagnostic
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .find(|binding| {
            crate::sketch_region::PartitionOccurrenceIdentity::capture(&binding.context.occurrence)
                == trim.output
        })
        .unwrap();
    edge(&binding.context.occurrence).unwrap()
}
#[test]
fn edited_side_interior_preserves_endpoints_but_invalidates_the_actual_chart() {
    let mut receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let actual = side_surface(&receipt)
        .basis_curve()
        .cast::<GeomBezierCurve>();
    let retained = take_inspection(&mut receipt);
    actual.set_pole(2, Point3::new(5., 0., 0.)).unwrap();
    fresh_inspection(&mut receipt);
    assert_identity_mismatch(
        &receipt,
        PrismImageRole::Generated,
        PrismChartStage::SideBasis,
    );
    assert_retained_scalar_proof(&retained);
    assert_body_incomplete(&receipt);
}
fn take_inspection(receipt: &mut PrismReceipt) -> PrismSurfaceInspection {
    receipt
        .diagnostic
        .as_mut()
        .unwrap()
        .surface_charts
        .take()
        .unwrap()
}
fn assert_retained_scalar_proof(retained: &PrismSurfaceInspection) {
    for entry in &retained.entries {
        assert!(entry.decision.is_ok());
        let mut proof = PrismSurfaceProof::default();
        prove_chart(
            entry.captured.as_ref().unwrap(),
            entry.role,
            &mut || false,
            &mut proof,
        )
        .unwrap();
    }
}
fn assert_identity_mismatch(
    receipt: &PrismReceipt,
    role: PrismImageRole,
    expected: PrismChartStage,
) {
    let result = &entry_by_role(receipt, role).decision;
    assert!(
        matches!(result, Err(PrismSurfaceCause::Identity { stage, .. }) if *stage == expected),
        "{result:?}"
    );
}
#[test]
fn edited_surface_direction_requires_a_new_chart_proof() {
    let mut receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    side_surface(&receipt).set_direction(Direction3::new(1., 0., 0.).unwrap());
    fresh_inspection(&mut receipt);
    assert_identity_mismatch(
        &receipt,
        PrismImageRole::Generated,
        PrismChartStage::SideDirection,
    );
}
#[test]
fn shared_copied_source_and_cap_edit_is_rejected_against_pre_sweep_snapshot() {
    let mut receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let face = live_face(&receipt, PrismImageRole::Last);
    let (surface, _) = BRepTool::surface_with_location(&face).unwrap();
    let plane = Plane3::new(
        Point3::new(0., 0., 3.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
    .unwrap();
    surface.cast::<GeomPlane>().set_pln(plane);
    assert_shared_support_changed(&receipt);
    fresh_inspection(&mut receipt);
    assert_identity_mismatch(&receipt, PrismImageRole::Last, PrismChartStage::CapSupportU);
}
#[test]
fn edited_actual_pcurve_range_is_incomplete_even_when_surface_coefficients_match() {
    let mut receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let edge = live_side_trim(&receipt, PrismTrimRole::First);
    let face = live_face(&receipt, PrismImageRole::Generated);
    BRepBuilder::default()
        .range_on_face(&edge, &face, 0., 0.5)
        .unwrap();
    fresh_inspection(&mut receipt);
    assert!(matches!(
        entry_by_role(&receipt, PrismImageRole::Generated).decision,
        Err(PrismSurfaceCause::Range {
            actual: [0., 0.5],
            expected: [0., 1.]
        })
    ));
}
#[test]
fn missing_history_is_explicit_for_every_actual_output_face() {
    let mut receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    receipt.diagnostic.as_mut().unwrap().bindings.clear();
    fresh_inspection(&mut receipt);
    assert_eq!(inspection(&receipt).uncovered_faces.len(), 3);
    assert!(inspection(&receipt).entries.is_empty());
    assert_body_incomplete(&receipt);
}
#[test]
fn changed_copied_source_cannot_reuse_a_previous_actual_surface_proof() {
    let mut receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let copied = &receipt.copied.as_ref().unwrap().base;
    let edges = trim_capture::face_edges(copied).unwrap();
    let raw = BRepTool::curve_with_location(&edge(&edges[0]).unwrap()).unwrap();
    let retained = take_inspection(&mut receipt);
    raw.curve
        .cast::<GeomBezierCurve>()
        .set_pole(2, Point3::new(6., 0., 0.))
        .unwrap();
    fresh_inspection(&mut receipt);
    assert_trim_edit_ownership(&receipt, &retained);
    assert_identity_mismatch(
        &receipt,
        PrismImageRole::Generated,
        PrismChartStage::TrimSpaceCurve,
    );
    assert_retained_scalar_proof(&retained);
}

fn assert_shared_support_changed(receipt: &PrismReceipt) {
    let source = &receipt.copied.as_ref().unwrap().base;
    let support = crate::sketch_region_source_conformance::capture_support(
        &rustycad_tkbrep::topods::face(source).unwrap(),
    )
    .unwrap();
    assert_eq!(support.frame.origin, [0., 0., 3.]);
    let snapshot = receipt
        .copied
        .as_ref()
        .unwrap()
        .chart_sources
        .as_ref()
        .unwrap();
    assert_eq!(snapshot.support.frame.origin, [0.; 3]);
}
#[test]
fn replaced_body_cannot_borrow_previous_face_history() {
    let mut receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let other = cubic_receipt([1., 1.], &mut progress(None, None));
    receipt.diagnostic.as_mut().unwrap().body = other.diagnostic.unwrap().body;
    fresh_inspection(&mut receipt);
    assert_eq!(inspection(&receipt).face_count, 3);
    assert_eq!(inspection(&receipt).uncovered_faces.len(), 6);
    assert!(inspection(&receipt).entries.is_empty());
}

fn assert_trim_edit_ownership(receipt: &PrismReceipt, retained: &PrismSurfaceInspection) {
    let sources = receipt
        .copied
        .as_ref()
        .unwrap()
        .chart_sources
        .as_ref()
        .unwrap();
    let side = retained
        .entries
        .iter()
        .find(|entry| entry.role == PrismImageRole::Generated)
        .unwrap();
    assert_eq!(side.source, sources.boundaries[0].occurrence);
    assert_source_pole_four(&sources.boundaries[0].space.curve);
    assert_distinct_basis_and_trim(receipt);
}
fn assert_distinct_basis_and_trim(receipt: &PrismReceipt) {
    let basis = side_surface(receipt)
        .basis_curve()
        .cast::<GeomBezierCurve>();
    assert_eq!(basis.pole(2).unwrap(), Point3::new(4., 0., 0.));
    let actual = live_side_trim(receipt, PrismTrimRole::First);
    let raw = BRepTool::curve_with_location(&actual).unwrap();
    assert_eq!(
        raw.curve.cast::<GeomBezierCurve>().pole(2).unwrap(),
        Point3::new(6., 0., 0.)
    );
}
fn assert_source_pole_four(curve: &crate::sketch_region_geometry_capture::CapturedMetricCurve<3>) {
    let crate::sketch_region_geometry_capture::CapturedMetricCurve::Bezier { poles, .. } = curve
    else {
        panic!("expected pre-sweep cubic");
    };
    assert_eq!(poles[1], [4., 0., 0.]);
}
