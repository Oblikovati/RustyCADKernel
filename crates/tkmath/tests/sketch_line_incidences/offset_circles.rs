// SPDX-License-Identifier: GPL-2.0-only
//! Circle offsets are the concentric circles of radius `r + d` (counterclockwise
//! travel) or `r - d` (clockwise arcs), with the basis sector (ADR 0080).
use super::*;

pub fn offset(basis: SketchIncidenceCurve, distance: f64) -> SketchIncidenceCurve {
    SketchIncidenceCurve::Offset(SketchIncidenceOffset {
        basis: Box::new(basis),
        distance,
    })
}
fn horizontal() -> SketchIncidenceSource {
    source([[-2., 0.], [2., 0.]], SketchLineDomain::Segment)
}
#[test]
fn a_full_circle_offset_grows_outward() {
    let report = certify(
        horizontal(),
        &[target(3, offset(circle([0., 0.], 1.), 0.5))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 2, "{report:#?}");
    assert_exact_point(contacts[0], [-1.5, 0.]);
    assert_exact_point(contacts[1], [1.5, 0.]);
    assert!(
        contacts
            .iter()
            .all(|c| c.transition() == SketchContactTransition::Crossing)
    );
}
// The clockwise arc from (2, 0) through (0, -2) and (-2, 0) to (0, 2) has its
// right normal toward the center, so a distance of 0.5 gives radius 1.5.
#[test]
fn a_clockwise_arc_offset_shrinks_and_keeps_its_sector() {
    let basis = arc([0., 0.], 2., [1., 0.], [0., 1.], false);
    let report = certify(horizontal(), &[target(3, offset(basis, 0.5))]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 2, "{report:#?}");
    assert_exact_point(contacts[0], [-1.5, 0.]);
    assert_eq!(contacts[0].target_position(), SketchLinePosition::Interior);
    assert_exact_point(contacts[1], [1.5, 0.]);
    assert_eq!(contacts[1].target_position(), SketchLinePosition::AtStart);
    assert_eq!(contacts[1].transition(), SketchContactTransition::TargetEnd);
}
// 1 + 0.1 is not a binary64 value; the contact x = +-(1 + 0.1) is certified exactly.
#[test]
fn an_inexact_offset_radius_is_certified_exactly() {
    let report = certify(
        horizontal(),
        &[target(3, offset(circle([0., 0.], 1.), 0.1))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 2, "{report:#?}");
    let radius = 1. + 0.1;
    assert_point(contacts[1], [radius, 0.]);
    let [lower, upper] = contacts[1].source_parameter_bounds().unwrap();
    assert!(lower <= (radius + 2.) / 4. && (radius + 2.) / 4. <= upper);
}
#[test]
fn a_circle_offset_source_equals_its_concentric_circle() {
    let targets = [
        target(1, infinite([0., -3.], [0., 3.])),
        target(2, circle([1., 1.], 1.5)),
        target(3, segment([-3., 0.5], [3., 1.])),
    ];
    let native = certify(
        curve_source(
            offset(circle([0.25, 0.], 1.), 1.),
            SketchLineDomain::Support,
        ),
        &targets,
    );
    let concentric = certify(
        curve_source(circle([0.25, 0.], 2.), SketchLineDomain::Support),
        &targets,
    );
    assert!(native.ordered().len() >= 6, "{native:#?}");
    assert_eq!(native.ordered(), concentric.ordered());
    for (left, right) in native.targets().iter().zip(concentric.targets()) {
        assert_eq!(left.contacts(), right.contacts());
    }
}
#[test]
fn a_zero_offset_is_its_basis() {
    let basis = spline(
        3,
        &[[-2., -1.], [-1., 2.], [0., -2.], [1., 2.], [2., -1.]],
        &[0., 0., 0., 0., 0.5, 1., 1., 1., 1.],
        [0., 1.],
    );
    let line = source([[-3., 0.], [3., 0.]], SketchLineDomain::Support);
    let native = certify(line.clone(), &[target(5, offset(basis.clone(), 0.))]);
    let plain = certify(line, &[target(5, basis)]);
    assert_eq!(
        native.targets()[0].contacts(),
        plain.targets()[0].contacts()
    );
}
