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

//! Behavior of the IntImp port beyond the numeric probe, which compares the templates
//! with their native TKGeomAlgo instances.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeomalgo::int_imp::*;
use rustycad_tkgeomalgo::int_surf::{IntSurfQuadric, IntSurfQuadricTool};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

type Curve = Handle<dyn Adaptor3dCurve>;

/// `IntCurveSurface_TheHCurveTool`, the curve tool of the IntCurveSurface instances.
use rustycad_tkgeomalgo::int_curve_surface::TheHCurveTool as HCurveTool;

fn adapted(
    surface: Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
) -> Adaptor3dSurfaceHandle {
    Handle::new(GeomAdaptorSurface::with_bounds(&surface, bounds).unwrap()).cast()
}

fn plane_at(z: f64) -> Adaptor3dSurfaceHandle {
    let ax = Axis3::from_normal(
        Point3::new(0.0, 0.0, z),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    adapted(
        Handle::new(GeomPlane::new(ax)).cast(),
        (-2.0, 2.0, -2.0, 2.0),
    )
}

fn sphere(radius: f64) -> Adaptor3dSurfaceHandle {
    let s = GeomSphericalSurface::new(Axis3::default(), radius).unwrap();
    let half = std::f64::consts::FRAC_PI_2;
    adapted(
        Handle::new(s).cast(),
        (0.0, 2.0 * std::f64::consts::PI, -half, half),
    )
}

#[test]
fn choix_ref_reads_the_isoparametric_table() {
    assert_eq!(
        choix_ref(0).unwrap(),
        IntImpConstIsoparametric::UIsoparametricOnCaro1
    );
    assert_eq!(
        choix_ref(3).unwrap(),
        IntImpConstIsoparametric::VIsoparametricOnCaro2
    );
    let error = choix_ref(4).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(error.message().contains('4'));
}

#[test]
fn perpendicular_patches_sort_their_isoparametrics() {
    let x = Vector3::new(1.0, 0.0, 0.0);
    let (y, z) = (Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0));
    let (mut tg, mut iso) = ([0.0; 4], [IntImpConstIsoparametric::default(); 4]);
    assert!(!compute_tangence(
        &[x, y, x, z],
        &[1e-7; 4],
        &mut tg,
        &mut iso
    ));
    // The intersection runs along X, the U direction of both patches: the U
    // isoparametrics (cosine 0) come first.
    assert_eq!(tg, [1.0, 0.0, 1.0, -0.0]);
    assert_eq!(iso[0], IntImpConstIsoparametric::UIsoparametricOnCaro1);
    assert_eq!(iso[1], IntImpConstIsoparametric::UIsoparametricOnCaro2);
    // A null derivative answers tangent and writes nothing.
    let mut untouched = [7.0; 4];
    assert!(compute_tangence(
        &[x, Vector3::default(), x, z],
        &[1e-7; 4],
        &mut untouched,
        &mut iso
    ));
    assert_eq!(untouched, [7.0; 4]);
}

#[test]
fn an_implicit_function_needs_its_surfaces() {
    let mut f = IntImpZerImpFunc::<HSurfaceTool, IntSurfQuadricTool>::new();
    let x = Vector::filled(1, 2, 0.0).unwrap();
    let mut v = Vector::filled(1, 1, 0.0).unwrap();
    let error = f.value(&x, &mut v).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    f.set_surface(&plane_at(0.0));
    f.set_implicit_surface(&IntSurfQuadric::new());
    assert!(f.value(&x, &mut v).unwrap());
    let mut d = Matrix::filled(1, 1, 1, 2, 0.0).unwrap();
    assert!(f.derivatives(&x, &mut d).unwrap());
}

#[test]
fn a_line_meets_a_plane_at_its_foot() {
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::new(0.2, 0.1, 2.0),
        Direction3::new(0.0, 0.0, -1.0).unwrap(),
    ))
    .cast();
    let curve: Curve = Handle::new(GeomAdaptorCurve::with_range(&line, 0.0, 4.0).unwrap()).cast();
    let f = IntImpZerCSParFunc::<HSurfaceTool, HCurveTool>::new(&plane_at(0.0), &curve);
    let e = IntImpIntCS::new([0.0, 0.0, 1.0], &f, 1e-7, 0.0).unwrap();
    assert!(e.is_done() && !e.is_empty().unwrap());
    let (u, v) = e.parameter_on_surface().unwrap();
    assert!((u - 0.2).abs() <= 1e-12 && (v - 0.1).abs() <= 1e-12);
    assert!((e.parameter_on_curve().unwrap() - 2.0).abs() <= 1e-12);
    let empty = IntImpIntCS::from_function(&f, 1e-7);
    assert_eq!(
        empty.point().unwrap_err().exception_type(),
        "Standard_DomainError"
    );
}

#[test]
fn a_walking_point_lies_on_both_patches() {
    let mut param: Array1<'static, f64> = Array1::with_bounds(1, 4).unwrap();
    (param[1], param[2], param[3], param[4]) = (1.2, 0.5, 0.4, 0.25);
    let s = IntImpInt2S::<HSurfaceTool>::with_start(&param, &plane_at(0.3), &sphere(1.5), 1e-6)
        .unwrap();
    assert!(s.is_done() && !s.is_empty().unwrap());
    let p = s.point().unwrap().value();
    assert!((p.z() - 0.3).abs() <= 1e-9, "{}", p.z());
    assert!((p.distance(&Point3::default()) - 1.5).abs() <= 1e-9);
    assert!(!s.is_tangent().unwrap());
    // The tangent of a plane-sphere circle is horizontal.
    assert!(s.direction().unwrap().z().abs() <= 1e-9);
}
