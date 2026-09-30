//! Behavior tests of BRepCheck_Wire and BRepCheck_Face, with the cases of the upstream
//! suite `BRepCheck_Face_Test.cxx`. That suite is not mapped: its faces come from
//! BRepBuilderAPI_MakeEdge, MakeWire and MakeFace, which the port does not provide yet;
//! here the same faces are built with BRep_Builder and each case checks the same
//! statuses (its IsValid assertion waits for BRepCheck_Analyzer).
mod brep_check_support;

use brep_check_support::{Planar, TOL, square, vertex, wire_of};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Face, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dOffsetCurve};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis2, Axis2d, Direction2, Direction3, Point2, Point3, Vector3};
use rustycad_tktopalgo::brep_check::{
    BRepCheckFace, BRepCheckResult, BRepCheckStatus, BRepCheckWire,
};
use std::f64::consts::TAU;

use BRepCheckStatus::*;

fn z() -> Direction3 {
    Direction3::new(0.0, 0.0, 1.0).unwrap()
}

/// A circular wire of `radius` around `centre` on the plane z = 0 of `surface`, reversed
/// to bound a hole; with `offset`, its pcurve is the offset of a smaller circle.
fn circular_hole(
    face: &mut Face,
    surface: &Handle<dyn GeomSurface>,
    centre: (f64, f64),
    radius: f64,
    offset: Option<f64>,
) {
    let b = BRepBuilder::default();
    let v = vertex(Point3::new(centre.0 + radius, centre.1, 0.0), TOL);
    let mut e = Edge::default();
    b.make_edge(&mut e).unwrap();
    let frame = Axis2::new(Point3::new(centre.0, centre.1, 0.0), z()).unwrap();
    let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(frame, radius).unwrap()).cast();
    b.update_edge_curve(&e, &circle, TOL).unwrap();
    let axis = Axis2d::new(
        Point2::new(centre.0, centre.1),
        Direction2::new(1.0, 0.0).unwrap(),
    );
    let base = radius - offset.unwrap_or(0.0);
    let pc: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::from_axis(axis, base, true).unwrap()).cast();
    let pc = match offset {
        Some(d) => Handle::new(Geom2dOffsetCurve::new(&pc, d, false).unwrap()).cast(),
        None => pc,
    };
    b.update_edge_pcurve(&e, &pc, surface, &Default::default(), TOL, None)
        .unwrap();
    b.range(&e, 0.0, TAU, false).unwrap();
    b.add(&mut e, &v.oriented(Orientation::Forward)).unwrap();
    b.add(&mut e, &v.oriented(Orientation::Reversed)).unwrap();
    let hole: Wire = wire_of(&[(&e, Orientation::Forward)]);
    b.add(face, &hole.reversed()).unwrap();
}

fn classify(face: &Face) -> BRepCheckStatus {
    BRepCheckFace::new(face)
        .unwrap()
        .classify_wires(false)
        .unwrap()
}

/// A line edge outside the face, finite (with vertices) or unbounded, in its own wire.
fn add_outside_edge(face: &mut Face, unbounded: bool) {
    let b = BRepBuilder::default();
    let mut e = Edge::default();
    b.make_edge(&mut e).unwrap();
    let x = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let start = if unbounded {
        Point3::new(0.0, 120.0, 0.0)
    } else {
        Point3::new(120.0, 10.0, 0.0)
    };
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(start, x)).cast();
    b.update_edge_curve(&e, &line, TOL).unwrap();
    if unbounded {
        b.range(&e, -2e100, 2e100, false).unwrap();
    } else {
        b.range(&e, 0.0, 10.0, false).unwrap();
        b.add(&mut e, &vertex(start, TOL).oriented(Orientation::Forward))
            .unwrap();
        b.add(
            &mut e,
            &vertex(Point3::new(130.0, 10.0, 0.0), TOL).oriented(Orientation::Reversed),
        )
        .unwrap();
    }
    b.add(face, &wire_of(&[(&e, Orientation::Forward)]))
        .unwrap();
}

#[test]
fn face_test_classify_wires_disjoint_holes_are_valid() {
    let sq = square(100.0, &[], &[], 0.0);
    let mut face = sq.faces[0].clone();
    for row in 0..5 {
        for column in 0..8 {
            let centre = (
                10.0 + 11.0 * f64::from(column),
                10.0 + 11.0 * f64::from(row),
            );
            circular_hole(&mut face, &sq.surfaces[0], centre, 2.0, None);
        }
    }
    assert_eq!(classify(&face), NoError);
}

#[test]
fn face_test_classify_wires_offset_pcurve() {
    let sq = square(100.0, &[], &[], 0.0);
    let mut face = sq.faces[0].clone();
    circular_hole(&mut face, &sq.surfaces[0], (50.0, 50.0), 5.0, Some(1.0));
    assert_eq!(classify(&face), NoError);
}

#[test]
fn face_test_classify_wires_open_wire_fallback_is_orientation_independent() {
    let sq = square(100.0, &[], &[], 0.0);
    let mut face = sq.faces[0].clone();
    add_outside_edge(&mut face, false);
    let forward = classify(&face);
    assert_eq!(forward, NoError);
    assert_eq!(classify(&Face::unchecked(face.reversed())), forward);
}

#[test]
fn face_test_classify_wires_unbounded_wire_fallback_is_orientation_independent() {
    let sq = square(100.0, &[], &[], 0.0);
    let mut face = sq.faces[0].clone();
    add_outside_edge(&mut face, true);
    let forward = classify(&face);
    assert_eq!(forward, NoError);
    assert_eq!(classify(&Face::unchecked(face.reversed())), forward);
}

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
