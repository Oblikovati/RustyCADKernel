// SPDX-License-Identifier: GPL-2.0-only
//! `build_planar_regions` against closed-form areas, independent adaptive face integration
//! and the complete curve to edge mapping.
use super::fixtures::*;
use rustycad_tkbo::profile_extrusion::*;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::f64::consts::PI;

/// Adaptive (not tessellated) area integration of the native face; independent of the
/// closed-form Green evaluation inside the operation.
fn integrated_area(region: &PlanarRegion) -> f64 {
    let mut props = GProps::new();
    BRepGProp::surface_properties_eps(region.face(), &mut props, 1e-10, false).unwrap();
    props.mass()
}

fn built(profile: &SketchProfile) -> PlanarRegionSet {
    build_planar_regions(profile, resolution()).unwrap()
}

fn assert_area(region: &PlanarRegion, expected: f64) {
    let area = region.area();
    assert!(area.encloses(expected), "{area:?} vs {expected}");
    assert!(area.error_bound < 1e-12, "{area:?}");
    let integrated = integrated_area(region);
    assert!(
        (integrated - expected).abs() < 1e-8,
        "{integrated} vs {expected}"
    );
}

/// Every caller curve maps to exactly one edge; that edge bounds exactly the listed regions.
fn assert_complete_mapping(profile: &SketchProfile, set: &PlanarRegionSet) {
    let mapped: Vec<_> = set.curve_edges().iter().map(|entry| entry.curve).collect();
    let authored: Vec<_> = profile.curves.iter().map(ProfileCurve::id).collect();
    assert_eq!(mapped, authored);
    for entry in set.curve_edges() {
        assert!(!entry.regions.is_empty(), "{:?}", entry.curve);
        for region in set.regions() {
            let uses = region
                .edges()
                .iter()
                .filter(|(curve, _)| *curve == entry.curve);
            let expected = usize::from(entry.regions.contains(&region.id()));
            assert_eq!(uses.clone().count(), expected);
            assert!(uses.into_iter().all(|(_, edge)| edge.is_same(&entry.edge)));
        }
    }
}

#[test]
fn rectangle_is_one_region_with_four_mapped_edges() {
    let profile = rectangle();
    let set = built(&profile);
    assert_eq!(set.regions().len(), 1);
    let region = &set.regions()[0];
    assert!(region.holes().is_empty());
    assert_eq!(region.outer().uses.len(), 4);
    assert_area(region, 6.0);
    assert_eq!(region.vertices().len(), 4);
    assert_complete_mapping(&profile, &set);
    assert_eq!(region.validation().tolerance, StageStatus::Passed);
}

#[test]
fn clockwise_rectangle_is_oriented_counterclockwise_in_its_face() {
    let profile = SketchFixture::new()
        .rectangle(0, [2., 3.], [0., 0.])
        .build();
    let set = built(&profile);
    assert_area(&set.regions()[0], 6.0);
}

#[test]
fn circle_is_one_region() {
    let profile = circle();
    let set = built(&profile);
    assert_eq!(set.regions().len(), 1);
    assert_area(&set.regions()[0], PI * 1.5 * 1.5);
    assert!(set.regions()[0].vertices().is_empty());
    assert_complete_mapping(&profile, &set);
}

#[test]
fn annulus_yields_the_ring_with_one_hole_and_the_inner_disc() {
    let profile = annulus();
    let set = built(&profile);
    assert_eq!(set.regions().len(), 2);
    let (ring, disc) = (&set.regions()[0], &set.regions()[1]);
    assert_eq!(ring.holes().len(), 1);
    assert_eq!(ring.holes()[0].uses[0].curve, ProfileCurveId(2));
    assert!(disc.holes().is_empty());
    assert_area(ring, 3.0 * PI);
    assert_area(disc, PI);
    assert_complete_mapping(&profile, &set);
    let inner = &set.curve_edges()[1];
    assert_eq!(inner.regions, vec![RegionId(0), RegionId(1)]);
}

#[test]
fn slot_of_lines_and_tangent_arcs_is_one_region() {
    let profile = slot();
    let set = built(&profile);
    assert_eq!(set.regions().len(), 1);
    assert_area(&set.regions()[0], 8.0 + PI);
    assert_complete_mapping(&profile, &set);
}

#[test]
fn two_disjoint_rectangles_are_two_regions() {
    let profile = two_rectangles();
    let set = built(&profile);
    assert_eq!(set.regions().len(), 2);
    assert_area(&set.regions()[0], 1.0);
    assert_area(&set.regions()[1], 2.0);
    assert_complete_mapping(&profile, &set);
    assert_eq!(set.certificate().loops, 2);
    // Eight line pieces: all distinct pairs plus every self domain.
    assert_eq!(set.certificate().pair_domains, 8 * 9 / 2);
}

#[test]
fn square_with_circular_hole_mixes_curve_kinds() {
    let profile = SketchFixture::new()
        .rectangle(0, [-2., -2.], [2., 2.])
        .circle(50, [0.5, 0.], 1.)
        .build();
    let set = built(&profile);
    assert_eq!(set.regions().len(), 2);
    assert_area(&set.regions()[0], 16.0 - PI);
    assert_area(&set.regions()[1], PI);
}

#[test]
fn building_twice_is_identical_and_leaves_the_input_unchanged() {
    let profile = annulus();
    let copy = profile.clone();
    let (first, second) = (built(&profile), built(&profile));
    assert_eq!(profile, copy);
    for (a, b) in first.regions().iter().zip(second.regions()) {
        assert_eq!(
            (a.id(), a.outer(), a.holes()),
            (b.id(), b.outer(), b.holes())
        );
        assert_eq!(a.area(), b.area());
    }
}

#[test]
fn circular_segment_with_a_general_arc_has_the_closed_form_area() {
    let root3 = 3f64.sqrt();
    let profile = SketchFixture::new()
        .vertex(0, [root3, 1.])
        .vertex(1, [-root3, 1.])
        .arc(1, [0, 1], [0., 0.], ArcSense::CounterClockwise)
        .line(2, 1, 0)
        .build();
    let set = built(&profile);
    let theta = 2.0 * PI / 3.0;
    let region = &set.regions()[0];
    assert_area(region, 2.0 * (theta - theta.sin()));
    assert_eq!(
        region.area().method,
        IntegrationMethod::RepresentationEnclosure
    );
}

#[test]
fn line_only_regions_carry_a_certified_area_bound() {
    let set = built(&rectangle());
    assert_eq!(
        set.regions()[0].area().method,
        IntegrationMethod::CertifiedEnclosure
    );
}
