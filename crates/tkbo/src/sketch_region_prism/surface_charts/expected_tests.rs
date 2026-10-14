// SPDX-License-Identifier: GPL-2.0-only
use super::super::{tests::*, *};
use super::{tests::*, *};
use crate::sketch_region_candidates::tests::{candidates, rational_closed_curve};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use rustycad_tkmath::{
    gp::{Transform3, Vector3},
    toploc::Location,
};

#[test]
fn cubic_coefficients_have_independently_expected_surface_and_trim_formulas() {
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let side = entry_by_role(&receipt, PrismImageRole::Generated);
    assert_expected_side(side);
    assert_expected_caps(&receipt);
    assert_expected_side_trims(side);
}
pub(super) fn assert_expected_side(side: &PrismSurfaceEntry) {
    let actual = &side.captured.as_ref().unwrap().actual;
    let CapturedPrismSurface::Extrusion {
        basis, direction, ..
    } = actual
    else {
        panic!("expected extrusion")
    };
    assert_eq!(*direction, [0., 0., -1.]);
    let CapturedMetricCurve::Bezier { poles, weights } = basis else {
        panic!("expected cubic")
    };
    assert_eq!(
        poles,
        &[[0., 0., 0.], [4., 0., 0.], [0., 4., 0.], [0., 0., 0.]]
    );
    assert!(weights.is_none());
    assert_eq!(side.proof.side_height, Some(2.));
    assert_eq!(side.proof.trim_roles.len(), 4);
}
pub(super) fn assert_expected_caps(receipt: &PrismReceipt) {
    for (role, height) in [(PrismImageRole::First, 0.), (PrismImageRole::Last, 2.)] {
        let chart = entry_by_role(receipt, role).captured.as_ref().unwrap();
        let CapturedPrismSurface::Plane(plane) = &chart.actual else {
            panic!("expected plane")
        };
        assert_eq!(plane.frame.linear, [[1., 0.], [0., 1.], [0., 0.]]);
        assert_eq!(plane.frame.origin, [0., 0., 0.]);
        assert_eq!(plane.placement.origin, [0., 0., height]);
        assert_eq!(chart.trims.len(), 1);
        assert_eq!(chart.trims[0].interval, [0., 1.]);
    }
}
#[test]
fn rational_boundary_uses_the_same_actual_chart_and_history_pipeline() {
    let receipt = retain_selected_prism(
        candidates(&[rational_closed_curve()]),
        request([1., 1.]),
        &mut progress(None, None),
    );
    assert_cubic_collection(inspection(&receipt));
    assert_body_incomplete(&receipt);
    let side = entry_by_role(&receipt, PrismImageRole::Generated);
    let CapturedPrismSurface::Extrusion { basis, .. } = &side.captured.as_ref().unwrap().actual
    else {
        panic!("expected extrusion")
    };
    let CapturedMetricCurve::Bezier { weights, .. } = basis else {
        panic!("expected rational Bézier")
    };
    assert_eq!(weights.as_deref(), Some([1., 2., 2., 1.].as_slice()));
}
#[test]
fn reversed_and_located_inputs_keep_the_same_complete_chart_domain() {
    for reversed in [false, true] {
        let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., reversed);
        let mut transform = Transform3::default();
        transform.set_translation(Vector3::new(8., -4., 0.));
        let source = source.located(&Location::new(transform), true).unwrap();
        let receipt = retain_selected_prism(
            candidates(&[source]),
            request([9., -3.]),
            &mut progress(None, None),
        );
        assert_cubic_collection(inspection(&receipt));
        assert_body_incomplete(&receipt);
    }
}
#[test]
fn canonicalized_straight_side_is_explicitly_outside_the_admitted_chart_domain() {
    let source = super::super::geometry_tests::mixed_boundary();
    let receipt = retain_selected_prism(
        candidates(&source),
        request([2., 1.]),
        &mut progress(None, None),
    );
    assert!(
        inspection(&receipt)
            .entries
            .iter()
            .any(|entry| entry.decision.is_err())
    );
    assert_body_incomplete(&receipt);
}
#[test]
fn normalized_oblique_direction_cannot_claim_exact_requested_translation() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let mut options = request([1., 1.]);
    options.extrusion = Vector3::new(1., 1., 2.);
    let receipt = retain_selected_prism(candidates(&[source]), options, &mut progress(None, None));
    let side = entry_by_role(&receipt, PrismImageRole::Generated);
    assert!(matches!(
        side.decision,
        Err(PrismSurfaceCause::Identity {
            stage: PrismChartStage::SideDirection,
            ..
        })
    ));
    assert_body_incomplete(&receipt);
}
#[test]
fn coincident_source_members_retain_every_original_parent_on_each_chart() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let batch = candidates(&[source.clone(), source]);
    let receipt = retain_selected_prism(batch, request([1., 1.]), &mut progress(None, None));
    assert_cubic_collection(inspection(&receipt));
    for entry in &inspection(&receipt).entries {
        assert!(entry.parents.iter().any(|parent| parent.source.0 == 1));
        assert!(entry.parents.iter().any(|parent| parent.source.0 == 2));
        assert_trim_parents(entry);
    }
}
fn assert_trim_parents(entry: &PrismSurfaceEntry) {
    for trim in &entry.captured.as_ref().unwrap().trims {
        assert!(trim.parents.iter().any(|parent| parent.source.0 == 1));
        assert!(trim.parents.iter().any(|parent| parent.source.0 == 2));
    }
}

pub(super) fn assert_expected_side_trims(side: &PrismSurfaceEntry) {
    let trims = &side.captured.as_ref().unwrap().trims;
    for (role, origin, direction, end) in [
        (PrismTrimRole::First, [0., 0.], [1., 0.], 1.),
        (PrismTrimRole::Last, [0., -2.], [1., 0.], 1.),
        (PrismTrimRole::Start, [0., 0.], [0., -1.], 2.),
        (PrismTrimRole::End, [1., 0.], [0., -1.], 2.),
    ] {
        let trim = trims.iter().find(|trim| trim.role == role).unwrap();
        assert_eq!(trim.interval, [0., end]);
        assert!(
            matches!(trim.pcurve, CapturedMetricCurve::Line { origin: o, direction: d } if o == origin && d == direction)
        );
    }
}
