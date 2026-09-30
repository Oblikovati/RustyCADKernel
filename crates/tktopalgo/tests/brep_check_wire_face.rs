//! Behavior tests of BRepCheck_Wire and BRepCheck_Face beyond the upstream suite
//! `BRepCheck_Face_Test.cxx` (translated 1:1 in `brep_check_face.rs`), on faces built
//! with BRep_Builder.
mod brep_check_support;

use brep_check_support::{Planar, TOL, square, wire_of};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Vector3;
use rustycad_tktopalgo::brep_check::{
    BRepCheckFace, BRepCheckResult, BRepCheckStatus, BRepCheckWire,
};

use BRepCheckStatus::*;

#[test]
fn face_wire_checks_report_holes_that_intersect_escape_or_face_the_wrong_way() {
    let cases = [
        (vec![(1.0, 1.0, 1.0)], NoError, NoError),
        (vec![(3.0, 1.0, 2.0)], IntersectingWires, IntersectingWires),
        // No wire holds the other: classified, but the face has no outer wire.
        (vec![(5.0, 5.0, 1.0)], NoError, InvalidImbricationOfWires),
        // The outer wire and the first hole both hold a wire.
        (
            vec![(1.0, 1.0, 2.0), (1.5, 1.5, 0.5)],
            InvalidImbricationOfWires,
            InvalidImbricationOfWires,
        ),
    ];
    for (holes, classified, oriented) in cases {
        let sq = square(4.0, &holes, &[], 0.0);
        let checker = BRepCheckFace::new(&sq.faces[0]).unwrap();
        assert_eq!(checker.classify_wires(true).unwrap(), classified);
        assert_eq!(checker.orientation_of_wires(true).unwrap(), oriented);
        assert_eq!(checker.is_unorientable().unwrap(), oriented != NoError);
    }
    // A hole wound counter-clockwise is inside the outer wire but not reversed.
    let mut planar = Planar::new(&[
        (0.0, 0.0, 0.0),
        (4.0, 0.0, 0.0),
        (4.0, 4.0, 0.0),
        (0.0, 4.0, 0.0),
        (1.0, 1.0, 0.0),
        (2.0, 1.0, 0.0),
        (2.0, 2.0, 0.0),
        (1.0, 2.0, 0.0),
    ]);
    planar.face(
        &[&[0, 1, 2, 3], &[4, 5, 6, 7]],
        Some(Vector3::new(0.0, 0.0, 1.0)),
        &[],
        0.0,
    );
    planar.finish();
    let checker = BRepCheckFace::new(&planar.faces[0]).unwrap();
    assert_eq!(
        checker.orientation_of_wires(false).unwrap(),
        BadOrientationOfSubshape
    );
}

#[test]
fn face_without_surface_and_a_wire_met_twice() {
    let b = BRepBuilder::default();
    let mut bare = Face::default();
    b.make_face(&mut bare);
    let checker = BRepCheckFace::new(&bare).unwrap();
    assert_eq!(checker.status().unwrap().first(), Some(&NoSurface));
    let sq = square(1.0, &[], &[], 0.0);
    let mut twice = Face::default();
    b.make_face_on_surface(&mut twice, &sq.surfaces[0], TOL)
        .unwrap();
    b.add(&mut twice, &sq.wires[0][0]).unwrap();
    b.add(&mut twice, &sq.wires[0][0].reversed()).unwrap();
    let checker = BRepCheckFace::new(&twice).unwrap();
    assert_eq!(checker.intersect_wires(true).unwrap(), RedundantWire);
    // The wire map is not reset: a second computation finds every wire again.
    let fresh = BRepCheckFace::new(&sq.faces[0]).unwrap();
    assert_eq!(fresh.intersect_wires(false).unwrap(), NoError);
    fresh.set_geometric_controls(false);
    fresh.set_geometric_controls(true);
    assert_eq!(fresh.intersect_wires(false).unwrap(), RedundantWire);
}

