// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_CurveComputation.pxx (BSplCLib_BuildCache, BSplCLib_PolesCoefficients_Bezier) and BSplCLib.cxx (FlatBezierKnots, UnitWeights, UnitWeightsData) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Conversion of a span to polynomial (Taylor) coefficients for fast evaluation.
use super::bohm::bohm;
use super::build::build_eval;
use super::curve_eval::{CurveSpan, prepare_eval};
use super::point::CurvePoint;
use crate::plib::access::array_cell;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// Size of the shared unit-weight buffer (`BSplCLib::MaxUnitWeightsSize()`);
/// `bsplclib::max_unit_weights_size() == 2049`.
pub const fn max_unit_weights_size() -> i32 {
    UNIT_WEIGHTS_SIZE as i32
}

const UNIT_WEIGHTS_SIZE: usize = 2049;
static UNIT_WEIGHTS: [f64; UNIT_WEIGHTS_SIZE] = [1.0; UNIT_WEIGHTS_SIZE];

/// The shared buffer of 2049 unit weights (`BSplCLib::UnitWeightsData()`);
/// `bsplclib::unit_weights_data()[0] == 1.0`.
pub fn unit_weights_data() -> &'static [f64; UNIT_WEIGHTS_SIZE] {
    &UNIT_WEIGHTS
}

/// Weights 1..n all equal to 1 (`BSplCLib::UnitWeights`): a non-owning view of the
/// shared buffer up to [`max_unit_weights_size`], an owned array beyond. A negative
/// count, which natively builds a view of absurd size, is rejected;
/// `bsplclib::unit_weights(10)?.is_deletable() == false`.
pub fn unit_weights(nb_elems: i32) -> Result<Array1<'static, f64>, Failure> {
    let count: usize = usize::try_from(nb_elems).map_err(|_| {
        Failure::out_of_range(&format!(
            "BSplCLib::UnitWeights: {nb_elems} elements; expected >= 0"
        ))
    })?;
    if count <= UNIT_WEIGHTS_SIZE {
        // Non-owning view over the shared array (zero allocation).
        return Ok(Array1::from_shared(&UNIT_WEIGHTS[..count], 1));
    }
    let mut result: Array1<'static, f64> = Array1::with_bounds(1, nb_elems)
        .map_err(|error| Failure::out_of_range(&error.to_string()))?;
    result.init(1.0);
    Ok(result)
}

// Flat knots of Bezier curves up to degree 25: 26 zeros then 26 ones.
static BEZIER_KNOTS: [f64; 52] = {
    let mut knots: [f64; 52] = [0.0; 52];
    let mut i: usize = 26;
    while i < 52 {
        knots[i] = 1.0;
        i += 1;
    }
    knots
};

/// Flat knots of a Bezier curve of `degree`: the leading `2*(degree+1)` values of the
/// returned slice (`BSplCLib::FlatBezierKnots`); fails for degrees outside 1..=25;
/// `bsplclib::flat_bezier_knots(3)?[..8] == [0., 0., 0., 0., 1., 1., 1., 1.]`.
pub fn flat_bezier_knots(degree: i32) -> Result<&'static [f64], Failure> {
    if !(1..=super::MAX_DEGREE).contains(&degree) {
        return Err(Failure::out_of_range(
            "Bezier curve degree greater than maximal supported",
        ));
    }
    Ok(&BEZIER_KNOTS[(25 - degree) as usize..])
}

// Bohm to all orders on the span of `u` located from flat knots.
fn span_taylor<P: CurvePoint>(
    u: f64,
    index: i32,
    periodic: bool,
    degree: i32,
    flat_knots: &Array1<'_, f64>,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
) -> Result<super::curve_eval::LocalSpan, Failure> {
    let span = CurveSpan {
        u,
        index,
        degree,
        periodic,
        knots: flat_knots,
        mults: None,
    };
    let mut local = prepare_eval(&span, weights, P::DIMENSION, |index, weights, lp| {
        build_eval(degree, index, poles, weights, lp)
    })?;
    bohm(
        local.u,
        degree,
        degree,
        &local.knots,
        local.dim as i32,
        &mut local.poles,
    )?;
    Ok(local)
}

