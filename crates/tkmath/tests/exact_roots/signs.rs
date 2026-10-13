// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkmath::predicates::roots::signs::*;
mod failures;
fn classify(source: &[f64], query: &[f64], interval: [f64; 2], width: f64) -> RootSignEvaluation {
    signs_at_polynomial_roots(source, query, interval, options(width), || false).unwrap()
}
fn coefficients(text: &str) -> Vec<f64> {
    text.split(',')
        .map(|bits| f64::from_bits(u64::from_str_radix(bits, 16).unwrap()))
        .collect()
}
#[test]
fn algebraic_signs_match_independent_complete_root_order_comparisons() {
    let corpus = include_str!("signs/oracle.tsv");
    assert_eq!(corpus.lines().count(), 20);
    for row in corpus.lines() {
        check_fixture(row);
    }
}
fn check_fixture(row: &str) {
    let fields: Vec<&str> = row.split('\t').collect();
    let source = coefficients(fields[1]);
    let query = coefficients(fields[2]);
    let expected: Vec<Ordering> = fields[3]
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<i8>().unwrap().cmp(&0))
        .collect();
    let result = classify(&source, &query, [-16., 16.], 0.0001);
    assert_eq!(result.signs(), expected, "{}", fields[0]);
    assert_eq!(*result.isolation(), isolate(&source, [-16., 16.]));
    verify_sign_scaling(&source, &query, &result);
}
fn verify_sign_scaling(source: &[f64], query: &[f64], expected: &RootSignEvaluation) {
    for factor in [-1., 2_f64.powi(900), 2_f64.powi(-900)] {
        let scaled: Vec<f64> = source.iter().map(|v| v * factor).collect();
        assert_eq!(classify(&scaled, query, [-16., 16.], 0.0001), *expected);
        let scaled: Vec<f64> = query.iter().map(|v| v * factor).collect();
        let result = classify(source, &scaled, [-16., 16.], 0.0001);
        let signs: Vec<Ordering> = expected
            .signs()
            .iter()
            .map(|s| if factor < 0. { s.reverse() } else { *s })
            .collect();
        assert_eq!(result.signs(), signs);
        assert_eq!(result.isolation(), expected.isolation());
    }
}
#[test]
fn algebraic_signs_do_not_use_a_bracket_midpoint_as_the_root() {
    let source = [-2., 0., 1.];
    let threshold = 2_f64.sqrt();
    let result = classify(&source, &[-threshold, 1.], [1., 2.], 1.);
    assert_eq!(result.signs(), &[Ordering::Less]);
    let root = &result.isolation().roots()[0];
    assert_eq!(root.bounds().0.compare_f64(1.).unwrap(), Ordering::Equal);
    assert_eq!(root.bounds().1.compare_f64(2.).unwrap(), Ordering::Equal);
    assert!(
        1.5 - threshold > 0.,
        "midpoint substitution gives the wrong sign"
    );
    assert_eq!(
        classify(&source, &[-threshold.next_down(), 1.], [1., 2.], 1.).signs(),
        &[Ordering::Greater]
    );
}
#[test]
fn algebraic_signs_allow_multiple_query_roots_inside_one_source_bracket() {
    let query = [1.875, -2.75, 1.];
    let result = classify(&[-2., 0., 1.], &query, [1., 2.], 1.);
    assert_eq!(result.signs(), &[Ordering::Less]);
    let endpoint_values = [
        query[0] + query[1] + query[2],
        query[0] + 2. * query[1] + 4. * query[2],
    ];
    assert!(endpoint_values.into_iter().all(|value| value > 0.));
    assert_eq!(result.isolation().bisections(), 0);
}
#[test]
fn algebraic_signs_preserve_exact_common_roots_and_source_multiplicity() {
    let source = [-4., 4., 4., -4., -1., 1.];
    let result = classify(&source, &[-2., -2., 1., 1.], [-2., 2.], 0.001);
    assert_eq!(
        result.signs(),
        &[Ordering::Equal, Ordering::Less, Ordering::Equal]
    );
    let multiplicities: Vec<usize> = result
        .isolation()
        .roots()
        .iter()
        .map(RootBracket::multiplicity)
        .collect();
    assert_eq!(multiplicities, [2, 1, 2]);
    assert_eq!(
        classify(&source, &[0., 1.], [-2., 2.], 0.001).signs(),
        &[Ordering::Less, Ordering::Greater, Ordering::Greater]
    );
}
#[test]
fn algebraic_signs_handle_point_roots_constants_zero_queries_and_empty_results() {
    for query in [vec![], vec![0.; 65], vec![-1., 0., 1.]] {
        assert_eq!(
            classify(&[-1., 0., 1.], &query, [-1., 1.], 0.1).signs(),
            &[Ordering::Equal; 2]
        );
    }
    assert_eq!(
        classify(&[-1., 0., 1.], &[0., 1.], [-1., 1.], 0.1).signs(),
        &[Ordering::Less, Ordering::Greater]
    );
    assert_eq!(
        classify(&[-2., 0., 1.], &[-3.], [-2., 2.], 0.1).signs(),
        &[Ordering::Less; 2]
    );
    assert_eq!(
        classify(&[-2., 0., 1.], &[3.], [-2., 2.], 0.1).signs(),
        &[Ordering::Greater; 2]
    );
}
#[test]
fn algebraic_signs_return_completed_empty_for_sources_without_roots() {
    for source in [vec![1.], vec![1., 0., 1.]] {
        let result = classify(&source, &[0., 1.], [-2., 2.], 0.1);
        assert!(result.signs().is_empty());
        assert!(result.isolation().roots().is_empty());
    }
}
#[test]
fn algebraic_signs_remain_exact_with_subnormal_queries_and_extreme_source_values() {
    let tiny = f64::from_bits(1);
    let source = [0., -tiny, f64::MAX];
    let result = classify(&source, &[0., tiny], [0., tiny], tiny);
    assert_eq!(result.signs(), &[Ordering::Equal, Ordering::Greater]);
    assert!(
        result.isolation().roots()[1]
            .bounds()
            .1
            .compare_f64(tiny)
            .unwrap()
            == Ordering::Less
    );
}
#[test]
fn algebraic_signs_match_ordinary_occt_curve_parameters_away_from_query_zeros() {
    // Exact projected numerators of the retained ADR 0006 ordinary curve fixtures.
    let cases: [(&str, &[f64]); 4] = [
        ("linear", &[-1., 2.]),
        ("tangent", &[1., -4., 4.]),
        ("endpoints", &[0., 2., -2.]),
        ("weighted_crossing", &[-1., 3.]),
    ];
    for (name, source) in cases {
        let native = native_threshold_signs(name);
        assert_eq!(
            classify(source, &[-0.25, 1.], [0., 1.], 0.0001).signs(),
            native
        );
    }
}

fn native_threshold_signs(name: &str) -> Vec<Ordering> {
    let receipt = include_str!("bezier_line/occt-8.0.0.tsv")
        .lines()
        .find(|row| row.split('\t').next() == Some(name))
        .unwrap();
    receipt
        .split('\t')
        .nth(2)
        .unwrap()
        .split(',')
        .map(|t| t.parse::<f64>().unwrap().partial_cmp(&0.25).unwrap())
        .collect()
}
