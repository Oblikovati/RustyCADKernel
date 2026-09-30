// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (KnotForm..IsRational) and BSplCLib_CurveComputation.pxx (BSplCLib_Reverse) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Knot distribution analysis, reparametrization and reversal.
use super::knots::{first_u_knot_index, last_u_knot_index};
use super::native::{epsilon, next_toward_real_last, std_max, std_min};
use super::{KnotDistribution, MultDistribution};
use crate::geom_abs::BSplKnotDistribution;
use crate::plib::access::{array_cell, array_value};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// Tests uniform knot spacing between K1 and K2 (`BSplCLib::KnotForm`);
/// `bsplclib::knot_form(&knots, 1, 4)? == KnotDistribution::Uniform`.
pub fn knot_form(
    knots: &Array1<'_, f64>,
    from_k1: i32,
    to_k2: i32,
) -> Result<KnotDistribution, Failure> {
    if from_k1 + 1 > knots.upper() {
        return Ok(KnotDistribution::Uniform);
    }
    let mut ui: f64 = array_value(knots, from_k1)?.abs();
    let mut uj: f64 = array_value(knots, from_k1 + 1)?.abs();
    let mut du0: f64 = (uj - ui).abs();
    let mut eps: f64 = epsilon(ui) + epsilon(uj) + epsilon(du0);
    for i in from_k1 + 1..to_k2 {
        ui = array_value(knots, i)?.abs();
        uj = array_value(knots, i + 1)?.abs();
        let du1: f64 = (uj - ui).abs();
        if (du1 - du0).abs() > eps {
            return Ok(KnotDistribution::NonUniform);
        }
        du0 = du1;
        eps = epsilon(ui) + epsilon(uj) + epsilon(du0);
    }
    Ok(KnotDistribution::Uniform)
}

/// Classifies the multiplicities between K1 and K2 (`BSplCLib::MultForm`);
/// `bsplclib::mult_form(&mults, 1, 3)? == MultDistribution::Constant`.
pub fn mult_form(
    mults: &Array1<'_, i32>,
    from_k1: i32,
    to_k2: i32,
) -> Result<MultDistribution, Failure> {
    let (first, last): (i32, i32) = (from_k1.min(to_k2), from_k1.max(to_k2));
    if first + 1 > mults.upper() {
        return Ok(MultDistribution::Constant);
    }
    let first_mult: i32 = array_value(mults, first)?;
    let mut form: MultDistribution = MultDistribution::Constant;
    let mut mult: i32 = array_value(mults, first + 1)?;
    let mut i: i32 = first + 1;
    while i <= last && form != MultDistribution::NonConstant {
        let current: i32 = array_value(mults, i)?;
        form = next_mult_form(form, i == first + 1, i == last, [first_mult, mult, current]);
        if i != first + 1 && i != last {
            mult = current;
        }
        i += 1;
    }
    Ok(form)
}

// One step of the native loop; `values` holds the first, previous and current multiplicity.
fn next_mult_form(
    form: MultDistribution,
    is_second: bool,
    is_last: bool,
    [first_mult, mult, current]: [i32; 3],
) -> MultDistribution {
    if is_second {
        // The native loop compares the cached second multiplicity with the first one.
        if mult != first_mult {
            return MultDistribution::QuasiConstant;
        }
        return form;
    }
    let reference: i32 = if is_last && form == MultDistribution::QuasiConstant {
        first_mult
    } else {
        mult
    };
    if reference != current {
        return MultDistribution::NonConstant;
    }
    form
}

/// Classifies the knot vector and finds the greatest interior multiplicity
/// (`BSplCLib::KnotAnalysis`); returns `(KnotForm, MaxKnotMult)`;
/// `bsplclib::knot_analysis(3, false, &knots, &mults)?`.
pub fn knot_analysis(
    degree: i32,
    periodic: bool,
    c_knots: &Array1<'_, f64>,
    c_mults: &Array1<'_, i32>,
) -> Result<(BSplKnotDistribution, i32), Failure> {
    let mut knot_form_result: BSplKnotDistribution = BSplKnotDistribution::NonUniform;
    if knot_form(c_knots, 1, c_knots.len() as i32)? == KnotDistribution::Uniform {
        knot_form_result = uniform_knot_form(degree, c_knots, c_mults)?;
    }
    let first_km: i32 = if periodic {
        c_knots.lower()
    } else {
        first_u_knot_index(degree, c_mults)?
    };
    let last_km: i32 = if periodic {
        c_knots.upper()
    } else {
        last_u_knot_index(degree, c_mults)?
    };
    let mut max_knot_mult: i32 = 0;
    if last_km - first_km != 1 {
        for i in first_km + 1..last_km {
            max_knot_mult = max_knot_mult.max(array_value(c_mults, i)?);
        }
    }
    Ok((knot_form_result, max_knot_mult))
}

// The GeomAbs form of uniformly spaced knots, from their multiplicity pattern.
fn uniform_knot_form(
    degree: i32,
    c_knots: &Array1<'_, f64>,
    c_mults: &Array1<'_, i32>,
) -> Result<BSplKnotDistribution, Failure> {
    match mult_form(c_mults, 1, c_mults.len() as i32)? {
        MultDistribution::NonConstant => Ok(BSplKnotDistribution::NonUniform),
        MultDistribution::Constant if c_knots.len() == 2 => {
            Ok(BSplKnotDistribution::PiecewiseBezier)
        }
        MultDistribution::Constant if array_value(c_mults, 1)? == 1 => {
            Ok(BSplKnotDistribution::Uniform)
        }
        MultDistribution::Constant => Ok(BSplKnotDistribution::NonUniform),
        MultDistribution::QuasiConstant => quasi_constant_form(degree, c_mults),
    }
}

