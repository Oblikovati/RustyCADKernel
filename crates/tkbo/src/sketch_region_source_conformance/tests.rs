// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region::{ArrangementSource, RegionSourceId};
use crate::sketch_region_certification::{BoundaryInputs, capture_boundary_inputs};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomBezierCurve, GeomCurve, GeomLine},
    top_abs::Orientation,
};
use rustycad_tkmath::{
    gp::{Axis1, Direction3, Plane3, Point3, Transform3},
    predicates::roots::{DyadicBound, RootIsolationOptions, metric_bounds::*},
    toploc::Location,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
fn capture(shapes: &[Shape], plane: Plane3) -> BoundaryInputs {
    let sources: Vec<_> = shapes
        .iter()
        .enumerate()
        .map(|(i, s)| ArrangementSource {
            identity: RegionSourceId(i as u64 + 1),
            shape: s,
        })
        .collect();
    capture_boundary_inputs(&sources, plane)
        .unwrap_or_else(|_| panic!("expected actual captured boundary"))
}
fn inspect(inputs: &BoundaryInputs, limit: f64) -> SourceConformanceInspection {
    inspect_source_conformance(
        inputs,
        SourceConformancePolicy {
            original_plane_limit: limit,
            construction_limit: limit,
            roots: RootIsolationOptions::new(0.001),
        },
        || false,
    )
}
fn assert_complete(report: &SourceConformanceInspection) {
    assert!(!report.entries.is_empty());
    for original in &report.entries {
        assert_eq!(original.stages.len(), 3);
        for stage in &original.stages {
            assert!(matches!(
                stage.decision,
                Ok(ResidualDecision::Within { .. })
            ));
            assert_eq!(stage.requested_limit, stage.effective_limit);
        }
    }
}
fn line_shape(origin: Point3, first: f64, last: f64) -> Shape {
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(origin, Direction3::X))).cast();
    BRepBuilderAPIMakeEdge::from_curve_params(&curve, first, last)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}
