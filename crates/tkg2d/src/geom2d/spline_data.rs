// Rust translation of ModelingData/TKG2d/Geom2d (the dimension-independent data checks and array steps of Geom2d_BezierCurve and Geom2d_BSplineCurve, which Geom_BezierCurve and Geom_BSplineCurve repeat) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Knot, multiplicity, weight and array steps that the 2D and 3D Bezier and
//! B-spline curves share word for word; the point dimension never enters them.
use super::arrays::{any_weight_step, at, one_based, put, rebased};
use super::errors;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, real};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::{BSplKnotDistribution, Shape};

/// The file-static `CheckCurveData` of Geom2d_BSplineCurve.cxx and Geom_BSplineCurve.cxx;
/// `check_curve_data(&poles, &k, &m, 3, false)?`.
pub fn check_curve_data<P>(
    poles: &Array1<'_, P>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    degree: i32,
    periodic: bool,
) -> Result<(), Failure> {
    if degree < 1 || degree > bsplclib::max_degree() {
        return Err(errors::construction(&format!(
            "BSpline curve: invalid degree {degree}; expected in [1, {}]",
            bsplclib::max_degree()
        )));
    }
    if poles.len() < 2 {
        return Err(errors::construction(&format!(
            "BSpline curve: {} poles; at least 2 poles required",
            poles.len()
        )));
    }
    if knots.len() != mults.len() {
        return Err(errors::construction(&format!(
            "BSpline curve: {} knots and {} multiplicities; expected equal sizes",
            knots.len(),
            mults.len()
        )));
    }
    check_knot_spacing(knots)?;
    let expected: i32 = bsplclib::nb_poles(degree, periodic, mults)?;
    if poles.len() as i32 != expected {
        return Err(errors::construction(&format!(
            "BSpline curve: {} poles; # Poles and degree mismatch, expected {expected}",
            poles.len()
        )));
    }
    Ok(())
}

fn check_knot_spacing(knots: &Array1<'_, f64>) -> Result<(), Failure> {
    for i in knots.lower()..knots.upper() {
        let (knot, next) = (at(knots, i)?, at(knots, i + 1)?);
        if next - knot <= real::epsilon(knot.abs()) {
            return Err(errors::construction(&format!(
                "BSpline curve: Knots({i}) = {knot}, Knots({}) = {next}; Knots interval values too close",
                i + 1
            )));
        }
    }
    Ok(())
}

/// The file-static `Rational` of the B-spline curves (over the bounds of the array);
/// `bspline_rational(&weights)?`.
pub fn bspline_rational(weights: &Array1<'_, f64>) -> Result<bool, Failure> {
    any_weight_step(weights, weights.lower(), weights.upper())
}

/// The file-static `Rational(W)` of the Bezier curves: it reads W(1)..W(n) with
/// one-based indices whatever the lower bound, so other bounds raise
/// Standard_OutOfRange; `bezier_rational(&weights)?`.
pub fn bezier_rational(weights: &Array1<'_, f64>) -> Result<bool, Failure> {
    any_weight_step(weights, 1, weights.len() as i32)
}

/// The continuity switch of `updateKnots`: `Degree - MaxKnotMult` with `default: C3`;
/// `smoothness(3, 1) == Shape::C2`.
pub fn smoothness(degree: i32, max_knot_mult: i32) -> Shape {
    if max_knot_mult == 0 {
        return Shape::CN;
    }
    match degree - max_knot_mult {
        0 => Shape::C0,
        1 => Shape::C1,
        2 => Shape::C2,
        _ => Shape::C3,
    }
}

/// What `updateKnots` (B-spline curves) and `updateUKnots`/`updateVKnots` (B-spline
/// surfaces) derive from the knots and multiplicities of one parametric direction.
pub struct KnotData {
    /// The knot distribution (`KnotAnalysis`).
    pub knot_set: BSplKnotDistribution,
    /// The flat knot sequence; a copy of the knots for uniform non-periodic data.
    pub flat_knots: Array1<'static, f64>,
    /// The continuity from the highest interior multiplicity.
    pub smooth: Shape,
}

