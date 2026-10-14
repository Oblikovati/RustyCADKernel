// SPDX-License-Identifier: GPL-2.0-only
//! Named invalid, cancelled and incomplete outcomes; none returns partial contacts.
use super::*;
use SketchIncidenceCause as Cause;
use rustycad_tkmath::predicates::roots::RootIsolationError;

const SEGMENT: SketchLineDomain = SketchLineDomain::Segment;

fn attempt(
    targets: &[SketchIncidenceTarget],
    options: SketchIncidenceOptions,
) -> SketchIncidenceFailure {
    certify_sketch_line_incidences(
        source([[-2., 0.5], [2., 0.5]], SEGMENT),
        targets,
        options,
        || false,
    )
    .expect_err("expected a named failure")
}
fn invalid(curve: SketchIncidenceCurve) -> (Option<u64>, SketchIncidenceCompletion, Cause) {
    let failure = attempt(&[target(6, curve)], SketchIncidenceOptions::v1(ERROR));
    assert_eq!(failure.stage(), SketchIncidenceStage::Input);
    (
        failure.target(),
        failure.completion(),
        failure.cause().clone(),
    )
}
#[test]
fn invalid_options_and_source_name_the_offending_value() {
    let mut options = SketchIncidenceOptions::v1(ERROR);
    options.version = 2;
    assert_eq!(attempt(&[], options).cause(), &Cause::OptionsVersion(2));
    for limit in [0., -1., f64::NAN, f64::INFINITY] {
        let failure = attempt(&[], SketchIncidenceOptions::v1(limit));
        assert!(
            matches!(failure.cause(), Cause::WorldErrorLimit(value) if value.to_bits() == limit.to_bits())
        );
        assert_eq!(failure.completion(), SketchIncidenceCompletion::Invalid);
    }
    let degenerate = source([[1., 1.], [1., 1.]], SEGMENT);
    let failure =
        certify_sketch_line_incidences(degenerate, &[], SketchIncidenceOptions::v1(ERROR), || {
            false
        })
        .unwrap_err();
    assert_eq!(
        failure.cause(),
        &Cause::Degenerate {
            field: "source.points"
        }
    );
    assert_eq!(failure.target(), None);
}
#[test]
fn invalid_targets_are_refused_before_any_work() {
    use SketchIncidenceCompletion::Invalid;
    let nan = invalid(circle([0., f64::NAN], 1.));
    assert!(
        matches!(nan, (Some(6), Invalid, Cause::NonFinite { field: "conic.center", index: 1, value }) if value.is_nan())
    );
    assert_eq!(
        invalid(circle([0., 0.], 0.)),
        (
            Some(6),
            Invalid,
            Cause::NonPositive {
                field: "conic.radii",
                index: 0,
                value: 0.
            }
        )
    );
    assert_eq!(
        invalid(ellipse([0., 0.], [0., 0.], [1., 1.])).2,
        Cause::Degenerate {
            field: "conic.major_axis"
        }
    );
    let same_ray = arc([0., 0.], 1., [1., 1.], [2., 2.], true);
    assert_eq!(
        invalid(same_ray).2,
        Cause::CoincidentSector {
            start: [1., 1.],
            end: [2., 2.]
        }
    );
    assert_eq!(
        invalid(segment([1., 2.], [1., 2.])).2,
        Cause::Degenerate {
            field: "line.points"
        }
    );
}
#[test]
fn invalid_splines_name_shape_knots_bounds_and_weights() {
    let poles = [[0., 0.], [1., 1.], [2., 0.]];
    let shape = spline(2, &poles, &[0., 0., 0., 1., 1.], [0., 1.]);
    assert!(matches!(
        invalid(shape).2,
        Cause::SplineShape {
            degree: 2,
            poles: 3,
            knots: 5,
            weights: None
        }
    ));
    let decreasing = spline(1, &poles, &[0., 0., 2., 1., 1.], [0., 1.]);
    assert_eq!(invalid(decreasing).2, Cause::KnotSequence { index: 3 });
    let split = spline(
        1,
        &[[0., 0.], [1., 1.], [2., 0.], [3., 1.]],
        &[0., 0., 1., 1., 2., 2.],
        [0., 2.],
    );
    assert_eq!(invalid(split).2, Cause::KnotSequence { index: 2 });
    let outside = spline(1, &poles, &[0., 0., 1., 2., 2.], [-1., 1.]);
    assert_eq!(
        invalid(outside).2,
        Cause::SplineBounds {
            bounds: [-1., 1.],
            domain: [0., 2.]
        }
    );
    let weighted = SketchIncidenceCurve::Spline(SketchIncidenceSpline {
        degree: 1,
        poles: poles.to_vec(),
        weights: Some(vec![1., -1., 1.]),
        flat_knots: vec![0., 0., 1., 2., 2.],
        bounds: [0., 2.],
    });
    assert_eq!(
        invalid(weighted).2,
        Cause::NonPositive {
            field: "spline.weights",
            index: 1,
            value: -1.
        }
    );
}
#[test]
fn cancellation_is_named_and_returns_no_contacts() {
    let failure = certify_sketch_line_incidences(
        source([[-2., 0.5], [2., 0.5]], SEGMENT),
        &[target(6, circle([0., 0.], 1.))],
        SketchIncidenceOptions::v1(ERROR),
        || true,
    )
    .unwrap_err();
    assert_eq!(failure.completion(), SketchIncidenceCompletion::Cancelled);
    assert_eq!(failure.target(), Some(6));
}
#[test]
fn exhausted_refinement_is_incomplete_construction() {
    let mut options = SketchIncidenceOptions::v1(ERROR);
    options.max_refinements = 0;
    let failure = attempt(&[target(6, circle([0., 0.], 1.))], options);
    assert_eq!(failure.cause(), &Cause::RefinementExhausted { limit: 0 });
    assert_eq!(failure.completion(), SketchIncidenceCompletion::Incomplete);
    assert_eq!(failure.target(), Some(6));
}
#[test]
fn unreachable_world_error_stops_at_the_binary64_resolution() {
    let failure = attempt(
        &[target(6, circle([0., 0.], 1.))],
        SketchIncidenceOptions::v1(1e-300),
    );
    assert_eq!(failure.stage(), SketchIncidenceStage::Construction);
    let Cause::ConstructionResolution {
        requested,
        achieved: Some(achieved),
    } = failure.cause()
    else {
        panic!("{failure}");
    };
    assert_eq!(*requested, 1e-300);
    assert!(*achieved > 1e-300 && *achieved < 1e-15);
}
#[test]
fn isolation_and_degree_limits_are_typed_root_failures() {
    let mut options = SketchIncidenceOptions::v1(ERROR);
    options.max_isolation_bisections = 0;
    let failure = attempt(&[target(6, circle([0., 0.], 1.))], options);
    assert_eq!(failure.stage(), SketchIncidenceStage::Isolation);
    assert!(matches!(
        failure.cause(),
        Cause::Root(RootIsolationError::SubdivisionLimit { limit: 0 })
    ));
    let mut options = SketchIncidenceOptions::v1(ERROR);
    options.max_degree = 1;
    let failure = attempt(&[target(6, circle([0., 0.], 1.))], options);
    assert!(matches!(
        failure.cause(),
        Cause::Root(RootIsolationError::Degree {
            actual: 2,
            limit: 1
        })
    ));
}
