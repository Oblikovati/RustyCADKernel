// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkmath::predicates::roots::bezier_line::*;
mod failures;
mod fixture;
mod ownership;
mod rays;
mod winding;
mod wire;
fn query(controls: &[[f64; 2]]) -> BezierLineQuery<'_> {
    BezierLineQuery {
        controls,
        weights: None,
        line: [[0., 0.], [1., 0.]],
        interval: [0., 1.],
        options: options(0.0001),
    }
}
fn contacts(request: BezierLineQuery<'_>) -> RootIsolation {
    match intersect_bezier_line(request, || false).unwrap() {
        BezierLineContact::Isolated(roots) => roots,
        other => panic!("contact {other:?}; expected isolated contacts"),
    }
}
fn decode_bits(text: &str) -> Vec<f64> {
    text.split(',')
        .map(|bits| f64::from_bits(u64::from_str_radix(bits, 16).unwrap()))
        .collect()
}
#[test]
fn bezier_contacts_match_independent_exact_rational_projection_and_root_identities() {
    let corpus = include_str!("bezier_line/oracle.tsv");
    assert_eq!(corpus.lines().count(), 12);
    for line in corpus.lines() {
        check_fixture(line);
    }
}
fn check_fixture(line: &str) -> RootIsolation {
    let fields: Vec<&str> = line.split('\t').collect();
    let controls: Vec<[f64; 2]> = fields[1]
        .split(';')
        .map(|p| decode_bits(p).try_into().unwrap())
        .collect();
    let weights = (fields[2] != "-").then(|| decode_bits(fields[2]));
    let line_points = decode_bits(fields[3]);
    let mut request = query(&controls);
    request.weights = weights.as_deref();
    request.line = [
        [line_points[0], line_points[1]],
        [line_points[2], line_points[3]],
    ];
    let result = contacts(request);
    certify_fixture(&result, fields[4]);
    check_reversal_and_weight_sign(request, &result);
    result
}
fn check_reversal_and_weight_sign(mut request: BezierLineQuery<'_>, result: &RootIsolation) {
    request.line.reverse();
    assert_eq!(contacts(request), *result);
    let negative: Vec<f64> = (0..request.controls.len())
        .map(|index| -request.weights.map_or(1., |weights| weights[index]))
        .collect();
    let negated = BezierLineQuery {
        weights: Some(&negative),
        ..request
    };
    assert_eq!(contacts(negated), *result);
}

fn certify_fixture(result: &RootIsolation, identities: &str) {
    let roots: Vec<&str> = identities
        .split('|')
        .filter(|root| !root.is_empty())
        .collect();
    assert_eq!(result.roots().len(), roots.len());
    verify_order_and_width(result, 0.0001);
    for (root, identity) in result.roots().iter().zip(roots) {
        super::oracle::certify_interval(root, identity);
    }
}
#[test]
fn bezier_exact_projection_does_not_turn_a_near_line_point_into_coincidence() {
    let epsilon = f64::EPSILON;
    let endpoint = [1. + epsilon, 1.];
    let point = [1., 1. - epsilon];
    assert_eq!(endpoint[0] * point[1] - endpoint[1] * point[0], 0.);
    let controls = [point];
    let mut request = query(&controls);
    request.line = [[0., 0.], endpoint];
    assert!(contacts(request).roots().is_empty());
}
#[test]
fn bezier_coincidence_is_distinct_from_completed_empty_and_endpoint_contacts() {
    let coincident = query(&[[0., 0.], [1., 0.], [2., 0.]]);
    assert_eq!(
        intersect_bezier_line(coincident, || false),
        Ok(BezierLineContact::Coincident { bisections: 0 })
    );
    assert!(contacts(query(&[[0., 1.], [1., 1.]])).roots().is_empty());
    let result = contacts(query(&[[0., 0.], [1., 1.], [2., 0.]]));
    assert_eq!(result.roots().len(), 2);
    for (root, parameter) in result.roots().iter().zip([0., 1.]) {
        assert_eq!(root.bounds().0, root.bounds().1);
        assert_eq!(
            root.bounds().0.compare_f64(parameter).unwrap(),
            Ordering::Equal
        );
    }
}
#[test]
fn bezier_affine_frames_preserve_the_entire_parameter_result() {
    let controls = [[0., 1.], [1., -2.], [2., 1.]];
    let reference = contacts(query(&controls));
    let changed: Vec<[f64; 2]> = controls
        .iter()
        .map(|p| [8. - 2. * p[0] + p[1], -4. + 4. * p[1]])
        .collect();
    let mut request = query(&changed);
    request.line = [[8., -4.], [6., -4.]];
    assert_eq!(contacts(request), reference);
    request.weights = Some(&[2., 2., 2.]);
    assert_eq!(contacts(request), reference);
}
#[test]
fn bezier_line_is_infinite_and_closed_parameter_queries_do_not_rescale_the_basis() {
    let controls = [[100., -1.], [200., 1.]];
    let mut request = query(&controls);
    let result = contacts(request);
    assert_eq!(result.roots().len(), 1);
    request.interval = [0.5, 0.5];
    assert_eq!(contacts(request).roots(), result.roots());
    request.interval = [0., 0.25];
    assert!(contacts(request).roots().is_empty());
    request.interval = [1., 2.];
    assert!(contacts(request).roots().is_empty());
}

#[test]
fn bezier_ordinary_contacts_match_retained_occt_8_parameter_receipts() {
    let receipts = include_str!("bezier_line/occt-8.0.0.tsv");
    assert_eq!(receipts.lines().count(), 6);
    for receipt in receipts.lines() {
        let fields: Vec<&str> = receipt.split('\t').collect();
        let row = include_str!("bezier_line/oracle.tsv")
            .lines()
            .find(|row| row.split('\t').next() == Some(fields[0]))
            .unwrap();
        let result = check_fixture(row);
        assert_eq!(
            fields[1], "0",
            "native intersection segments are not isolated parameters"
        );
        certify_native_parameters(&result, fields[2]);
    }
}
fn certify_native_parameters(result: &RootIsolation, text: &str) {
    let parameters: Vec<f64> = text
        .split(',')
        .filter(|value| !value.is_empty())
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(result.roots().len(), parameters.len());
    for (root, parameter) in result.roots().iter().zip(parameters) {
        assert!(root.bounds().0.compare_f64(parameter).unwrap() != Ordering::Greater);
        assert!(root.bounds().1.compare_f64(parameter).unwrap() != Ordering::Less);
    }
}
#[test]
fn bezier_weighted_projection_preserves_contacts_across_every_finite_exponent() {
    let expected = scaled_contact(1.);
    for exponent in 0..2047_u64 {
        let scale = f64::from_bits((exponent << 52) | 1);
        assert_eq!(scaled_contact(scale), expected);
        assert_eq!(scaled_contact(-scale), expected);
    }
    assert_eq!(scaled_contact(f64::MAX), expected);
}
fn scaled_contact(scale: f64) -> RootIsolation {
    let controls = [[0., -scale], [scale, scale]];
    let weights = [scale, scale];
    let mut request = query(&controls);
    request.line = [[0., 0.], [scale, 0.]];
    request.weights = Some(&weights);
    contacts(request)
}
