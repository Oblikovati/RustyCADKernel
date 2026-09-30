// Rust translation of FoundationClasses/TKMath/Convert/Convert_PolynomialCosAndSin.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A polynomial (degree 7 Bezier) approximation of a circular arc, trimmed from
//! a fixed unit-circle Bezier curve.
use crate::bsplclib;
use crate::gp::{Point2, Transform2, Xy};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

// Bezier control points for a unit circle (degree-7 polynomial approximation).
// Coefficients from Tiller's algorithm for polynomial cos/sin representation.
const UNIT_CIRCLE_POLES: [[f64; 2]; 8] = [
    [1., 0.],
    [1., 1.013854],
    [-0.199043, 1.871905],
    [-1.937729, 1.057323],
    [-1.937729, -1.057323],
    [-0.199043, -1.871905],
    [1., -1.013854],
    [1., 0.],
];

/// Fills `num_poles` polynomial poles of the arc `[u_first, u_last]`, with unit
/// denominators (`BuildPolynomialCosAndSin`); only 8 poles match the fixed degree 7
/// Bezier; `build_polynomial_cos_and_sin(0.0, 1.0, 8, &mut cos, &mut sin, &mut den)?`.
pub fn build_polynomial_cos_and_sin(
    u_first: f64,
    u_last: f64,
    num_poles: i32,
    cos_numerator: &mut Array1<'_, f64>,
    sin_numerator: &mut Array1<'_, f64>,
    denominator: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let delta: f64 = u_last - u_first;
    let t_poles: Array1<'static, Point2> = bisector_circle(num_poles, 0.5e0 * delta - PI)?;
    let trim_max: f64 = locate(delta, &t_poles, trim_bounds(delta))?;
    let mut arc: Array1<'static, Point2> = trim_bezier(num_poles - 1, &t_poles, trim_max)?;
    readjust_ends(&mut arc, num_poles, delta)?;
    let rotation: Transform2 = Transform2::rotation(Point2::default(), wrapped_start(u_first));
    for ii in 1..=num_poles {
        let point: Point2 = checked(&arc, ii)?.transformed(rotation);
        *checked_mut(cos_numerator, ii)? = point.x();
        *checked_mut(sin_numerator, ii)? = point.y();
        *checked_mut(denominator, ii)? = 1.;
    }
    Ok(())
}

// Returns UFirst in [-2PI; 2PI] to make rotations without risk.
fn wrapped_start(u_first: f64) -> f64 {
    const PI2: f64 = 2.0 * PI;
    let mut start: f64 = u_first;
    while start > PI2 {
        start -= PI2;
    }
    while start < -PI2 {
        start += PI2;
    }
    start
}

// The unit-circle Bezier rotated so that its middle lies on the bisector of the
// arc turned onto -Ox; parameter 1/2 is then exactly on the bisector.
fn bisector_circle(num_poles: i32, angle: f64) -> Result<Array1<'static, Point2>, Failure> {
    let mut poles: Array1<'static, Point2> = Array1::with_size(UNIT_CIRCLE_POLES.len());
    poles.update_lower_bound(1);
    for (pole, [x, y]) in poles.iter_mut().zip(UNIT_CIRCLE_POLES) {
        *pole = Point2::new(x, y);
    }
    let rotation: Transform2 = Transform2::rotation(Point2::default(), angle);
    for ii in 1..=num_poles {
        *checked_mut(&mut poles, ii)? = checked(&poles, ii)?.transformed(rotation);
    }
    Ok(poles)
}

fn trim_bounds(delta: f64) -> (f64, f64) {
    let mut t_min: f64 = 1.0e0 - (delta * 1.3e0 / PI);
    t_min *= 0.5e0;
    let mut t_max: f64 = 1.0e0 + (delta * 1.3e0 / PI);
    t_max *= 0.5e0;
    // std::max(t_min, 0.0) and std::min(t_max, 1.0) keep the first argument on NaN.
    let t_min: f64 = if t_min < 0.0e0 { 0.0e0 } else { t_min };
    let t_max: f64 = if 1.0e0 < t_max { 1.0e0 } else { t_max };
    (t_min, t_max)
}

