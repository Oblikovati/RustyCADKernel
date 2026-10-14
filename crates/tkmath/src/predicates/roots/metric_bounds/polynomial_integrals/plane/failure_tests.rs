// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[test]
fn boundary_failures_keep_curve_index_and_original_source_chain() {
    let invalid = [polynomial(&POINT), line([f64::NAN, 0.], [0., 1.])];
    let error = certify_polynomial_plane_measure(
        &invalid,
        frame(),
        placement(),
        Default::default(),
        || false,
    )
    .unwrap_err();
    assert!(matches!(&error.cause, PlaneFailure::Boundary(cause) if cause.curve_index == Some(1)));
    assert!(error.source().unwrap().source().unwrap().source().is_some());
    assert!(
        error
            .to_string()
            .contains("complete admitted native polynomial")
    );
}
#[test]
fn singular_zero_does_not_hide_nonfinite_placement() {
    let mut placed = placement();
    placed.origin[2] = f64::INFINITY;
    let error = certify_polynomial_plane_measure(
        &occurrences("point"),
        frame(),
        placed,
        Default::default(),
        || false,
    )
    .unwrap_err();
    assert!(matches!(&error.cause, PlaneFailure::Metric(cause)
        if matches!(cause.as_ref(), super::super::super::super::ResidualError::Value { role:"origin", index:2, .. })));
    assert!(error.source().unwrap().source().is_some());
    assert!(error.to_string().contains("finite placed support"));
}
#[test]
fn every_observed_cancellation_poll_refuses_and_retries_identically() {
    let mut polls = 0;
    let curves = occurrences("cubic_ccw");
    let baseline = plane_attempt(&curves, Default::default(), || {
        polls += 1;
        false
    })
    .unwrap();
    for cutoff in 1..=polls {
        assert_cancelled_at(&curves, cutoff);
        assert_eq!(plane_report(&curves, frame()), baseline);
    }
}
fn assert_cancelled_at(curves: &[OrientedMetricCurve<'_>], cutoff: usize) {
    let mut count = 0;
    let error = plane_attempt(curves, Default::default(), || {
        count += 1;
        count == cutoff
    })
    .unwrap_err();
    assert!(error.to_string().contains("Cancelled"));
}

#[test]
fn reports_own_scalars_after_source_mutation_failure_cancel_and_reuse() {
    let mut poles = CUBIC;
    let original = plane_report(&[polynomial(&poles)], frame());
    let copy = original.clone();
    let curves = [polynomial(&poles)];
    assert!(plane_attempt(&curves, Default::default(), || true).is_err());
    let zero = PolynomialIntegralOptions {
        absolute_error_limit: 0.,
        ..Default::default()
    };
    assert!(plane_attempt(&curves, zero, || false).is_err());
    assert_eq!(poles, CUBIC);
    poles.fill([0.; 2]);
    assert_eq!(
        plane_report(&[polynomial(&poles)], frame()).enclosure(),
        [0., 0.]
    );
    assert_eq!(original, copy);
    assert_squared_enclosure(&original, &rational("144", "25"));
}
#[test]
fn independent_threads_return_identical_owned_reports() {
    let baseline = plane_report(&occurrences("cubic_ccw"), frame());
    let threads: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(|| {
                (0..32)
                    .map(|_| plane_report(&occurrences("cubic_ccw"), frame()))
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    for thread in threads {
        for report in thread.join().unwrap() {
            assert_eq!(report, baseline);
        }
    }
}

#[test]
fn empty_policy_and_rational_domains_remain_typed_incomplete() {
    let empty = plane_attempt(&[], Default::default(), || false).unwrap_err();
    assert!(
        matches!(empty.cause, PlaneFailure::Boundary(error) if matches!(error.cause, Failure::EmptyBoundary))
    );
    let options = PolynomialIntegralOptions {
        behavior_version: 2,
        ..Default::default()
    };
    let invalid = plane_attempt(&square_occurrences(), options, || false).unwrap_err();
    assert!(
        matches!(invalid.cause, PlaneFailure::Boundary(error) if matches!(error.cause, Failure::Policy(actual) if actual == options))
    );
}
#[test]
fn rational_plane_boundary_retains_unsupported_curve_index() {
    let weighted = OrientedMetricCurve {
        curve: MetricCurve::Bezier(MetricBezier {
            poles: &CUBIC,
            weights: Some(&[1.; 4]),
        }),
        reversed: false,
    };
    let rational = plane_attempt(&[weighted], Default::default(), || false).unwrap_err();
    assert!(
        matches!(rational.cause, PlaneFailure::Boundary(error) if error.curve_index == Some(0) && matches!(error.cause, Failure::RationalWeights { count:4 }))
    );
}
