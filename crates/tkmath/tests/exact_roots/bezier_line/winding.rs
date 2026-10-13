// SPDX-License-Identifier: GPL-2.0-only
use super::*;
mod failures;
mod oracle;
fn winding(request: BezierLineQuery<'_>) -> BezierWinding {
    bezier_winding_contribution(request, || false).unwrap()
}
#[test]
fn winding_oriented_crossings_retain_boundary_and_ignore_behind_contacts() {
    for (x, expected) in [
        (-1., BezierWinding::Clear(0)),
        (0., BezierWinding::Boundary),
        (1., BezierWinding::Clear(1)),
    ] {
        let controls = [[x, -1.], [x, 1.]];
        assert_eq!(winding(query(&controls)), expected);
        let reversed = [controls[1], controls[0]];
        let opposite = if x > 0. {
            BezierWinding::Clear(-1)
        } else {
            expected
        };
        assert_eq!(winding(query(&reversed)), opposite);
    }
}
#[test]
fn winding_tangent_multiplicity_and_stationary_crossings_are_exact() {
    for (ordinates, expected) in [
        (vec![1., -1., 1.], 0),
        (vec![-1., 1., -1., 1.], 1),
        (vec![1., -1., 1., -1., 1.], 0),
    ] {
        let controls: Vec<[f64; 2]> = ordinates.into_iter().map(|y| [1., y]).collect();
        assert_eq!(winding(query(&controls)), BezierWinding::Clear(expected));
    }
}
#[test]
fn winding_half_open_endpoints_count_crossings_once_and_cancel_touches() {
    for (ordinates, expected) in [
        ([-1., 0.], 0),
        ([0., 1.], 1),
        ([1., 0.], -1),
        ([0., -1.], 0),
    ] {
        let controls = ordinates.map(|y| [1., y]);
        assert_eq!(winding(query(&controls)), BezierWinding::Clear(expected));
    }
    assert_eq!(
        winding(query(&[[1., 0.], [1., 0.], [1., 1.]])),
        BezierWinding::Clear(1)
    );
    assert_eq!(
        winding(query(&[[1., 1.], [1., 0.], [1., 0.]])),
        BezierWinding::Clear(-1)
    );
}
#[test]
fn winding_coincident_curves_use_exact_origin_incidence() {
    for (abscissae, expected) in [
        (vec![-1., 1.], BezierWinding::Boundary),
        (vec![1., 2.], BezierWinding::Clear(0)),
        (vec![-2., -1.], BezierWinding::Clear(0)),
        (vec![0., 0.], BezierWinding::Boundary),
        (vec![1., -0.25, 1.], BezierWinding::Clear(0)),
        (vec![1., -1., 1.], BezierWinding::Boundary),
    ] {
        let controls: Vec<[f64; 2]> = abscissae.into_iter().map(|x| [x, 0.]).collect();
        assert_eq!(winding(query(&controls)), expected);
    }
}
#[test]
fn winding_point_parameter_intervals_contribute_only_boundary() {
    for (x, expected) in [(0., BezierWinding::Boundary), (1., BezierWinding::Clear(0))] {
        let controls = [[x, -1.], [x, 1.]];
        let mut request = query(&controls);
        request.interval = [0.5, 0.5];
        assert_eq!(winding(request), expected);
    }
}
#[test]
fn winding_exact_closed_polygon_corpus_covers_vertices_holes_and_self_crossings() {
    let square = [[-2., -2.], [2., -2.], [2., 2.], [-2., 2.], [-2., -2.]];
    assert_eq!(polygon(&square, [0., 0.]), BezierWinding::Clear(1));
    assert_eq!(polygon(&square, [3., 0.]), BezierWinding::Clear(0));
    assert_eq!(polygon(&square, [0., -2.]), BezierWinding::Boundary);
    assert_eq!(polygon(&square, [2., 2.]), BezierWinding::Boundary);
    let diamond = [[0., -2.], [2., 0.], [0., 2.], [-2., 0.], [0., -2.]];
    assert_eq!(polygon(&diamond, [0., 0.]), BezierWinding::Clear(1));
    let bow = [[-2., -2.], [2., 2.], [-2., 2.], [2., -2.], [-2., -2.]];
    assert_eq!(polygon(&bow, [0., 1.]), BezierWinding::Clear(1));
    assert_eq!(polygon(&bow, [0., -1.]), BezierWinding::Clear(-1));
    assert_eq!(polygon(&bow, [0., 0.]), BezierWinding::Boundary);
}
fn polygon(points: &[[f64; 2]], seed: [f64; 2]) -> BezierWinding {
    assert_eq!(points.first(), points.last());
    let mut total = 0;
    for edge in points.windows(2) {
        let mut request = query(edge);
        request.line = [seed, [seed[0] + 1., seed[1]]];
        match winding(request) {
            BezierWinding::Boundary => return BezierWinding::Boundary,
            BezierWinding::Clear(count) => total += count,
        }
    }
    BezierWinding::Clear(total)
}
#[test]
fn winding_oppositely_oriented_nested_wires_cancel_inside_the_hole() {
    let outer = [[-2., -2.], [2., -2.], [2., 2.], [-2., 2.], [-2., -2.]];
    let inner = [[-1., -1.], [-1., 1.], [1., 1.], [1., -1.], [-1., -1.]];
    assert_eq!(polygon(&outer, [0., 0.]), BezierWinding::Clear(1));
    assert_eq!(polygon(&inner, [0., 0.]), BezierWinding::Clear(-1));
    assert_eq!(polygon(&inner, [1.5, 0.]), BezierWinding::Clear(0));
}
#[test]
fn winding_rational_weight_sign_and_all_finite_exponents_preserve_contribution() {
    for exponent in 0..2047_u64 {
        let scale = f64::from_bits((exponent << 52) | 1);
        for signed in [scale, -scale] {
            assert_eq!(scaled_winding(signed), BezierWinding::Clear(1));
        }
    }
    assert_eq!(scaled_winding(f64::MAX), BezierWinding::Clear(1));
}
fn scaled_winding(scale: f64) -> BezierWinding {
    let controls = [[scale, -scale], [scale, scale]];
    let weights = [-scale, -scale];
    let mut request = query(&controls);
    request.weights = Some(&weights);
    request.line[1] = [scale, 0.];
    winding(request)
}
