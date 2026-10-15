// SPDX-License-Identifier: GPL-2.0-only
//! Ellipse and spline offsets (ADR 0080), with hand-derived points: an offset
//! point is `C(u) + d (C'_y, -C'_x) / |C'(u)|`.
use super::offset_circles::offset;
use super::*;

/// The ellipse x^2/4 + y^2 = 1 offset outward by 1: (3, 0), (0, 2), (-3, 0), (0, -2).
fn grown_ellipse() -> SketchIncidenceCurve {
    offset(ellipse([0., 0.], [1., 0.], [2., 1.]), 1.)
}
/// The quadratic arch (0,0), (1,2), (2,0) offset by 0.25 to the right of travel;
/// its apex (1, 1) has tangent (2, 0), so the offset apex is (1, 0.75).
fn lowered_arch() -> SketchIncidenceCurve {
    let arch = spline(
        2,
        &[[0., 0.], [1., 2.], [2., 0.]],
        &[0., 0., 0., 1., 1., 1.],
        [0., 1.],
    );
    offset(arch, 0.25)
}
#[test]
fn lines_meet_an_ellipse_offset_at_its_vertices() {
    let horizontal = certify(
        source([[-4., 0.], [4., 0.]], SketchLineDomain::Segment),
        &[target(1, grown_ellipse())],
    );
    let contacts = ordered(&horizontal);
    assert_eq!(contacts.len(), 2, "{horizontal:#?}");
    assert_exact_point(contacts[0], [-3., 0.]);
    assert_exact_point(contacts[1], [3., 0.]);
    for contact in contacts {
        assert_eq!(contact.transition(), SketchContactTransition::Crossing);
        assert_eq!(contact.multiplicity(), [Some(1), Some(1)]);
        assert_eq!(contact.target_parameter_bounds(), None);
    }
    let vertical = certify(
        source([[0., -4.], [0., 4.]], SketchLineDomain::Segment),
        &[target(1, grown_ellipse())],
    );
    let contacts = ordered(&vertical);
    assert_eq!(contacts.len(), 2, "{vertical:#?}");
    assert_exact_point(contacts[0], [0., -2.]);
    assert_exact_point(contacts[1], [0., 2.]);
}
// The line y = 2 touches the grown ellipse at its top vertex (0, 2).
#[test]
fn a_line_touches_an_ellipse_offset() {
    let report = certify(
        source([[-4., 2.], [4., 2.]], SketchLineDomain::Segment),
        &[target(1, grown_ellipse())],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1, "{report:#?}");
    assert_exact_point(contacts[0], [0., 2.]);
    assert_eq!(contacts[0].transition(), SketchContactTransition::Touching);
}
#[test]
fn an_ellipse_offset_source_sweeps_from_its_basis_axis() {
    let report = certify(
        curve_source(grown_ellipse(), SketchLineDomain::Segment),
        &[
            target(1, infinite([-4., 0.], [4., 0.])),
            target(2, infinite([0., -4.], [0., 4.])),
        ],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 4, "{report:#?}");
    let expected = [[3., 0.], [0., 2.], [-3., 0.], [0., -2.]];
    for (contact, point) in contacts.iter().zip(expected) {
        assert_exact_point(contact, point);
    }
    let charts: Vec<SketchSweepChart> = contacts
        .iter()
        .map(|c| c.source_sweep().unwrap().chart)
        .collect();
    use SketchSweepChart::{Lower, Opposite, Reference, Upper};
    assert_eq!(charts, [Reference, Upper, Opposite, Lower]);
}
// The grown ellipse reaches distance 3 from its center only at (+-3, 0), where the
// circle of radius 3 touches it from outside.
#[test]
fn a_circle_source_touches_an_ellipse_offset_at_its_far_vertices() {
    let around = curve_source(circle([0., 0.], 3.), SketchLineDomain::Support);
    let report = certify(around, &[target(1, grown_ellipse())]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 2, "{report:#?}");
    assert_exact_point(contacts[0], [3., 0.]);
    assert_exact_point(contacts[1], [-3., 0.]);
    for contact in contacts {
        assert_eq!(contact.transition(), SketchContactTransition::Touching);
    }
}
#[test]
fn a_vertical_line_crosses_a_spline_offset_at_its_apex() {
    let report = certify(
        source([[1., -3.], [1., 3.]], SketchLineDomain::Segment),
        &[target(2, lowered_arch())],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1, "{report:#?}");
    assert_exact_point(contacts[0], [1., 0.75]);
    assert_eq!(contacts[0].target_parameter_bounds(), Some([0.5, 0.5]));
}
#[test]
fn a_spline_offset_source_touches_its_apex_tangent() {
    let report = certify(
        curve_source(lowered_arch(), SketchLineDomain::Segment),
        &[target(2, infinite([-1., 0.75], [3., 0.75]))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1, "{report:#?}");
    assert_exact_point(contacts[0], [1., 0.75]);
    assert_eq!(contacts[0].source_parameter_bounds(), Some([0.5, 0.5]));
    assert_eq!(contacts[0].transition(), SketchContactTransition::Touching);
    assert_eq!(contacts[0].multiplicity(), [Some(2), Some(2)]);
}
#[test]
fn a_spline_offset_source_trims_by_parameter() {
    let report = certify(
        curve_source(lowered_arch(), SketchLineDomain::Segment),
        &[
            target(1, infinite([1., -3.], [1., 3.])),
            target(2, infinite([-1., 0.5], [3., 0.5])),
        ],
    );
    assert_eq!(report.ordered().len(), 3, "{report:#?}");
    let around = report
        .select_trim_at(SketchTrimPick::Parameter(0.45), || false)
        .unwrap();
    assert_eq!(
        around,
        SketchTrimSelection {
            before: Some(0),
            after: Some(1)
        }
    );
}
#[test]
fn ellipse_offset_sources_refuse_point_picks() {
    let report = certify(
        curve_source(grown_ellipse(), SketchLineDomain::Segment),
        &[target(1, infinite([-4., 0.], [4., 0.]))],
    );
    let refusal = report.select_trim([0., 2.], || false).unwrap_err();
    assert!(matches!(
        refusal,
        SketchSelectionError::Refused(SketchSelectionRefusal::PickKind { .. })
    ));
}
fn pair_refusal(
    source: SketchIncidenceSource,
    curve: SketchIncidenceCurve,
) -> SketchIncidenceCause {
    let options = SketchIncidenceOptions::v1(ERROR);
    let failure = certify_sketch_line_incidences(source, &[target(7, curve)], options, || false)
        .expect_err("uncertified offset pair");
    assert_eq!(failure.completion(), SketchIncidenceCompletion::Incomplete);
    assert_eq!(failure.target(), Some(7));
    failure.cause().clone()
}
#[test]
fn offsets_without_an_implicit_partner_are_refused_by_name() {
    let arch = spline(
        2,
        &[[0., 0.], [1., 2.], [2., 0.]],
        &[0., 0., 0., 1., 1., 1.],
        [0., 1.],
    );
    let from_spline = curve_source(arch.clone(), SketchLineDomain::Segment);
    assert_eq!(
        pair_refusal(from_spline, grown_ellipse()),
        SketchIncidenceCause::OffsetPair {
            source: "spline",
            target: "offset ellipse"
        }
    );
    let from_offset = curve_source(lowered_arch(), SketchLineDomain::Segment);
    assert_eq!(
        pair_refusal(from_offset.clone(), arch),
        SketchIncidenceCause::OffsetPair {
            source: "offset spline",
            target: "spline"
        }
    );
    assert_eq!(
        pair_refusal(from_offset, lowered_arch()),
        SketchIncidenceCause::OffsetPair {
            source: "offset spline",
            target: "offset spline"
        }
    );
}
