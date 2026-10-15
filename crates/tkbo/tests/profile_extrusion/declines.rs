// SPDX-License-Identifier: GPL-2.0-only
//! Named declines of `build_planar_regions`: typed stage, class and cause; no partial set.
use super::fixtures::*;
use rustycad_tkbo::profile_extrusion::*;

fn declined(profile: &SketchProfile, resolution: Resolution) -> OperationDecline {
    build_planar_regions(profile, resolution).unwrap_err()
}

#[test]
fn open_profile_names_the_dangling_vertex() {
    let profile = SketchFixture::new()
        .vertex(0, [0., 0.])
        .vertex(1, [1., 0.])
        .vertex(2, [1., 1.])
        .line(10, 0, 1)
        .line(11, 1, 2)
        .build();
    let decline = declined(&profile, resolution());
    assert_eq!(decline.operation, OperationKind::BuildPlanarRegions);
    assert_eq!(decline.stage, OperationStage::Loops);
    assert_eq!(decline.class(), DeclineClass::InvalidGeometry);
    let open = DeclineCause::OpenProfile {
        vertex: ProfileVertexId(0),
        uses: 1,
    };
    assert_eq!(decline.cause, open);
}

#[test]
fn branching_profile_is_declined() {
    let mut fixture = SketchFixture::new();
    fixture
        .rectangle(0, [0., 0.], [1., 1.])
        .vertex(9, [2., 0.])
        .line(30, 1, 9);
    let decline = declined(&fixture.build(), resolution());
    assert!(
        matches!(
            decline.cause,
            DeclineCause::BranchingProfile { uses: 3, .. }
        ),
        "{decline}"
    );
}

#[test]
fn self_intersecting_bow_tie_names_both_curves_and_the_crossing() {
    let profile = SketchFixture::new()
        .vertex(0, [0., 0.])
        .vertex(1, [2., 2.])
        .vertex(2, [2., 0.])
        .vertex(3, [0., 2.])
        .line(10, 0, 1)
        .line(11, 1, 2)
        .line(12, 2, 3)
        .line(13, 3, 0)
        .build();
    let decline = declined(&profile, resolution());
    assert_eq!(decline.stage, OperationStage::Certification);
    let expected = DeclineCause::SelfIntersecting {
        curves: [ProfileCurveId(10), ProfileCurveId(12)],
        point: Some([1., 1.]),
    };
    assert_eq!(decline.cause, expected);
}

#[test]
fn crossing_circles_are_self_intersecting() {
    let profile = SketchFixture::new()
        .circle(1, [0., 0.], 1.)
        .circle(2, [1., 0.], 1.)
        .build();
    let decline = declined(&profile, resolution());
    assert!(
        matches!(decline.cause, DeclineCause::SelfIntersecting { .. }),
        "{decline}"
    );
}

#[test]
fn tolerance_cap_below_the_stored_tolerance_is_refused() {
    let mut strict = resolution();
    strict.cap = ToleranceCap::Refuse(1e-9);
    let decline = declined(&rectangle(), strict);
    assert_eq!(decline.stage, OperationStage::Tolerance);
    assert_eq!(decline.class(), DeclineClass::OverBudget);
    let DeclineCause::ToleranceOverBudget { achieved, cap } = decline.cause else {
        panic!("{decline}")
    };
    assert_eq!(cap, 1e-9);
    assert!(achieved > cap);
}

#[test]
fn report_cap_keeps_the_region() {
    let mut lenient = resolution();
    lenient.cap = ToleranceCap::Report(1e-9);
    assert!(build_planar_regions(&rectangle(), lenient).is_ok());
}

#[test]
fn unsupported_and_invalid_inputs_are_named() {
    let mut ellipse = SketchFixture::new();
    ellipse.profile.curves.push(ProfileCurve::Ellipse {
        id: ProfileCurveId(4),
        center: [0., 0.],
        major_axis: [2., 0.],
        minor_radius: 1.,
    });
    let decline = declined(&ellipse.build(), resolution());
    assert_eq!(decline.class(), DeclineClass::Unsupported);
    let empty = SketchFixture::new().build();
    assert_eq!(
        declined(&empty, resolution()).cause,
        DeclineCause::EmptyProfile
    );
    let mut bad = resolution();
    bad.sew = f64::NAN;
    assert!(matches!(
        declined(&rectangle(), bad).cause,
        DeclineCause::InvalidResolution { field: "sew", .. }
    ));
}

#[test]
fn arc_whose_ends_disagree_on_the_radius_is_declined() {
    let profile = SketchFixture::new()
        .vertex(0, [1., 0.])
        .vertex(1, [-1.1, 0.])
        .arc(1, [0, 1], [0., 0.], ArcSense::CounterClockwise)
        .line(2, 1, 0)
        .build();
    let decline = declined(&profile, resolution());
    assert!(
        matches!(decline.cause, DeclineCause::ArcRadiusMismatch { .. }),
        "{decline}"
    );
}
