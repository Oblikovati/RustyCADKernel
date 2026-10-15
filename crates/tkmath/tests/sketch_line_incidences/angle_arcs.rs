// SPDX-License-Identifier: GPL-2.0-only
//! Arcs bounded by parameter angles (ADR 0067): exact frame angles, never an
//! algebraic contact on a nonzero bound, wrapped sweep order on sources.
use super::*;
use SketchLinePosition::{AfterEnd, AtStart, Interior};

const SEGMENT: SketchLineDomain = SketchLineDomain::Segment;
const SUPPORT: SketchLineDomain = SketchLineDomain::Support;

fn angle_arc(
    radius: f64,
    axis: [f64; 2],
    counterclockwise: bool,
    bounds: [f64; 2],
) -> SketchIncidenceCurve {
    let angles = SketchConicAngles {
        axis,
        counterclockwise,
        bounds,
    };
    SketchIncidenceCurve::Circle {
        center: [0., 0.],
        radius,
        sector: Some(angles.into()),
    }
}
fn upright() -> SketchIncidenceSource {
    source([[0., -2.], [0., 2.]], SEGMENT)
}
#[test]
fn a_rounded_right_angle_bound_excludes_the_exact_quarter_point() {
    // The binary64 FRAC_PI_2 is below pi/2, so (0, 1) at exactly pi/2 is outside.
    let short = angle_arc(1., [1., 0.], true, [0., std::f64::consts::FRAC_PI_2]);
    assert!(certify(upright(), &[target(1, short)]).ordered().is_empty());
    // One ulp more contains pi/2 strictly: the contact is interior, never AtEnd.
    let long = angle_arc(
        1.,
        [1., 0.],
        true,
        [
            0.,
            f64::from_bits(std::f64::consts::FRAC_PI_2.to_bits() + 1),
        ],
    );
    let report = certify(upright(), &[target(1, long)]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_exact_point(contacts[0], [0., 1.]);
    assert_eq!(contacts[0].target_position(), Interior);
}
#[test]
fn a_zero_angle_bound_is_the_exact_axis_ray() {
    let arc = angle_arc(1., [1., 0.], true, [0., 1.]);
    let report = certify(source([[0., 0.], [2., 0.]], SEGMENT), &[target(1, arc)]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_eq!(summary(contacts[0]).2, AtStart);
    assert_eq!(summary(contacts[0]).3, SketchContactTransition::TargetEnd);
}
#[test]
fn clockwise_and_elliptic_angle_frames_follow_their_parameterization() {
    // Clockwise from the axis, angles up to 1.6 reach (0, -1) at pi/2.
    let clockwise = angle_arc(1., [1., 0.], false, [0., 1.6]);
    let report = certify(upright(), &[target(1, clockwise)]);
    assert_eq!(points_of(&report), [[0., -1.]]);
    // x^2/4 + y^2 = 1 with parameter angles [1, 2] contains (0, 1) at t = pi/2.
    let ellipse = SketchIncidenceCurve::Ellipse {
        center: [0., 0.],
        major_axis: [-3., 0.],
        radii: [2., 1.],
        sector: Some(
            SketchConicAngles {
                axis: [-3., 0.],
                counterclockwise: false,
                bounds: [1., 2.],
            }
            .into(),
        ),
    };
    // With axis -x and the clockwise sense the frame maps t to (-2 cos t, sin t),
    // so t = pi/2 is (0, 1).
    let report = certify(upright(), &[target(2, ellipse)]);
    assert_eq!(points_of(&report), [[0., 1.]]);
}
fn points_of(report: &SketchLineIncidenceReport) -> Vec<[f64; 2]> {
    ordered(report)
        .iter()
        .map(|contact| contact.point())
        .collect()
}
#[test]
fn an_angle_arc_source_sweeps_from_its_start_angle_and_wraps() {
    // Unit circle swept counterclockwise over [-1, 1] radians from the x axis.
    let arc = angle_arc(1., [1., 0.], true, [-1., 1.]);
    let targets = [
        target(1, segment([0.9, -2.], [0.9, 2.])),
        target(2, segment([0.5, -2.], [0.5, 2.])),
    ];
    let report = certify(curve_source(arc, SUPPORT), &targets);
    let facts: Vec<_> = ordered(&report)
        .iter()
        .map(|contact| {
            let sweep = contact.source_sweep().unwrap();
            (
                contact.target_id(),
                contact.source_position(),
                sweep.wrapped,
                contact.point()[1] > 0.,
            )
        })
        .collect();
    assert_eq!(
        facts,
        // Places before the start angle -1 in the frame (from the axis) are wrapped.
        [
            (1, Interior, false, false),
            (1, Interior, true, true),
            (2, AfterEnd, true, true),
            (2, AfterEnd, true, false),
        ]
    );
    let end = report
        .select_extension(SketchLineEnd::End, || false)
        .unwrap()
        .unwrap();
    assert!(report.contact(report.ordered()[end].members()[0]).point()[1] > 0.);
    let start = report
        .select_extension(SketchLineEnd::Start, || false)
        .unwrap()
        .unwrap();
    assert!(report.contact(report.ordered()[start].members()[0]).point()[1] < 0.);
}
#[test]
fn an_angle_arc_source_trims_between_its_contacts() {
    let arc = angle_arc(1., [1., 0.], true, [-1., 1.]);
    let report = certify(
        curve_source(arc, SEGMENT),
        &[target(1, segment([0.9, -2.], [0.9, 2.]))],
    );
    assert_eq!(report.ordered().len(), 2);
    let selection = report.select_trim([1., 0.], || false).unwrap();
    assert_eq!((selection.before, selection.after), (Some(0), Some(1)));
    assert!(report.select_trim([1., -3.], || false).is_err());
}
#[test]
fn angle_bounds_are_admitted_only_below_one_turn_and_on_the_major_axis() {
    let invalid = |curve: SketchIncidenceCurve| {
        certify_sketch_line_incidences(
            upright(),
            &[target(3, curve)],
            SketchIncidenceOptions::v1(ERROR),
            || false,
        )
        .unwrap_err()
        .cause()
        .clone()
    };
    for bounds in [[1., 0.5], [0., 6.3], [0., f64::NAN], [-100., 0.]] {
        assert!(matches!(
            invalid(angle_arc(1., [1., 0.], true, bounds)),
            SketchIncidenceCause::AngleBounds { .. } | SketchIncidenceCause::NonFinite { .. }
        ));
    }
    // 2 pi rounded to binary64 is below the exact full turn: admitted.
    let report = certify(
        upright(),
        &[target(
            3,
            angle_arc(1., [1., 0.], true, [0., std::f64::consts::TAU]),
        )],
    );
    assert_eq!(report.ordered().len(), 2);
    let skew = SketchIncidenceCurve::Ellipse {
        center: [0., 0.],
        major_axis: [1., 0.],
        radii: [2., 1.],
        sector: Some(
            SketchConicAngles {
                axis: [1., 1.],
                counterclockwise: true,
                bounds: [0., 1.],
            }
            .into(),
        ),
    };
    assert_eq!(
        invalid(skew),
        SketchIncidenceCause::AngleAxis {
            axis: [1., 1.],
            major_axis: [1., 0.]
        }
    );
}
