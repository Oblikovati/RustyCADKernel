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

//! Behavior of the ApproxInt templates and their GeomInt instances (design section 20)
//! beyond the numeric probe, which compares them with native OCCT bit for bit.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::approx_int::{
    ApproxIntKnotPointSets, ApproxIntKnotTools, ApproxIntMultiLineLayout, ApproxIntOffsets,
    ApproxIntRange, ApproxIntSvComputed, ApproxIntSvSurfaces, PatchBounds, SvSurfacesRef,
};
use rustycad_tkgeomalgo::geom_int::*;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchIntersection, IntPatchIntersectionOptions, IntPatchPointLine, IntPatchWLine,
};
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfQuadric};
use rustycad_tkgeombase::app_par_curves::{LineTool, MultiCurveEval};
use rustycad_tkgeombase::approx::{ParametrizationType, Status};
use rustycad_tkmath::gp::{Axis3, Direction3, Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::Vector;
use std::cell::RefCell;
use std::rc::Rc;

/// A Bezier patch over [-2, 1]^2 with a gentle bump.
fn bump() -> Adaptor3dSurfaceHandle {
    let mut poles =
        rustycad_tkernel::ncollection::Array2::<Point3>::with_bounds(1, 4, 1, 4).unwrap();
    for i in 1..=4 {
        for j in 1..=4 {
            let (x, y) = (f64::from(i - 1), f64::from(j - 1));
            poles[(i, j)] = Point3::new(-2.0 + x, -2.0 + y, 0.4 * (x + 2.0 * y).sin());
        }
    }
    let s: Handle<dyn GeomSurface> = Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast();
    Handle::new(GeomAdaptorSurface::with_surface(&s).unwrap()).cast()
}

/// The plane z = 0.1 over [-3, 3]^2.
fn level() -> Adaptor3dSurfaceHandle {
    let axis = Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.1),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let s: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(axis)).cast();
    Handle::new(GeomAdaptorSurface::with_bounds(&s, (-3.0, 3.0, -3.0, 3.0)).unwrap()).cast()
}

/// The first walking line of the intersection of the two surfaces.
fn first_wline(s1: &Adaptor3dSurfaceHandle, s2: &Adaptor3dSurfaceHandle) -> Handle<IntPatchWLine> {
    let (d1, d2) = (
        Adaptor3dTopolTool::with_surface(s1).unwrap(),
        Adaptor3dTopolTool::with_surface(s2).unwrap(),
    );
    let mut inter = IntPatchIntersection::new();
    inter
        .perform(
            s1,
            &d1,
            s2,
            &d2,
            (1e-7, 1e-7),
            IntPatchIntersectionOptions::default(),
        )
        .unwrap();
    (1..=inter.nb_lines().unwrap())
        .map(|i| inter.line(i).unwrap().cast::<IntPatchWLine>())
        .find(|w| !w.is_null())
        .expect("a walking line")
}

#[test]
fn knot_tools_curvature_and_knots() {
    // A straight set: the curvature is null, the knots are the ends.
    let n = 12;
    let pars = {
        let mut p = Vector::filled(1, n, 0.0).unwrap();
        (1..=n).for_each(|i| p[i] = f64::from(i - 1) / f64::from(n - 1));
        p
    };
    let coords: Vec<f64> = (0..n)
        .flat_map(|i| [f64::from(i), 0.5 * f64::from(i), 0.0])
        .collect();
    let mut curv = Array1::<f64>::with_bounds(1, n).unwrap();
    assert_eq!(
        ApproxIntKnotTools::build_curvature(&coords, 3, &pars, &mut curv).unwrap(),
        0.0
    );
    let pts: Vec<Point3> = (0..n)
        .map(|i| Point3::new(f64::from(i), 0.5 * f64::from(i), 0.0))
        .collect();
    let xyz = Array1::from_shared(&pts, 1);
    let uv = Array1::<Point2>::with_bounds(1, n).unwrap();
    let sets = ApproxIntKnotPointSets {
        xyz: &xyz,
        u1v1: &uv,
        u2v2: &uv,
        approx_xyz: true,
        approx_u1v1: false,
        approx_u2v2: false,
    };
    // BuildKnots appends to its output without clearing it.
    let mut knots = vec![-7];
    ApproxIntKnotTools::build_knots(sets, &pars, 3, &mut knots).unwrap();
    assert_eq!(knots, vec![-7, 1, 12]);
    // No set used: the output is cleared.
    let none = ApproxIntKnotPointSets {
        approx_xyz: false,
        ..sets
    };
    ApproxIntKnotTools::build_knots(none, &pars, 3, &mut knots).unwrap();
    assert!(knots.is_empty());
}

