// SPDX-License-Identifier: GPL-2.0-only
//! Generated invalid charts never inherit completion from valid coefficient snapshots.
use super::super::{tests::*, *};
use super::{tests::*, *};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;

fn captured_side() -> CapturedSurfaceChart {
    let mut receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let charts = receipt
        .diagnostic
        .as_mut()
        .unwrap()
        .surface_charts
        .as_mut()
        .unwrap();
    charts
        .entries
        .iter_mut()
        .find(|entry| entry.role == PrismImageRole::Generated)
        .unwrap()
        .captured
        .take()
        .unwrap()
}
fn scalar_decision(chart: &CapturedSurfaceChart) -> Result<(), PrismSurfaceCause> {
    let mut proof = PrismSurfaceProof::default();
    prove_chart(chart, PrismImageRole::Generated, &mut || false, &mut proof)
}
#[test]
fn zero_negative_and_nonfinite_heights_are_incomplete() {
    for height in [0., -2., f64::INFINITY, f64::NAN] {
        let mut chart = captured_side();
        let last = chart
            .trims
            .iter_mut()
            .find(|trim| trim.role == PrismTrimRole::Last)
            .unwrap();
        last.pcurve = CapturedMetricCurve::Line {
            origin: [0., -height],
            direction: [1., 0.],
        };
        assert!(matches!(
            scalar_decision(&chart),
            Err(PrismSurfaceCause::InvalidHeight { .. })
        ));
    }
}
#[test]
fn missing_seam_roles_are_incomplete() {
    for count in [0, 1, 2, 3] {
        let mut chart = captured_side();
        chart.trims.truncate(count);
        assert!(
            matches!(scalar_decision(&chart), Err(PrismSurfaceCause::Count { actual, expected: 4, .. }) if actual == count)
        );
    }
}
#[test]
fn duplicate_seam_role_is_incomplete() {
    let mut chart = captured_side();
    chart
        .trims
        .iter_mut()
        .find(|trim| trim.role == PrismTrimRole::End)
        .unwrap()
        .role = PrismTrimRole::Start;
    assert!(matches!(
        scalar_decision(&chart),
        Err(PrismSurfaceCause::Count {
            actual: 2,
            expected: 1,
            ..
        })
    ));
}
