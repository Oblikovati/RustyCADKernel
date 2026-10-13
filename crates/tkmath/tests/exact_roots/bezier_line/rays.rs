// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkmath::predicates::roots::signs::RootSignEvaluation;
mod failures;
mod oracle;
fn directed(request: BezierLineQuery<'_>) -> RootSignEvaluation {
    let result = classify_bezier_ray_contacts(request, || false).unwrap();
    assert_eq!(result.isolation(), &contacts(request));
    result
}
#[test]
fn ray_contacts_retain_behind_origin_and_ahead_parameters() {
    for (x, side) in [
        (-1., Ordering::Less),
        (0., Ordering::Equal),
        (1., Ordering::Greater),
    ] {
        let controls = [[x, -1.], [x, 1.]];
        let result = directed(query(&controls));
        assert_eq!(result.signs(), &[side]);
        assert_eq!(result.isolation().roots()[0].multiplicity(), 1);
    }
}
#[test]
fn ray_contacts_retain_endpoint_and_tangent_multiplicity() {
    let result = directed(query(&[[-1., 0.], [0., 1.], [1., 0.]]));
    assert_eq!(result.signs(), &[Ordering::Less, Ordering::Greater]);
    let result = directed(query(&[[2., 1.], [2., -1.], [2., 1.]]));
    assert_eq!(result.signs(), &[Ordering::Greater]);
    assert_eq!(result.isolation().roots()[0].multiplicity(), 2);
}
#[test]
fn ray_common_irrational_roots_remain_distinct_origin_contacts() {
    let controls = [[-2., -2.], [-2., -2.], [-1., -1.]];
    let mut request = query(&controls);
    request.interval = [-2., 2.];
    let result = directed(request);
    assert_eq!(result.signs(), &[Ordering::Equal, Ordering::Equal]);
    for root in result.isolation().roots() {
        super::super::oracle::certify_interval(root, "1,0,-2:1");
    }
}
#[test]
fn ray_side_at_irrational_contact_does_not_use_bracket_midpoint() {
    let above = f64::from_bits(0x3ff6a09e667f3bcd);
    let below = f64::from_bits(0x3ff6a09e667f3bcc);
    for (offset, side) in [(above, Ordering::Less), (below, Ordering::Greater)] {
        let controls = [[-offset, -2.], [0.5 - offset, -2.], [1. - offset, -1.]];
        let mut request = query(&controls);
        request.interval = [1., 2.];
        request.options = options(1.);
        let result = directed(request);
        assert_eq!(result.signs(), &[side]);
        assert!(1.5 - offset > 0.);
        assert_eq!(result.isolation().bisections(), 0);
    }
}
#[test]
fn ray_direction_reversal_and_weight_negation_preserve_contact_identity() {
    let controls = [[-2., 1.], [0., -2.], [2., 1.]];
    let mut request = query(&controls);
    let original = directed(request);
    request.line[1] = [-1., 0.];
    let reversed = directed(request);
    assert_eq!(original.isolation(), reversed.isolation());
    assert_eq!(reversed.signs(), &[Ordering::Greater, Ordering::Less]);
    request.weights = Some(&[-1., -1., -1.]);
    assert_eq!(directed(request), reversed);
}
#[test]
fn ray_internal_product_degree_does_not_consume_input_degree_allowance() {
    let controls = [[1., -1.], [1., 0.], [1., 1.]];
    let mut request = query(&controls);
    request.weights = Some(&[1., 2., 4.]);
    request.options.max_degree = 2;
    assert_eq!(directed(request).signs(), &[Ordering::Greater]);
}
#[test]
fn ray_scaling_covers_all_finite_binary64_exponents() {
    let expected = scaled_ray(1.);
    for exponent in 0..2047_u64 {
        let scale = f64::from_bits((exponent << 52) | 1);
        assert_eq!(scaled_ray(scale), expected);
        assert_eq!(scaled_ray(-scale), expected);
    }
    assert_eq!(scaled_ray(f64::MAX), expected);
}
fn scaled_ray(scale: f64) -> RootSignEvaluation {
    let controls = [[0., -scale], [scale, scale]];
    let weights = [scale, scale];
    let mut request = query(&controls);
    request.line[1] = [scale, 0.];
    request.weights = Some(&weights);
    directed(request)
}
#[test]
fn ray_result_owns_contacts_after_source_mutation() {
    let mut controls = vec![[-0., -1.], [1., 1.]];
    let before: Vec<[u64; 2]> = controls.iter().map(|p| p.map(f64::to_bits)).collect();
    let result = directed(query(&controls));
    assert_eq!(
        controls
            .iter()
            .map(|p| p.map(f64::to_bits))
            .collect::<Vec<_>>(),
        before
    );
    controls.clear();
    assert_eq!(result.signs(), &[Ordering::Greater]);
    assert_eq!(result.isolation().roots().len(), 1);
}
