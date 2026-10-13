// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region::{ArrangementSource, RegionSourceId};
use crate::sketch_region_candidates::tests::{arranged, rectangle};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{gp::Plane3, predicates::roots::RootIsolationOptions};

fn capture(shapes: &[Shape]) -> BoundaryInputs {
    let sources: Vec<_> = shapes
        .iter()
        .enumerate()
        .map(|(i, shape)| ArrangementSource {
            identity: RegionSourceId(i as u64 + 1),
            shape,
        })
        .collect();
    capture_boundary_inputs(&sources, Plane3::default())
        .unwrap_or_else(|_| panic!("expected genuine copied source boundaries"))
}
fn policy() -> CensusPolicy {
    CensusPolicy {
        roots: RootIsolationOptions::new(0.001),
        max_world_error: 1e-8,
    }
}
fn census(shapes: &[Shape]) -> BoundaryCensus {
    enumerate_boundary_census(capture(shapes), policy(), || false).unwrap_or_else(|_| {
        panic!("expected explicit complete census domains, not complete operation")
    })
}
const EXPECTED_PAIR_ORDER: [[usize; 2]; 10] = [
    [0, 0],
    [0, 1],
    [0, 2],
    [0, 3],
    [1, 1],
    [1, 2],
    [1, 3],
    [2, 2],
    [2, 3],
    [3, 3],
];
#[test]
fn every_cross_and_self_domain_is_retained_in_stable_order() {
    let census = census(&rectangle(0., 0., 4., 4.));
    let pairs: Vec<_> = census
        .domains
        .iter()
        .map(|domain| domain.occurrences)
        .collect();
    assert_eq!(pairs, EXPECTED_PAIR_ORDER);
    assert_eq!(count_gap(&census, CensusGap::SelfOffDiagonal), 4);
}
fn count_gap(census: &BoundaryCensus, gap: CensusGap) -> usize {
    census
        .domains
        .iter()
        .filter(|domain| domain.evidence == DomainEvidence::Unsupported(gap))
        .count()
}
#[test]
fn reversed_shared_occurrence_reuses_pair_report_without_losing_original_keys() {
    let mut shapes = rectangle(0., 0., 4., 4.);
    shapes.push(shapes[0].oriented(Orientation::Reversed));
    let census = census(&shapes);
    assert_eq!(census.domains.len(), 15);
    assert_eq!(census.inputs.segments[0].parents.len(), 2);
    assert_eq!(count_gap(&census, CensusGap::SamePhysicalCurve), 1);
    assert_reuse_receipt(&census);
}
fn assert_reuse_receipt(census: &BoundaryCensus) {
    let references = census
        .domains
        .iter()
        .filter(|d| matches!(d.evidence, DomainEvidence::StoredIncidence { .. }))
        .count();
    assert!(references > census.reports.len());
    let fresh_work: u64 = census
        .reports
        .iter()
        .map(|report| u64::from(report.bisections()))
        .sum();
    assert_eq!(census.bisections, fresh_work);
}
#[test]
fn coincidence_and_self_contacts_never_become_empty_passes() {
    use rustycad_tkmath::predicates::roots::bezier_line::{
        BezierIncidenceCause, BezierIncidenceCompletion, BezierRayError,
    };
    let first = rectangle(0., 0., 4., 4.).remove(0);
    let second = rectangle(0., 0., 4., 4.).remove(0);
    let census = census(&[first, second]);
    assert!(census.reports.is_empty());
    assert_eq!(census.failures.len(), 1);
    assert_eq!(
        census.failures[0].completion(),
        BezierIncidenceCompletion::Declined
    );
    assert!(matches!(
        census.failures[0].cause(),
        BezierIncidenceCause::Classification(BezierRayError::SupportingLineCoincidence)
    ));
    assert_eq!(count_gap(&census, CensusGap::SelfOffDiagonal), 2);
}
#[test]
fn cubic_self_domain_is_explicitly_incomplete_not_fixture_specific_success() {
    let edge = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let census = census(&[edge]);
    assert_eq!(census.domains.len(), 1);
    assert_eq!(
        census.domains[0].evidence,
        DomainEvidence::Unsupported(CensusGap::SelfOffDiagonal)
    );
    assert!(census.reports.is_empty());
}
#[test]
fn actual_mapped_split_witnesses_require_history_and_shared_vertex_incidence() {
    let mut edges = rectangle(0., 0., 4., 4.);
    edges.extend(rectangle(2., -1., 1., 6.));
    let census = census(&edges);
    let arranged = arranged(&edges);
    let evidence = inspect_contact_coverage(&census, &arranged, Plane3::default()).unwrap();
    assert!(evidence.iter().any(|row| !row.vertices.is_empty()));
    assert!(
        evidence
            .iter()
            .flat_map(|row| &row.vertices)
            .all(|v| v.generated_from_both)
    );
    assert!(evidence.iter().all(|row| !row.complete));
}
#[test]
fn source_mutation_does_not_rewrite_retained_owned_coefficients_or_pcurves() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let inputs = capture(std::slice::from_ref(&source));
    let copied = &inputs.segments[0];
    replace_second_pole(&source, 8.);
    source.set_closed(false).unwrap();
    assert_eq!(second_pole_x(&copied.edge), 4.);
    assert!(copied.edge.closed().unwrap());
    replace_second_pole(&copied.edge, 6.);
    assert_eq!(second_pole_x(&source), 8.);
    assert_eq!(copied.controls[1], [4., 0.]);
    assert_eq!(capture(&[source]).segments[0].controls[1], [8., 0.]);
}
fn source_curve(
    source: &Shape,
) -> rustycad_tkernel::standard::Handle<rustycad_tkg3d::geom::GeomBezierCurve> {
    use rustycad_tkbrep::{brep::BRepTool, topods::edge};
    let (curve, _, _) = BRepTool::curve(&edge(source).unwrap()).unwrap();
    curve.cast()
}
#[test]
fn cancellation_preserves_owned_prefix_and_actual_next_domain() {
    let shapes = rectangle(0., 0., 4., 4.);
    let mut polls = 0;
    let result = enumerate_boundary_census(capture(&shapes), policy(), || {
        polls += 1;
        polls > 1
    });
    let Err(CensusCause::Cancelled {
        next_domain,
        completed,
        ..
    }) = result
    else {
        panic!("expected cancelled domain census");
    };
    assert_eq!(next_domain, [0, 1]);
    assert_eq!(completed.domains.len(), 1);
    assert_eq!(completed.inputs.segments.len(), 4);
}
#[test]
fn candidate_parent_coverage_keeps_missing_partition_certificate() {
    let shapes = rectangle(0., 0., 4., 4.);
    let census = census(&shapes);
    let candidates = crate::sketch_region_candidates::tests::candidates(&shapes);
    let coverage = inspect_candidate_coverage(&census, &candidates);
    assert_eq!(coverage.len(), 4);
    assert!(
        coverage
            .iter()
            .all(|row| !row.represented_occurrences.is_empty())
    );
    assert!(coverage.iter().all(|row| !row.parameter_coverage_certified));
    assert!(
        census
            .missing
            .contains(&CertificationObligation::CandidateSearchAndCoverage)
    );
}
#[test]
fn typed_empty_and_invalid_inputs_preserve_actual_admission_cause() {
    assert!(matches!(
        capture_boundary_inputs(&[], Plane3::default()),
        Err(CensusCause::Input(
            crate::sketch_region::ArrangementCause::Empty
        ))
    ));
    let invalid = [ArrangementSource {
        identity: RegionSourceId(7),
        shape: &Shape::default(),
    }];
    assert!(matches!(
        capture_boundary_inputs(&invalid, Plane3::default()),
        Err(CensusCause::Input(
            crate::sketch_region::ArrangementCause::NullSource(RegionSourceId(7))
        ))
    ));
}
fn curved_crossing() -> Vec<Shape> {
    use rustycad_tkernel::{ncollection::Array1, standard::Handle};
    use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};
    use rustycad_tkmath::gp::Point3;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    let points = [
        Point3::new(0., 0., 0.),
        Point3::new(0.5, 0., 0.),
        Point3::new(1., 1., 0.),
    ];
    let poles = Array1::from_shared(&points, 1);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast();
    let bezier = BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap();
    vec![bezier.into_shape(), straight_crossing()]
}
fn straight_crossing() -> Shape {
    use rustycad_tkmath::gp::Point3;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    BRepBuilderAPIMakeEdge::from_points(Point3::new(-1., 0.25, 0.), Point3::new(2., 0.25, 0.))
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

#[test]
fn native_curved_split_keeps_actual_parameter_and_construction_difference() {
    use rustycad_tkbrep::{brep::BRepTool, topods::vertex};
    let edges = curved_crossing();
    let census = census(&edges);
    let arranged = arranged(&edges);
    let evidence = inspect_contact_coverage(&census, &arranged, Plane3::default()).unwrap();
    let rows = native_endpoints();
    let witnesses: Vec<_> = evidence.iter().flat_map(|row| &row.vertices).collect();
    assert!(!witnesses.is_empty());
    for witness in witnesses {
        let point = BRepTool::pnt(&vertex(&witness.vertex).unwrap()).unwrap();
        assert_eq!(witness.parameter, rows[0][0]);
        assert_eq!(witness.line_parameter, rows[1][0]);
        assert_eq!([point.x(), point.y(), point.z()], rows[0][1..4]);
        assert!(!witness.parameter_in_bracket && !witness.world_inside_enclosure);
    }
}
fn native_endpoints() -> Vec<Vec<f64>> {
    include_str!("../../tests/fixtures/boundary-census-native.txt")
        .lines()
        .map(|row| {
            let fields: Vec<_> = row.split_whitespace().collect();
            assert_eq!(
                fields.len(),
                7,
                "expected endpoint source parameter world3 incidents, received {row}"
            );
            assert_eq!(fields[0], "endpoint");
            assert_eq!(fields[6], "4");
            fields[2..6]
                .iter()
                .map(|field| field.parse().unwrap())
                .collect()
        })
        .collect()
}
#[test]
fn nonlinear_cross_pairs_remain_visible_and_unsupported() {
    let first = curved_crossing().remove(0);
    let second = curved_crossing().remove(0);
    let census = census(&[first, second]);
    assert_eq!(census.domains.len(), 3);
    assert_eq!(count_gap(&census, CensusGap::BezierPair), 1);
    assert_eq!(count_gap(&census, CensusGap::SelfOffDiagonal), 2);
    assert!(census.reports.is_empty());
}
#[test]
fn missing_generated_parent_does_not_get_repaired_by_world_proximity() {
    let edges = curved_crossing();
    let census = census(&edges);
    let mut arranged = arranged(&edges);
    for relation in &mut arranged.relations {
        relation
            .generated
            .retain(|parent| parent.source != RegionSourceId(2));
    }
    let evidence = inspect_contact_coverage(&census, &arranged, Plane3::default()).unwrap();
    assert!(evidence.iter().all(|contact| contact.vertices.is_empty()));
}
#[test]
fn captured_result_survives_later_failure_cancel_and_source_alias_changes() {
    let edges = curved_crossing();
    let published = census(&edges);
    let source = published.reports[0].source().clone();
    let contacts = published.reports[0].contacts().to_vec();
    assert!(matches!(
        capture_boundary_inputs(&[], Plane3::default()),
        Err(CensusCause::Input(_))
    ));
    assert!(matches!(
        enumerate_boundary_census(capture(&edges), policy(), || true),
        Err(CensusCause::Cancelled { .. })
    ));
    edges[0].set_closed(true).unwrap();
    published.inputs.segments[0].edge.set_closed(true).unwrap();
    assert_eq!(published.reports[0].source(), &source);
    assert_eq!(published.reports[0].contacts(), contacts);
    assert!(!edges[0].is_partner(&published.inputs.segments[0].edge));
}
#[test]
fn numerical_cancellation_retains_actual_failed_domain_evidence() {
    let mut polls = 0;
    let result = enumerate_boundary_census(capture(&curved_crossing()), policy(), || {
        polls += 1;
        polls >= 3
    });
    let Err(CensusCause::Cancelled {
        during_domain: true,
        completed,
        ..
    }) = result
    else {
        panic!("expected provider cancellation with retained failed domain");
    };
    assert_eq!(completed.domains.len(), 2);
    assert_eq!(completed.failures.len(), 1);
}
#[test]
fn invalid_policy_is_typed_even_when_all_domains_are_unsupported() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let mut invalid = policy();
    invalid.max_world_error = -2.;
    assert!(matches!(
        enumerate_boundary_census(capture(&[source]), invalid, || false),
        Err(CensusCause::Policy(CensusPolicyViolation::WorldError {
            received: -2.
        }))
    ));
}

