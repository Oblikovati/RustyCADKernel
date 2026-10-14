// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkmath::predicates::roots::metric_bounds::*;
mod failures;
mod generated;
mod ownership;
mod replay;
mod signed;
mod stored_degree;
const CUBIC: [[f64; 3]; 4] = [[0., 0., 0.], [4., 0., 0.], [0., 4., 0.], [0., 0., 0.]];
const IRRATIONAL: [[f64; 3]; 4] = [[0., 0., 0.], [1., 0., 0.], [-1., 0., 0.], [0., 0., 0.]];
fn query(poles: &[[f64; 3]]) -> BezierRegularityQuery<'_> {
    BezierRegularityQuery::new(41, poles, BezierRegularityOptions::v1(options(1e-6)))
}
fn report(poles: &[[f64; 3]]) -> BezierRegularityReport {
    certify_bezier_regularity(query(poles), || false).unwrap()
}
fn stationary(report: &BezierRegularityReport) -> &[RootBracket] {
    match report.decision() {
        BezierRegularityDecision::Stationary {
            zero_speed_isolation,
        } => zero_speed_isolation.roots(),
        other => panic!("expected stationary isolation, got {other:?}"),
    }
}
fn exact_root(root: &RootBracket, parameter: f64, multiplicity: usize) {
    assert_eq!(root.bounds().0, root.bounds().1);
    assert_eq!(
        root.bounds().0.compare_f64(parameter).unwrap(),
        Ordering::Equal
    );
    assert_eq!(root.multiplicity(), multiplicity);
}
#[test]
fn closed_cubic_is_regular_without_claiming_a_smooth_seam() {
    // C'=(12(1-4t+3t²),12(2t-3t²),0), norm²=16+32(9t²-9t+2)².
    let result = report(&CUBIC);
    assert!(matches!(
        result.decision(),
        BezierRegularityDecision::Regular { .. }
    ));
    assert_eq!(result.completion(), BezierRegularityCompletion::Complete);
    assert_eq!(result.denominator_degree(), 0);
    assert_eq!(result.speed_degree(), Some(4));
    assert!(result.denominator_isolation().roots().is_empty());
    assert_eq!(result.source().curve_id(), 41);
    assert_eq!(result.source().poles(), CUBIC);
}
#[test]
fn closed_endpoint_and_singleton_queries_keep_zero_speed() {
    let poles = [[0.; 3], [0.; 3], [1., 0., 0.]]; // x=t².
    exact_root(&stationary(&report(&poles))[0], 0., 2);
    let mut request = query(&poles);
    request.interval = [0.25, 1.];
    assert!(matches!(
        certify_bezier_regularity(request, || false)
            .unwrap()
            .decision(),
        BezierRegularityDecision::Regular { .. }
    ));
    request.interval = [0., 0.];
    exact_root(
        &stationary(&certify_bezier_regularity(request, || false).unwrap())[0],
        0.,
        2,
    );
    let reversed = [[1., 0., 0.], [0.; 3], [0.; 3]];
    exact_root(&stationary(&report(&reversed))[0], 1., 2);
}
#[test]
fn injective_cubic_can_have_an_interior_stationary_parameter() {
    let poles = [[-1., 0., 0.], [1., 0., 0.], [-1., 0., 0.], [1., 0., 0.]];
    // x=(2t-1)^3 is injective; Q=36(2t-1)^4.
    let result = report(&poles);
    assert_eq!(stationary(&result).len(), 1);
    exact_root(&stationary(&result)[0], 0.5, 4);
}
#[test]
fn constant_and_collapsed_placements_are_everywhere_stationary() {
    for poles in [vec![[7., 8., 9.]], vec![[7., 8., 9.]; 3]] {
        let result = report(&poles);
        assert_eq!(
            result.decision(),
            &BezierRegularityDecision::EverywhereStationary
        );
        assert_eq!(result.speed_degree(), None);
    }
    let mut request = query(&CUBIC);
    request.placement.scale = 0.;
    assert_eq!(
        certify_bezier_regularity(request, || false)
            .unwrap()
            .decision(),
        &BezierRegularityDecision::EverywhereStationary
    );
}
#[test]
fn irrational_stationary_parameters_are_complete_and_separate() {
    // x=3t-9t²+6t³; C'=3(1-6t+6t²), roots (3±sqrt(3))/6.
    let result = report(&IRRATIONAL);
    let roots = stationary(&result);
    assert_eq!(roots.len(), 2);
    for (root, (lower, upper)) in roots.iter().zip([(0.125, 0.25), (0.75, 0.875)]) {
        assert_eq!(
            root.bounds().0.compare_f64(lower).unwrap(),
            Ordering::Greater
        );
        assert_eq!(root.bounds().1.compare_f64(upper).unwrap(), Ordering::Less);
        assert_eq!(root.multiplicity(), 2);
    }
    assert!(result.bisections() > 0);
}
#[test]
fn finite_extreme_scales_never_use_float_squared_speed() {
    for magnitude in [f64::from_bits(1), f64::MIN_POSITIVE, 1., f64::MAX] {
        for sign in [-1., 1.] {
            let poles = [[0.; 3], [sign * magnitude, 0., 0.]];
            let mut request = query(&poles);
            request.placement.scale = magnitude;
            request.placement.origin = [f64::MAX; 3];
            assert!(matches!(
                certify_bezier_regularity(request, || false)
                    .unwrap()
                    .decision(),
                BezierRegularityDecision::Regular { .. }
            ));
        }
    }
}
