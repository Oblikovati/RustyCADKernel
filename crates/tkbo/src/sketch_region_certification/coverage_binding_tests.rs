// SPDX-License-Identifier: GPL-2.0-only
use super::source_binding_tests::source_policy;
use super::tests::{capture, census, curved_crossing, policy};
use super::*;
use crate::sketch_region_candidates::tests::{arranged, rectangle};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::gp::Plane3;
#[test]
fn exact_native_membership_cannot_promote_canonical_contact_dual_axis_or_partition() {
    let mut shapes = rectangle(0., 0., 4., 4.);
    shapes.extend(rectangle(2., -1., 1., 6.));
    let census =
        enumerate_source_bound_census(capture(&shapes), policy(), source_policy(), || false)
            .unwrap_or_else(|_| panic!("census"));
    let coverage =
        inspect_contact_coverage(&census, &arranged(&shapes), Plane3::default()).unwrap();
    assert!(coverage.iter().any(|row| !row.vertices.is_empty()));
    for row in coverage {
        assert_remaining_contact_gaps(&row);
    }
}
fn assert_remaining_contact_gaps(row: &ContactCoverage) {
    assert!(!row.complete);
    assert!(
        !row.missing
            .contains(&EndpointGap::RepresentationUncertified)
    );
    for gap in [
        EndpointGap::CanonicalContactIdentityUncertified,
        EndpointGap::DualAxisParameterUncertified,
        EndpointGap::World3dErrorUncertified,
        EndpointGap::PartitionUncertified,
    ] {
        assert!(row.missing.contains(&gap));
    }
}
#[test]
fn old_native_curved_split_raw_parameter_and_point_survive_new_exact_transport() {
    let shapes = curved_crossing();
    let census = census(&shapes);
    let coverage =
        inspect_contact_coverage(&census, &arranged(&shapes), Plane3::default()).unwrap();
    let mut count = 0;
    for vertex in coverage.iter().flat_map(|row| &row.vertices) {
        let membership = vertex.native_membership.as_ref().unwrap();
        assert_eq!(membership.belongs, vertex.parameter_in_bracket);
        assert!(!membership.belongs && !membership.singleton_equal);
        assert_eq!(
            membership
                .supplied_parameter
                .compare_f64(vertex.parameter)
                .unwrap(),
            std::cmp::Ordering::Equal
        );
        count += 1;
    }
    assert!(count > 0);
}
#[test]
fn altered_incidence_curve_or_line_coefficients_never_accept_source_id_only() {
    let shapes = curved_crossing();
    let mut curve = census(&shapes);
    curve.inputs.segments[0].controls[1][0] += 0.5;
    let coverage = inspect_contact_coverage(&curve, &arranged(&shapes), Plane3::default()).unwrap();
    assert_unbound(&coverage);
    let mut line = census(&shapes);
    line.inputs.segments[1].controls[0][1] += 0.5;
    assert_unbound(
        &inspect_contact_coverage(&line, &arranged(&shapes), Plane3::default()).unwrap(),
    );
}
fn assert_unbound(coverage: &[ContactCoverage]) {
    assert!(!coverage.is_empty());
    assert!(
        coverage.iter().all(|row| row.vertices.is_empty()
            && row.missing.contains(&EndpointGap::IncidenceSourceUnbound))
    );
}
#[test]
fn changed_incidence_root_or_world_policy_is_not_reused_from_identical_ids() {
    let shapes = curved_crossing();
    let mut changed = census(&shapes);
    changed.requested.roots.max_width *= 2.;
    assert_unbound(
        &inspect_contact_coverage(&changed, &arranged(&shapes), Plane3::default()).unwrap(),
    );
    let mut changed = census(&shapes);
    changed.requested.max_world_error *= 2.;
    assert_unbound(
        &inspect_contact_coverage(&changed, &arranged(&shapes), Plane3::default()).unwrap(),
    );
}
#[test]
fn valid_reversed_physical_reuse_preserves_actual_descriptor_and_contact_incompleteness() {
    let mut shapes = curved_crossing();
    shapes.push(shapes[0].reversed());
    let census = census(&shapes);
    let coverage =
        inspect_contact_coverage(&census, &arranged(&shapes), Plane3::default()).unwrap();
    assert!(
        coverage
            .iter()
            .all(|row| !row.missing.contains(&EndpointGap::IncidenceSourceUnbound))
    );
    assert!(coverage.iter().all(|row| !row.complete));
}
#[test]
fn parent_or_construction_mutation_after_attachment_reinserts_observed_representation_gap() {
    let shapes = curved_crossing();
    let mut census =
        enumerate_source_bound_census(capture(&shapes), policy(), source_policy(), || false)
            .unwrap_or_else(|_| panic!("census"));
    assert!(
        !census
            .missing
            .contains(&CertificationObligation::SourceConversionAndPlane)
    );
    census.inputs.segments[0].original_interval[1] *= 0.5;
    assert_stale_representation_visible(&census, &shapes);
}

