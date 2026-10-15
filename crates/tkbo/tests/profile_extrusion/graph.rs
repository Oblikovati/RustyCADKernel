// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0084 through the public seam: regions sharing an edge, loops touching at a vertex,
//! bridges, and the named refusals (pinched regions, tangent branches).
use super::extrusion::{extruded, options};
use super::fixtures::*;
use rustycad_tkbo::profile_extrusion::*;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::f64::consts::PI;

fn built(profile: &SketchProfile) -> PlanarRegionSet {
    build_planar_regions(profile, resolution()).unwrap()
}

/// Closed form, and the native face integrated adaptively (no tessellation).
fn assert_area(region: &PlanarRegion, expected: f64) {
    assert!(
        region.area().encloses(expected),
        "{:?} vs {expected}",
        region.area()
    );
    let mut props = GProps::new();
    BRepGProp::surface_properties_eps(region.face(), &mut props, 1e-10, false).unwrap();
    assert!(
        (props.mass() - expected).abs() < 1e-8,
        "{} vs {expected}",
        props.mass()
    );
}

/// A 3 x 2 rectangle split by the line x = 1 (curve 20 from vertex 1 up to vertex 4):
/// a 1 x 2 left region and a 2 x 2 right region sharing curve 20.
fn split_rectangle() -> SketchProfile {
    let corners = [[0., 0.], [1., 0.], [3., 0.], [3., 2.], [1., 2.], [0., 2.]];
    let mut fixture = SketchFixture::new();
    for (id, corner) in corners.iter().enumerate() {
        fixture.vertex(id as u64, *corner);
    }
    for id in 0..6 {
        fixture.line(10 + id, id, (id + 1) % 6);
    }
    fixture.line(20, 1, 4).build()
}

fn region_with_area(set: &PlanarRegionSet, area: f64) -> &PlanarRegion {
    let found = set
        .regions()
        .iter()
        .find(|r| (r.area().value - area).abs() < 1e-9);
    found.unwrap_or_else(|| panic!("no region of area {area}"))
}

#[test]
fn rectangle_split_by_a_line_is_two_regions_sharing_the_edge() {
    let profile = split_rectangle();
    let set = built(&profile);
    assert_eq!(set.regions().len(), 2);
    assert_eq!(
        (set.certificate().loops, set.certificate().components),
        (2, 1)
    );
    let (left, right) = (region_with_area(&set, 2.0), region_with_area(&set, 4.0));
    assert_area(left, 2.0);
    assert_area(right, 4.0);
    let chord = set
        .curve_edges()
        .iter()
        .find(|e| e.curve == ProfileCurveId(20))
        .unwrap();
    assert_eq!(chord.regions, vec![RegionId(0), RegionId(1)]);
    let uses = |region: &PlanarRegion| {
        let used = region
            .outer()
            .uses
            .iter()
            .find(|u| u.curve == ProfileCurveId(20));
        used.expect("both regions use the chord").reversed
    };
    assert_ne!(uses(left), uses(right), "opposite directions");
    for region in set.regions() {
        let (_, edge) = region
            .edges()
            .iter()
            .find(|(c, _)| *c == ProfileCurveId(20))
            .unwrap();
        assert!(edge.is_same(&chord.edge), "one shared native edge");
        assert_eq!(region.outer().uses.len(), 4);
        assert!(region.validation().tolerance == StageStatus::Passed);
    }
    for entry in set
        .curve_edges()
        .iter()
        .filter(|e| e.curve != ProfileCurveId(20))
    {
        assert_eq!(entry.regions.len(), 1, "{:?}", entry.curve);
    }
}

#[test]
fn each_region_of_the_split_rectangle_extrudes() {
    let set = built(&split_rectangle());
    for (index, expected) in set.regions().iter().map(|r| r.area().value).enumerate() {
        let solid = extruded(&split_rectangle(), index, [0., 0., 1.5]);
        assert!(
            solid.volume().encloses(1.5 * expected),
            "{:?}",
            solid.volume()
        );
        assert_eq!(solid.faces().len(), 6);
        assert!(solid.validation().is_valid());
        let side = solid.history().of(HistorySource::Curve(ProfileCurveId(20)));
        assert!(side.iter().any(|r| r.role == HistoryRole::SideFace));
    }
}

/// Two triangles touching at vertex 0 (a figure eight); each has area 1.
fn figure_eight() -> SketchProfile {
    SketchFixture::new()
        .vertex(0, [0., 0.])
        .vertex(1, [-1., 1.])
        .vertex(2, [-1., -1.])
        .vertex(3, [1., -1.])
        .vertex(4, [1., 1.])
        .line(10, 0, 1)
        .line(11, 1, 2)
        .line(12, 2, 0)
        .line(13, 0, 3)
        .line(14, 3, 4)
        .line(15, 4, 0)
        .build()
}

