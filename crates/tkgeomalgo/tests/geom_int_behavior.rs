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

//! Behavior of GeomInt (design section 20) beyond the numeric probe, which compares it
//! with native OCCT bit for bit.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane, GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_api::IntSS;
use rustycad_tkgeomalgo::geom_int::*;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchLine, IntPatchPointLine, IntPatchRLine, IntPatchWLine,
};
use rustycad_tkgeomalgo::int_start::TheTopolTool;
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Axis2, Axis3, Direction2, Direction3, Point2, Point3};
use std::f64::consts::PI;
use std::rc::Rc;

fn z() -> Direction3 {
    Direction3::new(0.0, 0.0, 1.0).unwrap()
}

fn plane(height: f64, normal: Direction3) -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(
        Axis3::from_normal(Point3::new(0.0, 0.0, height), normal).unwrap(),
    ))
    .cast()
}

fn cylinder() -> Handle<dyn GeomSurface> {
    let axis = Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), z()).unwrap();
    Handle::new(GeomCylindricalSurface::new(axis, 1.0).unwrap()).cast()
}

#[test]
fn adjust_periodic_and_parameter_orientation() {
    assert_eq!(adjust_periodic(-7.0, 0.0, 6.0, 6.0, 0.0), (true, 5.0, 12.0));
    assert_eq!(
        adjust_periodic(13.0, 0.0, 6.0, 6.0, 0.0),
        (false, 1.0, -12.0)
    );
    assert_eq!(adjust_periodic(3.0, 0.0, 6.0, 6.0, 0.0), (false, 3.0, 0.0));
    let mut p = GeomIntParameterAndOrientation::new();
    assert_eq!(
        (p.parameter(), p.orientation1(), p.orientation2()),
        (0.0, Orientation::Forward, Orientation::Forward)
    );
    p.set_orientation2(Orientation::Reversed);
    assert_eq!(p.orientation2(), Orientation::Reversed);
}

#[test]
fn plane_and_cylinder_give_a_circle_with_2d_curves() {
    let flags = GeomIntIntSSApprox {
        approx: true,
        approx_s1: true,
        approx_s2: true,
    };
    let inter = GeomIntIntSS::with_surfaces(&plane(0.5, z()), &cylinder(), 1e-7, flags).unwrap();
    assert!(inter.is_done());
    assert_eq!(inter.nb_lines(), 1);
    let c = inter.line(1).unwrap();
    let p = c.value(1.0).unwrap();
    assert!(
        (p.z() - 0.5).abs() < 1e-12 && ((p.x() * p.x() + p.y() * p.y()).sqrt() - 1.0).abs() < 1e-12
    );
    assert!(inter.has_line_on_s1(1).unwrap() && inter.has_line_on_s2(1).unwrap());
    assert!(inter.line(0).is_err() && inter.line(2).is_err());
    assert_eq!(inter.nb_boundaries().unwrap(), 0);
    assert!(inter.boundary(1).is_err());
    // Without the 2d curves.
    let bare = GeomIntIntSS::with_surfaces(
        &plane(0.5, z()),
        &cylinder(),
        1e-7,
        GeomIntIntSSApprox::default(),
    )
    .unwrap();
    assert!(!bare.has_line_on_s1(1).unwrap());
    // Not performed: the accessors refuse.
    let empty = GeomIntIntSS::new();
    assert!(!empty.is_done() && empty.line(1).is_err() && empty.nb_points().is_err());
}

#[test]
fn a_second_perform_keeps_the_2d_curves_of_the_first() {
    let flags = GeomIntIntSSApprox {
        approx: true,
        approx_s1: true,
        approx_s2: false,
    };
    let mut inter = GeomIntIntSS::new();
    inter
        .perform(&plane(0.5, z()), &cylinder(), 1e-7, flags)
        .unwrap();
    let first = inter.line_on_s1(1).unwrap();
    inter
        .perform(&plane(0.25, z()), &cylinder(), 1e-7, flags)
        .unwrap();
    assert_eq!(inter.nb_lines(), 1);
    // The 2d sequences are not cleared: the first index still reads the first curve.
    assert!(inter.line_on_s1(1).unwrap() == first);
    let api = IntSS::with_surfaces(&plane(0.5, z()), &cylinder(), 1e-7).unwrap();
    assert!(api.is_done() && api.nb_lines() == 1);
}

