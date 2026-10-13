// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn winding_denominator_poles_decline_before_coincident_boundary() {
    let mut request = query(&[[0., 0.], [0., 0.]]);
    request.weights = Some(&[1., -1.]);
    assert!(matches!(
        bezier_winding_contribution(request, || false),
        Err(BezierRayError::Line(BezierLineError::DenominatorRoots(_)))
    ));
    request.weights = Some(&[0., 0.]);
    assert_eq!(
        bezier_winding_contribution(request, || false),
        Err(BezierRayError::Line(BezierLineError::ZeroDenominator))
    );
}
#[test]
fn winding_cancellation_discards_every_partial_contribution() {
    for controls in [
        vec![[1., -1.], [1., 1.]],
        vec![[1., 1.], [1., -2.], [1., 1.]],
        vec![[0., 0.], [0., 0.]],
        vec![[-1., 0.], [1., 0.]],
        vec![[1., 0.], [2., 0.]],
    ] {
        let mut polls = 0;
        let request = query(&controls);
        let _ = bezier_winding_contribution(request, || {
            polls += 1;
            false
        });
        for stop in 1..=polls {
            cancelled_at(request, stop);
        }
    }
}
fn cancelled_at(request: BezierLineQuery<'_>, stop: usize) {
    let mut count = 0;
    let error = bezier_winding_contribution(request, || {
        count += 1;
        count == stop
    })
    .unwrap_err();
    assert!(error.to_string().contains("cancel"), "{stop}: {error}");
}
#[test]
fn winding_success_failure_and_cancellation_preserve_source_bits() {
    let controls = [[-0., -1.], [1., 1.]];
    let before = controls.map(|p| p.map(f64::to_bits));
    assert_eq!(winding(query(&controls)), BezierWinding::Clear(1));
    assert!(bezier_winding_contribution(query(&controls), || true).is_err());
    let mut request = query(&controls);
    request.options.version = 2;
    assert!(bezier_winding_contribution(request, || false).is_err());
    assert_eq!(controls.map(|p| p.map(f64::to_bits)), before);
    assert_eq!(winding(query(&controls)), BezierWinding::Clear(1));
}
#[test]
fn winding_coincident_origin_search_obeys_degree_and_work_limits() {
    let controls = [[1., 0.], [-2., 0.], [1., 0.]];
    let mut request = query(&controls);
    request.options.max_degree = 1;
    assert_eq!(
        bezier_winding_contribution(request, || false),
        Err(BezierRayError::Direction(RootIsolationError::Degree {
            actual: 2,
            limit: 1
        }))
    );
    request.options = options(0.0001);
    request.options.max_bisections = 0;
    assert_eq!(
        bezier_winding_contribution(request, || false),
        Err(BezierRayError::Direction(
            RootIsolationError::SubdivisionLimit { limit: 0 }
        ))
    );
}
