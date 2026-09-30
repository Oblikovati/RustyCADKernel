// Rust translation of the static helpers of Geom2dAPI_Interpolate.cxx and
// GeomAPI_Interpolate.cxx (CheckPoints, CheckTangents, CheckParameters,
// BuildParameters, BuildPeriodicTangent, BuildTangents, ScaleTangents) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The checks and the tangent construction of Geom2dAPI_Interpolate and
//! GeomAPI_Interpolate, generic over their space (docs/geomalgo-design.md section 15).
use super::interpolate_space::InterpolateSpace;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_small;
use rustycad_tkmath::bsplclib::CurvePoint;
use rustycad_tkmath::plib::{self, PolePoint};

// Both native classes; the Array1 errors name the index and the bounds.
const MEMBER: &str = "Geom2dAPI_Interpolate/GeomAPI_Interpolate";

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
pub(super) fn check_points<S: InterpolateSpace>(
    points: &Array1<'_, S::Point>,
    tolerance: f64,
) -> Result<bool, Failure> {
    let tolerance_squared: f64 = tolerance * tolerance;
    let mut result: bool = true;
    let mut ii: i32 = points.lower();
    while result && ii < points.upper() {
        let distance_squared: f64 = S::square_distance(&at(points, ii)?, &at(points, ii + 1)?);
        result = distance_squared >= tolerance_squared;
        ii += 1;
    }
    Ok(result)
}

