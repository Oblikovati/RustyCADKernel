// Copyright (c) 1995-1999 Matra Datavision
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

//! Behavior of the IntSurf port beyond the upstream suites (the numeric members are
//! compared with OCCT bit for bit by the numeric probe).
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_surf::*;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis3, Direction2, Point2, Point3, Sphere3, Vector2, Vector3};

fn pnt_on_2s(x: f64, u: f64) -> IntSurfPntOn2S {
    let mut p = IntSurfPntOn2S::new();
    p.set_value_all(Point3::new(x, 0.0, 0.0), u, u, -u, -u);
    p
}

#[test]
fn transition_accessors_raise_on_the_wrong_type() {
    let undecided = IntSurfTransition::new();
    let error = undecided.is_tangent().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    let mut t = IntSurfTransition::touch(true, IntSurfSituation::Inside, true);
    assert_eq!(t.situation().unwrap(), IntSurfSituation::Inside);
    assert!(t.is_opposite().unwrap());
    t.set_type(false, IntSurfTypeTrans::In);
    assert!(t.situation().is_err() && !t.is_tangent().unwrap());
    // SetValue(Tangent, Type) keeps the situation of the earlier TOUCH transition.
    t.set_type(false, IntSurfTypeTrans::Touch);
    assert_eq!(t.situation().unwrap(), IntSurfSituation::Inside);
    t.set_undecided();
    assert_eq!(t.transition_type(), IntSurfTypeTrans::Undecided);
    let mut touch = IntSurfTransition::with_type(true, IntSurfTypeTrans::Out);
    touch.set_touch(false, IntSurfSituation::Outside, false);
    assert_eq!(touch.situation().unwrap(), IntSurfSituation::Outside);
}

#[test]
fn pnt_on_2s_setters_and_getters_agree() {
    let mut p = IntSurfPntOn2S::new();
    p.set_value_on(Point3::new(1.0, 2.0, 3.0), true, 0.5, 0.25);
    p.set_parameters_on(false, 4.0, 5.0);
    assert_eq!(p.parameters(), (0.5, 0.25, 4.0, 5.0));
    assert_eq!(p.parameters_on_s1(), (0.5, 0.25));
    assert_eq!(p.parameters_on_s2(), (4.0, 5.0));
    assert_eq!(p.value_on_surface(false), Point2::new(4.0, 5.0));
    p.set_point(Point3::new(0.0, 0.0, 0.0));
    p.set_parameters(1.0, 2.0, 3.0, 4.0);
    assert_eq!(p.parameters_on_surface(true), (1.0, 2.0));
    assert_eq!(p.value(), Point3::new(0.0, 0.0, 0.0));
    let couple = IntSurfCouple::with_indices(3, 4);
    assert_eq!((couple.first(), couple.second()), (3, 4));
    assert_eq!(IntSurfCouple::new().first(), 0);
}

#[test]
fn line_on_2s_edits_keep_the_points_in_order() {
    let line = IntSurfLineOn2S::new(None);
    for i in 0..3 {
        line.add(&pnt_on_2s(f64::from(i), f64::from(i)));
    }
    line.insert_before(1, &pnt_on_2s(-1.0, -1.0)).unwrap();
    line.insert_before(9, &pnt_on_2s(9.0, 9.0)).unwrap();
    assert!(line.insert_before(0, &pnt_on_2s(0.0, 0.0)).is_err());
    assert_eq!(line.nb_points(), 5);
    line.remove_point(5).unwrap();
    line.reverse();
    assert_eq!(line.value(1).unwrap().value().x(), 2.0);
    line.set_uv(1, true, 0.5, 0.75).unwrap();
    line.set_uv(1, false, 7.0, 8.0).unwrap();
    assert_eq!(line.value(1).unwrap().parameters(), (0.5, 0.75, 7.0, 8.0));
    assert!(line.value(6).is_err() && line.remove_point(0).is_err());
    assert!(line.split(9).is_err());
    assert!(line.is_out_surf2_box(Point2::new(100.0, 0.0)));
    line.clear();
    assert_eq!(line.nb_points(), 0);
    assert_eq!(IntSurfLineOn2S::static_type().name(), "IntSurf_LineOn2S");
}

#[test]
fn built_boxes_grow_with_added_points() {
    let line = IntSurfLineOn2S::new(None);
    line.add(&pnt_on_2s(0.0, 0.0));
    line.add(&pnt_on_2s(1.0, 1.0));
    assert!(line.is_out_box(Point3::new(5.0, 0.0, 0.0)));
    assert!(line.is_out_surf1_box(Point2::new(5.0, 5.0)));
    line.add(&pnt_on_2s(5.0, 5.0));
    assert!(!line.is_out_box(Point3::new(5.0, 0.0, 0.0)));
    assert!(!line.is_out_surf1_box(Point2::new(5.0, 5.0)));
}

