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

//! Behavior of the IntPatch analytic intersections beyond the numeric probe, which
//! compares them with native OCCT bit for bit: the results lie on both surfaces, the
//! consultation members check the state and the indices, and the special points land on
//! the poles.
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{
    GeomBezierSurface, GeomCylindricalSurface, GeomPlane, GeomSphericalSurface, GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_patch::*;
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfTypeTrans};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Xyz};
use std::f64::consts::PI;

const TWO_PI: f64 = 2.0 * PI;

fn frame(origin: Point3, normal: (f64, f64, f64)) -> Axis3 {
    let n = Direction3::new(normal.0, normal.1, normal.2).unwrap();
    Axis3::from_normal(origin, n).unwrap()
}

fn bounded(s: Handle<dyn GeomSurface>, b: (f64, f64, f64, f64)) -> Adaptor3dSurfaceHandle {
    Handle::new(GeomAdaptorSurface::with_bounds(&s, b).unwrap()).cast()
}

fn natural(s: Handle<dyn GeomSurface>) -> Adaptor3dSurfaceHandle {
    Handle::new(GeomAdaptorSurface::with_surface(&s).unwrap()).cast()
}

/// The plane `z = z0` over `[-3, 3]^2`.
fn plane_z(z0: f64) -> Adaptor3dSurfaceHandle {
    let p = GeomPlane::new(frame(Point3::new(0.0, 0.0, z0), (0.0, 0.0, 1.0)));
    bounded(Handle::new(p).cast(), (-3.0, 3.0, -3.0, 3.0))
}

/// The cylinder of radius `r` around the Z axis, V in `[-2, 2]`.
fn cylinder_z(r: f64, u: (f64, f64)) -> Adaptor3dSurfaceHandle {
    let c = GeomCylindricalSurface::new(frame(Point3::new(0.0, 0.0, 0.0), (0.0, 0.0, 1.0)), r);
    bounded(Handle::new(c.unwrap()).cast(), (u.0, u.1, -2.0, 2.0))
}

fn cylinder_x(r: f64) -> Adaptor3dSurfaceHandle {
    let c = GeomCylindricalSurface::new(frame(Point3::new(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)), r);
    bounded(Handle::new(c.unwrap()).cast(), (0.0, TWO_PI, -3.0, 3.0))
}

fn sphere(center: Point3, r: f64) -> Adaptor3dSurfaceHandle {
    let s = GeomSphericalSurface::new(frame(center, (0.0, 0.0, 1.0)), r).unwrap();
    natural(Handle::new(s).cast())
}

/// A bilinear Bezier patch of the four corners.
fn bezier(c: [[f64; 3]; 4]) -> Adaptor3dSurfaceHandle {
    let mut poles: Array2<'static, Point3> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    poles[(1, 1)] = Point3::new(c[0][0], c[0][1], c[0][2]);
    poles[(1, 2)] = Point3::new(c[1][0], c[1][1], c[1][2]);
    poles[(2, 1)] = Point3::new(c[2][0], c[2][1], c[2][2]);
    poles[(2, 2)] = Point3::new(c[3][0], c[3][1], c[3][2]);
    natural(Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast())
}

fn domain(s: &Adaptor3dSurfaceHandle) -> Adaptor3dTopolTool {
    Adaptor3dTopolTool::with_surface(s).unwrap()
}

/// Every point of the point lines of `lines`, with the number of lines of each type.
fn line_points(lines: &[Handle<dyn IntPatchLine>]) -> Vec<Point3> {
    let mut out = Vec::new();
    for l in lines {
        let pl: Handle<dyn IntPatchPointLine> = l.cast();
        let Some(pl) = pl.get() else { continue };
        for i in 1..=pl.nb_pnts().unwrap() {
            out.push(pl.point(i).unwrap().value());
        }
    }
    out
}

fn impimp_lines(x: &IntPatchImpImpIntersection) -> Vec<Handle<dyn IntPatchLine>> {
    (1..=x.nb_lines().unwrap())
        .map(|i| x.line(i).unwrap().clone())
        .collect()
}

fn impprm_lines(x: &IntPatchImpPrmIntersection) -> Vec<Handle<dyn IntPatchLine>> {
    (1..=x.nb_lines().unwrap())
        .map(|i| x.line(i).unwrap().clone())
        .collect()
}