#[test]
fn native_trim_and_normalized_endpoint_basis_have_independent_exact_evidence() {
    let shape = line_shape(Point3::new(0., 2., 0.), 2., 5.);
    let inputs = capture(&[shape], Plane3::default());
    assert_eq!(inputs.constructions[0].native_interval, [2., 5.]);
    assert_eq!(inputs.constructions[0].basis_interval, [0., 1.]);
    assert_eq!(
        inputs.originals[0]
            .representation
            .as_ref()
            .ok()
            .unwrap()
            .interval,
        [2., 5.]
    );
    assert_complete(&inspect(&inputs, 0.));
    assert_eq!(
        inputs.originals[0].original,
        inputs.constructions[0].parents[0]
    );
}
#[test]
fn shared_reversed_occurrences_retain_full_source_keys_and_support_location() {
    let shape = line_shape(Point3::default(), 0., 4.);
    let reversed = shape.oriented(Orientation::Reversed);
    let placed = shape
        .located(&Location::new(Transform3::translation([0., 3., 0.])), false)
        .unwrap();
    let inputs = capture(&[shape, reversed, placed], Plane3::default());
    assert_eq!(inputs.originals.len(), 3);
    assert_ne!(inputs.originals[0].original, inputs.originals[1].original);
    assert_ne!(
        inputs.originals[0].original.location,
        inputs.originals[2].original.location
    );
    assert_complete(&inspect(&inputs, 0.));
}
#[test]
fn full_coefficient_capture_survives_source_and_copy_curve_mutation() {
    let shape = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let inputs = capture(std::slice::from_ref(&shape), Plane3::default());
    assert_complete(&inspect(&inputs, 1e-12));
    mutate_curve(&shape);
    mutate_curve(&inputs.segments[0].edge);
    assert_complete(&inspect(&inputs, 1e-12));
    let changed = capture(&[shape], Plane3::default());
    assert_exceeds(&inspect(&changed, 1e-12));
}
fn mutate_curve(shape: &Shape) {
    BRepTool::curve_with_location(&edge(shape).unwrap())
        .unwrap()
        .curve
        .cast::<GeomBezierCurve>()
        .set_pole(2, Point3::new(4., 0., 1.))
        .unwrap();
}
fn assert_exceeds(report: &SourceConformanceInspection) {
    assert!(matches!(
        report.entries[0].stages[0].decision,
        Ok(ResidualDecision::Exceeds { .. })
    ));
    assert_eq!(report.entries[0].stages.len(), 1);
}
#[test]
fn off_plane_source_never_becomes_valid_through_projection_or_copy_flags() {
    let inputs = capture(
        &[line_shape(Point3::new(0., 0., 0.25), 0., 4.)],
        Plane3::default(),
    );
    assert_exceeds(&inspect(&inputs, 0.1));
    assert_complete(&inspect(&inputs, 0.25));
}
#[test]
fn actual_original_range_mismatch_is_incomplete_without_using_output_trim_as_source() {
    let mut inputs = capture(&[line_shape(Point3::default(), 2., 5.)], Plane3::default());
    inputs.originals[0]
        .representation
        .as_mut()
        .ok()
        .unwrap()
        .interval = [1., 5.];
    assert!(matches!(
        inspect(&inputs, 0.).entries[0].stages[0].decision,
        Err(SourceConformanceCause::NativeRange {
            original: [1., 5.],
            copied: [2., 5.],
            stored: [2., 5.]
        })
    ));
}
#[test]
fn absent_original_full_key_is_incomplete_and_never_pointer_recovered() {
    let mut inputs = capture(&[line_shape(Point3::default(), 0., 2.)], Plane3::default());
    inputs.originals.clear();
    let report = inspect(&inputs, 0.);
    assert!(matches!(
        report.entries[0].stages[0].decision,
        Err(SourceConformanceCause::MissingOriginal(_))
    ));
}
#[test]
fn cancellation_and_retry_keep_captured_state_and_requested_limits() {
    let inputs = capture(&[line_shape(Point3::default(), 0., 2.)], Plane3::default());
    let policy = SourceConformancePolicy {
        original_plane_limit: 0.,
        construction_limit: 0.,
        roots: RootIsolationOptions::new(0.003),
    };
    let failed = inspect_source_conformance(&inputs, policy, || true);
    assert_eq!(failed.bisections, 0);
    assert!(matches!(
        failed.entries[0].stages[0].decision,
        Err(SourceConformanceCause::Exact(_))
    ));
    assert_complete(&inspect_source_conformance(&inputs, policy, || false));
    assert_eq!(policy.roots.max_width, 0.003);
}
#[test]
fn native_parameter_membership_avoids_normalization_rounding_and_never_snaps() {
    let map = MetricParameterMap::Endpoints([1., 1e16]);
    let bound = DyadicBound::from_f64(0.5).unwrap();
    let evidence = native_parameter_membership(map, &[bound.clone(), bound], 5e15).unwrap();
    assert!(!evidence.belongs && !evidence.singleton_equal);
    assert!(evidence.supplied_parameter < evidence.native_bracket[0]);
}
#[test]
fn approximate_native_split_membership_does_not_claim_algebraic_identity() {
    let bounds = [
        DyadicBound::from_f64(0.49).unwrap(),
        DyadicBound::from_f64(0.51).unwrap(),
    ];
    let evidence =
        native_parameter_membership(MetricParameterMap::Endpoints([2., 6.]), &bounds, 4.).unwrap();
    assert!(evidence.belongs);
    assert!(!evidence.singleton_equal);
    assert!(native_parameter_membership(MetricParameterMap::Identity, &bounds, f64::NAN).is_err());
}

