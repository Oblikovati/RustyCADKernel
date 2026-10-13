// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn wire_reversal_negates_winding_and_preserves_boundary_occurrence_order() {
    let points = [[-2., -2.], [2., -2.], [2., 2.], [-2., 2.], [-2., -2.]];
    let mut segments = polygon_segments(&points);
    segments.reverse();
    for segment in &mut segments {
        segment.reversed = true;
    }
    assert_eq!(
        classify(&segments, [0., 0.]).location(),
        &BezierWireLocation::Inside { winding: -1 }
    );
    assert_eq!(
        classify(&segments, [2., 2.]).location(),
        &BezierWireLocation::Boundary {
            segment_ids: vec![2, 1]
        }
    );
}
#[test]
fn wire_repeated_traversals_preserve_nonzero_winding_multiplicity() {
    let controls = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    let mut segments = [segment(0, &controls), segment(u64::MAX, &controls)];
    assert_eq!(
        classify(&segments, [0.5, 0.5]).location(),
        &BezierWireLocation::Inside { winding: 2 }
    );
    segments[1].reversed = true;
    assert_eq!(
        classify(&segments, [0.5, 0.5]).location(),
        &BezierWireLocation::Outside
    );
}
#[test]
fn wire_result_and_source_bits_remain_isolated() {
    let mut controls = vec![[-0., 0.], [4., 0.], [0., 4.], [-0., 0.]];
    let before: Vec<[u64; 2]> = controls.iter().map(|p| p.map(f64::to_bits)).collect();
    let result = classify(&[segment(8, &controls)], [0., 0.]);
    assert_eq!(
        controls
            .iter()
            .map(|p| p.map(f64::to_bits))
            .collect::<Vec<_>>(),
        before
    );
    controls.clear();
    assert_eq!(
        result.location(),
        &BezierWireLocation::Boundary {
            segment_ids: vec![8]
        }
    );
}
#[test]
fn wire_exact_closure_survives_every_finite_scale_and_weight_sign() {
    for exponent in 0..2047_u64 {
        let scale = f64::from_bits((exponent << 52) | 1);
        assert_eq!(
            scaled_square(scale).location(),
            &BezierWireLocation::Inside { winding: 1 }
        );
        assert_eq!(
            scaled_square(-scale).location(),
            &BezierWireLocation::Inside { winding: 1 }
        );
    }
    assert_eq!(
        scaled_square(f64::MAX).location(),
        &BezierWireLocation::Inside { winding: 1 }
    );
}
fn scaled_square(scale: f64) -> BezierWireResult {
    let points = [
        [-scale, -scale],
        [scale, -scale],
        [scale, scale],
        [-scale, scale],
        [-scale, -scale],
    ];
    let weights = [-scale, -scale];
    let mut segments = polygon_segments(&points);
    for segment in &mut segments {
        segment.weights = Some(&weights);
    }
    let request = BezierWireQuery {
        segments: &segments,
        line: [[0., 0.], [scale, 0.]],
        options: options(0.0001),
    };
    classify_bezier_wire(request, || false).unwrap()
}
#[test]
fn wire_reversed_subnormal_trim_does_not_round_a_parameter_transform() {
    let controls = [[0., 0.], [1., 0.]];
    let interval = [f64::from_bits(1), f64::from_bits(2)];
    assert_eq!(1. - interval[0], 1. - interval[1]);
    let mut segments = [segment(1, &controls), segment(2, &controls)];
    for segment in &mut segments {
        segment.interval = interval;
    }
    segments[1].reversed = true;
    assert_eq!(
        classify(&segments, [0., 1.]).location(),
        &BezierWireLocation::Outside
    );
}
#[test]
fn wire_self_crossing_path_returns_signed_locations_and_all_origin_incidents() {
    let points = [[-2., -2.], [2., 2.], [-2., 2.], [2., -2.], [-2., -2.]];
    let segments = polygon_segments(&points);
    assert_eq!(
        classify(&segments, [0., 1.]).location(),
        &BezierWireLocation::Inside { winding: 1 }
    );
    assert_eq!(
        classify(&segments, [0., -1.]).location(),
        &BezierWireLocation::Inside { winding: -1 }
    );
    assert_eq!(
        classify(&segments, [0., 0.]).location(),
        &BezierWireLocation::Boundary {
            segment_ids: vec![0, 2]
        }
    );
}