/// The shared body of `updateKnots`, `updateUKnots` and `updateVKnots`;
/// `knot_data(3, false, &knots, &mults)?.smooth`.
pub fn knot_data(
    degree: i32,
    periodic: bool,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
) -> Result<KnotData, Failure> {
    let (knot_set, max_knot_mult) = bsplclib::knot_analysis(degree, periodic, knots, mults)?;
    let flat_knots: Array1<'static, f64> = if knot_set == BSplKnotDistribution::Uniform && !periodic
    {
        // The stored knots are one-based, so the rebased copy has their bounds.
        rebased(knots)?
    } else {
        let length: i32 = bsplclib::knot_sequence_length(mults, degree, periodic)?;
        let mut flat: Array1<'static, f64> = one_based(length)?;
        bsplclib::knot_sequence_with_degree(knots, mults, degree, periodic, &mut flat)?;
        flat
    };
    Ok(KnotData {
        knot_set,
        flat_knots,
        smooth: smoothness(degree, max_knot_mult),
    })
}

/// The `IsCN` switch of the B-spline curves and the `IsCNu`/`IsCNv` switch of the
/// surfaces, for `n >= 0`: C3 data is CN up to `Degree - MaxKnotMult` of the interior
/// knots, computed only when `n > 3`; `spline_is_cn(Shape::C2, 2, 3, &mults)?`.
pub fn spline_is_cn(
    smooth: Shape,
    n: i32,
    degree: i32,
    mults: &Array1<'_, i32>,
) -> Result<bool, Failure> {
    Ok(match smooth {
        Shape::CN => true,
        Shape::C0 | Shape::G1 => n <= 0,
        Shape::C1 | Shape::G2 => n <= 1,
        Shape::C2 => n <= 2,
        Shape::C3 => {
            n <= 3
                || n <= degree
                    - bsplclib::max_knot_mult(mults, mults.lower() + 1, mults.upper() - 1)?
        }
    })
}

/// The deprecated `Knots(K)` / `KnotSequence(K)`: K's bounds must lie inside the
/// source's (Standard_DomainError); `copy_inside(&knots, &mut k, "Geom_BSplineCurve::Knots")?`.
pub fn copy_inside(
    source: &Array1<'_, f64>,
    target: &mut Array1<'_, f64>,
    member: &str,
) -> Result<(), Failure> {
    if target.lower() < source.lower() || target.upper() > source.upper() {
        return Err(errors::domain(&format!(
            "{member}: bounds [{}, {}]; expected inside [{}, {}]",
            target.lower(),
            target.upper(),
            source.lower(),
            source.upper()
        )));
    }
    for index in target.lower()..=target.upper() {
        put(target, index, at(source, index)?)?;
    }
    Ok(())
}

/// The last branch of `LocateU`: hunt, clamp, then skip knots equal within the
/// tolerance; `locate_inside(&knots, 0.5, 1e-9)?`.
pub fn locate_inside(
    knots: &Array1<'_, f64>,
    u: f64,
    tolerance: f64,
) -> Result<(i32, i32), Failure> {
    let mut i1: i32 = 1;
    bsplclib::hunt(knots, u, &mut i1)?;
    i1 = i1.min(knots.upper()).max(knots.lower());
    while i1 < knots.upper() && (at(knots, i1 + 1)? - u).abs() <= tolerance {
        i1 += 1;
    }
    if (at(knots, i1)? - u).abs() <= tolerance {
        return Ok((i1, i1));
    }
    Ok((i1, i1 + 1))
}

