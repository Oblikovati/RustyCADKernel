// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0068 corpus: circles, annuli, polygons with holes, side-by-side and nested
//! loops through one general pipeline and one region proof.
use super::*;
use std::f64::consts::PI;

fn region(certificate: &LoopRegionCertificate, name: u64) -> &RegionProof {
    certificate
        .regions
        .iter()
        .find(|r| r.name == name)
        .expect("selected region")
}
fn parent(certificate: &LoopRegionCertificate, name: u64) -> Option<u64> {
    certificate
        .loops
        .iter()
        .find(|l| l.name == name)
        .expect("loop")
        .parent
}

#[test]
fn single_rational_circle_is_one_disc() {
    let mut profile = Profile::default();
    profile.circle(10, [0., 0.], 1., false);
    profile.select(1, [0.25, -0.5]);
    let certificate = certified(profile);
    // Four quarters: ten domains, four tangent joins, three carrier distinct pairs.
    assert_eq!((certificate.pair_domains, certificate.joins.len()), (10, 4));
    assert_eq!(certificate.carrier_pairs, 6);
    let disc = region(&certificate, 1);
    assert_eq!((disc.outer, disc.holes.clone()), (1010, vec![]));
    assert_eq!(disc.windings, vec![(1010, 1)]);
    assert_area(disc, PI);
}
#[test]
fn clockwise_circle_has_negative_winding_and_same_face() {
    let mut profile = Profile::default();
    profile.circle(10, [3., -2.], 0.5, true);
    profile.select(1, [3., -2.]);
    let certificate = certified(profile);
    assert_eq!(region(&certificate, 1).windings, vec![(1010, -1)]);
    assert_area(region(&certificate, 1), 0.25 * PI);
}
#[test]
fn circle_in_circle_annulus_and_inner_disc() {
    let mut profile = Profile::default();
    profile.circle(10, [0., 0.], 2., false);
    profile.circle(20, [0.25, 0.], 0.5, false);
    profile.select(1, [-1.5, 0.]);
    profile.select(2, [0.25, 0.]);
    let certificate = certified(profile);
    assert_eq!(parent(&certificate, 1020), Some(1010));
    assert_eq!(parent(&certificate, 1010), None);
    let annulus = region(&certificate, 1);
    assert_eq!((annulus.outer, annulus.holes.clone()), (1010, vec![1020]));
    assert_eq!(annulus.windings, vec![(1010, 1), (1020, 0)]);
    assert_area(annulus, 3.75 * PI);
    let disc = region(&certificate, 2);
    assert_eq!((disc.outer, disc.holes.clone()), (1020, vec![]));
    assert_area(disc, 0.25 * PI);
    // Arcs of different circles are disjoint by the generic route.
    assert_eq!(certificate.carrier_pairs, 12);
    assert_eq!(certificate.pair_domains, 36);
}
#[test]
fn square_with_circular_hole() {
    let mut profile = Profile::default();
    profile.polygon(10, &[[-2., -2.], [2., -2.], [2., 2.], [-2., 2.]]);
    profile.circle(20, [0., 0.], 1., true);
    profile.select(1, [1.5, 1.5]);
    let certificate = certified(profile);
    let face = region(&certificate, 1);
    assert_eq!((face.outer, face.holes.clone()), (1010, vec![1020]));
    assert_area(face, 16. - PI);
}
#[test]
fn side_by_side_circles_are_separate_faces() {
    let mut profile = Profile::default();
    profile.circle(10, [-2., 0.], 1., false);
    profile.circle(20, [2., 0.], 1., false);
    profile.select(1, [-2., 0.5]);
    profile.select(2, [2., -0.5]);
    let certificate = certified(profile);
    assert_eq!(
        (parent(&certificate, 1010), parent(&certificate, 1020)),
        (None, None)
    );
    assert_eq!(region(&certificate, 1).outer, 1010);
    assert_eq!(region(&certificate, 2).outer, 1020);
    assert!(region(&certificate, 2).holes.is_empty());
}
#[test]
fn three_nested_loops_select_disc_and_outer_annulus() {
    let mut profile = Profile::default();
    profile.circle(10, [0., 0.], 4., false);
    profile.polygon(20, &[[-2., -2.], [2., -2.], [2., 2.], [-2., 2.]]);
    profile.circle(30, [0., 0.], 1., false);
    profile.select(1, [3., 0.]);
    profile.select(2, [0., 0.5]);
    profile.select(3, [1.5, 1.5]);
    let certificate = certified(profile);
    assert_eq!(parent(&certificate, 1030), Some(1020));
    assert_eq!(parent(&certificate, 1020), Some(1010));
    assert_eq!(region(&certificate, 1).holes, vec![1020]);
    assert_eq!(region(&certificate, 2).outer, 1030);
    let middle = region(&certificate, 3);
    assert_eq!((middle.outer, middle.holes.clone()), (1020, vec![1030]));
    assert_area(middle, 16. - PI);
    assert_area(region(&certificate, 1), 16. * PI - 16.);
}
#[test]
fn replay_is_identical_and_inputs_are_untouched() {
    let build = || {
        let mut profile = Profile::default();
        profile.circle(10, [0., 0.], 2., false);
        profile.circle(20, [0.25, 0.], 0.5, false);
        profile.select(1, [-1.5, 0.]);
        profile
    };
    let request = build().request();
    let edges = request.edges.clone();
    let prepared = prepare_owned_profile(request, &mut Continue).ok().unwrap();
    assert_eq!(prepared.request.edges, edges);
    let first = prepared.inspect_regions(|| false).unwrap();
    assert_eq!(prepared.inspect_regions(|| false).unwrap(), first);
    assert_eq!(certified(build()), first);
}
#[test]
fn slot_of_lines_and_tangent_arcs_is_one_face() {
    let mut profile = Profile::default();
    let quarter = Some(&[1., 1., 2.][..]);
    profile.edge(1, &[[-1., -1.], [1., -1.]], None, [6, 1]);
    profile.edge(2, &[[1., -1.], [2., -1.], [2., 0.]], quarter, [1, 2]);
    profile.edge(3, &[[2., 0.], [2., 1.], [1., 1.]], quarter, [2, 3]);
    profile.edge(4, &[[1., 1.], [-1., 1.]], None, [3, 4]);
    profile.edge(5, &[[-1., 1.], [-2., 1.], [-2., 0.]], quarter, [4, 5]);
    profile.edge(6, &[[-2., 0.], [-2., -1.], [-1., -1.]], quarter, [5, 6]);
    profile.chain(100, &[1, 2, 3, 4, 5, 6]);
    profile.select(1, [1.75, 0.]);
    let certificate = certified(profile);
    assert_eq!(certificate.joins.len(), 6);
    assert_eq!(region(&certificate, 1).windings, vec![(100, 1)]);
    assert_area(region(&certificate, 1), 4. + PI);
}
#[test]
fn closed_single_edge_cubic_uses_its_closure_contact() {
    let mut profile = Profile::default();
    profile.edge(1, &[[0., 0.], [4., 0.], [0., 4.], [0., 0.]], None, [7, 7]);
    profile.chain(100, &[1]);
    profile.select(1, [1., 1.]);
    let certificate = certified(profile);
    let join = JoinProof {
        sources: [crate::sketch_region::RegionSourceId(1); 2],
        parameter: [0, 1],
    };
    assert_eq!(certificate.joins, vec![join]);
    assert_eq!(certificate.carrier_pairs, 0);
    assert_area(region(&certificate, 1), 2.4);
}
#[test]
fn two_edge_lens_joins_both_ends_of_one_pair() {
    // Two quarter arcs of different circles bounding a lens through (1,0) and (0,1).
    let mut profile = Profile::default();
    profile.edge(
        1,
        &[[1., 0.], [1., 1.], [0., 1.]],
        Some(&[1., 1., 2.]),
        [1, 2],
    );
    profile.edge(
        2,
        &[[0., 1.], [0., 0.], [1., 0.]],
        Some(&[1., 1., 2.]),
        [2, 1],
    );
    profile.chain(100, &[1, 2]);
    profile.select(1, [0.5, 0.5]);
    let certificate = certified(profile);
    assert_eq!(certificate.joins.len(), 2);
    assert_eq!(region(&certificate, 1).outer, 100);
}
