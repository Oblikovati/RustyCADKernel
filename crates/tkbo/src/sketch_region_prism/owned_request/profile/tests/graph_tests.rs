// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0084 plane graph proof: shared edges, touching loops, nested components and the
//! named refusals (pinched faces, tangent branches).
use super::*;
use crate::sketch_region_prism::region_certificate::loop_region::graph::*;
use std::f64::consts::PI;

fn graph_request(profile: Profile) -> OwnedProfileRequest {
    let mut request = profile.request();
    request.version = super::super::admission::GRAPH_PROFILE_VERSION;
    request.loops.clear();
    request.regions.clear();
    request
}

fn graph(profile: Profile) -> LoopRegionCheck<GraphRegionCertificate> {
    let prepared = prepare_owned_profile(graph_request(profile), &mut Continue)
        .unwrap_or_else(|failure| panic!("preparation {:?} at {:?}", failure.class, failure.phase));
    prepared.inspect_plane_graph(|| false)
}

fn line(profile: &mut Profile, id: u64, points: [[f64; 2]; 2], ends: [u64; 2]) {
    profile.edge(id, &points, None, ends);
}

fn assert_face_area(face: &GraphFaceProof, expected: f64) {
    let area = &face.area;
    let allowance = area.absolute_error_bound() + 4. * f64::EPSILON * expected.abs();
    assert!(
        (area.representative() - expected).abs() <= allowance,
        "{area:?} vs {expected}"
    );
}

/// A 2 x 1 rectangle split by the chord x = 1: vertices 1..=6 counterclockwise from the
/// origin, chord 7 from (1, 0) to (1, 1).
fn split_rectangle() -> Profile {
    let points = [[0., 0.], [1., 0.], [2., 0.], [2., 1.], [1., 1.], [0., 1.]];
    let mut profile = Profile::default();
    for i in 0..6 {
        let ends = [i as u64 + 1, (i as u64 + 1) % 6 + 1];
        line(
            &mut profile,
            i as u64 + 1,
            [points[i], points[(i + 1) % 6]],
            ends,
        );
    }
    line(&mut profile, 7, [points[1], points[4]], [2, 5]);
    profile
}

#[test]
fn split_rectangle_has_two_faces_sharing_the_chord() {
    let certificate = graph(split_rectangle()).unwrap();
    assert_eq!(certificate.faces.len(), 2);
    assert_eq!(certificate.components.len(), 1);
    // Four degree-2 vertices give one contact each, two degree-3 vertices three each.
    assert_eq!(certificate.contacts, 10);
    for face in &certificate.faces {
        assert_eq!(face.uses.len(), 4);
        assert!(face.holes.is_empty());
        assert_face_area(face, 1.);
    }
    let chord = |face: &GraphFaceProof| {
        let found = face.uses.iter().find(|u| u.source == RegionSourceId(7));
        found.expect("chord bounds both faces").reversed
    };
    assert_ne!(chord(&certificate.faces[0]), chord(&certificate.faces[1]));
    assert_eq!(certificate.components[0].outer.len(), 6);
    assert_eq!(certificate.components[0].container, None);
}

#[test]
fn figure_eight_lobes_touching_at_a_vertex_are_two_faces() {
    let mut profile = Profile::default();
    let lobes = [
        [[0., 0.], [-1., 1.], [-1., -1.]],
        [[0., 0.], [1., -1.], [1., 1.]],
    ];
    let ids = [[1, 2, 3], [1, 4, 5]];
    for (lobe, (points, vertices)) in lobes.iter().zip(ids).enumerate() {
        for k in 0..3 {
            let id = 10 * lobe as u64 + k as u64 + 1;
            let ends = [vertices[k], vertices[(k + 1) % 3]];
            line(&mut profile, id, [points[k], points[(k + 1) % 3]], ends);
        }
    }
    let certificate = graph(profile).unwrap();
    assert_eq!(certificate.faces.len(), 2);
    certificate
        .faces
        .iter()
        .for_each(|face| assert_face_area(face, 1.));
    // The outer walk passes the touching vertex twice; it bounds no face, so it is admitted.
    assert_eq!(certificate.components[0].outer.len(), 6);
}

