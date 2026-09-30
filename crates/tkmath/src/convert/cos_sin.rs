// Rust translation of Convert_ConicToBSplineCurve::BuildCosAndSin(Parameterisation, UFirst, ULast, ...) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The trimmed BuildCosAndSin: a rational B-spline of `(cos, sin)` on an arc.
use super::ParameterisationType;
use super::arrays::one_based;
use super::cos_sin_eval::build_algorithmic;
use super::polynomial_cos_sin::build_polynomial_cos_and_sin;
use crate::bvh::BvhScalar;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

/// B-spline of the unit circle arc: numerators of cos and sin divided by the
/// denominator (the weights), degree, knots and multiplicities
/// (the output arguments of `BuildCosAndSin`);
/// `ConicToBSplineCurve::build_cos_and_sin(ParameterisationType::TgtThetaOver2)?.degree == 2`.
#[derive(Clone, Debug)]
pub struct CosAndSin {
    /// Cosine pole coordinates.
    pub cos_numerator: Array1<'static, f64>,
    /// Sine pole coordinates.
    pub sin_numerator: Array1<'static, f64>,
    /// Pole weights.
    pub denominator: Array1<'static, f64>,
    /// B-spline degree.
    pub degree: i32,
    /// Distinct knots.
    pub knots: Array1<'static, f64>,
    /// Knot multiplicities.
    pub mults: Array1<'static, i32>,
}

// The counts the native switch assigns before any array exists.
#[derive(Clone, Copy)]
pub(super) struct SpanLayout {
    pub(super) num_spans: i32,
    pub(super) num_poles: i32,
    pub(super) num_knots: i32,
    pub(super) degree: i32,
    pub(super) order: i32,
    pub(super) tangent_half_angle: bool,
}

/// `Standard_ConstructionError` with the offending value; `construction("bad 3")`.
pub(super) fn construction(message: &str) -> Failure {
    Failure::with_type("Standard_ConstructionError", message, "")
}

/// Builds `(cos, sin)` on `[u_first, u_last]`; `build_trimmed(ParameterisationType::QuasiAngular, 0.0, PI)?`.
pub(super) fn build_trimmed(
    parameterisation: ParameterisationType,
    u_first: f64,
    u_last: f64,
) -> Result<CosAndSin, Failure> {
    let layout: SpanLayout = span_layout(parameterisation, u_last - u_first)?;
    let mut result: CosAndSin = allocate(layout)?;
    if layout.tangent_half_angle {
        fill_tangent_half_angle(&mut result, layout.num_spans, u_first, u_last - u_first);
    } else if parameterisation != ParameterisationType::Polynomial {
        let quasi_angular: bool = parameterisation == ParameterisationType::QuasiAngular;
        build_algorithmic(&mut result, quasi_angular, layout, u_first, u_last)?;
    } else {
        fill_polynomial(&mut result, layout, u_first, u_last)?;
    }
    Ok(result)
}

// The native switch on the parameterisation.
fn span_layout(parameterisation: ParameterisationType, delta: f64) -> Result<SpanLayout, Failure> {
    use ParameterisationType::*;
    match parameterisation {
        // (int)std::trunc(...) is cvttsd2si on the oracle: INT_MIN when out of range.
        TgtThetaOver2 => Ok(tangent(
            (1.2 * delta / PI).trunc().truncate_to_int().wrapping_add(1),
        )),
        TgtThetaOver2_1 => limited_tangent(1, delta, 0.9999),
        TgtThetaOver2_2 => limited_tangent(2, delta, 1.9999),
        TgtThetaOver2_3 => Ok(tangent(3)),
        TgtThetaOver2_4 => Ok(tangent(4)),
        QuasiAngular => Ok(algorithmic(1, 7, 2, 6)),
        RationalC1 => Ok(algorithmic(2, 8, 3, 4)),
        Polynomial => Ok(SpanLayout {
            order: 0,
            ..algorithmic(1, 8, 2, 7)
        }),
    }
}

fn tangent(num_spans: i32) -> SpanLayout {
    SpanLayout {
        num_spans,
        num_poles: num_spans.wrapping_mul(2).wrapping_add(1),
        num_knots: 1,
        degree: 2,
        order: 0,
        tangent_half_angle: true,
    }
}

fn limited_tangent(num_spans: i32, delta: f64, limit: f64) -> Result<SpanLayout, Failure> {
    if delta > limit * PI {
        return Err(construction(&format!(
            "Convert_ConicToBSplineCurve::BuildCosAndSin: arc {delta} exceeds {limit} * PI for {num_spans} span(s)"
        )));
    }
    Ok(tangent(num_spans))
}

fn algorithmic(num_spans: i32, num_poles: i32, num_knots: i32, degree: i32) -> SpanLayout {
    SpanLayout {
        num_spans,
        num_poles,
        num_knots,
        degree,
        order: degree + 1,
        tangent_half_angle: false,
    }
}

fn allocate(layout: SpanLayout) -> Result<CosAndSin, Failure> {
    let knot_count: i32 = layout.num_spans.wrapping_add(1);
    Ok(CosAndSin {
        cos_numerator: one_based(layout.num_poles)?,
        sin_numerator: one_based(layout.num_poles)?,
        denominator: one_based(layout.num_poles)?,
        degree: layout.degree,
        knots: one_based(knot_count)?,
        mults: one_based(knot_count)?,
    })
}

// Rational quadratic spans of opening 2*alpha, each with its middle pole on the
// intersection of the end tangents.
fn fill_tangent_half_angle(result: &mut CosAndSin, num_spans: i32, u_first: f64, delta: f64) {
    let alpha: f64 = delta / (2.0e0 * f64::from(num_spans));
    let mut param: f64 = u_first;
    set_pole(result, 1, u_first.cos(), u_first.sin(), 1.0e0);
    result.knots[1] = param;
    result.mults[1] = result.degree + 1;
    let direct: f64 = alpha.cos();
    let inverse: f64 = 1.0e0 / direct;
    for ii in 1..=num_spans {
        let (middle, end) = (param + alpha, param + 2.0 * alpha);
        set_pole(
            result,
            2 * ii,
            inverse * middle.cos(),
            inverse * middle.sin(),
            direct,
        );
        set_pole(result, 2 * ii + 1, end.cos(), end.sin(), 1.0e0);
        result.knots[ii + 1] = param + 2.0 * alpha;
        result.mults[ii + 1] = 2;
        param += 2.0 * alpha;
    }
    result.mults[num_spans + 1] = result.degree + 1;
}

/// Stores one pole of the three parallel arrays; `set_pole(&mut cs, 1, 1.0, 0.0, 1.0)`.
pub(super) fn set_pole(result: &mut CosAndSin, index: i32, cos: f64, sin: f64, weight: f64) {
    result.cos_numerator[index] = cos;
    result.sin_numerator[index] = sin;
    result.denominator[index] = weight;
}

// One Bezier span of degree 7 parameterised on [0, 1].
fn fill_polynomial(
    result: &mut CosAndSin,
    layout: SpanLayout,
    u_first: f64,
    u_last: f64,
) -> Result<(), Failure> {
    result.knots[1] = 0.0;
    result.knots[layout.num_knots] = 1.0;
    result.mults[1] = layout.num_poles;
    result.mults[layout.num_knots] = layout.num_poles;
    build_polynomial_cos_and_sin(
        u_first,
        u_last,
        layout.num_poles,
        &mut result.cos_numerator,
        &mut result.sin_numerator,
        &mut result.denominator,
    )
}