#[test]
fn declared_oblique_plane_and_translated_line_are_certified_without_axis_snapping() {
    let plane = Plane3::new(
        Point3::new(7., 11., 13.),
        Direction3::new(1., 2., 3.).unwrap(),
    )
    .unwrap();
    let edge = oblique_edge(plane);
    let inputs = capture(&[edge], plane);
    assert_complete(&inspect(&inputs, 1e-12));
    assert!(
        inputs.constructions[0]
            .support
            .as_ref()
            .ok()
            .unwrap()
            .frame
            .linear[2]
            .iter()
            .any(|v| *v != 0.)
    );
}
fn oblique_edge(plane: Plane3) -> Shape {
    let frame = plane.position();
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
        frame.location(),
        frame.x_direction(),
    )))
    .cast();
    BRepBuilderAPIMakeEdge::from_curve_params(&curve, -2., 5.)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}
#[test]
fn actual_rational_weights_and_prior_snapshot_survive_success_failure_success() {
    let edge = rational_edge();
    let inputs = capture(&[edge], Plane3::default());
    assert_complete(&inspect(&inputs, 1e-12));
    let policy = SourceConformancePolicy {
        original_plane_limit: 1e-12,
        construction_limit: 1e-12,
        roots: RootIsolationOptions::new(0.001),
    };
    assert!(
        inspect_source_conformance(&inputs, policy, || true).entries[0].stages[0]
            .decision
            .is_err()
    );
    assert_complete(&inspect(&inputs, 1e-12));
}
#[test]
fn missing_curve_capture_keeps_typed_cause_and_original_key() {
    let mut edge = rustycad_tkbrep::topods::Edge::default();
    rustycad_tkbrep::brep::BRepBuilder::default()
        .make_edge(&mut edge)
        .unwrap();
    let shape = edge.into_shape();
    let captures = capture_originals(&[ArrangementSource {
        identity: RegionSourceId(91),
        shape: &shape,
    }])
    .unwrap();
    let cause = captures[0].representation.as_ref().err().unwrap();
    assert!(matches!(
        cause.as_ref(),
        crate::sketch_region_geometry_capture::CapturedGeometryCause::MissingCurve {
            representation: "3D curve"
        }
    ));
    assert_eq!(captures[0].original.source, RegionSourceId(91));
}
#[test]
fn source_same_parameter_flag_is_only_a_precondition_not_a_projection_proof() {
    let shape = line_shape(Point3::default(), 0., 2.);
    rustycad_tkbrep::brep::BRepBuilder::default()
        .same_parameter(&edge(&shape).unwrap(), false)
        .unwrap();
    let inputs = capture(&[shape], Plane3::default());
    assert!(matches!(
        inspect(&inputs, 0.).entries[0].stages[0].decision,
        Err(SourceConformanceCause::UnknownParameterBinding)
    ));
}

