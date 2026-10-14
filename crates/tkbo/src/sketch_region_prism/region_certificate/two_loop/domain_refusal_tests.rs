// SPDX-License-Identifier: GPL-2.0-only
//! Closed-form separation/crossing fixtures exercise the same pair provider, never an OCCT oracle.
use super::{fixture::*, fixture_bindings, operation_tests::*, pair_capture, work::*};
#[test]
fn nested_disjoint_and_separated_disjoint_pairs_are_distinct_from_containment() {
    for offset in [[0.5, 0.5], [4., 4.]] {
        let outer = cubic(1., [0., 0.]);
        let inner = cubic(0.5, offset);
        let sources = [
            crate::sketch_region::ArrangementSource {
                identity: OUTER,
                shape: &outer,
            },
            crate::sketch_region::ArrangementSource {
                identity: INNER,
                shape: &inner,
            },
        ];
        let census = census(&sources);
        let mut progress = OperationProgress;
        let mut work = TwoLoopWork::new(77, fixture_bindings::options(), &mut progress);
        let evidence = pair_capture::domains(&census, [OUTER, INNER], &mut work)
            .unwrap_or_else(|e| panic!("disjoint proof: {:?}", e.cause));
        assert_eq!(evidence.iter().filter(|p| p.closure.is_none()).count(), 1);
    }
}
#[test]
fn touching_and_crossing_boundaries_cannot_be_certified_as_disjoint() {
    // Shared closure is an exact touch. The second curve starts at the known interior center,
    // but its t=1/3 x=17/9 exceeds the outer maximum16/9, proving a crossing independently.
    for offset in [[0., 0.], [1., 1.]] {
        let outer = cubic(1., [0., 0.]);
        let inner = cubic(0.5, offset);
        let sources = [
            crate::sketch_region::ArrangementSource {
                identity: OUTER,
                shape: &outer,
            },
            crate::sketch_region::ArrangementSource {
                identity: INNER,
                shape: &inner,
            },
        ];
        let census = census(&sources);
        let mut progress = OperationProgress;
        let mut work = TwoLoopWork::new(77, fixture_bindings::options(), &mut progress);
        assert!(pair_capture::domains(&census, [OUTER, INNER], &mut work).is_err());
    }
}
#[test]
fn same_parent_labels_never_merge_actual_source_identity_or_supply_nesting() {
    use super::{operation::*, outcome::*, request::*};
    let fixture = TwoLoopFixture::nested();
    let sources = fixture.sources();
    let mut identities = fixture_bindings::identities(&fixture);
    std::mem::swap(&mut identities.outer, &mut identities.inner);
    let request = TwoLoopRegionRequest {
        bridge: &fixture.bridge,
        sources: &sources,
        identities: &identities,
        inner_interior: [1., 1.],
        material_seed: [0.25, 0.25],
        requested: fixture_bindings::options(),
    };
    assert!(matches!(
        inspect_two_loop_region(&request, &mut OperationProgress),
        TwoLoopRegionOutcome::Incomplete(_)
    ));
}

#[test]
fn coincident_distinct_authored_curves_never_become_complete_empty_cross_domain() {
    let outer = cubic(1., [0., 0.]);
    let inner = cubic(1., [0., 0.]);
    let sources = [
        crate::sketch_region::ArrangementSource {
            identity: OUTER,
            shape: &outer,
        },
        crate::sketch_region::ArrangementSource {
            identity: INNER,
            shape: &inner,
        },
    ];
    assert!(!outer.is_partner(&inner));
    let census = census(&sources);
    let mut progress = OperationProgress;
    let mut work = TwoLoopWork::new(77, fixture_bindings::options(), &mut progress);
    assert!(pair_capture::domains(&census, [OUTER, INNER], &mut work).is_err());
}
#[test]
fn complete_disjoint_unnested_boundaries_fail_the_native_closure_nesting_witness() {
    use super::{failure::*, request::*};
    let fixture = TwoLoopFixture::for_edges(cubic(1., [0., 0.]), cubic(0.5, [4., 4.]));
    let sources = fixture.sources();
    let captured =
        super::source_tests::capture(&fixture).unwrap_or_else(|_| panic!("actual source proof"));
    let identities = TwoLoopRegionIdentities {
        operation: 77,
        outer: OUTER,
        inner: INNER,
        selected: 301,
        candidates: Vec::new(),
    };
    let request = TwoLoopRegionRequest {
        bridge: &fixture.bridge,
        sources: &sources,
        identities: &identities,
        inner_interior: [4.5, 4.5],
        material_seed: [0.25, 0.25],
        requested: fixture_bindings::options(),
    };
    let mut progress = OperationProgress;
    let mut work = TwoLoopWork::new(77, request.requested, &mut progress);
    let failure = super::nesting::capture(&request, &captured, &mut work)
        .err()
        .unwrap();
    assert!(matches!(
        failure.cause,
        TwoLoopRegionCause::WindingLocation { source: OUTER, .. }
    ));
}
