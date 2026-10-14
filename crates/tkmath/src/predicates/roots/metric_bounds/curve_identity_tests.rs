// SPDX-License-Identifier: GPL-2.0-only
//! Algebraic identities are independent of evaluator samples, endpoint coincidence or degree.
use super::*;

fn identity() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
fn chain(curve: MetricCurve<'_, 3>) -> MetricCurveChain<'_, 3> {
    MetricCurveChain {
        curve,
        embedding: identity(),
        placements: &[],
    }
}
fn polynomial(poles: &[[f64; 3]]) -> MetricCurve<'_, 3> {
    MetricCurve::Bezier(MetricBezier {
        poles,
        weights: None,
    })
}
#[test]
fn exact_identity_accepts_degree_elevation() {
    let first = [[0.; 3], [2., 4., 6.]];
    let second = [[0.; 3], [1., 2., 3.], [2., 4., 6.]];
    let elevated = compare_curve_chains(
        chain(polynomial(&first)),
        chain(polynomial(&second)),
        || false,
    );
    assert_eq!(elevated.unwrap(), CurveIdentityDecision::Identical);
}
#[test]
fn common_rational_weight_scaling_does_not_change_the_curve() {
    let second = [[0.; 3], [1., 2., 3.], [2., 4., 6.]];
    let a = MetricCurve::Bezier(MetricBezier {
        poles: &second,
        weights: Some(&[1., 2., 1.]),
    });
    let b = MetricCurve::Bezier(MetricBezier {
        poles: &second,
        weights: Some(&[4., 8., 4.]),
    });
    assert_eq!(
        compare_curve_chains(chain(a), chain(b), || false).unwrap(),
        CurveIdentityDecision::Identical
    );
}
#[test]
fn matching_endpoints_do_not_hide_an_edited_interior_coefficient() {
    let original = [[0.; 3], [1., 2., 0.], [2., 0., 0.]];
    let changed = [[0.; 3], [1., 3., 0.], [2., 0., 0.]];
    let decision = compare_curve_chains(
        chain(polynomial(&original)),
        chain(polynomial(&changed)),
        || false,
    );
    assert_eq!(
        decision.unwrap(),
        CurveIdentityDecision::Different {
            coordinate: 1,
            coefficient: 1
        }
    );
}
#[test]
fn separate_placements_preserve_translation_that_rounded_composition_loses() {
    let curve = MetricCurve::Line {
        origin: [0.; 3],
        direction: [1., 0., 0.],
    };
    let mut placements = [identity(); 3];
    placements[0].origin[0] = 9_007_199_254_740_992.;
    placements[1].origin[0] = 1.;
    placements[2].origin[0] = -placements[0].origin[0];
    let mut expected = chain(curve);
    expected.embedding.origin[0] = 1.;
    let actual = MetricCurveChain {
        placements: &placements,
        ..chain(curve)
    };
    assert_eq!(
        compare_curve_chains(expected, actual, || false).unwrap(),
        CurveIdentityDecision::Identical
    );
}
#[test]
fn finite_scale_family_keeps_constant_and_collapsed_maps_exact() {
    for scale in [f64::from_bits(1), 0., -1., f64::MAX] {
        let mut first = chain(MetricCurve::Line {
            origin: [1.; 3],
            direction: [0.; 3],
        });
        first.embedding.scale = scale;
        let second = chain(MetricCurve::Line {
            origin: [scale; 3],
            direction: [0.; 3],
        });
        assert_eq!(
            compare_curve_chains(first, second, || false).unwrap(),
            CurveIdentityDecision::Identical
        );
    }
}
#[test]
fn invalid_coefficients_are_typed() {
    let poles = [[0.; 3], [1.; 3]];
    let invalid = MetricCurve::Bezier(MetricBezier {
        poles: &poles,
        weights: Some(&[1., 0.]),
    });
    assert!(matches!(
        compare_curve_chains(chain(invalid), chain(polynomial(&poles)), || false),
        Err(ResidualError::Value {
            role: "weight",
            index: 1,
            value: 0.
        })
    ));
}
#[test]
fn cancellation_never_publishes_identity_and_does_not_poison_reuse() {
    let poles = [[0.; 3], [1.; 3]];
    let valid = chain(polynomial(&poles));
    let mut polls = 0;
    compare_curve_chains(valid, valid, || {
        polls += 1;
        false
    })
    .unwrap();
    for stop in 1..=polls {
        assert_cancelled_identity(valid, stop);
    }
    assert_eq!(
        compare_curve_chains(valid, valid, || false).unwrap(),
        CurveIdentityDecision::Identical
    );
}
fn assert_cancelled_identity(curve: MetricCurveChain<'_, 3>, stop: usize) {
    let mut calls = 0;
    let result = compare_curve_chains(curve, curve, || {
        calls += 1;
        calls == stop
    });
    assert!(matches!(
        result,
        Err(ResidualError::Roots(
            super::super::RootIsolationError::Cancelled
        ))
    ));
}

fn send_sync_identity<T: Send + Sync>(value: T) -> T {
    // The bound checks the public scalar types without a new test-only dependency.
    value
}
#[test]
fn owned_identity_and_borrowed_coefficients_are_send_sync() {
    let curve = chain(MetricCurve::Line {
        origin: [0.; 3],
        direction: [1.; 3],
    });
    let checked = send_sync_identity(curve);
    let decision = compare_curve_chains(checked, checked, || false).unwrap();
    assert_eq!(
        send_sync_identity(decision),
        CurveIdentityDecision::Identical
    );
}

#[test]
fn generated_identity_corpus_is_deterministic_across_threads_and_reuse() {
    let serial = identity_corpus();
    assert_eq!(serial.len(), 64);
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8).map(|_| scope.spawn(identity_corpus)).collect();
        for worker in workers {
            assert_eq!(worker.join().unwrap(), serial);
        }
    });
    assert_eq!(identity_corpus(), serial);
    println!("identity-corpus-v1:{serial:?}");
}
fn identity_corpus() -> Vec<CurveIdentityDecision> {
    (0..64).map(identity_case).collect()
    // Each expectation is the independently expanded quadratic middle-pole delta.
}
fn identity_case(index: usize) -> CurveIdentityDecision {
    let scale = index as f64 - 32.;
    let edit = (index % 2) as f64;
    let first = [[0.; 3], [2. * scale, 4. * scale, 0.]];
    let second = [[0.; 3], [scale, 2. * scale + edit, 0.], first[1]];
    let observed = compare_curve_chains(
        chain(polynomial(&first)),
        chain(polynomial(&second)),
        || false,
    )
    .unwrap();
    let expected = expected_middle_edit(edit);
    assert_eq!(observed, expected);
    observed
}

fn expected_middle_edit(edit: f64) -> CurveIdentityDecision {
    if edit == 0. {
        return CurveIdentityDecision::Identical;
    }
    CurveIdentityDecision::Different {
        coordinate: 1,
        coefficient: 1,
    }
}
