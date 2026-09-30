// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (PrepareUnperiodize, Unperiodize, PrepareTrimming, Trimming) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Conversion of periodic curves to clamped ones, and trimming.
use super::insert::insert_knots_flat;
use super::knots::pole_index;
use super::locate::locate_parameter;
use super::native::epsilon;
use super::prepare_insert::prepare_insert_knots;
use super::views::one_based;
use crate::plib::access::{array_cell, array_value};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const TRIMMING: &str = "BSplCLib::Trimming";

/// Numbers of knots and poles of the non-periodic curve equal to a periodic one
/// (`BSplCLib::PrepareUnperiodize`); returns `(NbKnots, NbPoles)`;
/// `bsplclib::prepare_unperiodize(3, &mults)?`.
pub fn prepare_unperiodize(degree: i32, mults: &Array1<'_, i32>) -> Result<(i32, i32), Failure> {
    let mut nb_knots: i32 = mults.len() as i32;
    let mut nb_poles: i32 = -degree - 1;
    for i in mults.lower()..=mults.upper() {
        nb_poles += array_value(mults, i)?;
    }
    // Knots added at the beginning to raise the multiplicity to Degree+1 ...
    let mut sigma: i32 = array_value(mults, mults.lower())?;
    let mut k: i32 = mults.upper() - 1;
    while sigma < degree + 1 {
        let mult: i32 = array_value(mults, k)?;
        sigma += mult;
        nb_poles += mult;
        k -= 1;
        nb_knots += 1;
    }
    // ... exactly up to Degree+1: the excess is suppressed.
    if sigma > degree + 1 {
        nb_poles -= sigma - degree - 1;
    }
    sigma = array_value(mults, mults.upper())?;
    k = mults.lower() + 1;
    while sigma < degree + 1 {
        let mult: i32 = array_value(mults, k)?;
        sigma += mult;
        nb_poles += mult;
        k += 1;
        nb_knots += 1;
    }
    if sigma > degree + 1 {
        nb_poles -= sigma - degree - 1;
    }
    Ok((nb_knots, nb_poles))
}

/// Rewrites a periodic curve as a non-periodic one (`BSplCLib::Unperiodize` on flat
/// poles; the Dimension argument is unused natively).
///
/// All arrays are addressed from index 1, whatever their lower bounds;
/// `bsplclib::unperiodize_flat(3, 3, &m, &k, &p, &mut nm, &mut nk, &mut np)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native eight-argument signature.
pub fn unperiodize_flat(
    degree: i32,
    _dimension: i32,
    mults: &Array1<'_, i32>,
    knots: &Array1<'_, f64>,
    poles: &Array1<'_, f64>,
    new_mults: &mut Array1<'_, i32>,
    new_knots: &mut Array1<'_, f64>,
    new_poles: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    // index: the number of knots to insert before knot(1) to reach Degree+1.
    let mut sigma: i32 = array_value(mults, mults.lower())?;
    let mut k: i32 = mults.upper() - 1;
    let mut index: i32 = 0;
    while sigma < degree + 1 {
        sigma += array_value(mults, k)?;
        k -= 1;
        index += 1;
    }
    let period: f64 = array_value(knots, knots.upper())? - array_value(knots, knots.lower())?;
    let length: i32 = knots.len() as i32;
    for k in 1..=length {
        *array_cell(new_knots, k + index)? = array_value(knots, k)?;
        *array_cell(new_mults, k + index)? = array_value(mults, k)?;
    }
    for k in 1..=index {
        *array_cell(new_knots, k)? = array_value(new_knots, k + length - 1)? - period;
        *array_cell(new_mults, k)? = array_value(new_mults, k + length - 1)?;
    }
    *array_cell(new_mults, 1)? -= sigma - degree - 1;
    let ends = UnperiodizedEnd {
        degree,
        length,
        period,
    };
    ends.set_ending_knots(index, new_knots, new_mults)?;
    let count: i32 = poles.len() as i32;
    for k in 1..=new_poles.len() as i32 {
        *array_cell(new_poles, k)? = array_value(poles, (k - 1) % count + 1)?;
    }
    Ok(())
}

struct UnperiodizedEnd {
    degree: i32,
    length: i32,
    period: f64,
}

impl UnperiodizedEnd {
    fn set_ending_knots(
        &self,
        index: i32,
        new_knots: &mut Array1<'_, f64>,
        new_mults: &mut Array1<'_, i32>,
    ) -> Result<(), Failure> {
        let mut sigma: i32 = array_value(new_mults, index + self.length)?;
        for k in self.length + index + 1..=new_knots.len() as i32 {
            let source: i32 = k - self.length + 1;
            *array_cell(new_knots, k)? = array_value(new_knots, source)? + self.period;
            let mult: i32 = array_value(new_mults, source)?;
            *array_cell(new_mults, k)? = mult;
            sigma += mult;
        }
        let last: i32 = new_mults.len() as i32;
        *array_cell(new_mults, last)? -= sigma - self.degree - 1;
        Ok(())
    }
}

