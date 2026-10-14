// SPDX-License-Identifier: GPL-2.0-only
//! Actual copied supports carry exact affine evidence while region/body completion stays incomplete.
use super::{affine_chart::*, tests::*, *};
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_candidates::tests::candidates;
use crate::sketch_region_geometry_capture::CapturedGeometryCause;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, face},
};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkmath::{
    gp::{Axis3, Direction3, Plane3, Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use std::cmp::Ordering;

fn plane_shape(plane: Plane3) -> Shape {
    BRepBuilderAPIMakeFace::from_plane(plane)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}
fn yz_plane() -> Plane3 {
    Plane3::from_position(
        Axis3::new(
            Point3::new(5., 6., 7.),
            Direction3::new(1., 0., 0.).unwrap(),
            Direction3::new(0., 1., 0.).unwrap(),
        )
        .unwrap(),
    )
}
fn assert_jacobian(observation: &PrismAffineChartObservation, expected: f64) {
    let evidence = observation
        .decision
        .as_ref()
        .unwrap_or_else(|_| panic!("expected actual planar support"));
    assert_eq!(evidence.version, 1);
    assert_eq!(
        evidence.signed_jacobian.compare_f64(expected),
        Ok(Ordering::Equal)
    );
}
#[test]
fn actual_placed_support_and_chart_handedness_determine_exact_jacobian() {
    let mut plane = yz_plane();
    for expected in [2., -2.] {
        let mut transform = Transform3::default();
        transform.set_translation(Vector3::new(11., 13., 17.));
        let shape = plane_shape(plane)
            .located(&Location::new(transform), true)
            .unwrap();
        let observation = observe_affine_chart(&shape, [2., 3., 4.]);
        assert_jacobian(&observation, expected);
        assert_eq!(
            observation.source,
            PartitionOccurrenceIdentity::capture(&shape)
        );
        assert_eq!(observation.extrusion, [2., 3., 4.]);
        plane.v_reverse();
    }
}
#[test]
fn tangent_and_zero_sweeps_have_explicit_singular_scalar_evidence() {
    let shape = plane_shape(yz_plane());
    for extrusion in [[0.; 3], [0., 1., 2.], [0., -3., 4.]] {
        assert_jacobian(&observe_affine_chart(&shape, extrusion), 0.);
    }
}
#[test]
fn actual_face_location_rotates_the_support_before_the_extrusion_comparison() {
    let shape = plane_shape(Plane3::default());
    let mut rotation = Transform3::default();
    rotation
        .set_values([[0., 0., 1., 5.], [1., 0., 0., 6.], [0., 1., 0., 7.]])
        .unwrap();
    let located = shape.located(&Location::new(rotation), true).unwrap();
    assert_jacobian(&observe_affine_chart(&shape, [2., 0., 0.]), 0.);
    assert_jacobian(&observe_affine_chart(&located, [2., 0., 0.]), 2.);
}
#[test]
fn unsupported_support_retains_original_capture_cause() {
    let sphere = BRepBuilderAPIMakeFace::from_sphere(rustycad_tkmath::gp::Sphere3::default())
        .unwrap()
        .face()
        .unwrap()
        .into_shape();
    let observation = observe_affine_chart(&sphere, [0., 0., 2.]);
    assert!(matches!(
        observation.decision,
        Err(PrismAffineChartCause::Capture(
            CapturedGeometryCause::Unsupported {
                representation: "support plane",
                ..
            }
        ))
    ));
}
#[test]
fn invalid_shape_retains_original_capture_cause() {
    assert!(matches!(
        observe_affine_chart(&Shape::default(), [0., 0., 2.]).decision,
        Err(PrismAffineChartCause::Capture(_))
    ));
}
#[test]
fn complete_existing_pipeline_retains_actual_copy_evidence_and_all_unfinished_gates() {
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    let observed = receipt.affine_chart.as_ref().unwrap();
    assert_jacobian(observed, 2.);
    assert_eq!(
        observed.source,
        PartitionOccurrenceIdentity::capture(&receipt.copied.as_ref().unwrap().base)
    );
    assert_open_body_gates(&receipt);
}
fn assert_open_body_gates(receipt: &PrismReceipt) {
    assert!(
        receipt
            .gaps
            .iter()
            .any(|gap| matches!(gap, PrismGap::PrismInjectivityAndSurfaceCharts))
    );
    let coverage = receipt
        .diagnostic
        .as_ref()
        .unwrap()
        .coverage
        .as_ref()
        .unwrap();
    assert!(!coverage.complete());
    assert!(receipt.limits.achieved_error.is_none());
}
#[test]
fn cancellation_and_failure_keep_the_exact_completed_prefix_without_a_success_body() {
    let early = cubic_receipt([1., 1.], &mut progress(Some(PrismPhase::Copy), None));
    assert!(early.affine_chart.is_none());
    for phase in [PrismPhase::Sweep, PrismPhase::History] {
        let cancelled = cubic_receipt([1., 1.], &mut progress(Some(phase), None));
        assert!(matches!(cancelled.completion,PrismCompletion::Cancelled(actual) if actual==phase));
        assert_jacobian(cancelled.affine_chart.as_ref().unwrap(), 2.);
        let failed = cubic_receipt([1., 1.], &mut progress(None, Some(phase)));
        assert!(matches!(failed.completion, PrismCompletion::Failed(_)));
        assert_jacobian(failed.affine_chart.as_ref().unwrap(), 2.);
    }
    assert_jacobian(
        cubic_receipt([1., 1.], &mut progress(None, None))
            .affine_chart
            .as_ref()
            .unwrap(),
        2.,
    );
}
#[test]
fn near_tangent_exact_nonzero_evidence_does_not_promote_the_diagnostic_to_success() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let mut options = request([1., 1.]);
    options.extrusion = Vector3::new(1., 0., f64::from_bits(1));
    let receipt = retain_selected_prism(
        candidates(&[source]),
        options,
        &mut progress(Some(PrismPhase::Sweep), None),
    );
    assert!(matches!(
        receipt.completion,
        PrismCompletion::Cancelled(PrismPhase::Sweep)
    ));
    assert_jacobian(receipt.affine_chart.as_ref().unwrap(), f64::from_bits(1));
    assert!(receipt.diagnostic.is_none());
}
#[test]
fn input_surface_mutation_cannot_change_retained_evidence_or_the_owned_copied_support() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let batch = candidates(&[source]);
    let surfaces: Vec<_> = batch
        .candidates
        .iter()
        .map(|candidate| surface_plane(&candidate.face))
        .collect();
    let receipt = retain_selected_prism(batch, request([1., 1.]), &mut progress(None, None));
    for surface in surfaces {
        surface.set_pln(yz_plane());
    }
    let copied = &receipt.copied.as_ref().unwrap().base;
    assert_jacobian(receipt.affine_chart.as_ref().unwrap(), 2.);
    assert_jacobian(&observe_affine_chart(copied, [0., 0., 2.]), 2.);
}
#[test]
fn copied_support_mutation_requires_a_new_observation_and_preserves_the_old_scalar() {
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let copied = &receipt.copied.as_ref().unwrap().base;
    surface_plane(&face(copied).unwrap()).set_pln(yz_plane());
    assert_jacobian(&observe_affine_chart(copied, [0., 0., 2.]), 0.);
    assert_jacobian(receipt.affine_chart.as_ref().unwrap(), 2.);
}
fn surface_plane(
    face: &rustycad_tkbrep::topods::Face,
) -> rustycad_tkernel::standard::Handle<GeomPlane> {
    let (surface, _) = BRepTool::surface_with_location(face).unwrap();
    surface.cast::<GeomPlane>()
}

static_assertions::assert_impl_all!(PrismAffineChartEvidence: Send, Sync);
static_assertions::assert_impl_all!(rustycad_tkmath::predicates::roots::DyadicBound: Send, Sync);
#[test]
fn invalid_extrusion_observation_retains_its_exact_coefficient_failure() {
    let shape = plane_shape(Plane3::default());
    let observed = observe_affine_chart(&shape, [0., 0., f64::INFINITY]);
    assert!(
        matches!(observed.decision, Err(PrismAffineChartCause::Exact(
        rustycad_tkmath::predicates::roots::metric_bounds::ResidualError::Value {
            role: "prism extrusion", index: 2, value
        })) if value == f64::INFINITY)
    );
}
