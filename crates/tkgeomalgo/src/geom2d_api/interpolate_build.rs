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
// Rust translation of the static helpers of Geom2dAPI_Interpolate.cxx (CheckPoints,
// CheckTangents, CheckParameters, BuildParameters, BuildPeriodicTangent, BuildTangents,
// ScaleTangents) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The checks and the tangent construction of Geom2dAPI_Interpolate.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_small;
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tkmath::plib;

const MEMBER: &str = "Geom2dAPI_Interpolate";

/// The value at `index` of a native array (Standard_OutOfRange outside it).
pub(super) fn at<T: Clone>(array: &Array1<'_, T>, index: i32) -> Result<T, Failure> {
    array
        .value(index)
        .cloned()
        .map_err(|e| e.into_failure(MEMBER))
}

/// `SetValue(index, value)` of a native array (Standard_OutOfRange outside it).
pub(super) fn put<T>(array: &mut Array1<'_, T>, index: i32, value: T) -> Result<(), Failure> {
    array
        .set_value(index, value)
        .map_err(|e| e.into_failure(MEMBER))
}

/// A fresh native array `lower..=upper`.
pub(super) fn array<T: Default>(lower: i32, upper: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(lower, upper).map_err(|e| e.into_failure(MEMBER))
}

/// `CheckPoints(PointArray, Tolerance)`: consecutive points are at least `tol` apart.
pub(super) fn check_points(points: &Array1<'_, Point2>, tolerance: f64) -> Result<bool, Failure> {
    let tolerance_squared: f64 = tolerance * tolerance;
    let mut result: bool = true;
    let mut ii: i32 = points.lower();
    while result && ii < points.upper() {
        let distance_squared: f64 = at(points, ii)?.square_distance(at(points, ii + 1)?);
        result = distance_squared >= tolerance_squared;
        ii += 1;
    }
    Ok(result)
}

/// `CheckTangents(Tangents, TangentFlags, Tolerance)`: every flagged tangent is at
/// least `tol` long (the flags are read from their own lower bound).
pub(super) fn check_tangents(
    tangents: &Array1<'_, Vector2>,
    tangent_flags: &Array1<'_, bool>,
    tolerance: f64,
) -> Result<bool, Failure> {
    let tolerance_squared: f64 = tolerance * tolerance;
    let mut result: bool = true;
    let mut index: i32 = tangent_flags.lower();
    let mut ii: i32 = tangents.lower();
    while result && ii <= tangents.upper() {
        if at(tangent_flags, index)? {
            result = at(tangents, ii)?.square_magnitude() >= tolerance_squared;
        }
        index += 1;
        ii += 1;
    }
    Ok(result)
}

/// `CheckParameters(Parameters)`: the parameters increase by at least RealSmall().
pub(super) fn check_parameters(parameters: &Array1<'_, f64>) -> Result<bool, Failure> {
    let mut result: bool = true;
    let mut ii: i32 = parameters.lower();
    while result && ii < parameters.upper() {
        result = at(parameters, ii + 1)? - at(parameters, ii)? >= real_small();
        ii += 1;
    }
    Ok(result)
}

/// `BuildParameters(PeriodicFlag, PointsArray, ParametersPtr)`: the cumulated chord
/// lengths from 0, one more for the closing chord of a periodic curve. The native
/// code reads the previous parameter at the POINT index, so points not starting
/// at 1 raise Standard_OutOfRange.
pub(super) fn build_parameters(
    periodic: bool,
    points: &Array1<'_, Point2>,
) -> Result<Array1<'static, f64>, Failure> {
    let num_parameters: i32 = points.len() as i32 + i32::from(periodic);
    let mut parameters: Array1<'static, f64> = array(1, num_parameters)?;
    put(&mut parameters, 1, 0.0e0)?;
    let mut index: i32 = 2;
    let mut ii: i32 = points.lower();
    while ii < points.upper() {
        let distance: f64 = at(points, ii)?.distance(at(points, ii + 1)?);
        let value: f64 = at(&parameters, ii)? + distance;
        put(&mut parameters, index, value)?;
        index += 1;
        ii += 1;
    }
    if periodic {
        let distance: f64 = at(points, points.upper())?.distance(at(points, points.lower())?);
        let value: f64 = at(&parameters, ii)? + distance;
        put(&mut parameters, index, value)?;
    }
    Ok(parameters)
}