#[test]
fn multi_line_values_offsets_and_status() {
    let points = Handle::new(IntSurfLineOn2S::new(None));
    for i in 0..5 {
        let mut p = IntSurfPntOn2S::new();
        let t = f64::from(i);
        p.set_value_all(Point3::new(t, 0.0, 0.0), t, 1.0, 2.0 * t, 3.0);
        points.add(&p);
    }
    let wline = Handle::new(IntPatchWLine::new_undecided(&points, false));
    let layout = ApproxIntMultiLineLayout {
        nb_p3d: 1,
        nb_p2d: 1,
        approx_u1v1: false,
        approx_u2v2: true,
        offsets: ApproxIntOffsets {
            xo: 1.0,
            u2o: -1.0,
            ..Default::default()
        },
        p2d_on_first: false,
        ind_min: 5,
        ind_max: 2,
    };
    let line = GeomIntTheMultiLineOfWLApprox::with_line(wline, layout);
    assert_eq!((line.first_point(), line.last_point()), (2, 5));
    assert_eq!(line.what_status(), Status::NoPointsAdded);
    let mut p3 = Array1::<Point3>::with_bounds(1, 1).unwrap();
    let mut p2 = Array1::<Point2>::with_bounds(1, 1).unwrap();
    LineTool::value(&line, 3, &mut p3, &mut p2).unwrap();
    assert_eq!(p3[1], Point3::new(3.0, 0.0, 0.0));
    assert_eq!(p2[1], Point2::new(3.0, 3.0));
    // Without SvSurfaces: no tangent, no point added, an empty sub-line.
    let mut v = Array1::<Vector3>::with_bounds(1, 1).unwrap();
    assert!(!line.tangency_3d(3, &mut v).unwrap());
    assert!(line.make_ml_one_more_point(2, 5, 3).unwrap().is_none());
    let empty = line.make_ml_between(2, 5, 4).unwrap();
    assert_eq!((empty.first_point(), empty.last_point()), (1, 1));
    assert!(line.value_3d(9, &mut p3).is_err());
    let mut text = String::new();
    line.dump(&mut text).unwrap();
    assert_eq!(text.lines().count(), 4);
}

#[test]
fn prm_prm_sv_surfaces_compute_and_cache() {
    let (s1, s2) = (bump(), level());
    let w = first_wline(&s1, &s2);
    let (u1, v1, u2, v2) = w.point(3).unwrap().parameters();
    let mut sv = GeomIntThePrmPrmSvSurfacesOfWLApprox::new(&s1, &s2).unwrap();
    assert!(!sv.get_use_solver());
    let mut uv = [u1, v1, u2, v2];
    let mut out = ApproxIntSvComputed::default();
    assert!(sv.compute(&mut uv, &mut out).unwrap());
    assert!(
        (out.point.z() - 0.1).abs() < 1e-6,
        "the point is on the plane: {:?}",
        out.point
    );
    assert!((out.tangent.magnitude() - 1.0).abs() < 1e-12);
    // The same parameters hit the stored result.
    let (flag, tg) = sv.tangency([u1, v1, u2, v2]).unwrap();
    assert!(flag);
    assert_eq!(tg, out.tangent);
    let (_, t2) = sv.tangency_on_surf2([u1, v1, u2, v2]).unwrap();
    assert_eq!(t2, out.tangent_uv2);
    assert!(sv.seek_point([u1, v1, u2, v2]).unwrap().is_some());
}

