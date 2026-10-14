// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::super::tests::{progress, request};
use super::super::super::tests::fixture_for;
use super::*;
use rustycad_tkmath::gp::Vector3;

#[test]
fn generated_scaled_reversed_boundaries_keep_closed_form_world_measures() {
    for (scale, reversed) in [(0.5, false), (1., true), (2., false), (2., true)] {
        let source = crate::closed_analytic_cubic::closed_analytic_cubic(scale, reversed);
        let (bridge, shapes) = fixture_for(vec![source]);
        let receipt = with_request(&bridge, &shapes, request([scale, scale]));
        let measured = measured(&bridge, &shapes, &receipt);
        assert_measure_bits(&measured, scale, 2.);
    }
}
pub(super) fn with_request(
    bridge: &PairedContactTransaction,
    shapes: &[rustycad_tkbrep::topods::Shape],
    request: PrismRequest,
) -> PrismReceipt {
    retain_source_bound_prism(
        bridge,
        &binding_sources(shapes),
        request,
        &mut progress(None, None),
    )
    .ok()
    .unwrap()
}
fn assert_measure_bits(measured: &PrismMeasureInspection, scale: f64, height: f64) {
    // Exact rational 12/5 lies strictly between these independently pinned binary64s.
    let lower = f64::from_bits(0x4003333333333333) * scale * scale;
    let upper = f64::from_bits(lower.to_bits() + 1);
    assert_eq!(measured.measures.first_cap.enclosure(), [lower, upper]);
    assert_eq!(measured.measures.last_cap.enclosure(), [lower, upper]);
    let expected = if height > 0. {
        [lower * height, upper * height]
    } else {
        [upper * height, lower * height]
    };
    assert_eq!(measured.measures.signed_volume.enclosure(), expected);
    assert_eq!(measured.region.selected_winding, 1);
}
#[test]
fn translated_last_cap_and_reversed_sweep_keep_actual_placement_and_sign() {
    for height in [4., -2.] {
        let (bridge, shapes) = fixture();
        let mut requested = request([1., 1.]);
        requested.extrusion = Vector3::new(0., 0., height);
        let receipt = with_request(&bridge, &shapes, requested);
        let measured = measured(&bridge, &shapes, &receipt);
        assert_measure_bits(&measured, 1., height);
        let last = measured
            .charts
            .iter()
            .find(|chart| chart.role == PrismImageRole::Last)
            .unwrap();
        let CapturedPrismSurface::Plane(support) = &last.captured.actual else {
            panic!("expected actual plane")
        };
        assert_eq!(support.placement.origin, [0., 0., height]);
        assert_eq!(last.captured.extrusion, [0., 0., height]);
    }
}
#[test]
fn exact_rational_jordan_region_stays_incomplete_for_polynomial_measurement() {
    let poles = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    let source =
        crate::sketch_region_certification::census_bezier_fixture(&poles, Some(&[1., 2., 2., 1.]));
    let (bridge, shapes) = fixture_for(vec![source]);
    let receipt = run(&bridge, &shapes);
    assert!(receipt.region.as_ref().unwrap().decision.is_ok());
    let failure = measurement_attempt(&bridge, &shapes, &receipt, options(), || false)
        .err()
        .unwrap();
    assert!(matches!(
        failure,
        PrismMeasureFailure::Sweep(PolynomialIntegralError {
            cause: PolynomialIntegralFailure::RationalWeights { count: 4 },
            curve_index: Some(0),
            ..
        })
    ));
}
#[test]
fn separate_area_and_volume_budgets_preserve_failure_quantity() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let mut limits = options();
    limits.cap_area.absolute_error_limit = 0.;
    let failure = measurement_attempt(&bridge, &shapes, &receipt, limits, || false)
        .err()
        .unwrap();
    assert!(matches!(
        failure,
        PrismMeasureFailure::Cap {
            role: PrismImageRole::First,
            cause: PolynomialPlaneError {
                cause: PolynomialPlaneFailure::ErrorLimit { requested: 0., .. },
                ..
            }
        }
    ));
}
#[test]
fn separate_volume_budget_preserves_failure_quantity() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let mut limits = options();
    limits.signed_volume.absolute_error_limit = 0.;
    let failure = measurement_attempt(&bridge, &shapes, &receipt, limits, || false)
        .err()
        .unwrap();
    assert!(matches!(
        failure,
        PrismMeasureFailure::Sweep(PolynomialIntegralError {
            cause: PolynomialIntegralFailure::ErrorLimit { requested: 0., .. },
            ..
        })
    ));
}
#[test]
fn stale_nonzero_jacobian_observation_cannot_certify_current_in_plane_sweep() {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    receipt.limits.requested.extrusion = Vector3::new(1., 0., 0.);
    receipt.affine_chart.as_mut().unwrap().extrusion = [1., 0., 0.];
    let failure = measurement_attempt(&bridge, &shapes, &receipt, options(), || false)
        .err()
        .unwrap();
    assert!(matches!(
        failure,
        PrismMeasureFailure::Region(RegionCertificateCause::Hypothesis {
            name: "fresh nonzero measurement sweep determinant",
            ..
        })
    ));
}
