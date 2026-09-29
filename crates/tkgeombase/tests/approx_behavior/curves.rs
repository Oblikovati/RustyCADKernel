// Copyright (c) 1997-1999 Matra Datavision
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

//! The adaptor approximations: Curve3d, Curve2d, CurveOnSurface, CurvlinFunc,
//! CurvilinearParameter and SameParameter.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::approx::{
    Curve2d, Curve3d, CurveOnSurface, CurveOnSurfaceOptions, CurvilinearOptions,
    CurvilinearParameter, CurvlinFunc, SameParameter,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3};

fn line_3d() -> Handle<dyn GeomCurve> {
    let direction = Direction3::new(3.0, 4.0, 0.0).unwrap();
    Handle::new(GeomLine::from_point(Point3::new(0.0, 0.0, 0.0), direction)).cast()
}

fn line_2d() -> Handle<dyn Geom2dCurve> {
    let direction = Direction2::new(3.0, 4.0).unwrap();
    Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), direction)).cast()
}

fn plane() -> Handle<dyn GeomSurface> {
    let normal = Direction3::new(0.0, 0.0, 1.0).unwrap();
    Handle::new(GeomPlane::from_point(Point3::new(0.0, 0.0, 0.0), normal).unwrap()).cast()
}

fn adapted_3d(first: f64, last: f64) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    Ok(Handle::new(GeomAdaptorCurve::with_range(&line_3d(), first, last)?).cast())
}

fn adapted_2d(first: f64, last: f64) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
    Ok(Handle::new(Geom2dAdaptorCurve::with_range(&line_2d(), first, last)?).cast())
}

fn adapted_plane() -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
    Ok(Handle::new(GeomAdaptorSurface::with_surface(&plane())?).cast())
}

fn cos_options(only_3d: bool, only_2d: bool) -> CurveOnSurfaceOptions {
    CurveOnSurfaceOptions {
        max_segments: 16,
        max_degree: 14,
        continuity: Shape::C2,
        only_3d,
        only_2d,
    }
}

#[test]
fn curve3d_approximates_a_segment() -> Result<(), Failure> {
    let approx = Curve3d::new(&adapted_3d(0.0, 5.0)?, 1.0e-6, Shape::C2, 10, 8)?;
    assert!(approx.is_done() && approx.has_result());
    assert!(approx.max_error() < 1.0e-6);
    assert!(!approx.curve().is_null());
    Ok(())
}

#[test]
fn curve2d_approximates_a_segment() -> Result<(), Failure> {
    let approx = Curve2d::new(
        &adapted_2d(0.0, 5.0)?,
        0.0,
        5.0,
        1.0e-6,
        1.0e-6,
        Shape::C1,
        8,
        10,
    )?;
    assert!(approx.is_done() && approx.has_result());
    assert!(approx.max_error_2d_u() < 1.0e-6 && approx.max_error_2d_v() < 1.0e-6);
    Ok(())
}

#[test]
fn curve_on_surface_refuses_only_3d_and_only_2d() -> Result<(), Failure> {
    let mut approx =
        CurveOnSurface::without_perform(&adapted_2d(0., 5.)?, &adapted_plane()?, 0., 5., 1e-6);
    assert!(!approx.is_done() && approx.curve_3d().is_null() && approx.curve_2d().is_null());
    let failure = approx.perform(cos_options(true, true)).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    Ok(())
}

#[test]
fn curve_on_surface_builds_both_curves() -> Result<(), Failure> {
    let approx = CurveOnSurface::new(
        &adapted_2d(0., 5.)?,
        &adapted_plane()?,
        0.,
        5.,
        1e-6,
        cos_options(false, false),
    )?;
    assert!(approx.is_done() && approx.has_result());
    assert!(!approx.curve_3d().is_null() && !approx.curve_2d().is_null());
    assert!(approx.max_error_3d() < 1.0e-6);
    Ok(())
}

#[test]
fn curve_on_surface_only_2d_leaves_the_3d_curve_null() -> Result<(), Failure> {
    let approx = CurveOnSurface::new(
        &adapted_2d(0., 5.)?,
        &adapted_plane()?,
        0.,
        5.,
        1e-6,
        cos_options(false, true),
    )?;
    assert!(approx.is_done());
    assert!(approx.curve_3d().is_null() && !approx.curve_2d().is_null());
    Ok(())
}

fn segment_function() -> Result<CurvlinFunc, Failure> {
    CurvlinFunc::from_curve(&adapted_3d(0.0, 5.0)?, 1.0e-7)
}

