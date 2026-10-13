// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkmath::predicates::roots::isolate_bernstein_roots;
mod limits;
fn bernstein(controls: &[f64], interval: [f64; 2]) -> RootIsolation {
    isolate_bernstein_roots(controls, interval, options(0.0001), || false).unwrap()
}
#[test]
fn bernstein_roots_match_complete_independent_jacobian_identities() {
    let corpus = include_str!("bernstein/oracle.tsv");
    assert_eq!(corpus.lines().count(), 20);
    for line in corpus.lines() {
        verify_bernstein_fixture(line);
    }
}
fn verify_bernstein_fixture(line: &str) {
    let fields: Vec<&str> = line.split('\t').collect();
    let controls: Vec<f64> = fields[1]
        .split(',')
        .map(|bits| f64::from_bits(u64::from_str_radix(bits, 16).unwrap()))
        .collect();
    let expected: Vec<&str> = fields[2]
        .split('|')
        .filter(|root| !root.is_empty())
        .collect();
    let result = bernstein(&controls, [-16., 16.]);
    assert_eq!(result.roots().len(), expected.len(), "{}", fields[0]);
    verify_order_and_width(&result, 0.0001);
    for (root, identity) in result.roots().iter().zip(expected) {
        super::oracle::certify_interval(root, identity);
    }
    let negated: Vec<f64> = controls.iter().map(|value| -value).collect();
    assert_eq!(bernstein(&negated, [-16., 16.]), result);
}
#[test]
fn bernstein_rounding_cannot_create_tangencies_or_merge_crossings() {
    let above = [1., -1., 1_f64.next_up()];
    let below = [1., -1., 1_f64.next_down()];
    assert!(bernstein(&above, [0., 1.]).roots().is_empty());
    let distinct = bernstein(&below, [0., 1.]);
    assert_eq!(distinct.roots().len(), 2);
    assert!(distinct.roots().iter().all(|root| root.multiplicity() == 1));
    for controls in [above, below] {
        let rounded = [
            controls[0],
            2. * (controls[1] - controls[0]),
            controls[0] - 2. * controls[1] + controls[2],
        ];
        assert_eq!(rounded, [1., -4., 4.]);
        let false_tangent = isolate(&rounded, [0., 1.]);
        assert_eq!(false_tangent.roots().len(), 1);
        assert_eq!(false_tangent.roots()[0].multiplicity(), 2);
    }
}
#[test]
fn bernstein_extreme_scales_preserve_full_results_without_power_overflow() {
    let reference = bernstein(&[1., -1., 1.], [0., 1.]);
    for exponent in 0..2047_u64 {
        let scale = f64::from_bits((exponent << 52) | 1);
        for signed in [scale, -scale] {
            assert_eq!(bernstein(&[signed, -signed, signed], [0., 1.]), reference);
        }
    }
    assert_eq!(
        bernstein(&[f64::MAX, -f64::MAX, f64::MAX], [0., 1.]),
        reference
    );
}
#[test]
fn bernstein_can_separate_a_root_below_the_binary64_parameter_range() {
    let tiny = f64::from_bits(1);
    let result = bernstein(&[0., -tiny, f64::MAX], [0., tiny]);
    assert_eq!(result.roots().len(), 2);
    let root = &result.roots()[1];
    assert_eq!(root.multiplicity(), 1);
    assert_eq!(root.bounds().0.compare_f64(0.).unwrap(), Ordering::Greater);
    assert_eq!(root.bounds().1.compare_f64(tiny).unwrap(), Ordering::Less);
    let maximal_units =
        (num_bigint::BigInt::from(1_u64 << 53) - 1) * (num_bigint::BigInt::from(1) << 2045_usize);
    super::oracle::certify_interval(root, &format!("{},-2:1", maximal_units + 2));
}
#[test]
fn bernstein_degree_elevation_preserves_legacy_power_results_bit_for_bit() {
    let controls = [0., 2., -2., 0.];
    let elevated = [0., 6., 0., -6., 0.];
    let power = [0., 6., -18., 12.];
    for interval in [[-16., 16.], [0., 1.], [0.25, 0.75], [0.5, 0.5], [1., 2.]] {
        let expected = isolate(&power, interval);
        assert_eq!(bernstein(&controls, interval), expected);
        assert_eq!(bernstein(&elevated, interval), expected);
    }
}
#[test]
fn bernstein_reversal_maps_endpoint_multiplicities_and_extrapolated_roots() {
    let controls = [0., 0., 2., 6.]; // 6t², nominal degree 3.
    let reversed: Vec<f64> = controls.iter().rev().copied().collect();
    assert_eq!(
        bernstein(&controls, [-2., 3.]),
        isolate(&[0., 0., 6.], [-2., 3.])
    );
    assert_eq!(
        bernstein(&reversed, [-2., 3.]),
        isolate(&[6., -12., 6.], [-2., 3.])
    );
    let outside = [1., 2.]; // 1+t: the query interval never rescales the basis.
    assert!(bernstein(&outside, [0., 1.]).roots().is_empty());
    assert_eq!(
        bernstein(&outside, [-1., -1.]),
        isolate(&[1., 1.], [-1., -1.])
    );
}
#[test]
fn bernstein_high_degree_multiplicity_is_not_limited_to_the_oracle_degree() {
    for degree in [32_u16, 64] {
        let controls: Vec<f64> = (0..=degree)
            .map(|index| if index % 2 == 0 { 1. } else { -1. })
            .collect();
        let mut policy = options(0.0001);
        policy.max_degree = degree;
        let result = isolate_bernstein_roots(&controls, [0., 1.], policy, || false).unwrap();
        assert_eq!(result.roots().len(), 1);
        let root = &result.roots()[0];
        assert_eq!(root.multiplicity(), usize::from(degree));
        assert_eq!(root.bounds().0, root.bounds().1);
        assert_eq!(root.bounds().0.compare_f64(0.5).unwrap(), Ordering::Equal);
    }
}