/// `LocateU` on already normalized `u`: the knot interval `(I1, I2)`, equal indices
/// on a knot, 0 or NbKnots+1 outside; `locate_u_in(&knots, 0.5, 1e-9)?`.
pub fn locate_u_in(
    knots: &Array1<'_, f64>,
    new_u: f64,
    parametric_tolerance: f64,
) -> Result<(i32, i32), Failure> {
    let count: i32 = knots.len() as i32;
    let (first, last) = (at(knots, 1)?, at(knots, count)?);
    let tolerance: f64 = parametric_tolerance.abs();
    if (new_u - first).abs() <= tolerance {
        return Ok((1, 1));
    }
    if (new_u - last).abs() <= tolerance {
        return Ok((count, count));
    }
    if new_u < first {
        return Ok((0, 1));
    }
    if new_u > last {
        return Ok((count, count + 1));
    }
    locate_inside(knots, new_u, tolerance)
}

/// `NCollection_Array1<T>(I1, I2)` with the native bound check; `bounded::<f64>(2, 3)?`.
pub fn bounded<T: Default>(i1: i32, i2: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(i1, i2).map_err(|error| {
        errors::out_of_range(&format!(
            "NCollection_Array1({i1}, {i2}): {error}; expected I1 <= I2 + 1"
        ))
    })
}

/// Poles or weights starting at one-based `index`, then the ones before it
/// (`SetOrigin`); `rotated(&poles, 3)?`.
pub fn rotated<T: Copy + Default>(
    values: &Array1<'_, T>,
    index: i32,
) -> Result<Array1<'static, T>, Failure> {
    let count: i32 = values.len() as i32;
    let mut result: Array1<'static, T> = one_based(count)?;
    for (k, i) in (1_i32..).zip((index..=values.upper()).chain(values.lower()..index)) {
        put(&mut result, k, at(values, i)?)?;
    }
    Ok(result)
}

/// Values `first..=last` of a one-based array, re-indexed from 1 (`Segment`);
/// `slice(&poles, 2, 4)?`.
pub fn slice<T: Copy + Default>(
    values: &Array1<'_, T>,
    first: i32,
    last: i32,
) -> Result<Array1<'static, T>, Failure> {
    let mut result: Array1<'static, T> = one_based(last - first + 1)?;
    for (k, i) in (first..=last).enumerate() {
        put(&mut result, k as i32 + 1, at(values, i)?)?;
    }
    Ok(result)
}

/// The Bezier knots `{0, 1}` (`Knots`); `bezier_knots()[2] == 1.0`.
pub fn bezier_knots() -> Array1<'static, f64> {
    static THE_DATA: [f64; 2] = [0.0, 1.0];
    Array1::from_shared(&THE_DATA, 1)
}

fn check_bezier_degree(degree: i32, class: &str) -> Result<(), Failure> {
    if degree < 0 {
        return Err(errors::program(&format!(
            "{class}: empty poles; expected at least 2 poles"
        )));
    }
    Ok(())
}

/// The Bezier multiplicities `{Degree+1, Degree+1}` (`Multiplicities`), Standard_ProgramError
/// without poles; `bezier_multiplicities(3, "Geom_BezierCurve")?[1] == 4`.
pub fn bezier_multiplicities(degree: i32, class: &str) -> Result<Array1<'static, i32>, Failure> {
    check_bezier_degree(degree, class)?;
    let mut mults: Array1<'static, i32> = one_based(2)?;
    mults.init(degree + 1);
    Ok(mults)
}

/// The Bezier flat knots: `Degree+1` zeros then `Degree+1` ones (`KnotSequence`);
/// `bezier_knot_sequence(2, "Geom_BezierCurve")?.len() == 6`.
pub fn bezier_knot_sequence(degree: i32, class: &str) -> Result<Array1<'static, f64>, Failure> {
    check_bezier_degree(degree, class)?;
    let flat: &'static [f64] = bsplclib::flat_bezier_knots(degree)?;
    Ok(Array1::from_shared(&flat[..2 * (degree as usize + 1)], 1))
}