fn quasi_constant_form(
    degree: i32,
    c_mults: &Array1<'_, i32>,
) -> Result<BSplKnotDistribution, Failure> {
    if array_value(c_mults, 1)? != degree + 1 {
        return Ok(BSplKnotDistribution::NonUniform);
    }
    // The native code stores CMults(2) in a double before comparing.
    let second: f64 = f64::from(array_value(c_mults, 2)?);
    if second == f64::from(degree) {
        return Ok(BSplKnotDistribution::PiecewiseBezier);
    }
    if second == 1.0 {
        return Ok(BSplKnotDistribution::QuasiUniform);
    }
    Ok(BSplKnotDistribution::NonUniform)
}

/// Maps the knots onto `[min(U1,U2), max(U1,U2)]` keeping their form
/// (`BSplCLib::Reparametrize`); `bsplclib::reparametrize(0.0, 1.0, &mut knots)?`.
pub fn reparametrize(u1: f64, u2: f64, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
    let (lower, upper): (i32, i32) = (knots.lower(), knots.upper());
    let u_first: f64 = std_min(u1, u2);
    let new_length: f64 = std_max(u1, u2) - u_first;
    if knot_form(knots, lower, upper)? == KnotDistribution::Uniform {
        let du: f64 = new_length / f64::from(upper - lower);
        *array_cell(knots, lower)? = u_first;
        for i in lower + 1..=upper {
            *array_cell(knots, i)? = array_value(knots, i - 1)? + du;
        }
        return Ok(());
    }
    let mut k1: f64 = array_value(knots, lower)?;
    let length: f64 = array_value(knots, upper)? - k1;
    *array_cell(knots, lower)? = u_first;
    for i in lower + 1..=upper {
        let k2: f64 = array_value(knots, i)?;
        let ratio: f64 = (k2 - k1) / length;
        let previous: f64 = array_value(knots, i - 1)?;
        let mut knot: f64 = previous + (new_length * ratio);
        // For CheckCurveData: keep the knots strictly increasing.
        let eps: f64 = epsilon(previous.abs());
        if knot - previous <= eps {
            knot = next_toward_real_last(previous + eps);
        }
        *array_cell(knots, i)? = knot;
        k1 = k2;
    }
    Ok(())
}

/// Reverses a knot vector keeping its first and last values
/// (`BSplCLib::Reverse(Knots)`); `bsplclib::reverse_knots(&mut knots)?`.
pub fn reverse_knots(knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
    let (mut first, mut last): (i32, i32) = (knots.lower(), knots.upper());
    let mut kfirst: f64 = array_value(knots, first)?;
    let mut klast: f64 = array_value(knots, last)?;
    let (mut tfirst, mut tlast): (f64, f64) = (kfirst, klast);
    first += 1;
    last -= 1;
    while first <= last {
        tfirst += klast - array_value(knots, last)?;
        tlast -= array_value(knots, first)? - kfirst;
        kfirst = array_value(knots, first)?;
        klast = array_value(knots, last)?;
        *array_cell(knots, first)? = tfirst;
        *array_cell(knots, last)? = tlast;
        first += 1;
        last -= 1;
    }
    Ok(())
}

/// Reverses the multiplicities (`BSplCLib::Reverse(Mults)`); `bsplclib::reverse_mults(&mut mults)`.
pub fn reverse_mults(mults: &mut Array1<'_, i32>) {
    mults.as_mut_slice().reverse();
}

/// Reverses poles or weights so that entry `last` becomes the first
/// (`BSplCLib::Reverse(Poles, Last)` / `Reverse(Weights, Last)`).
///
/// `Lower..L` and `L+1..Upper` are reversed separately, with
/// `L = Lower + (last - Lower) % Length`; `bsplclib::reverse_poles(&mut poles, poles.upper())?`.
pub fn reverse_poles<T>(array: &mut Array1<'_, T>, last: i32) -> Result<(), Failure> {
    let lower: i32 = array.lower();
    let length: i32 = array.len() as i32;
    if length == 0 {
        return Err(Failure::out_of_range(&format!(
            "BSplCLib::Reverse: empty array with lower bound {lower}"
        )));
    }
    let split: i32 = lower + (last - lower) % length;
    if split < lower {
        return Err(Failure::out_of_range(&format!(
            "BSplCLib::Reverse: Last = {last} maps to index {split}; expected >= {lower}"
        )));
    }
    let offset: usize = (split - lower) as usize;
    let values: &mut [T] = array.as_mut_slice();
    values[..=offset].reverse();
    values[offset + 1..].reverse();
    Ok(())
}

/// Tests whether weights differ between I1 and I2, cyclically (`BSplCLib::IsRational`).
///
/// The Epsilon argument is ignored natively: weights are compared exactly;
/// `bsplclib::is_rational(&weights, 1, 3, 0.0)?`.
pub fn is_rational(
    weights: &Array1<'_, f64>,
    i1: i32,
    i2: i32,
    _epsilon: f64,
) -> Result<bool, Failure> {
    let f: i32 = weights.lower();
    let l: i32 = weights.len() as i32;
    if l == 0 {
        return Err(Failure::out_of_range(
            "BSplCLib::IsRational: empty weight array; expected at least one weight",
        ));
    }
    let i3: i32 = i2 - f;
    for i in i1 - f..i3 {
        if array_value(weights, f + (i % l))? != array_value(weights, f + ((i + 1) % l))? {
            return Ok(true);
        }
    }
    Ok(false)
}
