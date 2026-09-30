// Copyright (c) 2026 OPEN CASCADE SAS
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

//! The cases of GeomFill_CorrectedFrenet_Test.cxx (at 3d097a0328e71b826377d4814ab05ec3c3d23871)
//! that need only GeomFill. The suite is not mapped: its fourth case builds a wire with
//! BRepBuilderAPI and ShapeExtend and adapts it with BRepAdaptor_CompCurve, which are not
//! ported (and TKShHealing is not in the native oracle closure).
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::geom_fill::{GeomFillCorrectedFrenet, GeomFillTrihedronLaw, TrihedronD0};
use rustycad_tkmath::gp::Point3;

/// An adaptor on the B-spline of `poles` on `[0, 1]` without inner knots.
fn bspline_adaptor(poles: &[Point3], degree: i32) -> Handle<dyn Adaptor3dCurve> {
    let mut p: Array1<'static, Point3> = Array1::with_bounds(1, poles.len() as i32).unwrap();
    p.as_mut_slice().copy_from_slice(poles);
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    knots[2] = 1.0;
    let mut mults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    mults.init(degree + 1);
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomBSplineCurve::new(&p, &knots, &mults, degree, false).unwrap()).cast();
    Handle::new(GeomAdaptorCurve::with_curve(&curve).unwrap()).cast()
}

fn frame_at(law: &GeomFillCorrectedFrenet, param: f64) -> TrihedronD0 {
    let mut frame = TrihedronD0::default();
    law.d0(param, &mut frame).unwrap();
    frame
}

fn law_on(poles: &[Point3], degree: i32) -> GeomFillCorrectedFrenet {
    let law = GeomFillCorrectedFrenet::with_evaluation(false);
    law.set_curve(&bspline_adaptor(poles, degree)).unwrap();
    law
}

#[test]
fn corrected_frenet_ends_of_a_turning_cubic_have_frames() {
    let poles = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let law = law_on(&poles, 3);
    for param in [0.0, 1.0] {
        let f = frame_at(&law, param);
        assert!(f.tangent.magnitude() > 1e-10);
        assert!(f.normal.magnitude() > 1e-10);
        assert!(f.binormal.magnitude() > 1e-10);
    }
}

#[test]
fn corrected_frenet_evaluates_a_tiny_segment() {
    let poles = [Point3::new(0.0, 0.0, 0.0), Point3::new(1e-10, 0.0, 0.0)];
    let law = law_on(&poles, 1);
    let mut frame = TrihedronD0::default();
    assert!(law.d0(0.5, &mut frame).is_ok());
}

#[test]
fn corrected_frenet_parameters_progress_on_a_parabola() {
    let poles = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.5, 0.5, 0.0),
        Point3::new(1.0, 0.0, 0.0),
    ];
    let law = law_on(&poles, 2);
    let mut param = 0.1;
    while param <= 0.9 {
        let f = frame_at(&law, param);
        assert!(f.tangent.magnitude() > 1e-12);
        assert!(f.normal.magnitude() > 1e-12);
        assert!(f.binormal.magnitude() > 1e-12);
        param += 0.1;
    }
}