#[test]
fn interior_point_and_its_tool_read_the_same_fields() {
    let tangent2d = Vector2::new(0.0, 2.0);
    let origin = Point3::new(1.0, 2.0, 3.0);
    let ip =
        IntSurfInteriorPoint::with_value(origin, 0.5, 0.25, Vector3::new(1.0, 0.0, 0.0), tangent2d);
    assert_eq!(IntSurfInteriorPointTool::value3d(&ip), ip.value());
    let parameters = (ip.u_parameter(), ip.v_parameter());
    assert_eq!(IntSurfInteriorPointTool::value2d(&ip), parameters);
    assert_eq!(IntSurfInteriorPointTool::direction3d(&ip), ip.direction());
    let unit = Direction2::new(0.0, 1.0).unwrap();
    assert_eq!(IntSurfInteriorPointTool::direction2d(&ip).unwrap(), unit);
    let mut null = IntSurfInteriorPoint::new();
    assert!(IntSurfInteriorPointTool::direction2d(&null).is_err());
    null.set_value(origin, 1.0, 2.0, Vector3::new(0.0, 0.0, 1.0), tangent2d);
    assert_eq!(
        (null.parameters(), null.direction2d()),
        ((1.0, 2.0), tangent2d)
    );
}

#[test]
fn path_point_shares_its_parameters_between_copies() {
    let mut p = IntSurfPathPoint::with_value(Point3::new(0.0, 0.0, 0.0), 0.5, 0.25);
    let copy = p.clone();
    copy.add_uv(1.0, 2.0).unwrap();
    assert_eq!(IntSurfPathPointTool::multiplicity(&p).unwrap(), 1);
    assert_eq!(IntSurfPathPointTool::parameters(&p, 1).unwrap(), (1.0, 2.0));
    assert!(p.parameters(2).is_err() && p.parameters(-1).is_err());
    assert_eq!(p.parameters(0).unwrap(), (0.5, 0.25));
    assert_eq!(IntSurfPathPointTool::value2d(&p).unwrap(), (0.5, 0.25));
    assert!(IntSurfPathPointTool::is_tangent(&p) && IntSurfPathPointTool::is_passing_pnt(&p));
    let error = IntSurfPathPointTool::direction3d(&p).unwrap_err();
    assert_eq!(error.exception_type(), "StdFail_UndefinedDerivative");
    assert!(IntSurfPathPointTool::direction2d(&p).is_err());
    p.set_directions(
        Vector3::new(1.0, 0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
    );
    p.set_passing(false);
    assert_eq!(
        IntSurfPathPointTool::direction3d(&p).unwrap(),
        Vector3::new(1.0, 0.0, 0.0)
    );
    assert!(IntSurfPathPointTool::direction2d(&p).is_ok() && !p.is_passing_pnt());
    p.set_tangency(true);
    p.set_value(Point3::new(1.0, 1.0, 1.0), 3.0, 4.0);
    assert_eq!(
        IntSurfPathPointTool::value3d(&p),
        Point3::new(1.0, 1.0, 1.0)
    );
    assert_eq!(p.multiplicity().unwrap(), 0);
    let empty = IntSurfPathPoint::new();
    let error = empty.value2d().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    assert!(empty.add_uv(0.0, 0.0).is_err() && empty.multiplicity().is_err());
}

#[test]
fn set_period_reads_the_periodic_directions() {
    let cylinder = GeomCylindricalSurface::new(Axis3::default(), 2.0).unwrap();
    let surface: Handle<dyn GeomSurface> = Handle::new(cylinder).cast();
    let adaptor = GeomAdaptorSurface::with_surface(&surface).unwrap();
    let periods = set_period(&adaptor, &adaptor).unwrap();
    let full = 2.0 * std::f64::consts::PI;
    assert_eq!(periods, [full, 0.0, full, 0.0]);
}

#[test]
fn quadric_tool_forwards_to_the_quadric() {
    let quadric = IntSurfQuadric::from_sphere(&Sphere3::new(Axis3::default(), 2.0).unwrap());
    assert_eq!(IntSurfQuadricTool::value(&quadric, 3.0, 0.0, 0.0), 1.0);
    let gradient = IntSurfQuadricTool::gradient(&quadric, 3.0, 0.0, 0.0);
    assert_eq!(gradient, Vector3::new(1.0, 0.0, 0.0));
    let (value, _) = IntSurfQuadricTool::value_and_gradient(&quadric, 3.0, 0.0, 0.0).unwrap();
    assert_eq!(value, 1.0);
    assert!(quadric.sphere().is_ok() && quadric.cone().is_err());
    assert_eq!(
        IntSurfQuadric::default().type_quadric(),
        SurfaceType::OtherSurface
    );
    let mut q = IntSurfQuadric::new();
    q.set_cylinder(&quadric.cylinder().unwrap());
    assert!(q.torus().is_ok() && q.cone().is_err());
    assert_eq!(q.plane().position().location(), Point3::new(0.0, 0.0, 0.0));
}
