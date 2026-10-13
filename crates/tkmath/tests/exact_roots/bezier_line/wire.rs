// SPDX-License-Identifier: GPL-2.0-only
use super::*;
mod failures;
mod invariants;
mod oracle;
fn segment(id: u64, controls: &[[f64; 2]]) -> BezierWireSegment<'_> {
    BezierWireSegment {
        id,
        controls,
        weights: None,
        interval: [0., 1.],
        reversed: false,
    }
}
fn wire_query<'a>(segments: &'a [BezierWireSegment<'a>], seed: [f64; 2]) -> BezierWireQuery<'a> {
    BezierWireQuery {
        segments,
        line: [seed, [seed[0] + 1., seed[1]]],
        options: options(0.0001),
    }
}
fn classify(segments: &[BezierWireSegment<'_>], seed: [f64; 2]) -> BezierWireResult {
    classify_bezier_wire(wire_query(segments, seed), || false).unwrap()
}
fn polygon_segments(points: &[[f64; 2]]) -> Vec<BezierWireSegment<'_>> {
    points
        .windows(2)
        .enumerate()
        .map(|(index, edge)| segment(index as u64, edge))
        .collect()
}
#[test]
fn wire_closed_polygon_inside_outside_and_all_boundary_occurrences() {
    let points = [[-2., -2.], [2., -2.], [2., 2.], [-2., 2.], [-2., -2.]];
    let segments = polygon_segments(&points);
    assert_eq!(
        classify(&segments, [0., 0.]).location(),
        &BezierWireLocation::Inside { winding: 1 }
    );
    assert_eq!(
        classify(&segments, [3., 0.]).location(),
        &BezierWireLocation::Outside
    );
    assert_eq!(
        classify(&segments, [2., 2.]).location(),
        &BezierWireLocation::Boundary {
            segment_ids: vec![1, 2]
        }
    );
    assert_eq!(
        classify(&segments, [0., -2.]).location(),
        &BezierWireLocation::Boundary {
            segment_ids: vec![0]
        }
    );
}
#[test]
fn wire_single_closed_curved_segment_has_exact_membership() {
    let controls = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    let segments = [segment(91, &controls)];
    assert_eq!(
        classify(&segments, [0.5, 0.5]).location(),
        &BezierWireLocation::Inside { winding: 1 }
    );
    assert_eq!(
        classify(&segments, [2., 2.]).location(),
        &BezierWireLocation::Outside
    );
    assert_eq!(
        classify(&segments, [0., 0.]).location(),
        &BezierWireLocation::Boundary {
            segment_ids: vec![91]
        }
    );
}
#[test]
fn wire_curved_lens_and_reversed_occurrences_use_shared_endpoint_ownership() {
    let bottom = [[-2., 0.], [0., -2.], [2., 0.]];
    let top = [[-2., 0.], [0., 2.], [2., 0.]];
    let mut segments = [segment(10, &bottom), segment(20, &top)];
    segments[1].reversed = true;
    assert_eq!(
        classify(&segments, [0., 0.]).location(),
        &BezierWireLocation::Inside { winding: 1 }
    );
    assert_eq!(
        classify(&segments, [0., 1.]).location(),
        &BezierWireLocation::Boundary {
            segment_ids: vec![20]
        }
    );
    assert_eq!(
        classify(&segments, [0., 1.5]).location(),
        &BezierWireLocation::Outside
    );
    assert_eq!(
        classify(&segments, [2., 0.]).location(),
        &BezierWireLocation::Boundary {
            segment_ids: vec![10, 20]
        }
    );
}
#[test]
fn wire_trimmed_rational_endpoints_keep_unreduced_relative_scales() {
    let rational = [[0., 0.], [3., 0.]];
    let line = [[0., 0.], [3., 0.]];
    let weights = [1., 2.];
    let mut segments = [segment(1, &rational), segment(2, &line)];
    segments[0].weights = Some(&weights);
    segments[0].interval = [0., 0.5];
    segments[1].controls = &[[2., 0.], [0., 0.]];
    assert_eq!(
        classify(&segments, [1., 1.]).location(),
        &BezierWireLocation::Outside
    );
    assert_eq!(
        classify(&segments, [1., 0.]).location(),
        &BezierWireLocation::Boundary {
            segment_ids: vec![1, 2]
        }
    );
}
#[test]
fn wire_distinct_nonbinary_rational_parameterizations_can_close_exactly() {
    let controls = [[0., 0.], [1., 0.]];
    let weights = [1., 2.];
    let elevated = [[0., 0.], [0.5, 0.], [1., 0.]];
    let elevated_weights = [1., 2., 4.];
    let mut segments = [segment(41, &controls), segment(42, &elevated)];
    segments[0].weights = Some(&weights);
    segments[1].weights = Some(&elevated_weights);
    segments[0].interval = [0., 0.5];
    segments[1].interval = [0., 0.5];
    segments[1].reversed = true;
    assert_eq!(
        classify(&segments, [0., 1.]).location(),
        &BezierWireLocation::Outside
    );
}
#[test]
fn wire_trim_intervals_outside_the_basis_are_not_rescaled() {
    let controls = [[0., 0.], [1., 0.]];
    let back = [[2., 0.], [-1., 0.]];
    let mut segments = [segment(1, &controls), segment(2, &back)];
    segments[0].interval = [-1., 2.];
    assert_eq!(
        classify(&segments, [0., 0.]).location(),
        &BezierWireLocation::Boundary {
            segment_ids: vec![1, 2]
        }
    );
}
#[test]
fn wire_zero_width_occurrence_can_close_but_does_not_claim_face_validity() {
    let controls = [[0., 0.], [2., 2.]];
    let mut segments = [segment(7, &controls)];
    segments[0].interval = [0.5, 0.5];
    assert_eq!(
        classify(&segments, [1., 1.]).location(),
        &BezierWireLocation::Boundary {
            segment_ids: vec![7]
        }
    );
    assert_eq!(
        classify(&segments, [0., 0.]).location(),
        &BezierWireLocation::Outside
    );
}