#[test]
fn imp_prm_sv_surfaces_compute_seek_and_start() {
    let (s1, s2) = (level(), bump());
    let w = first_wline(&s1, &s2);
    let (u1, v1, u2, v2) = w.point(4).unwrap().parameters();
    let quadric = IntSurfQuadric::from_plane(&s1.plane().unwrap());
    let mut sv = GeomIntTheImpPrmSvSurfacesOfWLApprox::quadric_first(&quadric, &s2);
    assert!(sv.get_use_solver());
    let mut out = ApproxIntSvComputed::default();
    let mut uv = [u1, v1, u2, v2];
    assert!(sv.compute(&mut uv, &mut out).unwrap());
    assert!((out.point.z() - 0.1).abs() < 1e-6);
    let p = sv.seek_point([u1, v1, u2, v2]).unwrap().expect("a point");
    assert!((p.value().z() - 0.1).abs() < 1e-6);
    let bounds = PatchBounds {
        binfu: 0.0,
        bsupu: 1.0,
        binfv: 0.0,
        bsupv: 1.0,
    };
    let mut x = Vector::filled(1, 2, 0.0).unwrap();
    // A start point outside a non-periodic range fails.
    assert!(
        sv.fill_initial_vector_of_solution([0.0, 0.0, 3.0, 0.5], bounds, &mut x)
            .unwrap()
            .is_none()
    );
    let shift = sv
        .fill_initial_vector_of_solution([0.0, 0.0, 0.5, 0.5], bounds, &mut x)
        .unwrap();
    assert_eq!(shift, Some((0.0, 0.0)));
    assert_eq!((x[1], x[2]), (0.5, 0.5));
}

#[test]
fn multi_line_with_sv_surfaces_adds_points() {
    let (s1, s2) = (bump(), level());
    let w = first_wline(&s1, &s2);
    let n = w.nb_pnts().unwrap();
    let sv: SvSurfacesRef = Rc::new(RefCell::new(
        GeomIntThePrmPrmSvSurfacesOfWLApprox::new(&s1, &s2).unwrap(),
    ));
    let layout = ApproxIntMultiLineLayout {
        nb_p3d: 1,
        nb_p2d: 2,
        approx_u1v1: true,
        approx_u2v2: true,
        p2d_on_first: true,
        ind_min: 1,
        ind_max: n,
        ..Default::default()
    };
    let line = GeomIntTheMultiLineOfWLApprox::with_sv_surfaces(w, Some(sv.clone()), layout);
    assert_eq!(line.what_status(), Status::PointsAdded);
    let mut v = Array1::<Vector3>::with_bounds(1, 1).unwrap();
    let mut v2 = Array1::<Vector2>::with_bounds(1, 2).unwrap();
    assert!(line.tangency(2, &mut v, &mut v2).unwrap());
    let between = line.make_ml_between(1, n.min(6), 5).unwrap();
    assert!(between.last_point() >= between.first_point());
    // The solver switch is restored after the insertion.
    assert!(!sv.borrow().get_use_solver());
}

#[test]
fn wlapprox_on_surfaces_and_on_the_line() {
    let (s1, s2) = (bump(), level());
    let w = first_wline(&s1, &s2);
    let n = w.nb_pnts().unwrap();
    let mut approx = GeomIntWLApprox::new();
    assert_eq!(approx.nb_multi_curves(), 1);
    approx.set_parameters(
        1e-7,
        1e-7,
        4,
        8,
        0,
        30,
        true,
        ParametrizationType::ChordLength,
    );
    let range = ApproxIntRange {
        indicemin: 1,
        indicemax: n,
        ..Default::default()
    };
    approx.perform_on_surfaces(&s1, &s2, &w, range).unwrap();
    assert!(approx.is_done());
    assert!(approx.tol_reached3d() >= 0.0);
    assert!(approx.value(1).nb_poles() >= 2);
    approx
        .perform(
            &w,
            ApproxIntRange {
                approx_u1v1: false,
                approx_u2v2: false,
                ..range
            },
        )
        .unwrap();
    assert!(approx.is_done());
    // The chord-length parameters run from 0 to 1.
    let line = GeomIntTheMultiLineOfWLApprox::with_line(
        w.clone(),
        ApproxIntMultiLineLayout {
            nb_p3d: 1,
            ind_min: 1,
            ind_max: n,
            ..Default::default()
        },
    );
    let mut pars = Vector::filled(1, n, 0.0).unwrap();
    GeomIntWLApprox::parameters(&line, 1, n, ParametrizationType::ChordLength, &mut pars).unwrap();
    assert_eq!((pars[1], pars[n]), (0.0, 1.0));
    let kind = ApproxIntKnotTools::define_par_type(&w, 1, 2, (true, true, true)).unwrap();
    assert_eq!(kind, ParametrizationType::IsoParametric);
}
