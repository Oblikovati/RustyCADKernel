// SPDX-License-Identifier: GPL-2.0-only
use super::incidences::request;
use super::*;
use DirectedBezierPosition::*;

pub(super) fn directed(mut query: BezierIncidenceQuery<'_>) -> DirectedBezierIncidenceReport {
    query.domain = BezierLineDomain::Support;
    certify_directed_bezier_incidences(DirectedBezierIncidenceQuery::v1(query), || false).unwrap()
}
#[test]
fn rational_third_compares_exactly_against_neighboring_binary64_endpoints() {
    let controls = [[0., -1.], [1., 2.]];
    let mut query = request(&controls);
    query.curve.weights = Some(&[2., 1.]);
    let lower = f64::from_bits(0x3fd5555555555555);
    let upper = f64::from_bits(0x3fd5555555555556);
    for (line, expected) in [
        ([[lower, 0.], [1., 0.]], Interior),
        ([[upper, 0.], [1., 0.]], BeforeOrigin),
        ([[0., 0.], [lower, 0.]], BeyondTerminal),
        ([[0., 0.], [upper, 0.]], Interior),
    ] {
        query.curve.line = line;
        let result = directed(query);
        assert_eq!(result.positions(), &[expected]);
        assert_half_parameter(&result);
    }
}
fn assert_half_parameter(result: &DirectedBezierIncidenceReport) {
    let root = result.incidence().contacts()[0].parameter();
    assert_eq!(root.bounds().0.compare_f64(0.5).unwrap(), Ordering::Equal);
    assert_eq!(root.bounds().1.compare_f64(0.5).unwrap(), Ordering::Equal);
}
#[test]
fn generated_dyadic_contacts_cover_five_positions_and_line_reversal() {
    for exponent in -4..=4 {
        let scale = 2_f64.powi(exponent);
        for (offset, forward, backward) in [
            (-1., BeforeOrigin, BeyondTerminal),
            (0., AtOrigin, AtTerminal),
            (1., Interior, Interior),
            (2., AtTerminal, AtOrigin),
            (3., BeyondTerminal, BeforeOrigin),
        ] {
            check_generated_position(offset * scale, scale, forward, backward);
        }
    }
}
fn check_generated_position(
    x: f64,
    scale: f64,
    forward: DirectedBezierPosition,
    backward: DirectedBezierPosition,
) {
    let controls = [[x, -scale], [x, scale]];
    let mut query = request(&controls);
    query.curve.line = [[0., 0.], [2. * scale, 0.]];
    let report = directed(query);
    assert_eq!(report.positions(), &[forward]);
    query.curve.line.reverse();
    query.curve.weights = Some(&[-1., -1.]);
    let reversed = directed(query);
    assert_eq!(reversed.positions(), &[backward]);
    assert_eq!(
        report.incidence().contacts(),
        reversed.incidence().contacts()
    );
    assert_eq!(
        report.incidence().support_isolation(),
        reversed.incidence().support_isolation()
    );
}
#[test]
fn tangent_root_identity_and_segment_filter_share_complete_isolation() {
    let controls = [[0., 27.], [1., -45.], [2., 59.], [3., -45.], [4., 27.]];
    let mut query = request(&controls);
    query.curve.line = [[2., 0.], [3., 0.]];
    let report =
        certify_directed_bezier_incidences(DirectedBezierIncidenceQuery::v1(query), || false)
            .unwrap();
    assert_eq!(report.version(), 1);
    assert_eq!(report.positions(), &[BeforeOrigin, AtTerminal]);
    assert_eq!(report.incidence().in_domain(), &[false, true]);
    assert_eq!(report.incidence().contacts()[0].support_root_index(), 1);
    assert_eq!(
        report.incidence().contacts()[0].parameter().multiplicity(),
        2
    );
    assert_eq!(
        report.incidence(),
        &certify_bezier_line_incidences(query, || false).unwrap()
    );
}
#[test]
fn parameter_order_is_not_directed_distance_order() {
    let controls = [[3., -9.], [2., 13.], [1., -13.], [0., 9.]];
    let mut query = request(&controls);
    query.curve.line = [[1., 0.], [2., 0.]];
    let report = directed(query);
    assert_eq!(
        report.positions(),
        &[BeyondTerminal, Interior, BeforeOrigin]
    );
    for (index, contact) in report.incidence().contacts().iter().enumerate() {
        assert_eq!(contact.support_root_index(), index);
        assert_eq!(contact.point(), [2.25 - 0.75 * index as f64, 0.]);
    }
}
#[test]
fn exact_origin_and_terminal_tangencies_remain_equal_after_weight_sign_reversal() {
    let controls = [[0., 27.], [1., -45.], [2., 59.], [3., -45.], [4., 27.]];
    let mut query = request(&controls);
    query.curve.line = [[1., 0.], [3., 0.]];
    assert_eq!(directed(query).positions(), &[AtOrigin, AtTerminal]);
    query.curve.weights = Some(&[-1.; 5]);
    query.curve.line.reverse();
    assert_eq!(directed(query).positions(), &[AtTerminal, AtOrigin]);
}
#[test]
fn signed_and_zero_individual_weights_use_certified_denominator_signs() {
    let controls = [[1., -1.], [1., 0.], [1., 1.]];
    for weights in [[2., -1., 2.], [-2., 1., -2.], [2., 0., 2.]] {
        let mut query = request(&controls);
        query.curve.line = [[0., 0.], [1., 0.]];
        query.curve.weights = Some(&weights);
        let report = directed(query);
        assert_eq!(report.positions(), &[AtTerminal]);
        assert_eq!(report.incidence().contacts()[0].point(), [1., 0.]);
        assert!(
            report
                .incidence()
                .denominator_isolation()
                .roots()
                .is_empty()
        );
    }
}
#[test]
fn empty_selected_domain_retains_complete_outside_positions() {
    let mut query = request(&[[0., -1.], [1., 1.]]);
    query.curve.line = [[5., 0.], [6., 0.]];
    let report =
        certify_directed_bezier_incidences(DirectedBezierIncidenceQuery::v1(query), || false)
            .unwrap();
    assert_eq!(report.positions(), &[BeforeOrigin]);
    assert_eq!(report.incidence().in_domain(), &[false]);
    assert!(report.incidence().contacts().is_empty());
    assert_eq!(report.incidence().support_isolation().roots().len(), 1);
    assert_eq!(
        report.incidence().completion(),
        BezierIncidenceCompletion::Complete
    );
}
#[test]
fn irrational_root_positions_share_original_support_isolation_and_construction() {
    // y=6t²-6t+1, x=2t: contacts x=1±sqrt(1/3) lie outside [1/2,3/2]
    // because 1/3 > (1/2)². No rounded square root supplies the expected signs.
    let mut query = request(&[[0., 1.], [1., -2.], [2., 1.]]);
    query.domain = BezierLineDomain::Support;
    query.curve.line = [[0.5, 0.], [1.5, 0.]];
    let original = certify_bezier_line_incidences(query, || false).unwrap();
    let result = directed(query);
    assert_eq!(result.positions(), &[BeforeOrigin, BeyondTerminal]);
    assert_eq!(
        result.incidence().support_isolation(),
        original.support_isolation()
    );
    assert_eq!(result.incidence().contacts(), original.contacts());
    assert_eq!(result.incidence().bisections(), original.bisections());
}
