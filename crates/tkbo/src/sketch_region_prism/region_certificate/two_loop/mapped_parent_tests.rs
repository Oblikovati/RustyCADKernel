// SPDX-License-Identifier: GPL-2.0-only
//! Authored native roles and independent structural-unit arithmetic pin the shared lookup adapter.
use super::{
    failure::*, fixture::TwoLoopFixture, fixture_bindings, mapped_parents::*, member_types::*,
    request::*,
};
use crate::sketch_region::{OriginalMemberIdentity, PartitionOccurrenceIdentity, RegionSourceId};
fn exercise<R>(run: impl FnOnce(TwoLoopRegionRequest<'_, '_>, OriginalMemberIdentity) -> R) -> R {
    let fixture = TwoLoopFixture::nested();
    let sources = fixture.sources();
    let identities = fixture_bindings::identities(&fixture);
    let actual = PartitionOccurrenceIdentity::capture(sources[0].shape);
    let parent = OriginalMemberIdentity {
        source: RegionSourceId(101),
        topology_address: actual.topology_address,
        location: actual.location,
        orientation: actual.orientation,
    };
    let request = TwoLoopRegionRequest {
        bridge: &fixture.bridge,
        sources: &sources,
        identities: &identities,
        inner_interior: [1., 1.],
        material_seed: [0.25, 0.25],
        requested: fixture_bindings::options(),
    };
    run(request, parent)
}
#[test]
fn parent_mapper_preserves_duplicate_kind_and_empty_rows_with_exact_budget() {
    exercise(|mut request, parent| {
        let parents = [parent.clone(), parent.clone(), parent];
        request.requested.maximum_retained_units = 15; // Six bindings + four children + two rows + three parents.
        let result = map_two_loop_original_parents(&request, &[&parents, &[]])
            .unwrap_or_else(|e| panic!("{:?}", e.cause));
        assert_eq!(result.retained_units, 15);
        assert_eq!(
            result.rows,
            [
                vec![
                    TwoLoopOriginalUse {
                        source: RegionSourceId(101),
                        role: TwoLoopMemberRole::Boundary
                    };
                    3
                ],
                vec![]
            ]
        );
    });
}
#[test]
fn parent_mapper_one_short_budget_keeps_original_limit_cause() {
    exercise(|mut request, parent| {
        let parents = [parent.clone(), parent.clone(), parent];
        request.requested.maximum_retained_units = 14;
        let Err(failure) = map_two_loop_original_parents(&request, &[&parents, &[]]) else {
            panic!("expected exact aggregate limit refusal");
        };
        assert!(matches!(
            failure.cause,
            TwoLoopRegionCause::Limit {
                requested: 14,
                measured: 15,
                ..
            }
        ));
        assert_eq!(failure.operation, 77);
        assert_eq!(failure.stage, TwoLoopRegionStage::Membership);
    });
}
#[test]
fn parent_mapper_refuses_foreign_authored_source_without_coordinate_matching() {
    exercise(|request, mut parent| {
        parent.source = RegionSourceId(999);
        let Err(failure) = map_two_loop_original_parents(&request, &[&[parent]]) else {
            panic!("expected foreign parent refusal");
        };
        assert!(matches!(
            failure.cause,
            TwoLoopRegionCause::Count {
                actual: 0,
                expected: 1,
                ..
            }
        ));
    });
}
#[test]
fn parent_mapper_checks_zero_one_three_sources_before_budget_or_native_query() {
    for count in [0, 1, 3] {
        exercise(|mut request, _| {
            let null = rustycad_tkbrep::topods::Shape::default();
            let sources: Vec<_> = (0..count).map(|i| crate::sketch_region::ArrangementSource { identity: RegionSourceId(101 + i as u64), shape: &null }).collect();
            request.requested.maximum_retained_units = 0;
            let request = TwoLoopRegionRequest { sources: &sources, ..request };
            let Err(failure) = map_two_loop_original_parents(&request, &[]) else { panic!("expected source cardinality refusal"); };
            assert!(matches!(failure.cause, TwoLoopRegionCause::Count { actual, expected: 2, .. } if actual == count));
            assert_eq!(failure.retained_units, 0);
        });
    }
}
#[test]
fn original_parent_budget_keeps_prior_reservation_and_original_limit() {
    exercise(|mut request, parent| {
        request.requested.maximum_retained_units = 20;
        let result = map_two_loop_original_parents_after(&request, &[&[parent.clone()]], 8).unwrap_or_else(|_| panic!("exact 8+6+4+1+1 budget"));
        assert_eq!(result.retained_units, 20);
        request.requested.maximum_retained_units = 19;
        let Err(failure) = map_two_loop_original_parents_after(&request, &[&[parent]], 8) else { panic!("expected cumulative limit"); };
        assert!(matches!(failure.cause, TwoLoopRegionCause::Limit { requested: 19, measured: 20, .. }));
    });
}
#[test]
fn polynomial_descriptor_admission_retains_original_caps_and_negative_count_cause() {
    exercise(|mut request, _| {
        request.requested.maximum_poles = 4;
        request.requested.maximum_retained_units = 10;
        assert_eq!(super::admit_two_loop_polynomial_counts(&request, &[4,0], 4).unwrap_or_else(|_| panic!("exact cumulative geometry cap")),10);
        let Err(failure) = super::admit_two_loop_polynomial_counts(&request, &[5], 0) else { panic!("expected pole cap"); };
        assert!(matches!(failure.cause, TwoLoopRegionCause::Limit { operand: "polynomial poles", requested:4, measured:5 }));
        let Err(failure) = super::admit_two_loop_polynomial_counts(&request, &[-1], 0) else { panic!("expected negative count"); };
        assert!(matches!(failure.cause, TwoLoopRegionCause::Scalar { operand: "pole count", actual:-1., .. }));
    });
}