#[test]
fn curvlin_func_measures_and_maps_a_segment() -> Result<(), Failure> {
    let f = segment_function()?;
    assert_eq!((f.first_parameter(), f.last_parameter()), (0.0, 1.0));
    assert!((f.get_length() - 5.0).abs() < 1.0e-9);
    assert!((f.get_s_parameter(2.5)? - 0.5).abs() < 1.0e-9);
    let c = adapted_3d(0.0, 5.0)?;
    assert!((f.get_u_parameter(c.get().unwrap(), 0.25, 1)? - 1.25).abs() < 1.0e-7);
    Ok(())
}

#[test]
fn curvlin_func_rejects_abscissas_outside_the_unit_range() -> Result<(), Failure> {
    let f = segment_function()?;
    let c = adapted_3d(0.0, 5.0)?;
    let failure = f.get_u_parameter(c.get().unwrap(), 1.5, 1).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    assert_eq!(
        f.trim(-0.5, 0.5, 1e-9).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    Ok(())
}

#[test]
fn curvlin_func_evaluates_only_its_own_case() -> Result<(), Failure> {
    let f = segment_function()?;
    let mut result = Array1::with_bounds(0, 6).unwrap();
    let failure = f.eval_case2(0.5, 0, &mut result).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    // An order beyond 2 returns false and zeros.
    result.init(-1.0);
    assert!(!f.eval_case1(0.5, 3, &mut result)?);
    assert_eq!((result[0], result[1], result[2]), (0.0, 0.0, 0.0));
    assert!(f.eval_case1(0.5, 0, &mut result)?);
    assert!((result[0] - 1.5).abs() < 1.0e-7 && (result[1] - 2.0).abs() < 1.0e-7);
    Ok(())
}

#[test]
fn curvilinear_parameter_reparametrizes_a_segment() -> Result<(), Failure> {
    let options = CurvilinearOptions {
        order: Shape::C1,
        max_degree: 8,
        max_segments: 10,
    };
    let approx = CurvilinearParameter::from_curve(&adapted_3d(0.0, 5.0)?, 1.0e-6, options)?;
    assert!(approx.is_done() && approx.has_result());
    assert!(approx.curve_2d1().is_null() && approx.curve_2d2().is_null());
    let curve = approx.curve_3d();
    let curve = curve.get().unwrap();
    // The result is parametrized by the normalized abscissa, not the length.
    assert_eq!(
        (curve.first_parameter(), curve.last_parameter()),
        (0.0, 1.0)
    );
    let end = curve.value(1.0)?;
    assert!((end.x() - 3.0).abs() < 1.0e-6 && (end.y() - 4.0).abs() < 1.0e-6);
    Ok(())
}

#[test]
fn same_parameter_keeps_a_pcurve_that_already_matches() -> Result<(), Failure> {
    // Quirk: an already same-parameter pcurve leaves Curve2d() null.
    let approx =
        SameParameter::from_geometry(&line_3d_trimmed()?, &line_2d_trimmed()?, &plane(), 1e-7)?;
    assert!(approx.is_done() && approx.is_same_parameter());
    assert!(approx.curve_2d().is_null());
    assert!(approx.tol_reached() <= 1.0e-7);
    Ok(())
}

#[test]
fn same_parameter_reparametrizes_a_non_uniform_pcurve() -> Result<(), Failure> {
    // A quadratic Bezier on the segment: same geometry, another speed.
    let mut poles = Array1::with_bounds(1, 3).unwrap();
    (poles[1], poles[2], poles[3]) = (
        Point2::new(0., 0.),
        Point2::new(0.3, 0.4),
        Point2::new(3., 4.),
    );
    let c2d: Handle<dyn Geom2dCurve> = Handle::new(Geom2dBezierCurve::new(&poles)?).cast();
    let c3d = trimmed_3d(line_3d(), 0.0, 5.0)?;
    let approx = SameParameter::from_geometry(&c3d, &c2d, &plane(), 1e-7)?;
    assert!(approx.is_done() && !approx.curve_2d().is_null());
    assert!(!approx.curve_on_surface().is_null());
    Ok(())
}

fn trimmed_3d(
    c: Handle<dyn GeomCurve>,
    u1: f64,
    u2: f64,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(Handle::new(rustycad_tkg3d::geom::GeomTrimmedCurve::new(
        &c, u1, u2, true, true,
    )?)
    .cast())
}

fn trimmed_2d(
    c: Handle<dyn Geom2dCurve>,
    u1: f64,
    u2: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(Handle::new(rustycad_tkg2d::geom2d::Geom2dTrimmedCurve::new(
        &c, u1, u2, true, true,
    )?)
    .cast())
}

fn line_3d_trimmed() -> Result<Handle<dyn GeomCurve>, Failure> {
    trimmed_3d(line_3d(), 0.0, 5.0)
}

fn line_2d_trimmed() -> Result<Handle<dyn Geom2dCurve>, Failure> {
    trimmed_2d(line_2d(), 0.0, 5.0)
}
