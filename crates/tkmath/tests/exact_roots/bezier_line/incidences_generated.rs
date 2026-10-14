// SPDX-License-Identifier: GPL-2.0-only
use super::incidences::request;
use super::*;

#[test]
fn higher_rational_degrees_use_same_complete_even_root_pipeline() {
    for degree in [8usize, 16] {
        verify_higher_rational_degree(degree);
    }
}
fn verify_higher_rational_degree(degree: usize) {
    let weights: Vec<f64> = (0..=degree)
        .map(|i| if i % 2 == 0 { 1. } else { 2. })
        .collect();
    let controls = alternating_homogeneous_controls(&weights, degree);
    let mut query = request(&controls);
    query.curve.weights = Some(&weights);
    let report = certify_bezier_line_incidences(query, || false).unwrap();
    assert_eq!(report.contacts().len(), 1);
    assert_eq!(report.contacts()[0].parameter().multiplicity(), degree);
    assert_eq!(report.contacts()[0].point(), [0.5, 0.]);
    assert_eq!(report.contacts()[0].achieved_world_error(), 0.);
    assert_eq!(report.degrees().normal(), degree);
    assert!(report.denominator_isolation().roots().is_empty());
}
fn alternating_homogeneous_controls(weights: &[f64], degree: usize) -> Vec<[f64; 2]> {
    // The homogeneous normal is (2t-1)^degree in the original Bernstein basis.
    weights
        .iter()
        .enumerate()
        .map(|(i, weight)| {
            let sign = if i % 2 == 0 { 1. } else { -1. };
            [i as f64 / degree as f64, sign / weight]
        })
        .collect()
}

#[test]
fn repeated_quartic_incidence_is_exact_across_scale_axis_orientation_and_reversal() {
    for scale in [2_f64.powi(-900), 1., 2_f64.powi(900)] {
        for reflected in [false, true] {
            verify_transformed_quartic(scale, reflected);
        }
    }
}
fn verify_transformed_quartic(scale: f64, reflected: bool) {
    let ordinates = [27., -45., 59., -45., 27.];
    let controls: Vec<[f64; 2]> = ordinates
        .iter()
        .enumerate()
        .map(|(index, y)| transform_point([index as f64, *y], scale, reflected))
        .collect();
    let mut query = request(&controls);
    query.curve.line = [
        transform_point([1., 0.], scale, reflected),
        transform_point([3., 0.], scale, reflected),
    ];
    let original = certify_bezier_line_incidences(query, || false).unwrap();
    verify_exact_quartic(&original, scale, reflected);
    query.curve.line.reverse();
    query.curve.weights = Some(&[-1.; 5]);
    let reversed = certify_bezier_line_incidences(query, || false).unwrap();
    assert_eq!(reversed.contacts(), original.contacts());
    assert_eq!(reversed.source().controls(), controls);
}
fn transform_point(point: [f64; 2], scale: f64, reflected: bool) -> [f64; 2] {
    if reflected {
        return [-point[1] * scale, point[0] * scale];
    }
    [point[0] * scale, point[1] * scale]
}
fn verify_exact_quartic(report: &BezierIncidenceReport, scale: f64, reflected: bool) {
    assert_eq!(report.contacts().len(), 2);
    for (contact, x) in report.contacts().iter().zip([1., 3.]) {
        assert_eq!(contact.point(), transform_point([x, 0.], scale, reflected));
        assert_eq!(contact.achieved_world_error(), 0.);
        assert_eq!(contact.parameter().multiplicity(), 2);
    }
}
#[test]
fn occurrence_permutation_keeps_each_owned_source_and_parameter_order() {
    let controls = [[0., -9.], [1., 13.], [2., -13.], [3., 9.]];
    let forward: Vec<BezierIncidenceReport> = [7, 19, 31]
        .into_iter()
        .map(|id| occurrence(&controls, id))
        .collect();
    let reversed: Vec<BezierIncidenceReport> = [31, 19, 7]
        .into_iter()
        .map(|id| occurrence(&controls, id))
        .collect();
    assert!(forward.iter().eq(reversed.iter().rev()));
    for report in &forward {
        assert!(
            report
                .contacts()
                .windows(2)
                .all(|pair| pair[0].parameter().bounds().1 < pair[1].parameter().bounds().0)
        );
    }
}
fn occurrence(controls: &[[f64; 2]], id: u64) -> BezierIncidenceReport {
    let mut query = request(controls);
    query.curve_id = id;
    certify_bezier_line_incidences(query, || false).unwrap()
}
