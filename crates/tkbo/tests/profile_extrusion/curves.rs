// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0083: ellipses, elliptic arcs and clamped B-splines in `build_planar_regions` and
//! `extrude`, against closed-form areas and volumes; their named declines.
use super::extrusion::{extruded, options};
use super::fixtures::*;
use rustycad_tkbo::profile_extrusion::*;
use std::f64::consts::PI;

fn built(profile: &SketchProfile) -> PlanarRegionSet {
    build_planar_regions(profile, resolution()).unwrap()
}

fn declined(profile: &SketchProfile) -> DeclineCause {
    build_planar_regions(profile, resolution())
        .unwrap_err()
        .cause
}

/// Encloses the closed form with a tight representation bound and the estimated method.
fn assert_area(region: &PlanarRegion, expected: f64) {
    let area = region.area();
    assert!(area.encloses(expected), "{area:?} vs {expected}");
    assert!(area.error_bound < 1e-11, "{area:?}");
    assert_eq!(area.method, IntegrationMethod::RepresentationEnclosure);
}

#[test]
fn ellipse_disc_has_area_pi_a_b_and_extrudes_to_an_elliptic_cylinder() {
    let set = built(&ellipse());
    assert_eq!(set.regions().len(), 1);
    let region = &set.regions()[0];
    assert_area(region, PI);
    assert!(region.vertices().is_empty());
    assert_eq!(set.curve_edges()[0].regions, vec![RegionId(0)]);
    // Four quarter pieces: ten pair domains. Their corners are rounded off the tilted
    // ellipse, so the pieces are certified by the generic pair route, not as one carrier.
    assert_eq!(set.certificate().pair_domains, 10);
    let solid = extruded(&ellipse(), 0, [0., 0., 3.]);
    assert!(solid.volume().encloses(3.0 * PI), "{:?}", solid.volume());
    assert_eq!(
        (
            solid.vertices().len(),
            solid.edges().len(),
            solid.faces().len()
        ),
        (2, 3, 3)
    );
    assert!(solid.validation().is_valid());
    let roles = solid.history().of(HistorySource::Curve(ProfileCurveId(3)));
    assert!(
        roles.iter().any(|r| r.role == HistoryRole::LateralEdge),
        "seam history"
    );
}

#[test]
fn half_ellipse_and_general_elliptic_segment_have_closed_form_areas() {
    let half = SketchFixture::new()
        .vertex(0, [3., 0.])
        .vertex(1, [-3., 0.])
        .elliptic_arc(
            1,
            [0, 1],
            ([0., 0.], [3., 0.], 2.),
            ArcSense::CounterClockwise,
        )
        .line(2, 1, 0)
        .build();
    assert_area(&built(&half).regions()[0], 3.0 * PI);
    // A tilted ellipse, a = 2.5, b = 1: the segment between parameters 0.3 and 2.4 has the
    // affine image of the circular segment area, a b (dt - sin dt) / 2.
    let (center, axis, b) = ([0.5, -1.], [1.5, 2.], 1.);
    let point = |t: f64| {
        let (u, v) = ([0.6, 0.8], [-0.8, 0.6]);
        [0, 1].map(|k| center[k] + 2.5 * t.cos() * u[k] + b * t.sin() * v[k])
    };
    let segment = SketchFixture::new()
        .vertex(0, point(0.3))
        .vertex(1, point(2.4))
        .elliptic_arc(1, [0, 1], (center, axis, b), ArcSense::CounterClockwise)
        .line(2, 1, 0)
        .build();
    let dt: f64 = 2.1;
    assert_area(
        &built(&segment).regions()[0],
        2.5 * b * (dt - dt.sin()) / 2.0,
    );
    let solid = extruded(&segment, 0, [0., 0., -2.]);
    assert!(solid.volume().encloses(2.5 * (dt - dt.sin())));
}

#[test]
fn polynomial_spline_bounded_region_is_the_parabola_segment() {
    let set = built(&parabola_segment());
    assert_area(&set.regions()[0], 16.0 / 3.0);
    let solid = extruded(&parabola_segment(), 0, [0., 0., 1.5]);
    assert!(solid.volume().encloses(8.0), "{:?}", solid.volume());
    assert_eq!(solid.faces().len(), 4);
    assert!(solid.validation().is_valid(), "{:?}", solid.validation());
}

#[test]
fn rational_spline_bounded_region_is_the_half_disc() {
    let set = built(&rational_half_disc());
    assert_area(&set.regions()[0], PI / 2.0);
    let solid = extruded(&rational_half_disc(), 0, [0., 0., 2.]);
    assert!(solid.volume().encloses(PI), "{:?}", solid.volume());
}

#[test]
fn curve_regions_are_deterministic() {
    for profile in [ellipse(), parabola_segment(), rational_half_disc()] {
        let (a, b) = (built(&profile), built(&profile));
        assert_eq!(a.regions()[0].area(), b.regions()[0].area());
        assert_eq!(a.regions()[0].outer(), b.regions()[0].outer());
        assert_eq!(a.certificate(), b.certificate());
    }
}

#[test]
fn invalid_and_unsupported_curves_are_named() {
    let c = ProfileCurveId(1);
    let flat = SketchFixture::new()
        .ellipse(1, [0., 0.], [1., 0.], 2.)
        .build();
    let reason = "minor radius above the major radius";
    assert_eq!(
        declined(&flat),
        DeclineCause::InvalidCurve { curve: c, reason }
    );
    let off = SketchFixture::new()
        .vertex(0, [3.1, 0.])
        .vertex(1, [-3., 0.])
        .elliptic_arc(
            1,
            [0, 1],
            ([0., 0.], [3., 0.], 2.),
            ArcSense::CounterClockwise,
        )
        .line(2, 1, 0)
        .build();
    assert!(matches!(declined(&off), DeclineCause::EndOffCurve { curve, .. } if curve == c));
    let mut spline = parabola_segment();
    let ProfileCurve::Spline { degree, knots, .. } = &mut spline.curves[0] else {
        panic!()
    };
    *degree = 17;
    knots.clear();
    let kind = "spline above degree 16";
    assert_eq!(
        declined(&spline),
        DeclineCause::UnsupportedCurve { curve: c, kind }
    );
    let mut short = parabola_segment();
    let ProfileCurve::Spline { knots, .. } = &mut short.curves[0] else {
        panic!()
    };
    knots.pop();
    assert!(matches!(
        declined(&short),
        DeclineCause::InvalidCurve { .. }
    ));
    let mut moved = parabola_segment();
    let ProfileCurve::Spline { poles, .. } = &mut moved.curves[0] else {
        panic!()
    };
    poles[0] = [0., 0.01];
    assert!(matches!(declined(&moved), DeclineCause::EndOffCurve { .. }));
    assert_eq!(options().resolution, resolution());
}