#[test]
fn line_tool_and_constructor_on_a_walking_line() {
    let points = Handle::new(IntSurfLineOn2S::new(None));
    for i in 0..6 {
        let x = 0.2 + 0.3 * f64::from(i);
        let mut p = IntSurfPntOn2S::new();
        p.set_value_all(Point3::new(x, 0.0, 0.0), x, 0.5, x, 0.0);
        points.add(&p);
    }
    let w = Handle::new(IntPatchWLine::new_undecided(&points, false));
    let l: Handle<dyn IntPatchLine> = w.clone().cast();
    assert_eq!(GeomIntLineTool::nb_vertex(&l).unwrap(), 0);
    assert_eq!(GeomIntLineTool::first_parameter(&l).unwrap(), 1.0);
    assert_eq!(GeomIntLineTool::last_parameter(&l).unwrap(), 6.0);
    assert!(GeomIntLineTool::vertex(&l, 1).is_err());
    let s1: Handle<GeomAdaptorSurface> = Handle::new(
        GeomAdaptorSurface::with_bounds(&plane(0.0, z()), (0.0, 4.0, 0.0, 1.0)).unwrap(),
    );
    let s2: Handle<GeomAdaptorSurface> = Handle::new(
        GeomAdaptorSurface::with_bounds(
            &plane(0.0, Direction3::new(0.0, 1.0, 0.0).unwrap()),
            (-5.0, 5.0, -5.0, 5.0),
        )
        .unwrap(),
    );
    let d1: Rc<dyn TheTopolTool> = Rc::new(Adaptor3dTopolTool::with_surface(&s1.cast()).unwrap());
    let d2: Rc<dyn TheTopolTool> = Rc::new(Adaptor3dTopolTool::with_surface(&s2.cast()).unwrap());
    let mut lc = GeomIntLineConstructor::new();
    // Before Load, a line without vertices needs no domain and has no part.
    lc.perform(&l).unwrap();
    assert!(lc.is_done());
    assert_eq!(lc.nb_parts().unwrap(), 0);
    lc.load(d1, d2, &s1, &s2);
    lc.perform(&l).unwrap();
    assert!(lc.is_done());
    // No vertex: no part.
    assert_eq!(lc.nb_parts().unwrap(), 0);
    assert!(lc.part(1).is_err());
    let mut new_lines = Vec::new();
    assert!(
        !GeomIntLineTool::decomposition_of_wline(&w, &s1, &s2, 2e-7, &lc, &mut new_lines).unwrap()
    );
    let bsp = GeomIntIntSS::make_bspline(&w, 2, 5).unwrap();
    assert_eq!((bsp.first_parameter(), bsp.last_parameter()), (0.0, 3.0));
    let bsp2 = GeomIntIntSS::make_bspline2d(&w, 1, 6, false).unwrap();
    assert_eq!(bsp2.nb_poles(), 6);
    assert!(GeomIntIntSS::make_bspline(&w, 4, 9).is_err());
}

#[test]
fn pcurves_trimming_and_restriction_lines() {
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    ))
    .cast();
    let mut c2d: Handle<dyn Geom2dCurve> = Handle::default();
    let mut tol = 1e-7;
    GeomIntIntSS::build_pcurves(0.0, 2.0, &mut tol, &plane(0.0, z()), &line, &mut c2d).unwrap();
    let p = c2d.value(1.0).unwrap();
    assert!((p.x() - 1.0).abs() < 1e-12 && p.y().abs() < 1e-12);
    // A preset 2d curve or a null surface: nothing is done.
    let before = c2d.clone();
    GeomIntIntSS::build_pcurves(0.0, 2.0, &mut tol, &plane(0.0, z()), &line, &mut c2d).unwrap();
    assert!(c2d == before);
    let circle: Handle<dyn GeomCurve> = Handle::new(
        GeomCircle::new(Axis2::new(Point3::new(0.0, 0.0, 0.3), z()).unwrap(), 1.0).unwrap(),
    )
    .cast();
    let mut on_cyl: Handle<dyn Geom2dCurve> = Handle::default();
    GeomIntIntSS::build_pcurves_bounds(
        1.0,
        4.0,
        (0.0, 2.0 * PI, -1.0, 1.0),
        &mut tol,
        &cylinder(),
        &circle,
        &mut on_cyl,
    )
    .unwrap();
    assert!(!on_cyl.is_null());
    let seg: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(-1.0, 0.5),
        Direction2::X,
    ))
    .cast();
    let (mut b1, mut b2) = (BoundingBox2::default(), BoundingBox2::default());
    b1.add_point(Point2::new(0.0, 0.0));
    b1.add_point(Point2::new(1.0, 1.0));
    b2.add_point(Point2::new(0.0, 0.0));
    b2.add_point(Point2::new(1.0, 1.0));
    let mut params = vec![3.0];
    GeomIntIntSS::trim_iline_on_surf_boundaries(&seg, &Handle::default(), &b1, &b2, &mut params)
        .unwrap();
    assert_eq!(params, vec![1.0, 2.0, 3.0]);
    // A restriction line on no arc gives no curve.
    let rl = Handle::new(IntPatchRLine::new_undecided(false));
    let mut out = GeomIntRLineCurves {
        tol_reached: -1.0,
        ..Default::default()
    };
    let hs: Handle<GeomAdaptorSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&cylinder()).unwrap());
    GeomIntIntSS::treat_rline(&rl, &hs, &hs, &mut out).unwrap();
    assert!(out.c3d.is_null() && out.tol_reached == -1.0);
    assert_eq!(rl.nb_vertex(), 0);
}