/// `CheckTangents(Tangents, TangentFlags, Tolerance)`: every flagged tangent is at
/// least `tol` long (the flags are read from their own lower bound).
pub(super) fn check_tangents<S: InterpolateSpace>(
    tangents: &Array1<'_, S::Vector>,
    tangent_flags: &Array1<'_, bool>,
    tolerance: f64,
) -> Result<bool, Failure> {
    let tolerance_squared: f64 = tolerance * tolerance;
    let mut result: bool = true;
    let mut index: i32 = tangent_flags.lower();
    let mut ii: i32 = tangents.lower();
    while result && ii <= tangents.upper() {
        if at(tangent_flags, index)? {
            result = S::square_magnitude(&at(tangents, ii)?) >= tolerance_squared;
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
pub(super) fn build_parameters<S: InterpolateSpace>(
    periodic: bool,
    points: &Array1<'_, S::Point>,
) -> Result<Array1<'static, f64>, Failure> {
    let num_parameters: i32 = points.len() as i32 + i32::from(periodic);
    let mut parameters: Array1<'static, f64> = array(1, num_parameters)?;
    put(&mut parameters, 1, 0.0e0)?;
    let mut index: i32 = 2;
    let mut ii: i32 = points.lower();
    while ii < points.upper() {
        let distance: f64 = S::distance(&at(points, ii)?, &at(points, ii + 1)?);
        let value: f64 = at(&parameters, ii)? + distance;
        put(&mut parameters, index, value)?;
        index += 1;
        ii += 1;
    }
    if periodic {
        let closing = (at(points, points.upper())?, at(points, points.lower())?);
        let value: f64 = at(&parameters, ii)? + S::distance(&closing.0, &closing.1);
        put(&mut parameters, index, value)?;
    }
    Ok(parameters)
}

/// The derivative at `parameter` of the Lagrange interpolant of the `degree + 1`
/// points and parameters starting at `from_point` and `from_parameter`
/// (`PLib::EvalLagrange` on the native arrays). The native result array is not
/// written when two parameters are too close (garbage then); it is 0 here.
fn lagrange_derivative<S: InterpolateSpace>(
    (points, from_point): (&Array1<'_, S::Point>, i32),
    (parameters, from_parameter): (&Array1<'_, f64>, i32),
    parameter: f64,
    degree: i32,
) -> Result<S::Vector, Failure> {
    let dimension: usize = <S::Point as PolePoint>::DIMENSION;
    let mut values: Vec<f64> = Vec::with_capacity(dimension * (degree as usize + 1));
    let mut knots: Vec<f64> = Vec::with_capacity(degree as usize + 1);
    for offset in 0..=degree {
        let point: S::Point = at(points, from_point + offset)?;
        values.extend((0..dimension).map(|k| point.pole_coordinate(k)));
        knots.push(at(parameters, from_parameter + offset)?);
    }
    let mut eval_result: Vec<f64> = vec![0.0; 2 * dimension];
    let dim: i32 = dimension as i32;
    plib::eval_lagrange(parameter, 1, degree, dim, &values, &knots, &mut eval_result)?;
    Ok(S::Point::vector_from(&eval_result[dimension..]))
}

/// `BuildPeriodicTangent(PointsArray, TangentsArray, TangentFlags, ParametersArray)`:
/// the first tangent from a cubic (quadratic for 3 points) Lagrange interpolant when
/// it is not given; Standard_ConstructionError for fewer than 3 points.
pub(super) fn build_periodic_tangent<S: InterpolateSpace>(
    points: &Array1<'_, S::Point>,
    tangents: &mut Array1<'_, S::Vector>,
    tangent_flags: &mut Array1<'_, bool>,
    parameters: &Array1<'_, f64>,
) -> Result<(), Failure> {
    if points.len() < 3 {
        return Err(construction::<S>("BuildPeriodicTangent", points.len()));
    }
    if !at(tangent_flags, 1)? {
        let degree: i32 = if points.len() == 3 { 2 } else { 3 };
        put(tangent_flags, 1, true)?;
        let tangent = lagrange_derivative::<S>(
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
pub(super) fn build_tangents<S: InterpolateSpace>(
    points: &Array1<'_, S::Point>,
    tangents: &mut Array1<'_, S::Vector>,
    tangent_flags: &mut Array1<'_, bool>,
    parameters: &Array1<'_, f64>,
) -> Result<(), Failure> {
    if points.len() < 3 {
        return Err(construction::<S>("BuildTangents", points.len()));
    }
    let degree: i32 = if points.len() == 3 { 2 } else { 3 };
    if !at(tangent_flags, 1)? {
        put(tangent_flags, 1, true)?;
        let tangent = lagrange_derivative::<S>(
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
        let tangent = lagrange_derivative::<S>(
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
pub(super) fn scale_tangents<S: InterpolateSpace>(
    points: &Array1<'_, S::Point>,
    tangents: &mut Array1<'_, S::Vector>,
    tangent_flags: &Array1<'_, bool>,
    parameters: &Array1<'_, f64>,
) -> Result<(), Failure> {
    let degree: i32 = match points.len() {
        2 => 1,
        n if n >= 3 => 2,
        _ => 0,
    };
    let mut index: i32 = points.lower();
    for ii in tangent_flags.lower()..=tangent_flags.upper() {
        if !at(tangent_flags, ii)? {
            continue;
        }
        let derivative = lagrange_derivative::<S>(
            (points, index),
            (parameters, index),
            at(parameters, ii)?,
            degree,
        )?;
        let tangent: S::Vector = at(tangents, ii)?;
        let ratio: f64 = l1_norm::<S>(&derivative) / l1_norm::<S>(&tangent);
        put(tangents, ii, scaled::<S>(ratio, &tangent))?;
        if ii != tangent_flags.lower() {
            index += 1;
        }
        if index > points.upper() - degree {
            index = points.upper() - degree;
        }
    }
    Ok(())
}

// `value = 0.0e0; for (jj) value += Abs(V.Coord(jj))`.
fn l1_norm<S: InterpolateSpace>(v: &S::Vector) -> f64 {
    let dimension: usize = <S::Point as PolePoint>::DIMENSION;
    (0..dimension).fold(0.0e0, |value, k| {
        value + S::Point::vector_coordinate(v, k).abs()
    })
}

// `for (jj) a_vector.SetCoord(jj, ratio * V.Coord(jj))`.
fn scaled<S: InterpolateSpace>(ratio: f64, v: &S::Vector) -> S::Vector {
    let dimension: usize = <S::Point as PolePoint>::DIMENSION;
    let coordinates: Vec<f64> = (0..dimension)
        .map(|k| ratio * S::Point::vector_coordinate(v, k))
        .collect();
    S::Point::vector_from(&coordinates)
}

// The Standard_ConstructionError of the tangent builders.
fn construction<S: InterpolateSpace>(member: &str, nb_points: usize) -> Failure {
    Failure::with_type(
        "Standard_ConstructionError",
        &format!(
            "{} {member}() - {nb_points} points; expected at least 3",
            S::NAME
        ),
        "",
    )
}