/// The derivative at `parameter` of the Lagrange interpolant of the `degree + 1`
/// points and parameters starting at `from_point` and `from_parameter`
/// (`PLib::EvalLagrange` on the native arrays). The native result array is not
/// written when two parameters are too close (garbage then); it is 0 here.
fn lagrange_derivative(
    (points, from_point): (&Array1<'_, Point2>, i32),
    (parameters, from_parameter): (&Array1<'_, f64>, i32),
    parameter: f64,
    degree: i32,
) -> Result<Vector2, Failure> {
    let mut values: Vec<f64> = Vec::with_capacity(2 * (degree as usize + 1));
    let mut knots: Vec<f64> = Vec::with_capacity(degree as usize + 1);
    for offset in 0..=degree {
        let point: Point2 = at(points, from_point + offset)?;
        values.extend([point.x(), point.y()]);
        knots.push(at(parameters, from_parameter + offset)?);
    }
    let mut eval_result: [f64; 4] = [0.0; 4];
    plib::eval_lagrange(parameter, 1, degree, 2, &values, &knots, &mut eval_result)?;
    Ok(Vector2::new(eval_result[2], eval_result[3]))
}

/// `BuildPeriodicTangent(PointsArray, TangentsArray, TangentFlags, ParametersArray)`:
/// the first tangent from a cubic (quadratic for 3 points) Lagrange interpolant when
/// it is not given; Standard_ConstructionError for fewer than 3 points.
pub(super) fn build_periodic_tangent(
    points: &Array1<'_, Point2>,
    tangents: &mut Array1<'_, Vector2>,
    tangent_flags: &mut Array1<'_, bool>,
    parameters: &Array1<'_, f64>,
) -> Result<(), Failure> {
    if points.len() < 3 {
        return Err(construction("BuildPeriodicTangent", points.len()));
    }
    if !at(tangent_flags, 1)? {
        let degree: i32 = if points.len() == 3 { 2 } else { 3 };
        put(tangent_flags, 1, true)?;
        let tangent = lagrange_derivative(
            (points, points.lower()),
            (parameters, 1),
            at(parameters, 1)?,
            degree,
        )?;
        put(tangents, 1, tangent)?;
    }
    Ok(())
}

/// `BuildTangents(PointsArray, TangentsArray, TangentFlags, ParametersArray)`: the
/// missing end tangents from the Lagrange interpolants of the first and last
/// `degree + 1` points; Standard_ConstructionError for fewer than 3 points.
pub(super) fn build_tangents(
    points: &Array1<'_, Point2>,
    tangents: &mut Array1<'_, Vector2>,
    tangent_flags: &mut Array1<'_, bool>,
    parameters: &Array1<'_, f64>,
) -> Result<(), Failure> {
    if points.len() < 3 {
        return Err(construction("BuildTangents", points.len()));
    }
    let degree: i32 = if points.len() == 3 { 2 } else { 3 };
    if !at(tangent_flags, 1)? {
        put(tangent_flags, 1, true)?;
        let tangent = lagrange_derivative(
            (points, points.lower()),
            (parameters, 1),
            at(parameters, 1)?,
            degree,
        )?;
        put(tangents, 1, tangent)?;
    }
    if !at(tangent_flags, tangent_flags.upper())? {
        let upper: i32 = tangent_flags.upper();
        put(tangent_flags, upper, true)?;
        let at_end: f64 = at(parameters, parameters.upper())?;
        let source = (points, points.upper() - degree);
        let tangent = lagrange_derivative(
            source,
            (parameters, parameters.upper() - degree),
            at_end,
            degree,
        )?;
        let last: i32 = tangents.upper();
        put(tangents, last, tangent)?;
    }
    Ok(())
}

/// `ScaleTangents(PointsArray, TangentsArray, TangentFlags, ParametersArray)`: each
/// flagged tangent rescaled to the L1 norm of the derivative of the Lagrange
/// interpolant (linear for 2 points, quadratic otherwise) of the points from the
/// running index.
pub(super) fn scale_tangents(
    points: &Array1<'_, Point2>,
    tangents: &mut Array1<'_, Vector2>,
    tangent_flags: &Array1<'_, bool>,
    parameters: &Array1<'_, f64>,
) -> Result<(), Failure> {
    let num_points: usize = points.len();
    let degree: i32 = match num_points {
        2 => 1,
        n if n >= 3 => 2,
        _ => 0,
    };
    let mut index: i32 = points.lower();
    for ii in tangent_flags.lower()..=tangent_flags.upper() {
        if !at(tangent_flags, ii)? {
            continue;
        }
        let derivative = lagrange_derivative(
            (points, index),
            (parameters, index),
            at(parameters, ii)?,
            degree,
        )?;
        let tangent: Vector2 = at(tangents, ii)?;
        let value0: f64 = tangent.x().abs() + tangent.y().abs();
        let value1: f64 = derivative.x().abs() + derivative.y().abs();
        let ratio: f64 = value1 / value0;
        put(
            tangents,
            ii,
            Vector2::new(ratio * tangent.x(), ratio * tangent.y()),
        )?;
        if ii != tangent_flags.lower() {
            index += 1;
        }
        if index > points.upper() - degree {
            index = points.upper() - degree;
        }
    }
    Ok(())
}

// The Standard_ConstructionError of the tangent builders.
fn construction(member: &str, nb_points: usize) -> Failure {
    Failure::with_type(
        "Standard_ConstructionError",
        &format!("Geom2dAPI_Interpolate {member}() - {nb_points} points; expected at least 3"),
        "",
    )
}
