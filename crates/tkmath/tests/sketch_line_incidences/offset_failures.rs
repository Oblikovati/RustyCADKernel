// SPDX-License-Identifier: GPL-2.0-only
//! Every offset refusal is named and carries the offending value (ADR 0080).
use super::offset_circles::offset;
use super::*;

fn invalid(curve: SketchIncidenceCurve) -> SketchIncidenceCause {
    let line = source([[-3., 0.], [3., 0.]], SketchLineDomain::Segment);
    let options = SketchIncidenceOptions::v1(ERROR);
    let failure = certify_sketch_line_incidences(line, &[target(9, curve)], options, || false)
        .expect_err("refused offset");
    assert_eq!(failure.completion(), SketchIncidenceCompletion::Invalid);
    assert_eq!(failure.target(), Some(9));
    failure.cause().clone()
}
fn source_failure(curve: SketchIncidenceCurve, domain: SketchLineDomain) -> SketchIncidenceCause {
    let options = SketchIncidenceOptions::v1(ERROR);
    let failure =
        certify_sketch_line_incidences(curve_source(curve, domain), &[], options, || false)
            .expect_err("refused offset source");
    assert_eq!(failure.completion(), SketchIncidenceCompletion::Invalid);
    failure.cause().clone()
}
fn clamped_quadratic(poles: &[[f64; 2]]) -> SketchIncidenceCurve {
    spline(2, poles, &[0., 0., 0., 1., 1., 1.], [0., 1.])
}
#[test]
fn a_nonfinite_distance_names_its_field() {
    let cause = invalid(offset(circle([0., 0.], 1.), f64::INFINITY));
    assert_eq!(
        cause,
        SketchIncidenceCause::NonFinite {
            field: "offset.distance",
            index: 0,
            value: f64::INFINITY
        }
    );
}
#[test]
fn a_circle_offset_through_its_center_is_refused() {
    assert_eq!(
        invalid(offset(circle([0., 0.], 1.), -1.)),
        SketchIncidenceCause::OffsetRadius {
            radius: 1.,
            distance: -1.
        }
    );
    let clockwise = arc([0., 0.], 1., [1., 0.], [0., 1.], false);
    assert_eq!(
        invalid(offset(clockwise, 2.)),
        SketchIncidenceCause::OffsetRadius {
            radius: 1.,
            distance: 2.
        }
    );
}
#[test]
fn nested_offsets_and_irrational_ellipse_axes_are_refused() {
    let nested = offset(offset(circle([0., 0.], 1.), 0.5), 0.5);
    assert_eq!(invalid(nested), SketchIncidenceCause::OffsetBasis);
    let skew = offset(ellipse([0., 0.], [1., 1.], [2., 1.]), 0.25);
    assert_eq!(
        invalid(skew),
        SketchIncidenceCause::OffsetEllipseAxis {
            major_axis: [1., 1.]
        }
    );
}
#[test]
fn spline_offsets_need_a_regular_c1_basis() {
    let corner = spline(
        2,
        &[[0., 0.], [1., 1.], [2., 0.], [3., 1.], [4., 0.]],
        &[0., 0., 0., 0.5, 0.5, 1., 1., 1.],
        [0., 1.],
    );
    assert_eq!(
        invalid(offset(corner, 0.1)),
        SketchIncidenceCause::OffsetContinuity { index: 3 }
    );
    let returning = clamped_quadratic(&[[0., 0.], [1., 0.], [0., 0.]]);
    assert_eq!(
        invalid(offset(returning, 0.1)),
        SketchIncidenceCause::OffsetSingular { range: [0., 1.] }
    );
}
#[test]
fn offset_sources_keep_their_basis_domains() {
    let curve = offset(clamped_quadratic(&[[0., 0.], [1., 1.], [2., 0.]]), 0.1);
    assert_eq!(
        source_failure(curve, SketchLineDomain::Support),
        SketchIncidenceCause::SourceDomain {
            source: "offset spline",
            domain: SketchLineDomain::Support
        }
    );
    let unbounded = offset(infinite([0., 0.], [1., 0.]), 0.1);
    assert_eq!(
        source_failure(unbounded, SketchLineDomain::Segment),
        SketchIncidenceCause::UnboundedSource
    );
}
