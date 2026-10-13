// SPDX-License-Identifier: GPL-2.0-only
use super::*;
fn raw(id: u64, controls: &[[f64; 2]]) -> BezierWireSegment<'_> {
    BezierWireSegment {
        id,
        controls,
        weights: None,
        interval: [0., 1.],
        reversed: false,
    }
}
fn capture_raw(raw: BezierWireSegment<'_>, id: u64) -> CapturedBezierWireOccurrence {
    CapturedBezierWireOccurrence {
        id,
        reversed: raw.reversed,
        input: CapturedBezierWireInput::capture(raw.id, raw.controls, raw.weights, raw.interval),
    }
}
fn cold_pair(segments: &[BezierWireSegment<'_>], line: [[f64; 2]; 2]) {
    let options = RootIsolationOptions::new(0.001);
    let request = BezierWireQuery {
        segments,
        line,
        options,
    };
    let old = classify_bezier_wire(request, || false);
    let occurrences: Vec<_> = segments.iter().map(|s| capture_raw(*s, s.id)).collect();
    let new = CapturedBezierWireStore::new(line, options).classify(&occurrences, || false);
    match (old, new) {
        (Ok(a), Ok(b)) => {
            assert_eq!(a.location(), b.location());
            assert_eq!(a.bisections(), b.work().new_bisections);
        }
        (Err(a), Err(b)) => assert_eq!(&a, b.cause()),
        _ => panic!("expected matching cold completion or typed failure"),
    }
}
#[test]
fn exact_subnormal_and_rational_endpoint_gaps_are_not_welded() {
    let first = [[0., 0.], [1., 0.]];
    let second = [[1., f64::from_bits(1)], [0., 0.]];
    cold_pair(&[raw(1, &first), raw(2, &second)], [[0., 0.], [1., 0.]]);
    let back = [[2. / 3., 0.], [0., 0.]];
    let mut weighted = raw(1, &first);
    weighted.weights = Some(&[1., 2.]);
    weighted.interval = [0., 0.5];
    cold_pair(&[weighted, raw(2, &back)], [[0., 0.], [1., 0.]]);
}
#[test]
fn signed_regular_weights_degenerate_trim_and_poles_preserve_cold_causes() {
    let controls = [[1., 1.], [1., -1.], [1., 1.]];
    let mut segment = raw(1, &controls);
    segment.weights = Some(&[-1.; 3]);
    cold_pair(&[segment], [[0., 0.], [1., 0.]]);
    segment.interval = [0.5, 0.5];
    cold_pair(&[segment], [[0., 0.], [1., 0.]]);
    segment.interval = [0., 1.];
    segment.weights = Some(&[1., -2., 1.]);
    cold_pair(&[segment], [[0., 0.], [1., 0.]]);
}
#[test]
fn half_open_polygon_vertex_ownership_survives_reversed_reuse() {
    let points = [[-2., -2.], [2., -2.], [2., 2.], [-2., 2.], [-2., -2.]];
    let segments = polygon_segments(&points);
    let mut captured =
        CapturedBezierWireStore::new([[0., 0.], [1., 0.]], RootIsolationOptions::new(0.001));
    let forward: Vec<_> = segments.iter().map(|s| capture_raw(*s, s.id)).collect();
    assert_eq!(
        captured.classify(&forward, || false).unwrap().location(),
        &BezierWireLocation::Inside { winding: 1 }
    );
    let reverse = reverse_occurrences(&segments);
    let result = captured.classify(&reverse, || false).unwrap();
    assert_eq!(
        result.location(),
        &BezierWireLocation::Inside { winding: -1 }
    );
    assert_eq!(result.work().new_bisections, 0);
    assert_eq!(result.work().reused_certificates, 4);
}
fn reverse_occurrences(segments: &[BezierWireSegment<'_>]) -> Vec<CapturedBezierWireOccurrence> {
    segments
        .iter()
        .rev()
        .map(|s| {
            let mut occurrence = capture_raw(*s, s.id + 100);
            occurrence.reversed = true;
            occurrence
        })
        .collect()
}
#[test]
fn nonfinite_weights_empty_controls_and_changed_degree_policy_are_not_admitted() {
    let mut segment = raw(1, &CUBIC);
    segment.weights = Some(&[1., 1., f64::NAN, 1.]);
    let input = capture_raw(segment, 1);
    let failed = store(100).classify(&[input], || false).unwrap_err();
    assert_nonfinite_weight(&failed);
    cold_pair(&[raw(2, &[])], [[0., 0.], [1., 0.]]);
    let options = RootIsolationOptions {
        max_degree: 1,
        ..RootIsolationOptions::new(0.001)
    };
    let failed = CapturedBezierWireStore::new([[0.5, 0.5], [1.5, 0.5]], options)
        .classify(&[occurrence(7, 1, false)], || false)
        .unwrap_err();
    assert_eq!(failed.work().operation_bisections, 0);
}

fn polygon_segments(points: &[[f64; 2]]) -> Vec<BezierWireSegment<'_>> {
    points
        .windows(2)
        .enumerate()
        .map(|(i, edge)| raw(i as u64, edge))
        .collect()
}

fn assert_nonfinite_weight(failed: &CapturedBezierWireFailure) {
    assert!(matches!(
        failed.cause(),
        BezierWireError::Segment {
            cause: BezierRayError::Line(BezierLineError::Weight { index: 2, .. }),
            ..
        }
    ));
}
