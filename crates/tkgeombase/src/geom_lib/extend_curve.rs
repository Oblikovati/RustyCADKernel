// Created on: 1993-07-07
// Created by: Jean Claude VAUTHIER
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx (ExtendCurveToPoint)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib::ExtendCurveToPoint`: a Bezier extension of a bounded curve to a
//! point, joined to it with the required continuity.
use super::compute_lambda::compute_lambda;
use super::live;
use crate::geom_convert::CompCurveToBSplineCurve;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomBoundedCurve, GeomCurveD3};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};
use rustycad_tkmath::math::Matrix;
use rustycad_tkmath::plib;

const THE_MAX_EXTENSION_CONTINUITY: i32 = 3;
const THE_CURVE_EXTENSION_TOLERANCE: f64 = 1.0e-6;
const THE_CURVE_SPEED_SAMPLE_COUNT: i32 = 9;
const THE_MIN_CURVE_SPEED_RATIO: f64 = 0.75;
const THE_MAX_CURVE_SPEED_RATIO: f64 = 1.5;

/// `GeomLib::ExtendCurveToPoint(Curve, Point, Cont, After)`: replaces `curve` by the
/// B-spline of the curve extended to `point` at its end (`after`) or start, with a
/// C`cont` junction (`cont` in 1..=3, else nothing is done).
///
/// Nothing is done either when the point is within 1e-6 of the boundary. The
/// extension is the Bezier curve of the Hermite constraints (the boundary point and
/// derivatives scaled by powers of Lambda, the target point), Lambda starting from
/// the ratio of the target distance to the boundary speed (or to the mean speed of
/// 9 samples when the boundary speed is out of [0.75, 1.5] times it) and optimized
/// by ComputeLambda; Standard_ConstructionError ("ExtendCurveToPoint") when the
/// concatenation fails; `geom_lib::extend_curve_to_point(&mut curve, target, 1, true)?`.
pub fn extend_curve_to_point(
    curve: &mut Handle<dyn GeomBoundedCurve>,
    point: Point3,
    continuity: i32,
    after: bool,
) -> Result<(), Failure> {
    if !(1..=THE_MAX_EXTENSION_CONTINUITY).contains(&continuity) {
        return Ok(());
    }
    let constraint_count: i32 = continuity + 2;
    let mut tol: f64 = THE_CURVE_EXTENSION_TOLERANCE;
    let source: &dyn GeomBoundedCurve = live(curve, "GeomLib::ExtendCurveToPoint")?;
    // Convert the input (preserving the parameterization if possible).
    let mut concat =
        CompCurveToBSplineCurve::with_curve(curve, ParameterisationType::QuasiAngular)?;
    let ubord: f64 = if after {
        source.last_parameter()
    } else {
        source.first_parameter()
    };
    let mut mat_coefs: Matrix<'static> =
        crate::array_access::matrix(1, constraint_count, 1, constraint_count, 0.0)?;
    plib::hermite_coefficients(0.0, 1.0, continuity, 0, &mut mat_coefs)?;
    let boundary: Boundary = Boundary::at(source, ubord, after)?;
    let l1: f64 = boundary.point.distance(&point);
    if l1 <= tol {
        return Ok(()); // No extension.
    }
    let mut lambda: f64 = initial_lambda(source, &boundary, l1, tol)?;
    // Optimization of Lambda.
    let cons: Matrix<'static> = boundary.constraints(point, continuity)?;
    compute_lambda(&cons, &mat_coefs, l1, &mut lambda)?;
    let extrap_poles: Array1<'static, Point3> =
        boundary.extension_poles(point, continuity, lambda, &mat_coefs)?;
    let bezier: Handle<GeomBezierCurve> = Handle::new(GeomBezierCurve::new(&extrap_poles)?);
    let dist: f64 = extrap_poles[1].distance(&boundary.point);
    tol += dist;
    // Concatenation.
    if !concat.add(&bezier.cast::<dyn GeomBoundedCurve>(), tol, after, true, 0)? {
        return Err(failure("Standard_ConstructionError", "ExtendCurveToPoint"));
    }
    *curve = concat.bspline_curve().cast();
    Ok(())
}

// Lambda, the ratio to apply to the derivative of the curve to obtain the derivative
// of the extension (arbitrarily the length of the segment end of curve - target
// point): the extension keeps the average speed of the curve.
fn initial_lambda(
    source: &dyn GeomBoundedCurve,
    boundary: &Boundary,
    l1: f64,
    tol: f64,
) -> Result<f64, Failure> {
    let f: f64 = source.first_parameter();
    let mut dt: f64 = (source.last_parameter() - f) / f64::from(THE_CURVE_SPEED_SAMPLE_COUNT);
    let mut norm: f64 = boundary.d1.magnitude();
    let mut t: f64 = f + dt;
    for _ in 1..THE_CURVE_SPEED_SAMPLE_COUNT {
        norm += source.eval_d1(t)?.d1.magnitude();
        t += dt;
    }
    norm /= f64::from(THE_CURVE_SPEED_SAMPLE_COUNT);
    dt = boundary.d1.magnitude() / norm;
    if dt < THE_MAX_CURVE_SPEED_RATIO && dt > THE_MIN_CURVE_SPEED_RATIO {
        // The edge is within the average, keep it.
        return Ok(1.0 / std_max(boundary.d1.magnitude() / l1, tol));
    }
    Ok(1.0 / std_max(norm / l1, tol))
}

// The boundary point and its derivatives, the odd ones oriented outwards.
struct Boundary {
    point: Point3,
    d1: Vector3,
    d2: Vector3,
    d3: Vector3,
}

impl Boundary {
    fn at(source: &dyn GeomBoundedCurve, ubord: f64, after: bool) -> Result<Self, Failure> {
        let d3: GeomCurveD3 = source.eval_d3(ubord)?;
        let direction: f64 = if after { 1.0 } else { -1.0 };
        Ok(Self {
            point: d3.point,
            d1: d3.d1 * direction,
            d2: d3.d2,
            d3: d3.d3 * direction,
        })
    }

    // Cons: the boundary point, the derivatives up to the continuity and the target
    // in the last column, one row per coordinate.
    fn constraints(&self, point: Point3, continuity: i32) -> Result<Matrix<'static>, Failure> {
        let count: i32 = continuity + 2;
        let mut cons: Matrix<'static> = crate::array_access::matrix(1, 3, 1, count, 0.0)?;
        let mut set_col = |col: i32, xyz: Xyz| -> Result<(), Failure> {
            *cons.change_value(1, col)? = xyz.x();
            *cons.change_value(2, col)? = xyz.y();
            *cons.change_value(3, col)? = xyz.z();
            Ok(())
        };
        set_col(1, self.point.xyz())?;
        set_col(2, *self.d1.xyz())?;
        set_col(count, point.xyz())?;
        if continuity >= 2 {
            set_col(3, *self.d2.xyz())?;
        }
        if continuity >= 3 {
            set_col(4, *self.d3.xyz())?;
        }
        Ok(cons)
    }

    // The poles of the Bezier extension: the constraints in the polynomial basis of
    // the Hermite coefficients, converted to the Bernstein basis.
    fn extension_poles(
        &self,
        point: Point3,
        continuity: i32,
        lambda: f64,
        mat_coefs: &Matrix<'static>,
    ) -> Result<Array1<'static, Point3>, Failure> {
        let count: i32 = continuity + 2;
        let mut cont: Vec<Xyz> = vec![self.point.xyz(), *self.d1.xyz() * lambda];
        if continuity >= 2 {
            cont.push(*self.d2.xyz() * lambda.powf(2.0));
        }
        if continuity >= 3 {
            cont.push(*self.d3.xyz() * lambda.powf(3.0));
        }
        cont.push(point.xyz());
        let mut extra_coeffs: Array1<'static, Point3> = one_based(count)?;
        extra_coeffs.init(Point3::new(0.0, 0.0, 0.0));
        for (constraint, value) in (1..).zip(&cont) {
            for coeff in 1..=count {
                let mut sum: Xyz = extra_coeffs[coeff].xyz();
                sum += *value * *mat_coefs.value(constraint, coeff)?;
                extra_coeffs[coeff].set_xyz(sum);
            }
        }
        // Conversion to the Bernstein basis.
        let mut extrap_poles: Array1<'static, Point3> = one_based(count)?;
        plib::coefficients_poles(&extra_coeffs, None, &mut extrap_poles, None)?;
        Ok(extrap_poles)
    }
}