/// Taylor coefficients of the span containing `u`, scaled by powers of `span_domain`
/// (`BSplCLib::BuildCache` into point arrays indexed from 1).
///
/// A non-rational span of a weighted curve gets cache weights `1, 0, 0, ..`;
/// `bsplclib::build_cache(0.5, 1.0, false, 3, &flat, &poles, None, &mut cache, None)?`.
pub fn build_cache<P: CurvePoint>(
    u: f64,
    span_domain: f64,
    periodic: bool,
    degree: i32,
    flat_knots: &Array1<'_, f64>,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    cache_poles: &mut Array1<'_, P>,
    cache_weights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    let local = span_taylor(u, 0, periodic, degree, flat_knots, poles, weights)?;
    let stride: usize = local.dim;
    let mut value: f64 = 1.0e0;
    for ii in 1..=degree + 1 {
        let row: usize = (ii - 1) as usize * stride;
        let scaled: Vec<f64> = local.poles[row..row + P::DIMENSION]
            .iter()
            .map(|c| c * value)
            .collect();
        *array_cell(cache_poles, ii)? = P::point_from(&scaled);
        value *= span_domain / f64::from(ii);
    }
    if weights.is_none() {
        return Ok(());
    }
    let cache_weights: &mut Array1<'_, f64> = cache_weights.ok_or_else(|| {
        Failure::out_of_range(
            "BSplCLib::BuildCache: weighted poles need CacheWeights; received None",
        )
    })?;
    if !local.rational {
        for ii in 1..=degree + 1 {
            *array_cell(cache_weights, ii)? = 0.0e0;
        }
        *array_cell(cache_weights, 1)? = 1.0e0;
        return Ok(());
    }
    value = 1.0e0;
    for ii in 1..=degree + 1 {
        let row: usize = (ii - 1) as usize * stride;
        *array_cell(cache_weights, ii)? = local.poles[row + P::DIMENSION] * value;
        value *= span_domain / f64::from(ii);
    }
    Ok(())
}

/// Taylor coefficients of the span `span_index` written row by row into a cache
/// matrix of `Degree+1` rows (`BSplCLib::BuildCache` into NCollection_Array2).
///
/// Rows hold the coordinates, then the weight for rational spans; a weighted but
/// locally non-rational span gets a weight column `1, 0, 0, ..`;
/// `bsplclib::build_cache_matrix(0.5, 1.0, false, 3, 4, &flat, &poles, None, &mut cache)?`.
pub fn build_cache_matrix<P: CurvePoint>(
    parameter: f64,
    span_domain: f64,
    periodic_flag: bool,
    degree: i32,
    span_index: i32,
    flat_knots: &Array1<'_, f64>,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    cache_array: &mut Array2<'_, f64>,
) -> Result<(), Failure> {
    let local = span_taylor(
        parameter,
        span_index,
        periodic_flag,
        degree,
        flat_knots,
        poles,
        weights,
    )?;
    let dimension: usize = local.dim;
    let shifted: bool = weights.is_some() && !local.rational;
    let row_length: usize = dimension + usize::from(shifted);
    let rows: usize = (degree + 1).max(0) as usize;
    let cache: &mut [f64] = cache_array.as_mut_slice();
    if cache.len() < rows * row_length {
        return Err(Failure::out_of_range(&format!(
            "BSplCLib::BuildCache: cache of {} values; expected at least {}",
            cache.len(),
            rows * row_length
        )));
    }
    let mut coeff: f64 = 1.0;
    for i in 0..rows {
        for j in 0..dimension {
            cache[i * row_length + j] = local.poles[i * dimension + j] * coeff;
        }
        coeff *= span_domain / (i + 1) as f64;
        if shifted {
            cache[i * row_length + dimension] = 0.0;
        }
    }
    if shifted {
        let (row, col): (i32, i32) = (cache_array.lower_row(), cache_array.lower_col());
        let target: i32 = col + row_length as i32 - 1;
        cache_array
            .set_value(row, target, 1.0)
            .map_err(|error| Failure::out_of_range(&error.to_string()))?;
    }
    Ok(())
}

/// Polynomial coefficients of a Bezier curve (`BSplCLib::PolesCoefficients`; the
/// weightless overload passes None twice);
/// `bsplclib::poles_coefficients(&poles, None, &mut coefficients, None)?`.
pub fn poles_coefficients<P: CurvePoint>(
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    cache_poles: &mut Array1<'_, P>,
    cache_weights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    let degree: i32 = poles.len() as i32 - 1;
    let knots: &'static [f64] = flat_bezier_knots(degree)?;
    let flat: Array1<'static, f64> = Array1::from_shared(&knots[..2 * (degree as usize + 1)], 1);
    build_cache(
        0.,
        1.,
        false,
        degree,
        &flat,
        poles,
        weights,
        cache_poles,
        cache_weights,
    )
}
