// Copyright (c) 1992-1999 Matra Datavision
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

//! The exceptions and the preserved native quirks of the IntAna and IntAna2d ports
//! (docs/tkgeombase-progress.md); the numeric results are compared with the native
//! kernel by tools/verify_tkgeombase_numeric.py.
use rustycad_tkgeombase::int_ana::{
    Curve, IntConicQuad, IntLinTorus, IntQuadQuad, QuadQuadGeo, Quadric, ResultType,
};
use rustycad_tkgeombase::int_ana2d::{
    AnaIntersection, Conic, IntPoint, MyDirectPolynomialRoots, traitement_points_confondus,
};
use rustycad_tkmath::gp::{
    Axis3, Axis22d, Cylinder3, Direction2, Line2, Line3, Parabola2, Plane3, Point2, Point3,
    Sphere3, StandardDirection3,
};

fn z_frame() -> Axis3 {
    Axis3::with_standard(Point3::new(0.0, 0.0, 0.0), StandardDirection3::Z)
}

#[test]
fn empty_algorithms_raise_not_done() {
    let kind = |error: rustycad_tkernel::standard::Failure| error.exception_type().to_string();
    assert_eq!(
        kind(AnaIntersection::new().nb_points().unwrap_err()),
        "StdFail_NotDone"
    );
    assert_eq!(
        kind(IntConicQuad::new().nb_points().unwrap_err()),
        "StdFail_NotDone"
    );
    assert_eq!(
        kind(IntLinTorus::new().nb_points().unwrap_err()),
        "StdFail_NotDone"
    );
    assert_eq!(
        kind(IntQuadQuad::new().nb_curve().unwrap_err()),
        "StdFail_NotDone"
    );
    assert_eq!(
        kind(QuadQuadGeo::new().type_inter().unwrap_err()),
        "StdFail_NotDone"
    );
    assert!(!QuadQuadGeo::default().has_common_gen());
}

#[test]
fn parabola_conic_is_always_done() {
    let frame = Axis22d::with_directions(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        Direction2::new(0.0, 1.0).unwrap(),
    )
    .unwrap();
    let parabola = Parabola2::new(frame, 1.0).unwrap();
    let inter = AnaIntersection::parabola_conic(parabola, &Conic::from_parabola(parabola)).unwrap();
    assert!(inter.is_done());
    assert!(inter.identical_elements().unwrap());
    assert_eq!(inter.nb_points().unwrap(), 0);
}

#[test]
fn identical_lines_are_not_empty_and_have_no_point() {
    let line = Line2::new(Point2::new(1.0, 2.0), Direction2::new(0.6, 0.8).unwrap());
    let inter = AnaIntersection::lines(line, line);
    assert!(!inter.is_empty().unwrap());
    assert!(inter.parallel_elements().unwrap());
    assert_eq!(
        inter.point(1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn coincident_points_lose_their_second_parameter() {
    // The second point repeats the first: the third one moves down implicitly.
    let mut points = [
        IntPoint::new(1.0, 2.0, 0.5, 0.25),
        IntPoint::new(1.0, 2.0, 0.7, 0.8),
        IntPoint::new(3.0, 4.0, 0.1, 0.2),
        IntPoint::default(),
    ];
    assert_eq!(traitement_points_confondus(3, &mut points), 2);
    assert_eq!(points[1].value(), Point2::new(3.0, 4.0));
    assert_eq!(points[1].param_on_first(), 0.1);
    assert_eq!(
        points[1].param_on_second().unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    let roots = MyDirectPolynomialRoots::quadratic(0.0, 0.0, 0.0).unwrap();
    assert!(roots.infinite_roots() && roots.is_done());
    assert!(roots.value(17).is_err());
}

#[test]
fn cylinder_curves_are_never_linked() {
    let cylinder = Cylinder3::new(z_frame(), 2.0).unwrap();
    let sphere = Sphere3::new(
        Axis3::with_standard(Point3::new(1.5, 0.0, 0.0), StandardDirection3::Z),
        2.0,
    )
    .unwrap();
    let inter = IntQuadQuad::cylinder(cylinder, &Quadric::from_sphere(sphere), 1e-7).unwrap();
    assert!(inter.nb_curve().unwrap() > 1);
    for i in 1..=inter.nb_curve().unwrap() {
        assert!(!inter.has_next_curve(i).unwrap());
        assert!(!inter.has_previous_curve(i).unwrap());
    }
    let (mut u1, mut u2) = (1.0, 2.0);
    inter.parameters(1, &mut u1, &mut u2);
    assert_eq!((u1, u2), (1.0, 2.0));
}

#[test]
fn curve_domain_errors() {
    let mut curve = Curve::new();
    assert_eq!(
        curve.domain().unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    let cylinder = Cylinder3::new(z_frame(), 1.0).unwrap();
    let q = [1.0, 1.0, 1.0, 0.0, 0.0, 0.0, -1.5, 0.0, 0.0, -1.75];
    curve.set_cylinder_quad_values(cylinder, q, 1e-8, (0.3, 2.8), false, true);
    assert_eq!(curve.domain().unwrap(), (0.3, 2.8));
    assert!(curve.value(3.0).is_err());
    assert!(curve.set_domain(1.0, 1.0).is_err());
    assert!(!curve.is_constant());
}

#[test]
fn quadric_special_points_accumulate() {
    let sphere = Sphere3::new(z_frame(), 2.0).unwrap();
    let mut quadric = Quadric::from_sphere(sphere);
    assert_eq!(quadric.special_points().len(), 2);
    quadric.set_sphere(sphere);
    assert_eq!(quadric.special_points().len(), 4);
    quadric.set_plane(Plane3::from_position(z_frame())).unwrap();
    assert_eq!(quadric.special_points().len(), 4);
    assert_eq!(Quadric::new().coefficients()[9], 1.0);
}

#[test]
fn line_plane_and_plane_torus_results() {
    let plane = Plane3::from_position(z_frame());
    let line = Line3::new(
        Point3::new(0.0, 0.0, 1.0),
        rustycad_tkmath::gp::Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let inter = IntConicQuad::line_plane(line, plane, 1e-12, 0.0, 0.0).unwrap();
    assert!(inter.is_parallel().unwrap() && !inter.is_in_quadric().unwrap());
    assert_eq!(
        inter.nb_points().unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    let planes = QuadQuadGeo::planes(plane, plane, 1e-12, 1e-7).unwrap();
    assert_eq!(planes.type_inter().unwrap(), ResultType::Same);
    assert_eq!(ResultType::Same.name(), "IntAna_Same");
}