fn replace_second_pole(source: &Shape, x: f64) {
    use rustycad_tkmath::gp::Point3;
    source_curve(source)
        .get()
        .unwrap()
        .set_pole(2, Point3::new(x, 0., 0.))
        .unwrap();
}
fn second_pole_x(source: &Shape) -> f64 {
    // Inspect the genuine owned curve, independently of the stored scalar descriptor.
    source_curve(source).get().unwrap().pole(2).unwrap().x()
}
#[test]
fn original_locations_and_reversed_aliases_remain_distinct_source_domains() {
    use rustycad_tkmath::{
        gp::{Transform3, Vector3},
        toploc::Location,
    };
    let source = curved_crossing().remove(0);
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(10., 0., 0.));
    let location = Location::new(transform);
    let located = source
        .located(&location, true)
        .unwrap()
        .oriented(Orientation::Reversed);
    let inputs = capture(&[source, located]);
    assert_located_capture(&inputs, &location);
}

fn assert_located_capture(inputs: &BoundaryInputs, location: &rustycad_tkmath::toploc::Location) {
    assert_eq!(inputs.segments.len(), 2);
    assert_eq!(inputs.segments[0].controls[0], [0., 0.]);
    assert_eq!(inputs.segments[1].controls[0], [10., 0.]);
    let located = &inputs.segments[1];
    assert!(located.reversed);
    assert_eq!(&located.parents[0].location, location);
    assert_eq!(located.parents[0].orientation, Orientation::Reversed);
}