fn assert_stale_representation_visible(census: &BoundaryCensus, shapes: &[Shape]) {
    assert!(
        census
            .current_obligations()
            .contains(&CertificationObligation::SourceConversionAndPlane)
    );
    let coverage = inspect_contact_coverage(census, &arranged(shapes), Plane3::default()).unwrap();
    assert!(coverage.iter().all(|row| {
        row.missing
            .contains(&EndpointGap::RepresentationUncertified)
    }));
}

#[test]
fn nonzero_representation_error_never_establishes_original_contact_or_world_topology() {
    let shapes = offplane_crossing();
    let mut requested = source_policy();
    requested.original_plane_limit = 0.3;
    requested.construction_limit = 0.3;
    let census = enumerate_source_bound_census(capture(&shapes), policy(), requested, || false)
        .unwrap_or_else(|_| panic!("census"));
    assert!(
        census
            .source_binding
            .as_ref()
            .unwrap()
            .representation_certified()
    );
    let coverage =
        inspect_contact_coverage(&census, &arranged(&shapes), Plane3::default()).unwrap();
    assert!(coverage.iter().any(|row| !row.vertices.is_empty()));
    for row in &coverage {
        assert_remaining_contact_gaps(row);
    }
}

fn offplane_crossing() -> Vec<rustycad_tkbrep::topods::Shape> {
    use rustycad_tkmath::{gp::Transform3, toploc::Location};
    let mut shapes = rectangle(0., 0., 4., 4.);
    shapes.extend(rectangle(2., -1., 1., 6.));
    let location = Location::new(Transform3::translation([0., 0., 0.25]));
    let shapes: Vec<_> = shapes
        .iter()
        .map(|shape| shape.located(&location, false).unwrap())
        .collect();
    shapes
}

#[test]
fn orphan_report_and_duplicate_producer_id_are_visible_unbound_not_empty_passes() {
    let shapes = curved_crossing();
    let mut orphan = census(&shapes);
    orphan.domains.clear();
    assert_unbound(
        &inspect_contact_coverage(&orphan, &arranged(&shapes), Plane3::default()).unwrap(),
    );
    let mut duplicate = census(&shapes);
    duplicate.inputs.segments[1].occurrence = duplicate.inputs.segments[0].occurrence;
    assert_unbound(
        &inspect_contact_coverage(&duplicate, &arranged(&shapes), Plane3::default()).unwrap(),
    );
}

#[test]
fn missing_stale_or_out_of_range_native_map_returns_typed_incomplete_evidence() {
    use crate::sketch_region_source_conformance::SourceConformanceCause;
    let shapes = curved_crossing();
    let mut proof = census(&shapes);
    let contact = proof.reports[0].contacts()[0].clone();
    assert!(matches!(
        super::parameter_binding::contact_native_membership(&proof, 99, &contact, 0.5),
        Err(SourceConformanceCause::InvalidSegmentIndex { received: 99, .. })
    ));
    proof.inputs.constructions.clear();
    assert!(matches!(
        super::parameter_binding::contact_native_membership(&proof, 0, &contact, 0.5),
        Err(SourceConformanceCause::MissingConstruction { .. })
    ));
    let mut proof = census(&shapes);
    proof.inputs.segments[0].original_interval[1] *= 0.5;
    assert!(matches!(
        super::parameter_binding::contact_native_membership(&proof, 0, &contact, 0.5),
        Err(SourceConformanceCause::CurrentDescriptorMismatch { .. })
    ));
}

#[test]
fn altered_separately_declared_source_policy_cannot_reuse_old_attached_proof() {
    let shapes = curved_crossing();
    let mut census =
        enumerate_source_bound_census(capture(&shapes), policy(), source_policy(), || false)
            .unwrap_or_else(|_| panic!("census"));
    census.source_policy.as_mut().unwrap().construction_limit = 1.;
    assert!(!census.source_representation_certified());
    assert!(
        census
            .current_obligations()
            .contains(&CertificationObligation::SourceConversionAndPlane)
    );
    let rows = inspect_contact_coverage(&census, &arranged(&shapes), Plane3::default()).unwrap();
    assert!(rows.iter().all(|row| {
        row.missing
            .contains(&EndpointGap::RepresentationUncertified)
    }));
}