#[test]
fn figure_eight_touching_loops_are_admitted_as_two_regions() {
    let set = built(&figure_eight());
    assert_eq!(set.regions().len(), 2);
    for region in set.regions() {
        assert_area(region, 1.0);
        assert!(
            region
                .vertices()
                .iter()
                .any(|(v, _)| *v == ProfileVertexId(0))
        );
    }
    for index in 0..2 {
        let solid = extruded(&figure_eight(), index, [0., 0., 2.]);
        assert!(solid.volume().encloses(2.0));
        assert!(solid.validation().is_valid());
    }
}

#[test]
fn loop_touching_its_enclosing_loop_from_inside_is_a_named_pinched_region() {
    let mut fixture = SketchFixture::new();
    fixture.rectangle(0, [0., 0.], [4., 4.]);
    fixture.vertex(5, [2., 1.]).vertex(6, [1., 2.]);
    fixture.line(30, 0, 5).line(31, 5, 6).line(32, 6, 0);
    let decline = build_planar_regions(&fixture.build(), resolution()).unwrap_err();
    assert_eq!(decline.stage, OperationStage::Certification);
    assert_eq!(decline.class(), DeclineClass::Unsupported);
    let pinched = DeclineCause::PinchedRegion {
        vertex: ProfileVertexId(0),
        hole: false,
    };
    assert_eq!(decline.cause, pinched);
}

#[test]
fn branches_leaving_a_vertex_tangentially_are_named() {
    // A line and a tangent half circle both leave vertex 0 straight up.
    let profile = SketchFixture::new()
        .vertex(0, [0., 0.])
        .vertex(1, [2., 0.])
        .vertex(2, [0., 2.])
        .vertex(3, [2., 2.])
        .line(10, 0, 1)
        .arc(11, [1, 0], [1., 0.], ArcSense::CounterClockwise)
        .line(12, 0, 2)
        .line(13, 2, 3)
        .line(14, 3, 1)
        .build();
    let decline = build_planar_regions(&profile, resolution()).unwrap_err();
    assert_eq!(
        decline.cause,
        DeclineCause::TangentBranches {
            vertex: ProfileVertexId(0)
        }
    );
}

#[test]
fn bridge_between_two_squares_is_named() {
    let mut fixture = SketchFixture::new();
    fixture
        .rectangle(0, [0., 0.], [1., 1.])
        .rectangle(100, [3., 0.], [4., 1.]);
    fixture.line(50, 1, 100);
    let decline = build_planar_regions(&fixture.build(), resolution()).unwrap_err();
    assert_eq!(
        (decline.stage, decline.class()),
        (OperationStage::Loops, DeclineClass::Unsupported)
    );
    assert_eq!(
        decline.cause,
        DeclineCause::BridgeCurve {
            curve: ProfileCurveId(50)
        }
    );
}

#[test]
fn circle_in_one_half_of_the_split_rectangle_is_a_hole_of_that_half() {
    let mut profile = split_rectangle();
    profile.curves.push(ProfileCurve::Circle {
        id: ProfileCurveId(40),
        center: [2., 1.],
        radius: 0.5,
    });
    let set = built(&profile);
    assert_eq!(set.regions().len(), 3);
    let right = region_with_area(&set, 4.0 - PI / 4.0);
    assert_eq!(right.holes().len(), 1);
    assert_eq!(right.holes()[0].uses[0].curve, ProfileCurveId(40));
    assert!(right.holes()[0].uses[0].reversed, "holes run clockwise");
    assert_area(right, 4.0 - PI / 4.0);
    let solid = extruded(&profile, right.id().0 as usize, [0., 0., 1.]);
    assert!(solid.volume().encloses(4.0 - PI / 4.0));
    assert!(solid.validation().is_valid());
}

#[test]
fn shared_edge_regions_are_deterministic_across_threads() {
    let local = built(&split_rectangle());
    let remote = std::thread::spawn(|| built(&split_rectangle()))
        .join()
        .unwrap();
    for (a, b) in local.regions().iter().zip(remote.regions()) {
        assert_eq!(
            (a.id(), a.outer(), a.holes(), a.area()),
            (b.id(), b.outer(), b.holes(), b.area())
        );
    }
    assert_eq!(local.certificate(), remote.certificate());
    assert_eq!(options().resolution, resolution());
}