fn rational_edge() -> Shape {
    use rustycad_tkernel::ncollection::Array1;
    let poles = rational_poles();
    let curve: Handle<dyn GeomCurve> = Handle::new(
        GeomBezierCurve::with_weights(
            &Array1::from_shared(&poles, 1),
            &Array1::from_shared(&[1., 2., 2., 1.], 1),
        )
        .unwrap(),
    )
    .cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

#[test]
fn retained_report_owns_coefficients_after_capture_inputs_and_source_are_dropped() {
    let report = {
        let shape = line_shape(Point3::default(), 2., 5.);
        inspect(&capture(&[shape], Plane3::default()), 0.)
    };
    assert_complete(&report);
    assert_retained_report_identity(&report);
}
fn assert_retained_report_identity(report: &SourceConformanceInspection) {
    assert_eq!(
        report.originals[0]
            .representation
            .as_ref()
            .ok()
            .unwrap()
            .interval,
        [2., 5.]
    );
    assert_eq!(report.constructions[0].native_interval, [2., 5.]);
    assert_eq!(report.entries[0].original, report.originals[0].original);
    assert_retained_work_policy(report);
}
fn assert_retained_work_policy(report: &SourceConformanceInspection) {
    assert_eq!(
        report.entries[0].occurrence,
        report.constructions[0].occurrence
    );
    assert_eq!(
        report.entries[0].stages[0].stage,
        ConstructionStage::OriginalPlane
    );
    assert_eq!(report.entries[0].stages[0].effective_roots.max_width, 0.001);
    assert_eq!(
        report.bisections,
        report
            .entries
            .iter()
            .flat_map(|e| &e.stages)
            .map(|s| s.bisections)
            .sum()
    );
}

fn rational_poles() -> [Point3; 4] {
    [
        Point3::new(0., 0., 0.),
        Point3::new(4., 0., 0.),
        Point3::new(0., 4., 0.),
        Point3::new(0., 0., 0.),
    ]
}

#[test]
fn unsupported_original_basis_retains_named_cause_without_line_or_faceted_fallback() {
    let curve: Handle<dyn GeomCurve> = Handle::new(
        rustycad_tkg3d::geom::GeomCircle::new(rustycad_tkmath::gp::Axis2::default(), 1.).unwrap(),
    )
    .cast();
    let shape = BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape();
    let sources = [ArrangementSource {
        identity: RegionSourceId(72),
        shape: &shape,
    }];
    let originals = capture_originals(&sources).unwrap();
    let cause = originals[0].representation.as_ref().err().unwrap();
    assert!(
        matches!(cause.as_ref(),crate::sketch_region_geometry_capture::CapturedGeometryCause::Unsupported { representation:"3D curve",dynamic_type } if dynamic_type=="Geom_Circle")
    );
    assert_eq!(originals[0].original.source, RegionSourceId(72));
}

#[test]
fn absent_stored_pcurve_is_typed_incomplete_without_treating_projection_as_storage() {
    let inputs = capture(&[line_shape(Point3::default(), 0., 2.)], Plane3::default());
    let fresh_support =
        rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace::from_plane(Plane3::default())
            .unwrap()
            .face()
            .unwrap();
    let fresh = capture_construction(&inputs.segments[0], &fresh_support);
    let cause = fresh
        .pcurve
        .as_ref()
        .err()
        .expect("new support has no installed edge pcurve");
    assert!(matches!(
        cause.as_ref(),
        crate::sketch_region_geometry_capture::CapturedGeometryCause::MissingStoredPCurve
    ));
    assert_complete(&inspect(&inputs, 0.));
}
#[test]
fn nonfinite_support_coefficient_preserves_typed_value_and_never_certifies_a_plane() {
    let mut inputs = capture(&[line_shape(Point3::default(), 0., 2.)], Plane3::default());
    inputs.constructions[0]
        .support
        .as_mut()
        .ok()
        .unwrap()
        .frame
        .origin[0] = f64::INFINITY;
    assert!(
        matches!(inspect(&inputs,0.).entries[0].stages[0].decision,Err(SourceConformanceCause::Exact(ResidualError::Value { role:"origin",index:0,value })) if value==f64::INFINITY)
    );
}

#[test]
fn original_plane_failure_charges_real_work_and_does_not_refund_aggregate_budget() {
    let inputs = capture(&[varying_height_edge()], Plane3::default());
    let mut policy = SourceConformancePolicy {
        original_plane_limit: 2.,
        construction_limit: 2.,
        roots: RootIsolationOptions::new(1e-6),
    };
    policy.roots.max_bisections = 1;
    let failed = inspect_source_conformance(&inputs, policy, || false);
    assert_eq!(failed.bisections, 1);
    assert_eq!(failed.entries[0].stages[0].bisections, 1);
    assert!(matches!(
        failed.entries[0].stages[0].decision,
        Err(SourceConformanceCause::Exact(_))
    ));
    policy.roots.max_bisections = 1000;
    let retry = inspect_source_conformance(&inputs, policy, || false);
    assert_complete(&retry);
    assert!(retry.entries[0].stages[1].effective_roots.max_bisections < 1000);
    assert_eq!(failed.bisections, 1);
}
fn varying_height_edge() -> Shape {
    use rustycad_tkernel::ncollection::Array1;
    let poles = [
        Point3::default(),
        Point3::new(4., 0., 1.),
        Point3::new(0., 4., 0.3),
        Point3::default(),
    ];
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}
