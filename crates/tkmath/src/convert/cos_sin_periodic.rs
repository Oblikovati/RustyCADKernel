// Rust translation of Convert_ConicToBSplineCurve::BuildCosAndSin(Parameterisation, ...) (the periodic overload) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The periodic BuildCosAndSin of the full circle.
use super::ParameterisationType;
use super::arrays::one_based;
use super::cos_sin::{CosAndSin, build_trimmed, construction, set_pole};
use crate::bsplclib::{self, CurveSpan};
use crate::gp::Point3;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

// The periodic RationalC1 circle: degree 4, knots at multiples of PI / 2.
const RATIONAL_C1_DEGREE: i32 = 4;
const RATIONAL_C1_KNOTS: i32 = 5;

/// Periodic `(cos, sin)` on `[0, 2 PI]` for TgtThetaOver2 or RationalC1;
/// `build_periodic(ParameterisationType::RationalC1)?.degree == 4`.
pub(super) fn build_periodic(parameterisation: ParameterisationType) -> Result<CosAndSin, Failure> {
    match parameterisation {
        ParameterisationType::TgtThetaOver2 => periodic_tangent_half_angle(),
        ParameterisationType::RationalC1 => periodic_rational_c1(),
        other => Err(construction(&format!(
            "Convert_ConicToBSplineCurve::BuildCosAndSin: {other:?} has no periodic form; \
             expected TgtThetaOver2 or RationalC1"
        ))),
    }
}

// Three quadratic spans on [0, 2 PI] without the repeated last pole; every
// multiplicity becomes the degree.
fn periodic_tangent_half_angle() -> Result<CosAndSin, Failure> {
    let open: CosAndSin = build_trimmed(ParameterisationType::TgtThetaOver2_3, 0.0e0, 2.0 * PI)?;
    let count: i32 = open.cos_numerator.len() as i32 - 1;
    let mut result = CosAndSin {
        cos_numerator: one_based(count)?,
        sin_numerator: one_based(count)?,
        denominator: one_based(count)?,
        degree: open.degree,
        knots: open.knots,
        mults: open.mults,
    };
    for ii in 1..=count {
        let (cos, sin) = (open.cos_numerator[ii], open.sin_numerator[ii]);
        set_pole(&mut result, ii, cos, sin, open.denominator[ii]);
    }
    result.mults.init(result.degree);
    Ok(result)
}

// Interpolates the RationalC1 half circle, mirrored for the second half, with
// periodic C2 knots of multiplicity 3.
fn periodic_rational_c1() -> Result<CosAndSin, Failure> {
    let half: CosAndSin = build_trimmed(ParameterisationType::RationalC1, 0.0e0, PI)?;
    let num_flat_knots: i32 = (RATIONAL_C1_DEGREE - 1) * RATIONAL_C1_KNOTS + 2 * 2;
    let num_poles: i32 = num_flat_knots - RATIONAL_C1_DEGREE - 1;
    let flat_knots: Array1<'static, f64> = periodic_flat_knots(num_flat_knots)?;
    let mut parameters: Array1<'static, f64> = one_based(num_poles)?;
    bsplclib::build_schoenberg_points(RATIONAL_C1_DEGREE, &flat_knots, &mut parameters)?;
    let mut poles: Array1<'static, Point3> = half_circle_samples(&half, &parameters)?;
    let mut contacts: Array1<'static, i32> = one_based(num_poles)?;
    contacts.init(0);
    // The native call ignores the returned pivot index.
    let mut pivot_index_problem: i32 = 0;
    bsplclib::interpolate(
        RATIONAL_C1_DEGREE,
        &flat_knots,
        &parameters,
        &contacts,
        &mut poles,
        &mut pivot_index_problem,
    )?;
    periodic_result(&poles, num_poles - 2)
}

fn periodic_flat_knots(num_flat_knots: i32) -> Result<Array1<'static, f64>, Failure> {
    let half_pi: f64 = PI * 0.5e0;
    let mut flat_knots: Array1<'static, f64> = one_based(num_flat_knots)?;
    let mut index: i32 = 1;
    let mut push = |value: f64| {
        flat_knots[index] = value;
        index += 1;
    };
    (0..2).for_each(|_| push(-half_pi));
    for ii in 1..=RATIONAL_C1_KNOTS {
        (0..RATIONAL_C1_DEGREE - 1).for_each(|_| push(f64::from(ii - 1) * half_pi));
    }
    (0..2).for_each(|_| push(2.0 * PI + half_pi));
    Ok(flat_knots)
}

// Homogeneous samples of the half circle; parameters past PI take the opposite
// point of the first half, as the sign flip is never reset.
fn half_circle_samples(
    half: &CosAndSin,
    parameters: &Array1<'_, f64>,
) -> Result<Array1<'static, Point3>, Failure> {
    let mut poles: Array1<'static, Point3> = one_based(parameters.len() as i32)?;
    let mut inverse: f64 = 1.0e0;
    for ii in parameters.lower()..=parameters.upper() {
        let mut param: f64 = parameters[ii];
        if param > PI {
            inverse = -1.0e0;
            param -= PI;
        }
        let span = CurveSpan {
            u: param,
            index: 0,
            degree: half.degree,
            periodic: false,
            knots: &half.knots,
            mults: Some(&half.mults),
        };
        let value1: f64 = bsplclib::d0_scalar(&span, &half.cos_numerator, Some(&half.denominator))?;
        let value2: f64 = bsplclib::d0_scalar(&span, &half.sin_numerator, Some(&half.denominator))?;
        let value3: f64 = bsplclib::d0_scalar(&span, &half.denominator, None)?;
        poles[ii] = Point3::new(value1 * value3 * inverse, value2 * value3 * inverse, value3);
    }
    Ok(poles)
}

fn periodic_result(poles: &Array1<'_, Point3>, count: i32) -> Result<CosAndSin, Failure> {
    let mut result = CosAndSin {
        cos_numerator: one_based(count)?,
        sin_numerator: one_based(count)?,
        denominator: one_based(count)?,
        degree: RATIONAL_C1_DEGREE,
        knots: one_based(RATIONAL_C1_KNOTS)?,
        mults: one_based(RATIONAL_C1_KNOTS)?,
    };
    for ii in 1..=RATIONAL_C1_KNOTS {
        result.knots[ii] = f64::from(ii - 1) * (PI * 0.5e0);
        result.mults[ii] = RATIONAL_C1_DEGREE - 1;
    }
    for ii in 1..=count {
        let inverse: f64 = 1.0e0 / poles[ii].z();
        let (cos, sin) = (poles[ii].x() * inverse, poles[ii].y() * inverse);
        set_pole(&mut result, ii, cos, sin, poles[ii].z());
    }
    Ok(result)
}
