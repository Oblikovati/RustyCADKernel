// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Behavior of the IntImpParGen and IntCurve ports on simple conic pairs (the bits
//! of every pair are compared with OCCT by the intcurve probes).
use rustycad_tkgeomalgo::int_curve::{
    IConicTool, IntConicConic, IntImpConicParConic, PConic, PeriodicInterval,
};
use rustycad_tkgeomalgo::int_imp_par_gen::{TransitionSide, determine_transition};
use rustycad_tkgeomalgo::int_res2d::{Domain, Position, Transition, TypeTrans};
use rustycad_tkmath::gp::{
    Axis22d, Circle2, Direction2, Ellipse2, Line2, Parabola2, Point2, Vector2,
};
use std::f64::consts::PI;

fn line(x: f64, y: f64, dx: f64, dy: f64) -> Line2 {
    Line2::new(Point2::new(x, y), Direction2::new(dx, dy).unwrap())
}
fn circle(x: f64, y: f64, r: f64) -> Circle2 {
    let frame = Axis22d::new(Point2::new(x, y), Direction2::new(1.0, 0.0).unwrap(), true).unwrap();
    Circle2::new(frame, r).unwrap()
}
fn full_turn(c: &Circle2) -> Domain {
    let p = c.location().translated(Vector2::new(c.radius(), 0.0));
    let mut d = Domain::bounded(p, 0.0, 1e-9, p, 2.0 * PI, 1e-9);
    d.set_equivalent_parameters(0.0, 2.0 * PI).unwrap();
    d
}

#[test]
fn crossing_lines_meet_at_one_point_with_opposite_transitions() {
    let inter = IntConicConic::lines(
        line(0.0, 0.0, 1.0, 0.0),
        &Domain::new(),
        line(2.0, -1.0, 0.0, 1.0),
        &Domain::new(),
        1e-9,
        1e-9,
    )
    .unwrap();
    assert_eq!(
        (inter.nb_points().unwrap(), inter.nb_segments().unwrap()),
        (1, 0)
    );
    let p = inter.point(1).unwrap();
    assert_eq!((p.param_on_first(), p.param_on_second()), (2.0, 1.0));
    assert_eq!(p.transition_of_first().transition_type(), TypeTrans::Out);
    assert_eq!(p.transition_of_second().transition_type(), TypeTrans::In);
}

#[test]
fn coincident_lines_give_one_infinite_segment() {
    let inter = IntConicConic::lines(
        line(0.0, 0.0, 1.0, 0.0),
        &Domain::new(),
        line(5.0, 0.0, -1.0, 0.0),
        &Domain::new(),
        1e-9,
        1e-9,
    )
    .unwrap();
    assert_eq!(inter.nb_segments().unwrap(), 1);
    let segment = inter.segment(1).unwrap();
    assert!(segment.is_opposite() && !segment.has_first_point() && !segment.has_last_point());
}

#[test]
fn secant_circles_meet_twice_and_identical_circles_overlap() {
    let (c1, c2) = (circle(0.0, 0.0, 1.0), circle(1.0, 0.0, 1.0));
    let inter =
        IntConicConic::circles(c1, &full_turn(&c1), c2, &full_turn(&c2), 1e-9, 1e-9).unwrap();
    assert_eq!(inter.nb_points().unwrap(), 2);
    let same =
        IntConicConic::circles(c1, &full_turn(&c1), c1, &full_turn(&c1), 1e-9, 1e-9).unwrap();
    assert_eq!(same.nb_segments().unwrap(), 1);
}

#[test]
fn tangent_line_touches_the_circle() {
    let c = circle(0.0, 0.0, 1.0);
    let inter = IntConicConic::line_circle(
        line(-3.0, 1.0, 1.0, 0.0),
        &Domain::new(),
        c,
        &full_turn(&c),
        1e-9,
        1e-9,
    )
    .unwrap();
    assert_eq!(inter.nb_points().unwrap(), 1);
    let t = inter.point(1).unwrap().transition_of_first();
    assert_eq!(t.transition_type(), TypeTrans::Touch);
}

#[test]
fn ellipse_and_parabola_intersect_through_the_generic_intersector() {
    let frame = Axis22d::default();
    let ellipse = Ellipse2::new(frame, 2.0, 1.0).unwrap();
    let parabola = Parabola2::new(frame, 1.0).unwrap();
    let mut domain = Domain::bounded(
        Point2::new(2.0, 0.0),
        0.0,
        1e-9,
        Point2::new(2.0, 0.0),
        2.0 * PI,
        1e-9,
    );
    domain.set_equivalent_parameters(0.0, 2.0 * PI).unwrap();
    let inter =
        IntConicConic::ellipse_parabola(ellipse, &domain, parabola, &Domain::new(), 1e-9, 1e-9)
            .unwrap();
    assert!(inter.is_done());
    assert_eq!(inter.nb_points().unwrap(), 2);
}

#[test]
fn an_unbounded_parametric_domain_is_a_construction_error() {
    let itool = IConicTool::from_circle(circle(0.0, 0.0, 1.0));
    let pconic = PConic::from_line(line(-2.0, 0.0, 1.0, 0.0)).unwrap();
    let error = IntImpConicParConic::with_curves(
        &itool,
        &Domain::new(),
        &pconic,
        &Domain::new(),
        1e-9,
        1e-9,
    )
    .err()
    .unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
}

#[test]
fn null_tangents_make_the_transition_undecided() {
    let (mut t1, mut t2) = (Vector2::new(0.0, 0.0), Vector2::new(1.0, 0.0));
    let (mut x, mut y) = (Transition::new(), Transition::new());
    let side1 = TransitionSide {
        pos: Position::Head,
        tan: &mut t1,
        norm: Vector2::new(0.0, 0.0),
        trans: &mut x,
    };
    let side2 = TransitionSide {
        pos: Position::End,
        tan: &mut t2,
        norm: Vector2::new(0.0, 1.0),
        trans: &mut y,
    };
    determine_transition(side1, side2, 1e-7);
    assert_eq!(x.transition_type(), TypeTrans::Undecided);
    assert_eq!(y.position_on_curve(), Position::End);
}

#[test]
fn first_intersection_keeps_the_shift_of_its_argument() {
    let mut domain = PeriodicInterval::new(0.5, 1.0);
    let mut arc = PeriodicInterval {
        binf: -5.5,
        bsup: -5.0,
        isnull: false,
    };
    let common = domain.first_intersection(&mut arc);
    assert!(!common.isnull);
    assert!(
        arc.binf > 0.0,
        "the argument was shifted by periods: {arc:?}"
    );
}