#[test]
fn impimp_consultation_needs_a_done_intersection() {
    let x = IntPatchImpImpIntersection::new();
    assert!(!x.is_done());
    assert_eq!(
        x.is_empty().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        x.nb_lines().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        x.point(1).err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
}

#[test]
fn impimp_plane_and_cylinder_cut_a_circle() {
    let (s1, s2) = (plane_z(0.5), cylinder_z(1.0, (0.0, TWO_PI)));
    let (d1, d2) = (domain(&s1), domain(&s2));
    let x =
        IntPatchImpImpIntersection::with_surfaces(&s1, &d1, &s2, &d2, 1e-7, 1e-6, false).unwrap();
    assert!(x.is_done() && !x.is_empty().unwrap());
    assert_eq!(x.get_status(), IntPatchImpImpStatus::Ok);
    let lines = impimp_lines(&x);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].arc_type(), IntPatchIType::Circle);
    let g: Handle<IntPatchGLine> = lines[0].cast();
    assert!((g.get().unwrap().circle().unwrap().radius() - 1.0).abs() < 1e-12);
    assert_eq!(
        x.line(0).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn impimp_same_cylinder_is_tangent_faces() {
    let (s1, s2) = (cylinder_z(1.0, (0.0, TWO_PI)), cylinder_z(1.0, (1.0, 4.0)));
    let (d1, d2) = (domain(&s1), domain(&s2));
    let x =
        IntPatchImpImpIntersection::with_surfaces(&s1, &d1, &s2, &d2, 1e-7, 1e-6, false).unwrap();
    assert!(x.tangent_faces().unwrap());
    assert!(!x.opposite_faces().unwrap());
}

#[test]
fn impimp_crossing_cylinders_walk_on_both() {
    let (s1, s2) = (cylinder_z(1.0, (0.0, TWO_PI)), cylinder_x(0.6));
    let (d1, d2) = (domain(&s1), domain(&s2));
    let x =
        IntPatchImpImpIntersection::with_surfaces(&s1, &d1, &s2, &d2, 1e-7, 1e-6, false).unwrap();
    let points = line_points(&impimp_lines(&x));
    assert!(points.len() > 20);
    for p in points {
        assert!(((p.x() * p.x() + p.y() * p.y()).sqrt() - 1.0).abs() < 1e-6);
        assert!(((p.y() * p.y() + p.z() * p.z()).sqrt() - 0.6).abs() < 1e-6);
    }
}

#[test]
fn impimp_refuses_a_parametric_surface() {
    let (s1, s2) = (
        plane_z(0.0),
        bezier([
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
        ]),
    );
    let (d1, d2) = (domain(&s1), domain(&s2));
    let e = IntPatchImpImpIntersection::with_surfaces(&s1, &d1, &s2, &d2, 1e-7, 1e-6, false)
        .err()
        .unwrap();
    assert_eq!(e.exception_type(), "Standard_ConstructionError");
}

#[test]
fn aline_to_wline_samples_the_analytic_lines_on_both_surfaces() {
    let (s1, s2) = (
        cylinder_z(1.0, (0.0, TWO_PI)),
        sphere(Point3::new(0.8, 0.2, 0.3), 1.0),
    );
    let (d1, d2) = (domain(&s1), domain(&s2));
    let x =
        IntPatchImpImpIntersection::with_surfaces(&s1, &d1, &s2, &d2, 1e-7, 1e-6, false).unwrap();
    let mut c = IntPatchALineToWLine::new(&s1, &s2, 200).unwrap();
    c.set_tol_3d(1e-6);
    c.set_tol_transition(1e-7);
    c.set_tol_open_domain(1e-8);
    assert_eq!(
        (c.tol_3d(), c.tol_transition(), c.tol_open_domain()),
        (1e-6, 1e-7, 1e-8)
    );
    let mut walked: Vec<Handle<dyn IntPatchLine>> = Vec::new();
    for l in impimp_lines(&x) {
        let a: Handle<IntPatchALine> = l.cast();
        if !a.is_null() {
            c.make_wline(&a, &mut walked).unwrap();
        }
    }
    let points = line_points(&walked);
    assert!(!points.is_empty());
    let center = Point3::new(0.8, 0.2, 0.3);
    for p in points {
        assert!(((p.x() * p.x() + p.y() * p.y()).sqrt() - 1.0).abs() < 1e-6);
        assert!((p.distance(&center) - 1.0).abs() < 1e-6);
    }
}

#[test]
fn aline_to_wline_deflection_and_section_radius() {
    let (s1, s2) = (
        cylinder_z(1.0, (0.0, TWO_PI)),
        sphere(Point3::new(0.8, 0.2, 0.3), 1.0),
    );
    let c = IntPatchALineToWLine::new(&s1, &s2, 200).unwrap();
    // On the cylinder the section radius is its radius.
    // The radius of the section of the sphere (the cylinder has none) through the point.
    let r = c.get_section_radius(Point3::new(1.0, 0.0, 0.0)).unwrap();
    assert!((r - 0.91_f64.sqrt()).abs() < 1e-12);
    assert!(c.check_deflection(Xyz::new(1.0, 0.0, 0.0), 1e-3) >= -1);
}

#[test]
fn impprm_consultation_needs_a_done_intersection() {
    let x = IntPatchImpPrmIntersection::new();
    assert!(!x.is_done());
    assert_eq!(
        x.nb_pnts().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(x.line(1).err().unwrap().exception_type(), "StdFail_NotDone");
}

#[test]
fn impprm_plane_and_bezier_cut_on_the_plane() {
    let s1 = plane_z(0.1);
    let s2 = bezier([
        [-2.0, -2.0, -1.0],
        [-2.0, 2.0, 1.2],
        [2.0, -2.0, -0.8],
        [2.0, 2.0, 1.0],
    ]);
    let (d1, d2) = (domain(&s1), domain(&s2));
    let x = IntPatchImpPrmIntersection::with_surfaces(&s1, &d1, &s2, &d2, 1e-7, 1e-6, 0.01, 0.1)
        .unwrap();
    assert!(x.is_done() && !x.is_empty().unwrap());
    let points = line_points(&impprm_lines(&x));
    assert!(points.len() > 2);
    assert!(points.iter().all(|p| (p.z() - 0.1).abs() < 1e-5));
    assert_eq!(
        x.line(0).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn impprm_sphere_lines_are_split_at_the_poles() {
    let s1 = sphere(Point3::new(0.0, 0.0, 0.0), 1.2);
    let s2 = bezier([
        [0.0, -2.0, -2.0],
        [0.0, -2.0, 2.0],
        [0.0, 2.0, -2.0],
        [0.0, 2.0, 2.0],
    ]);
    let (d1, d2) = (domain(&s1), domain(&s2));
    let mut x = IntPatchImpPrmIntersection::new();
    // A start point on the circle (y = -2 + 4u, z = -2 + 4v on the patch).
    x.set_start_point(0.8, 0.5);
    x.perform(&s1, &d1, &s2, &d2, 1e-7, 1e-6, 0.01, 0.1)
        .unwrap();
    let lines = impprm_lines(&x);
    // The great circle x = 0 is cut at both poles.
    assert_eq!(lines.len(), 3);
    let is_pole = |p: Point3| (p.z().abs() - 1.2).abs() < 1e-9;
    for l in &lines {
        let w: Handle<IntPatchWLine> = l.cast();
        let w = w.get().unwrap();
        let (first, last) = (
            w.point(1).unwrap().value(),
            w.point(w.nb_pnts().unwrap()).unwrap().value(),
        );
        assert!(is_pole(first) || is_pole(last));
    }
    let points = line_points(&lines);
    assert!(
        points
            .iter()
            .all(|p| (p.distance(&Point3::new(0.0, 0.0, 0.0)) - 1.2).abs() < 1e-5)
    );
}

#[test]
fn impprm_refuses_two_parametric_surfaces() {
    let s = bezier([
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
    ]);
    let d = domain(&s);
    let e = IntPatchImpPrmIntersection::with_surfaces(&s, &d, &s, &d, 1e-7, 1e-6, 0.01, 0.1)
        .err()
        .unwrap();
    assert_eq!(e.exception_type(), "Standard_ConstructionError");
}

/// The circle `z = 0.3` of the unit cylinder around Z, with the cylinder parameters first.
fn circle_line() -> Handle<IntSurfLineOn2S> {
    let pts = Handle::new(IntSurfLineOn2S::new(None));
    for k in 0..40 {
        let u = TWO_PI * f64::from(k) / 39.0;
        let (x, y) = (u.cos(), u.sin());
        let mut p = IntSurfPntOn2S::new();
        p.set_value_all(Point3::new(x, y, 0.3), u, 0.3, x, y);
        pts.add(&p);
    }
    pts
}

#[test]
fn rst_int_puts_vertices_on_the_bounds_of_the_cylinder() {
    let (s, o) = (cylinder_z(1.0, (0.3, 5.0)), plane_z(0.3));
    let ds = domain(&s);
    let w: Handle<dyn IntPatchLine> = Handle::new(IntPatchWLine::new(
        &circle_line(),
        false,
        IntSurfTypeTrans::In,
        IntSurfTypeTrans::Out,
    ))
    .cast();
    IntPatchRstInt::put_vertex_on_line(&w, &s, &ds, &o, true, 1e-6).unwrap();
    let wl: Handle<IntPatchWLine> = w.cast();
    let wl = wl.get().unwrap();
    let us: Vec<f64> = (1..=wl.nb_vertex())
        .map(|i| wl.vertex(i).unwrap().parameters_on_s1().0)
        .collect();
    // A vertex where the circle leaves the patch at each U bound.
    for bound in [0.3, 5.0] {
        assert!(
            us.iter().any(|u| (u - bound).abs() < 1e-6),
            "no vertex at U {bound} in {us:?}"
        );
    }
}

#[test]
fn rst_int_refuses_a_conic_line() {
    let (s, o) = (cylinder_z(1.0, (0.3, 5.0)), plane_z(0.3));
    let (s1, s2) = (plane_z(0.5), cylinder_z(1.0, (0.0, TWO_PI)));
    let (d1, d2) = (domain(&s1), domain(&s2));
    let x =
        IntPatchImpImpIntersection::with_surfaces(&s1, &d1, &s2, &d2, 1e-7, 1e-6, false).unwrap();
    let g = x.line(1).unwrap().clone();
    let e = IntPatchRstInt::put_vertex_on_line(&g, &s, &domain(&s), &o, true, 1e-6).unwrap_err();
    assert_eq!(e.exception_type(), "Standard_DomainError");
}

#[test]
fn special_points_find_the_pole_of_the_sphere() {
    let q = sphere(Point3::new(0.0, 0.0, 0.0), 1.0);
    let p = plane_z(1.0);
    let mut vertex = IntPatchPoint::new();
    let mut at_pole = IntSurfPntOn2S::new();
    at_pole.set_value_all(Point3::new(0.0, 0.0, 1.0), 0.1, 1.4, 0.0, 0.0);
    vertex.set_pnt_on_2s(&at_pole);
    vertex.set_tolerance(1e-6);
    let mut added = IntSurfPntOn2S::new();
    let found = IntPatchSpecialPoints::add_singular_pole(
        &q,
        &p,
        &at_pole,
        &mut vertex,
        &mut added,
        false,
        false,
    )
    .unwrap();
    assert!(found);
    assert!(added.value().distance(&Point3::new(0.0, 0.0, 1.0)) < 1e-9);
    assert!((added.parameters_on_s1().1 - PI / 2.0).abs() < 1e-12);
}

#[test]
fn special_points_continue_only_after_a_special_point() {
    let q = sphere(Point3::new(0.0, 0.0, 0.0), 1.0);
    let p = plane_z(1.0);
    let r = IntSurfPntOn2S::new();
    let mut new_point = IntSurfPntOn2S::new();
    let t = IntPatchSpecPntType::SPntNone;
    assert!(
        !IntPatchSpecialPoints::continue_after_special_point(
            &q,
            &p,
            &r,
            t,
            1e-9,
            &mut new_point,
            false
        )
        .unwrap()
    );
    // The tangents of a cone of semi-angle PI/4 cut by the plane of normal X.
    let mut tangents = [Xyz::new(0.0, 0.0, 0.0); 2];
    let n = IntPatchSpecialPoints::get_tangent_to_int_line_for_cone(
        PI / 4.0,
        Xyz::new(1.0, 0.0, 0.0),
        &mut tangents,
    );
    assert!(n >= 1);
}

#[test]
fn special_points_adjust_the_periods_to_the_reference() {
    let mut reference = IntSurfPntOn2S::new();
    reference.set_value_all(Point3::new(0.0, 0.0, 0.0), 0.1, 0.0, 0.0, 0.0);
    let mut new_point = IntSurfPntOn2S::new();
    new_point.set_value_all(Point3::new(0.0, 0.0, 0.0), 0.1 + TWO_PI, 0.0, 0.0, 0.0);
    IntPatchSpecialPoints::adjust_point_and_vertex(
        &reference,
        &[TWO_PI, 0.0, 0.0, 0.0],
        &mut new_point,
        None,
    );
    assert!((new_point.parameters_on_s1().0 - 0.1).abs() < 1e-12);
}
