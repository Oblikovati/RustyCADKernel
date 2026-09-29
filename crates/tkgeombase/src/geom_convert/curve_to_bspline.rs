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
// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert.cxx
// (CurveToBSplineCurve, BSplineCurveBuilder) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert::CurveToBSplineCurve`: the B-spline of a bounded Geom curve.
use super::approx_curve::ApproxCurve;
use super::comp_curve::CompCurveToBSplineCurve;
use super::live_bspline;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomBoundedCurve, GeomCircle, GeomConic, GeomCurve,
    GeomEllipse, GeomGeometry, GeomHyperbola, GeomLine, GeomOffsetCurve, GeomParabola,
    GeomTrimmedCurve,
};
use rustycad_tkmath::convert::{ConicToBSplineCurve, ParameterisationType};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{
    self, Axis3, Circle2, Ellipse2, Hyperbola2, Parabola2, Point2, Point3, Transform3,
};

/// `GeomConvert::CurveToBSplineCurve(C, Parameterisation = Convert_TgtThetaOver2)`:
/// the B-spline of a circle, an ellipse, a Bezier, B-spline or offset curve, or of a
/// trimmed line, conic, Bezier, B-spline or offset curve.
///
/// `parameterisation` applies to circles and ellipses only; offset curves are
/// approximated (tolerance 1e-4, C2, 16 segments, degree 14) and raise
/// Standard_ConstructionError when the approximation has no result. A trimmed
/// periodic B-spline whose bounds are its whole period (within
/// Precision::Confusion()) is made non-periodic first (OCC7355). Any other curve
/// raises Standard_DomainError ("No such curve");
/// `geom_convert::curve_to_bspline_curve(&circle, ParameterisationType::TgtThetaOver2)?`.
pub fn curve_to_bspline_curve(
    curve: &Handle<dyn GeomCurve>,
    parameterisation: ParameterisationType,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    if curve.is_null() {
        return Err(failure(
            "Standard_NullObject",
            "GeomConvert::CurveToBSplineCurve: null curve; expected a Geom_Curve",
        ));
    }
    let trimmed: Handle<GeomTrimmedCurve> = curve.cast();
    match trimmed.get() {
        Some(ctrim) => trimmed_to_bspline(curve, ctrim, parameterisation),
        None => whole_to_bspline(curve, parameterisation),
    }
}

// The trimmed-curve branch: the bounds are clamped to a non-periodic basis, so that
// the basis Segment cannot raise.
fn trimmed_to_bspline(
    curve: &Handle<dyn GeomCurve>,
    ctrim: &GeomTrimmedCurve,
    parameterisation: ParameterisationType,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let basis: Handle<dyn GeomCurve> = ctrim.basis_curve();
    let (mut u1, mut u2) = (ctrim.first_parameter(), ctrim.last_parameter());
    if !basis.is_periodic() {
        if u1 < basis.first_parameter() {
            u1 = basis.first_parameter();
        }
        if u2 > basis.last_parameter() {
            u2 = basis.last_parameter();
        }
    }
    if basis.cast::<GeomLine>().is_some() {
        return trimmed_line(ctrim);
    }
    if let Some(circle) = basis.cast::<GeomCircle>().get() {
        let c2d = Circle2::from_x_axis(gp::ox2d(), circle.radius(), true)?;
        let convert =
            |a: f64, b: f64| ConicToBSplineCurve::from_circle_arc(&c2d, a, b, parameterisation);
        return elliptic_arc(circle, [u1, u2], parameterisation, &convert);
    }
    if let Some(ellipse) = basis.cast::<GeomEllipse>().get() {
        let e2d = Ellipse2::from_x_axis(
            gp::ox2d(),
            ellipse.major_radius(),
            ellipse.minor_radius(),
            true,
        )?;
        let convert =
            |a: f64, b: f64| ConicToBSplineCurve::from_ellipse_arc(&e2d, a, b, parameterisation);
        return elliptic_arc(ellipse, [u1, u2], parameterisation, &convert);
    }
    trimmed_other(curve, &basis, [u1, u2])
}