/// Numbers of knots and poles of the curve trimmed to `[u1, u2]`
/// (`BSplCLib::PrepareTrimming`); returns `(NbKnots, NbPoles)`;
/// `bsplclib::prepare_trimming(3, false, &knots, &mults, 0.2, 0.8)?`.
pub fn prepare_trimming(
    degree: i32,
    periodic: bool,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    u1: f64,
    u2: f64,
) -> Result<(i32, i32), Failure> {
    let (mut index1, mut index2): (i32, i32) = (0, 0);
    let (mut new_u1, mut new_u2): (f64, f64) = (0.0, 0.0);
    let (lower, upper): (i32, i32) = (knots.lower(), knots.upper());
    locate_parameter(
        degree,
        knots,
        mults,
        u1,
        periodic,
        lower,
        upper,
        &mut index1,
        &mut new_u1,
    )?;
    locate_parameter(
        degree,
        knots,
        mults,
        u2,
        periodic,
        lower,
        upper,
        &mut index2,
        &mut new_u2,
    )?;
    index1 += 1;
    if (array_value(knots, index2)? - u2).abs() <= epsilon(u1) {
        index2 -= 1;
    }
    let mut nb_poles: i32 = degree + 1;
    for i in index1..=index2 {
        nb_poles += array_value(mults, i)?;
    }
    Ok((index2 - index1 + 3, nb_poles))
}

/// Trims a curve with `dimension` interleaved pole values to `[u1, u2]`
/// (`BSplCLib::Trimming` on flat poles).
///
/// Size the outputs with [`prepare_trimming`]; they are addressed from index 1. Fails
/// with Standard_OutOfRange when the bounds are outside the curve, as natively;
/// `bsplclib::trimming_flat(3, false, 3, &k, &m, &p, 0.2, 0.8, &mut nk, &mut nm, &mut np)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native eleven-argument signature.
pub fn trimming_flat(
    degree: i32,
    periodic: bool,
    dimension: i32,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    poles: &Array1<'_, f64>,
    u1: f64,
    u2: f64,
    new_knots: &mut Array1<'_, f64>,
    new_mults: &mut Array1<'_, i32>,
    new_poles: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let mut bounds: Array1<'static, f64> = one_based(TRIMMING, 2)?;
    bounds.as_mut_slice().copy_from_slice(&[u1, u2]);
    let mut bound_mults: Array1<'static, i32> = one_based(TRIMMING, 2)?;
    bound_mults.init(degree);
    let (mut nbpoles, mut nbknots): (i32, i32) = (0, 0);
    let tolerance: f64 = epsilon(u1);
    let prepared: bool = prepare_insert_knots(
        degree,
        periodic,
        knots,
        mults,
        &bounds,
        Some(&bound_mults),
        &mut nbpoles,
        &mut nbknots,
        tolerance,
        false,
    )?;
    if !prepared {
        return Err(Failure::out_of_range(""));
    }
    let mut temp_poles: Array1<'static, f64> = one_based(TRIMMING, nbpoles * dimension)?;
    let mut temp_knots: Array1<'static, f64> = one_based(TRIMMING, nbknots)?;
    let mut temp_mults: Array1<'static, i32> = one_based(TRIMMING, nbknots)?;
    // Multiplicities are not added: they must stay below the degree.
    insert_knots_flat(
        degree,
        periodic,
        dimension,
        poles,
        knots,
        mults,
        &bounds,
        Some(&bound_mults),
        &mut temp_poles,
        &mut temp_knots,
        &mut temp_mults,
        tolerance,
        false,
    )?;
    // Finds in TempPoles the pole corresponding to U1.
    let (mut kindex, mut new_u1): (i32, f64) = (0, 0.0);
    let (lower, upper): (i32, i32) = (temp_knots.lower(), temp_knots.upper());
    locate_parameter(
        degree,
        &temp_knots,
        &temp_mults,
        u1,
        periodic,
        lower,
        upper,
        &mut kindex,
        &mut new_u1,
    )?;
    let pindex: i32 = pole_index(degree, kindex, periodic, &temp_mults)? * dimension;
    for i in 1..=new_poles.len() as i32 {
        *array_cell(new_poles, i)? = array_value(&temp_poles, pindex + i)?;
    }
    for i in 1..=new_knots.len() as i32 {
        *array_cell(new_knots, i)? = array_value(&temp_knots, kindex + i - 1)?;
        *array_cell(new_mults, i)? = array_value(&temp_mults, kindex + i - 1)?;
    }
    let last: i32 = new_mults.len() as i32;
    *array_cell(new_mults, 1)? = degree.min(array_value(new_mults, 1)?) + 1;
    *array_cell(new_mults, last)? = degree.min(array_value(new_mults, last)?) + 1;
    Ok(())
}
