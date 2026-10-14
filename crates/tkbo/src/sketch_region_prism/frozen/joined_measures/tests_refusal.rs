// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[test]
fn unsupported_behavior_preserves_requested_bits_before_callbacks() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut requested = options();
    requested.version = 7;
    let error = inspect_joined_prism_measures(&join, requested, || {
        panic!("version admitted before callback")
    })
    .err()
    .unwrap();
    assert_eq!(error.operation, join.projection().operation);
    assert_eq!(error.requested, requested);
    assert_eq!(error.class, JoinedMeasureFailureClass::Incomplete);
    assert!(matches!(
        error.cause,
        JoinedPrismMeasureCause::Version {
            actual: 7,
            expected: 1
        }
    ));
    assert_eq!(error.entities.len(), 3);
}
#[test]
fn every_observed_cancellation_returns_owned_cause_and_preserves_retry() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let baseline = measured(&join).replay_bytes();
    let mut polls = 0;
    inspect_joined_prism_measures(&join, options(), || {
        polls += 1;
        false
    })
    .unwrap();
    for cutoff in 1..=polls {
        let mut current = 0;
        let error = inspect_joined_prism_measures(&join, options(), || {
            current += 1;
            current == cutoff
        })
        .err()
        .unwrap();
        assert_eq!(
            error.class,
            JoinedMeasureFailureClass::Cancelled,
            "cutoff {cutoff}: {error:?}"
        );
        assert_eq!(error.requested, options());
        assert_eq!(current, cutoff);
    }
    assert_eq!(measured(&join).replay_bytes(), baseline);
    println!("joined_measure_cancel_polls={polls}");
}
#[test]
fn separate_area_volume_and_total_caps_refuse_without_widening() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    for quantity in 0..4 {
        let mut requested = options();
        match quantity {
            0 => requested.scalars.subset.cap_area.absolute_error_limit = 0.,
            1 => requested.scalars.subset.signed_volume.absolute_error_limit = 0.,
            2 => requested.scalars.side_area.max_subdivisions = 0,
            _ => requested.scalars.total_area.absolute_error_limit = 0.,
        }
        let error = inspect_joined_prism_measures(&join, requested, || false)
            .err()
            .unwrap();
        assert_eq!(error.class, JoinedMeasureFailureClass::Incomplete);
        assert_eq!(error.requested, requested);
        assert_quantity_cause(quantity, error.cause);
    }
}
fn assert_quantity_cause(quantity: usize, cause: JoinedPrismMeasureCause) {
    let JoinedPrismMeasureCause::Arithmetic(cause) = cause else {
        panic!("expected exact quantity cause")
    };
    match (quantity, *cause) {
        (
            0,
            BoundPrismMeasureFailure::Subset(PrismScalarFailure::Cap {
                role: PrismImageRole::First,
                cause,
            }),
        ) => assert!(matches!(
            cause.cause,
            PolynomialPlaneFailure::ErrorLimit { requested: 0., .. }
        )),
        (1, BoundPrismMeasureFailure::Subset(PrismScalarFailure::Sweep(cause))) => {
            assert!(matches!(
                cause.cause,
                PolynomialIntegralFailure::ErrorLimit { requested: 0., .. }
            ))
        }
        (2, BoundPrismMeasureFailure::Side { cause, .. }) => assert!(matches!(
            *cause.cause,
            PolynomialSideFailure::WorkLimit { requested: 0, .. }
        )),
        (3, BoundPrismMeasureFailure::Total(cause)) => assert!(matches!(
            cause.cause,
            EnclosureSumCause::ErrorLimit { requested: 0., .. }
        )),
        (_, cause) => panic!("unexpected quantity {quantity} failure: {cause:?}"),
    }
}
#[test]
fn missing_and_duplicate_named_faces_are_incomplete_without_panicking() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut names = join.faces().to_vec();
    names.pop();
    assert!(matches!(
        admission::name_triplet(&names),
        Err(JoinedPrismMeasureCause::Count {
            actual: 2,
            expected: 3,
            ..
        })
    ));
    names = join.faces().to_vec();
    names[1].role = names[0].role;
    assert!(matches!(
        admission::name_triplet(&names),
        Err(JoinedPrismMeasureCause::Count { expected: 1, .. })
    ));
}