// Bisection on the Bezier parameter whose polar angle equals the arc opening.
fn locate(
    angle_end: f64,
    t_poles: &Array1<'_, Point2>,
    (u_min, u_max): (f64, f64),
) -> Result<f64, Failure> {
    let (mut umin, mut umax) = (u_min, u_max);
    let p_tol: f64 = precision::angular();
    let u_tol: f64 = precision::p_confusion();
    while (umax - umin).abs() >= u_tol {
        let ptest: f64 = (umax + umin) / 2.;
        let value: Point2 = bsplclib::bezier_d0(ptest, t_poles, None)?;
        let mut theta: f64 = value.y().atan2(value.x());
        if theta < 0. {
            theta += 2. * PI;
        }
        if (theta - angle_end).abs() < p_tol {
            return Ok(ptest);
        }
        if theta < angle_end {
            umin = ptest;
        } else if theta > angle_end {
            umax = ptest;
        }
    }
    Ok((umin + umax) / 2.)
}

// As the Bezier is symmetric about the bisector, the arc is [1 - trim_max, trim_max].
fn trim_bezier(
    degree: i32,
    t_poles: &Array1<'_, Point2>,
    trim_max: f64,
) -> Result<Array1<'static, Point2>, Failure> {
    let trim_min: f64 = 1.0e0 - trim_max;
    let knots: Array1<'static, f64> = one_based_values(&[0.0e0, 1.0e0]);
    let mults: Array1<'static, i32> = one_based_values(&[degree + 1, degree + 1]);
    let (mut new_knots, mut new_mults) = (knots.clone(), mults.clone());
    let mut new_poles: Array1<'static, Point2> = Array1::with_size(8);
    new_poles.update_lower_bound(1);
    bsplclib::trimming(
        degree,
        false,
        &knots,
        &mults,
        t_poles,
        None,
        trim_min,
        trim_max,
        &mut new_knots,
        &mut new_mults,
        &mut new_poles,
        None,
    )?;
    Ok(new_poles)
}

fn one_based_values<T: Copy + Default>(values: &[T]) -> Array1<'static, T> {
    let mut array: Array1<'static, T> = Array1::with_size(values.len());
    array.update_lower_bound(1);
    array.as_mut_slice().copy_from_slice(values);
    array
}

// Readjustment (obviously redundant, says the native comment): the end poles move
// onto the exact arc ends with the end tangent lengths kept.
fn readjust_ends(arc: &mut Array1<'_, Point2>, num_poles: i32, delta: f64) -> Result<(), Failure> {
    let (sin_d, cos_d) = (delta.sin(), delta.cos());
    let start: Point2 = Point2::new(1., 0.);
    let end: Point2 = Point2::new(cos_d, sin_d);
    let mut dtg: f64 = checked(arc, 1)?.distance(checked(arc, 2)?);
    arc[1] = start;
    arc[2] = Point2::from(start.xy() + Xy::new(0., dtg));
    // Readjustment to Euclid.
    dtg = checked(arc, num_poles)?.distance(checked(arc, num_poles - 1)?);
    *checked_mut(arc, num_poles)? = end;
    let offset: Xy = Xy::new(dtg * sin_d, -dtg * cos_d);
    *checked_mut(arc, num_poles - 1)? = Point2::from(end.xy() + offset);
    Ok(())
}

fn checked(array: &Array1<'_, Point2>, index: i32) -> Result<Point2, Failure> {
    crate::plib::access::array_value(array, index)
}

fn checked_mut<'b, T>(array: &'b mut Array1<'_, T>, index: i32) -> Result<&'b mut T, Failure> {
    crate::plib::access::array_cell(array, index)
}