#[test]
fn circle_in_the_left_half_is_a_hole_of_that_face_only() {
    let mut profile = split_rectangle();
    profile.circle(20, [0.5, 0.5], 0.25, false);
    profile.loops.clear();
    let certificate = graph(profile).unwrap();
    assert_eq!(certificate.faces.len(), 3);
    assert_eq!(certificate.components.len(), 2);
    let container = certificate.components[1].container.expect("nested circle");
    let left = &certificate.faces[container];
    assert_eq!(left.holes, vec![1]);
    assert_face_area(left, 1. - PI / 16.);
    let disc = certificate.faces.iter().find(|f| f.component == 1).unwrap();
    assert_face_area(disc, PI / 16.);
}

#[test]
fn loop_touching_its_enclosing_loop_from_inside_is_a_pinched_face() {
    let mut profile = Profile::default();
    let square = [[0., 0.], [4., 0.], [4., 4.], [0., 4.]];
    for i in 0..4 {
        let ends = [i as u64 + 1, (i as u64 + 1) % 4 + 1];
        line(
            &mut profile,
            i as u64 + 1,
            [square[i], square[(i + 1) % 4]],
            ends,
        );
    }
    line(&mut profile, 5, [[0., 0.], [2., 1.]], [1, 5]);
    line(&mut profile, 6, [[2., 1.], [1., 2.]], [5, 6]);
    line(&mut profile, 7, [[1., 2.], [0., 0.]], [6, 1]);
    let failure = graph(profile).unwrap_err();
    assert_eq!(failure.cause, LoopRegionCause::PinchedFace { vertex: 1 });
}

#[test]
fn branches_leaving_a_vertex_in_one_direction_are_refused() {
    let mut profile = Profile::default();
    let (a, b, c, d, m) = ([0., 0.], [2., 0.], [0., 2.], [2., 2.], [1., 1.]);
    line(&mut profile, 1, [a, b], [1, 2]);
    // Exact quarters of the unit circle about (1, 0): M to A and B to M, both vertical at
    // their ends on the x axis, tangent to the sides x = 0 and x = 2.
    profile.edge(2, &[m, [0., 1.], a], Some(&[1., 1., 2.]), [5, 1]);
    profile.edge(3, &[b, [2., 1.], m], Some(&[1., 1., 2.]), [2, 5]);
    line(&mut profile, 4, [a, c], [1, 3]);
    line(&mut profile, 5, [c, d], [3, 4]);
    line(&mut profile, 6, [d, b], [4, 2]);
    let failure = graph(profile).unwrap_err();
    assert_eq!(
        failure.cause,
        LoopRegionCause::TangentBranches { vertex: 1 }
    );
}

#[test]
fn replay_is_identical() {
    assert_eq!(graph(split_rectangle()), graph(split_rectangle()));
}

#[test]
fn touching_lobes_inside_a_square_are_a_pinched_hole() {
    let mut profile = Profile::default();
    profile.polygon(100, &[[-4., -4.], [4., -4.], [4., 4.], [-4., 4.]]);
    let lobes = [
        [[0., 0.], [-1., 1.], [-1., -1.]],
        [[0., 0.], [1., -1.], [1., 1.]],
    ];
    let ids = [[1, 2, 3], [1, 4, 5]];
    for (lobe, (points, vertices)) in lobes.iter().zip(ids).enumerate() {
        for k in 0..3 {
            let id = 10 * lobe as u64 + k as u64 + 1;
            let ends = [vertices[k], vertices[(k + 1) % 3]];
            line(&mut profile, id, [points[k], points[(k + 1) % 3]], ends);
        }
    }
    let failure = graph(profile).unwrap_err();
    assert_eq!(failure.cause, LoopRegionCause::PinchedHole { vertex: 1 });
}

#[test]
fn cancellation_is_observed_at_every_stage() {
    let prepared = prepare_owned_profile(graph_request(split_rectangle()), &mut Continue)
        .unwrap_or_else(|failure| panic!("preparation {:?}", failure.phase));
    for stop in 0..6 {
        let mut polls = 0;
        let result = prepared.inspect_plane_graph(|| {
            polls += 1;
            polls > stop
        });
        assert_eq!(
            result.unwrap_err().cause,
            LoopRegionCause::Cancelled,
            "stop {stop}"
        );
    }
}