// The hyperbola, parabola, Bezier, B-spline and offset bases of a trimmed curve.
fn trimmed_other(
    curve: &Handle<dyn GeomCurve>,
    basis: &Handle<dyn GeomCurve>,
    [u1, u2]: [f64; 2],
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    if let Some(hyperbola) = basis.cast::<GeomHyperbola>().get() {
        let h2d = Hyperbola2::from_x_axis(
            gp::ox2d(),
            hyperbola.major_radius(),
            hyperbola.minor_radius(),
            true,
        )?;
        return bspline_curve_builder(
            hyperbola,
            &ConicToBSplineCurve::from_hyperbola(&h2d, u1, u2)?,
        );
    }
    if let Some(parabola) = basis.cast::<GeomParabola>().get() {
        let p2d = Parabola2::from_mirror_axis(gp::ox2d(), parabola.focal(), true)?;
        return bspline_curve_builder(parabola, &ConicToBSplineCurve::from_parabola(&p2d, u1, u2)?);
    }
    if basis.cast::<GeomBezierCurve>().is_some() {
        let bezier: Handle<GeomBezierCurve> = basis.copy()?.cast();
        live_bezier(&bezier)?.segment(u1, u2)?;
        return bezier_to_bspline(&bezier);
    }
    if basis.cast::<GeomBSplineCurve>().is_some() {
        return trimmed_bspline(basis, [u1, u2]);
    }
    if basis.cast::<GeomOffsetCurve>().is_some() {
        // The native code approximates the trimmed curve itself, not its basis.
        return approximate_offset(curve);
    }
    Err(failure("Standard_DomainError", "No such curve"))
}

// A copy of the B-spline basis, made non-periodic when the trim is exactly its
// period (modified by jgv, 14.01.05 for OCC7355), then segmented.
fn trimmed_bspline(
    basis: &Handle<dyn GeomCurve>,
    [mut u1, mut u2]: [f64; 2],
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let the_curve: Handle<GeomBSplineCurve> = basis.copy()?.cast();
    let bspline: &GeomBSplineCurve = live_bspline(&the_curve)?;
    if bspline.is_periodic() {
        let (uf, ul): (f64, f64) = (bspline.first_parameter(), bspline.last_parameter());
        (u1, u2) = elclib::adjust_periodic(uf, ul, precision::confusion(), u1, u2);
        if (u1 - uf).abs() <= precision::confusion() && (u2 - ul).abs() <= precision::confusion() {
            bspline.set_not_periodic()?;
        }
    }
    bspline.segment(u1, u2, precision::p_confusion())?;
    Ok(the_curve)
}

// A whole (untrimmed) ellipse, circle, Bezier, B-spline or offset curve.
fn whole_to_bspline(
    curve: &Handle<dyn GeomCurve>,
    parameterisation: ParameterisationType,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    if let Some(ellipse) = curve.cast::<GeomEllipse>().get() {
        let e2d = Ellipse2::from_x_axis(
            gp::ox2d(),
            ellipse.major_radius(),
            ellipse.minor_radius(),
            true,
        )?;
        let convert = ConicToBSplineCurve::from_ellipse(&e2d, parameterisation)?;
        return periodic(bspline_curve_builder(ellipse, &convert)?);
    }
    if let Some(circle) = curve.cast::<GeomCircle>().get() {
        let c2d = Circle2::from_x_axis(gp::ox2d(), circle.radius(), true)?;
        let convert = ConicToBSplineCurve::from_circle(&c2d, parameterisation)?;
        return periodic(bspline_curve_builder(circle, &convert)?);
    }
    if curve.cast::<GeomBezierCurve>().is_some() {
        return bezier_to_bspline(&curve.cast());
    }
    if curve.cast::<GeomBSplineCurve>().is_some() {
        return Ok(curve.copy()?.cast());
    }
    if curve.cast::<GeomOffsetCurve>().is_some() {
        return approximate_offset(curve);
    }
    Err(failure("Standard_DomainError", "No such curve"))
}

// SetPeriodic on a whole conic, for the polynomial and quasi-angular
// parameterisations as for the others.
fn periodic(bspline: Handle<GeomBSplineCurve>) -> Result<Handle<GeomBSplineCurve>, Failure> {
    live_bspline(&bspline)?.set_periodic()?;
    Ok(bspline)
}

// The trimmed line: its two end points, the trimmed curve's own bounds as knots
// (not the clamped ones) and degree 1.
fn trimmed_line(ctrim: &GeomTrimmedCurve) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let mut poles: Array1<'static, Point3> = one_based(2)?;
    poles[1] = ctrim.start_point()?;
    poles[2] = ctrim.end_point()?;
    let mut knots: Array1<'static, f64> = one_based(2)?;
    (knots[1], knots[2]) = (ctrim.first_parameter(), ctrim.last_parameter());
    let mut mults: Array1<'static, i32> = one_based(2)?;
    (mults[1], mults[2]) = (2, 2);
    Ok(Handle::new(GeomBSplineCurve::new(
        &poles, &knots, &mults, 1, false,
    )?))
}