#[test]
fn wire_checks_on_a_square_and_its_defects() {
    let sq = square(1.0, &[], &[], 0.0);
    let face = &sq.faces[0];
    let checker = BRepCheckWire::new(&sq.wires[0][0]).unwrap();
    assert!(checker.geometric_controls());
    checker.in_context(face).unwrap();
    assert_eq!(
        checker.status_on_shape_of(face).unwrap().first(),
        Some(&NoError)
    );
    let e = &sq.edges;
    let open = wire_of(&[
        (&e[&(0, 1)], Orientation::Forward),
        (&e[&(1, 2)], Orientation::Forward),
    ]);
    let checker = BRepCheckWire::new(&open).unwrap();
    assert_eq!(checker.closed(false).unwrap(), NotClosed);
    assert_eq!(checker.orientation(face, false).unwrap(), NoError);
    assert_eq!(checker.closed2d(face, false).unwrap(), NotClosed);
    let apart = wire_of(&[
        (&e[&(0, 1)], Orientation::Forward),
        (&e[&(2, 3)], Orientation::Forward),
    ]);
    assert_eq!(
        BRepCheckWire::new(&apart)
            .unwrap()
            .status()
            .unwrap()
            .first(),
        Some(&NotConnected)
    );
    let redundant = wire_of(&[
        (&e[&(0, 1)], Orientation::Forward),
        (&e[&(0, 1)], Orientation::Forward),
    ]);
    assert_eq!(
        BRepCheckWire::new(&redundant)
            .unwrap()
            .closed(true)
            .unwrap(),
        RedundantEdge
    );
    let flipped = wire_of(&[
        (&e[&(0, 1)], Orientation::Forward),
        (&e[&(1, 2)], Orientation::Reversed),
        (&e[&(2, 3)], Orientation::Reversed),
        (&e[&(0, 3)], Orientation::Reversed),
    ]);
    let checker = BRepCheckWire::new(&flipped).unwrap();
    assert_eq!(
        checker.orientation(face, true).unwrap(),
        BadOrientationOfSubshape
    );
    let empty = wire_of(&[]);
    let checker = BRepCheckWire::new(&empty).unwrap();
    let (mut e1, mut e2) = (Edge::default(), Edge::default());
    assert_eq!(
        checker
            .self_intersect(face, &mut e1, &mut e2, true)
            .unwrap(),
        EmptyWire
    );
    checker.set_status(InvalidWire).unwrap();
    assert_eq!(
        checker
            .status()
            .unwrap()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [EmptyWire, InvalidWire]
    );
}

#[test]
fn wire_self_intersect_keeps_the_second_edge_of_a_single_crossing() {
    let mut bow = Planar::new(&[
        (0.0, 0.0, 0.0),
        (1.0, 1.0, 0.0),
        (1.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
    ]);
    bow.face(
        &[&[0, 1, 2, 3]],
        Some(Vector3::new(0.0, 0.0, 1.0)),
        &[],
        0.0,
    );
    bow.finish();
    let checker = BRepCheckWire::new(&bow.wires[0][0]).unwrap();
    let marker = bow.edges[&(1, 2)].clone();
    let (mut e1, mut e2) = (marker.clone(), marker);
    let status = checker
        .self_intersect(&bow.faces[0], &mut e1, &mut e2, true)
        .unwrap();
    assert_eq!(status, SelfIntersectingWire);
    assert!(e1.is_same(&bow.edges[&(0, 1)]) && e2.is_same(&bow.edges[&(2, 3)]));
    checker.in_context(&bow.faces[0]).unwrap();
    let statuses = checker.status_on_shape_of(&bow.faces[0]).unwrap();
    assert_eq!(statuses.first(), Some(&SelfIntersectingWire));
    checker.set_geometric_controls(false);
    assert!(!checker.geometric_controls());
}
