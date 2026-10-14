// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::super::tests::request;
use super::super::super::measurements::tests::domain_tests::with_request;
use super::*;

pub(in crate::sketch_region_prism) fn pole_source(
    reverse_poles: bool,
    reverse_traversal: bool,
) -> Shape {
    let mut poles = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    if reverse_poles {
        poles.reverse();
    }
    let source = crate::sketch_region_certification::census_bezier_fixture(&poles, None);
    if reverse_traversal {
        source.reversed()
    } else {
        source
    }
}
#[test]
fn reversing_stored_poles_is_distinct_from_reversing_source_traversal() {
    for (poles, traversal, expected_q) in [
        (false, false, 1),
        (false, true, 1),
        (true, false, -1),
        (true, true, -1),
    ] {
        let (bridge, shapes) = fixture_for(vec![pole_source(poles, traversal)]);
        let receipt = run(&bridge, &shapes);
        let evidence = oriented(&bridge, &shapes, &receipt);
        assert_eq!(evidence.winding().factors().3, expected_q);
        assert_green_sign(&evidence, expected_q);
        assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    }
}
fn assert_green_sign(evidence: &PrismOrientationEvidence, expected: i8) {
    let boundary = &evidence.embedding().binding().region.copied.boundaries[0];
    let curve = OrientedMetricCurve {
        curve: boundary.pcurve.borrow(),
        reversed: false,
    };
    let integral = certify_polynomial_boundary_integral(
        &[curve],
        PolynomialIntegralOptions::default(),
        || false,
    )
    .unwrap();
    // Independent antiderivative of authored cubic: +12/5; pole reversal gives -12/5.
    let [lo, hi] = integral.enclosure();
    assert!(lo <= 2.4 * f64::from(expected) && hi >= 2.4 * f64::from(expected));
    assert_eq!(lo > 0., expected == 1);
    assert_eq!(hi < 0., expected == -1);
}
#[test]
fn negative_extrusion_changes_cap_and_side_outwardness() {
    use Orientation::{Forward as F, Reversed as R};
    for (height, expected) in [(2., [R, F, R]), (-2., [F, R, F])] {
        let (bridge, shapes) = fixture();
        let mut requested = request([1., 1.]);
        requested.extrusion = rustycad_tkmath::gp::Vector3::new(0., 0., height);
        let receipt = with_request(&bridge, &shapes, requested);
        let evidence = oriented(&bridge, &shapes, &receipt);
        assert_eq!(
            std::array::from_fn::<_, 3, _>(|i| evidence.faces()[i].orientations().0),
            expected
        );
    }
}
#[test]
fn historical_status_and_binding_order_do_not_choose_orientations() {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    let before = oriented(&bridge, &shapes, &receipt);
    receipt.region = None;
    let diagnostic = receipt.diagnostic.as_mut().unwrap();
    diagnostic.surface_charts = None;
    diagnostic.bindings.reverse();
    diagnostic.relations.reverse();
    let after = oriented(&bridge, &shapes, &receipt);
    assert_eq!(before.faces(), after.faces());
}