// A trimmed circle or ellipse. With RationalC1, an arc of 6 radians or more is
// converted as two halves joined by CompCurveToBSplineCurve, to avoid the numerical
// overflow of one conversion near 2 PI.
fn elliptic_arc(
    conic: &dyn GeomConic,
    [u1, u2]: [f64; 2],
    parameterisation: ParameterisationType,
    convert: &dyn Fn(f64, f64) -> Result<ConicToBSplineCurve, Failure>,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    if parameterisation != ParameterisationType::RationalC1 || u2 - u1 < 6.0 {
        return bspline_curve_builder(conic, &convert(u1, u2)?);
    }
    let umed: f64 = (u1 + u2) * 0.5;
    let first: Handle<GeomBSplineCurve> = bspline_curve_builder(conic, &convert(u1, umed)?)?;
    let second: Handle<GeomBSplineCurve> = bspline_curve_builder(conic, &convert(umed, u2)?)?;
    let mut joined = CompCurveToBSplineCurve::with_curve(
        &first.cast::<dyn GeomBoundedCurve>(),
        parameterisation,
    )?;
    joined.add(
        &second.cast::<dyn GeomBoundedCurve>(),
        precision::p_confusion(),
        true,
        true,
        0,
    )?;
    Ok(joined.bspline_curve())
}

/// `BSplineCurveBuilder`: the planar B-spline of a conversion made in the reference
/// frame, lifted to z = 0 and moved from gp::XOY() to the conic's position.
///
/// Unlike the 2D builder, the 3D one has no mirror step: the conic's gp_Ax2 is
/// always right-handed.
fn bspline_curve_builder(
    conic: &dyn GeomConic,
    convert: &ConicToBSplineCurve,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let poles_2d: &Array1<'static, Point2> = convert.poles();
    let mut poles: Array1<'static, Point3> = one_based(poles_2d.length())?;
    for i in poles_2d.lower()..=poles_2d.upper() {
        let p2d: Point2 = poles_2d[i];
        poles[i] = Point3::new(p2d.x(), p2d.y(), 0.0);
    }
    let the_curve = GeomBSplineCurve::with_weights(
        &poles,
        convert.weights(),
        convert.knots(),
        convert.multiplicities(),
        convert.degree(),
        convert.is_periodic(),
    )?;
    let mut t = Transform3::default();
    t.set_transformation_between(
        Axis3::from_axis2(conic.position()),
        Axis3::from_axis2(gp::xoy()),
    );
    Ok(the_curve.transformed(t)?.cast())
}

// A Bezier curve as a single-span B-spline on [0, 1].
pub(super) fn bezier_to_bspline(
    bezier: &Handle<GeomBezierCurve>,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let bezier: &GeomBezierCurve = live_bezier(bezier)?;
    let degree: i32 = bezier.degree();
    let poles: Array1<'static, Point3> = bezier.poles();
    let mut knots: Array1<'static, f64> = one_based(2)?;
    (knots[1], knots[2]) = (0.0, 1.0);
    let mut mults: Array1<'static, i32> = one_based(2)?;
    (mults[1], mults[2]) = (degree + 1, degree + 1);
    let bspline = match bezier.weights() {
        Some(weights) if bezier.is_rational() => {
            GeomBSplineCurve::with_weights(&poles, &weights, &knots, &mults, degree, false)?
        }
        _ => GeomBSplineCurve::new(&poles, &knots, &mults, degree, false)?,
    };
    Ok(Handle::new(bspline))
}

fn live_bezier(bezier: &Handle<GeomBezierCurve>) -> Result<&GeomBezierCurve, Failure> {
    bezier.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "GeomConvert: null Geom_BezierCurve; expected a curve",
        )
    })
}

// The offset-curve approximation of CurveToBSplineCurve.
fn approximate_offset(curve: &Handle<dyn GeomCurve>) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let (tol3d, order, max_segments, max_degree): (f64, Shape, i32, i32) =
        (1.0e-4, Shape::C2, 16, 14);
    let approx = ApproxCurve::from_curve(curve, tol3d, order, max_segments, max_degree)?;
    if approx.has_result() {
        return Ok(approx.curve());
    }
    Err(failure(
        "Standard_ConstructionError",
        "GeomConvert::CurveToBSplineCurve: the offset curve approximation has no result",
    ))
}
