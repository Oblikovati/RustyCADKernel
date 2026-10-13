// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use std::error::Error;
fn failure(segments: &[BezierWireSegment<'_>]) -> BezierWireError {
    let error = classify_bezier_wire(wire_query(segments, [0., 0.]), || false).unwrap_err();
    assert!(error.to_string().contains("expected"));
    error
}
#[test]
fn wire_empty_duplicate_and_open_inputs_are_named_declines() {
    assert_eq!(failure(&[]), BezierWireError::Empty);
    let controls = [[0., 0.], [1., 0.]];
    assert_eq!(
        failure(&[segment(8, &controls)]),
        BezierWireError::OpenJoin { from: 8, to: 8 }
    );
    assert_eq!(
        failure(&[segment(8, &controls), segment(8, &controls)]),
        BezierWireError::DuplicateId { id: 8 }
    );
    assert_eq!(
        failure(&[segment(1, &controls), segment(2, &controls)]),
        BezierWireError::OpenJoin { from: 1, to: 2 }
    );
}
#[test]
fn wire_boundary_does_not_hide_a_later_pole_or_open_closing_join() {
    let first = [[0., 0.], [1., 0.]];
    let second = [[1., 0.], [2., 0.]];
    let mut segments = [segment(10, &first), segment(20, &second)];
    assert_eq!(
        failure(&segments),
        BezierWireError::OpenJoin { from: 20, to: 10 }
    );
    segments[1].weights = Some(&[1., -1.]);
    let error = failure(&segments);
    assert!(matches!(
        error,
        BezierWireError::Segment {
            id: 20,
            cause: BezierRayError::Line(BezierLineError::DenominatorRoots(_))
        }
    ));
    assert!(error.source().unwrap().source().is_some());
}
#[test]
fn wire_one_subnormal_gap_is_not_welded_or_rounded_away() {
    let delta = f64::from_bits(1);
    let first = [[0., 0.], [1., 0.]];
    let second = [[1., delta], [0., 0.]];
    assert_eq!(
        failure(&[segment(1, &first), segment(2, &second)]),
        BezierWireError::OpenJoin { from: 1, to: 2 }
    );
}
#[test]
fn wire_cancellation_discards_boundary_lists_and_partial_winding() {
    let points = [[-1., -1.], [1., -1.], [1., 1.], [-1., 1.], [-1., -1.]];
    let segments = polygon_segments(&points);
    for seed in [[0., 0.], [1., 1.]] {
        check_cancellation(wire_query(&segments, seed));
    }
}
fn check_cancellation(request: BezierWireQuery<'_>) {
    let mut polls = 0;
    let expected = classify_bezier_wire(request, || {
        polls += 1;
        false
    })
    .unwrap();
    for stop in 1..=polls {
        let mut count = 0;
        let error = classify_bezier_wire(request, || {
            count += 1;
            count == stop
        })
        .unwrap_err();
        assert!(error.to_string().contains("cancel"), "{stop}: {error}");
    }
    assert_eq!(classify_bezier_wire(request, || false).unwrap(), expected);
}
#[test]
fn wire_subdivision_limit_is_shared_across_all_occurrences() {
    let controls = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    let segments = [segment(1, &controls), segment(2, &controls)];
    let mut request = wire_query(&segments, [0.5, 0.5]);
    let complete = classify_bezier_wire(request, || false).unwrap();
    let used = complete.bisections();
    assert!(used > 1);
    request.options.max_bisections = used;
    assert_eq!(classify_bezier_wire(request, || false).unwrap(), complete);
    request.options.max_bisections = used - 1;
    assert!(matches!(
        classify_bezier_wire(request, || false),
        Err(BezierWireError::Segment { id: 2, .. })
    ));
}
#[test]
fn wire_rounded_rational_endpoint_equality_is_not_exact_closure() {
    let controls = [[0., 0.], [1., 0.]];
    let rounded = f64::from_bits(0x3fe5555555555555);
    assert_eq!(rounded, 2. / 3.);
    let back = [[rounded, 0.], [0., 0.]];
    let mut segments = [segment(1, &controls), segment(2, &back)];
    segments[0].weights = Some(&[1., 2.]);
    segments[0].interval = [0., 0.5];
    assert_eq!(
        failure(&segments),
        BezierWireError::OpenJoin { from: 1, to: 2 }
    );
}
#[test]
fn wire_boundary_does_not_skip_a_later_isolation_limit() {
    let first = [[0., 0.]];
    let later = [[0., 0.], [1., 1.], [1., -2.], [0., 0.]];
    let segments = [segment(1, &first), segment(2, &later)];
    let mut request = wire_query(&segments, [0., 0.]);
    request.options.max_bisections = 0;
    assert!(matches!(
        classify_bezier_wire(request, || false),
        Err(BezierWireError::Segment { id: 2, .. })
    ));
}
