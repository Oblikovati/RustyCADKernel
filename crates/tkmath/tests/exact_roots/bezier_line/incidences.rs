// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(super) fn request(controls: &[[f64; 2]]) -> BezierIncidenceQuery<'_> {
    let curve = BezierLineQuery {
        controls,
        weights: None,
        line: [[0., 0.], [4., 0.]],
        interval: [0., 1.],
        options: RootIsolationOptions::new(0.001),
    };
    BezierIncidenceQuery {
        curve_id: 91,
        line_id: 27,
        curve,
        domain: BezierLineDomain::Segment,
        domain_sign: BezierDomainSignOptions::source_basis(),
        max_world_error: 1e-10,
    }
}
#[test]
fn cubic_and_even_quartic_roots_have_exact_world_points_and_multiplicity() {
    for (ordinates, expected, multiplicity) in [
        (vec![-9., 13., -13., 9.], vec![0.75, 1.5, 2.25], 1),
        (vec![27., -45., 59., -45., 27.], vec![1., 3.], 2),
    ] {
        let controls: Vec<[f64; 2]> = ordinates
            .iter()
            .enumerate()
            .map(|(index, y)| [index as f64, *y])
            .collect();
        let report = certify_bezier_line_incidences(request(&controls), || false).unwrap();
        assert_eq!(report.contacts().len(), expected.len());
        assert!(report.denominator_isolation().roots().is_empty());
        for (contact, x) in report.contacts().iter().zip(expected) {
            assert_eq!(contact.point(), [x, 0.]);
            assert_eq!(contact.achieved_world_error(), 0.);
            assert_eq!(contact.parameter().multiplicity(), multiplicity);
        }
    }
}
#[test]
fn closed_segment_endpoint_membership_survives_line_reversal_and_weight_sign() {
    let controls = [[0., 27.], [1., -45.], [2., 59.], [3., -45.], [4., 27.]];
    let mut query = request(&controls);
    query.curve.line = [[1., 0.], [3., 0.]];
    let original = certify_bezier_line_incidences(query, || false).unwrap();
    query.curve.line.reverse();
    query.curve.weights = Some(&[-1.; 5]);
    let reversed = certify_bezier_line_incidences(query, || false).unwrap();
    assert_eq!(original.contacts(), reversed.contacts());
    assert_eq!(original.in_domain(), &[true, true]);
    assert_eq!(original.bisections(), reversed.bisections());
}
#[test]
fn owned_certificate_preserves_source_after_caller_mutation_and_completed_empty() {
    let mut controls = vec![[0., -1.], [1., 1.]];
    let report = certify_bezier_line_incidences(request(&controls), || false).unwrap();
    controls.clear();
    assert_eq!(report.source().controls(), &[[0., -1.], [1., 1.]]);
    assert_eq!(report.source().curve_id(), 91);
    assert_eq!(report.source().line_id(), 27);
    let empty = certify_bezier_line_incidences(request(&[[0., 1.], [1., 1.]]), || false).unwrap();
    assert!(empty.contacts().is_empty());
    assert!(empty.support_isolation().roots().is_empty());
}
