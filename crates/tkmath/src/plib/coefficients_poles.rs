// Rust translation of FoundationClasses/TKMath/PLib/PLib.cxx (CoefficientsPoles on curves) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Conversion of canonical polynomial coefficients to Bezier poles.
use super::access::{array_cell, array_value, failure};
use super::binomial::bin;
use super::poles::{PolePoint, get_poles, set_poles};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const NAME: &str = "PLib::CoefficientsPoles";

/// Paired weight arrays of a rational conversion: (input coefficients, output weights).
type WeightPair<'r, 'a, 'b> = Option<(&'r Array1<'a, f64>, &'r mut Array1<'b, f64>)>;

/// Converts point coefficients to poles through flat temporaries (`PLib::CoefficientsPoles`).
///
/// Natively the pole temporary is seeded from the coefficients; longer pole arrays
/// would read uninitialized storage, so Rust rejects `poles.len() > coefs.len()`;
/// `plib::coefficients_poles(&coefs, None, &mut poles, None)?`.
pub fn coefficients_poles<P: PolePoint>(
    coefs: &Array1<'_, P>,
    wcoefs: Option<&Array1<'_, f64>>,
    poles: &mut Array1<'_, P>,
    weights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    if poles.len() > coefs.len() {
        return Err(failure(
            "Standard_DimensionError",
            &format!(
                "{NAME}: {} poles; expected at most {} coefficients",
                poles.len(),
                coefs.len()
            ),
        ));
    }
    let mut temp_c: Array1<'static, f64> = flat_array(P::DIMENSION * coefs.len())?;
    set_poles(coefs, &mut temp_c)?;
    let mut temp_p: Array1<'static, f64> = flat_array(P::DIMENSION * poles.len())?;
    set_poles(coefs, &mut temp_p)?;
    coefficients_poles_flat(P::DIMENSION as i32, &temp_c, wcoefs, &mut temp_p, weights)?;
    get_poles(&temp_p, poles)
}

pub(super) fn flat_array(length: usize) -> Result<Array1<'static, f64>, Failure> {
    let upper: i32 = i32::try_from(length).map_err(|_| {
        failure(
            "Standard_RangeError",
            &format!("{length} values exceed i32 bounds"),
        )
    })?;
    Array1::with_bounds(1, upper)
        .map_err(|error| failure("Standard_RangeError", &error.to_string()))
}

/// Converts scalar coefficients (`PLib::CoefficientsPoles` on real arrays, dimension 1);
/// `plib::coefficients_poles_scalar(&coefs, None, &mut poles, None)?`.
pub fn coefficients_poles_scalar(
    coefs: &Array1<'_, f64>,
    wcoefs: Option<&Array1<'_, f64>>,
    poles: &mut Array1<'_, f64>,
    weights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    coefficients_poles_flat(1, coefs, wcoefs, poles, weights)
}

/// Converts interleaved `dim`-component coefficients in place of the poles array
/// (`PLib::CoefficientsPoles(dim, ...)`).
///
/// Weighted input requires an output weight array (a null pointer natively);
/// `plib::coefficients_poles_flat(3, &coefs, None, &mut poles, None)?`.
pub fn coefficients_poles_flat(
    dim: i32,
    coefs: &Array1<'_, f64>,
    wcoefs: Option<&Array1<'_, f64>>,
    poles: &mut Array1<'_, f64>,
    weights: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    if dim <= 0 {
        return Err(Failure::out_of_range(&format!(
            "{NAME}: dim = {dim}; expected > 0"
        )));
    }
    let mut pair: WeightPair = match (wcoefs, weights) {
        (None, _) => None,
        (Some(source), Some(target)) => Some((source, target)),
        (Some(_), None) => {
            return Err(Failure::new(&format!(
                "{NAME}: weighted input without output weights"
            )));
        }
    };
    let reflen: i32 = coefs.len() as i32 / dim;
    copy_extremities(dim, coefs, poles, &mut pair)?;
    scale_interior(dim, reflen, coefs, poles, &mut pair)?;
    accumulate(dim, reflen, poles, &mut pair)?;
    divide_by_weights(dim, reflen, poles, &pair)
}

fn copy_extremities(
    dim: i32,
    coefs: &Array1<'_, f64>,
    poles: &mut Array1<'_, f64>,
    pair: &mut WeightPair,
) -> Result<(), Failure> {
    let (loc, upc, lop, upp) = (coefs.lower(), coefs.upper(), poles.lower(), poles.upper());
    for i in 0..dim {
        *array_cell(poles, lop + i)? = array_value(coefs, loc + i)?;
        *array_cell(poles, upp - i)? = array_value(coefs, upc - i)?;
    }
    if let Some((source, target)) = pair {
        let (lowp, upwp) = (target.lower(), target.upper());
        *array_cell(target, lowp)? = array_value(source, source.lower())?;
        *array_cell(target, upwp)? = array_value(source, source.upper())?;
    }
    Ok(())
}

// Divides interior coefficients by C(reflen-1, i-1).
fn scale_interior(
    dim: i32,
    reflen: i32,
    coefs: &Array1<'_, f64>,
    poles: &mut Array1<'_, f64>,
    pair: &mut WeightPair,
) -> Result<(), Failure> {
    let (loc, lop) = (coefs.lower(), poles.lower());
    for i in 2..reflen {
        let cnp: f64 = bin(reflen - 1, i - 1)?;
        if let Some((source, target)) = pair {
            let lowp: i32 = target.lower();
            *array_cell(target, lowp + i - 1)? = array_value(source, source.lower() + i - 1)? / cnp;
        }
        for j in 0..dim {
            *array_cell(poles, lop + dim * (i - 1) + j)? =
                array_value(coefs, loc + dim * (i - 1) + j)? / cnp;
        }
    }
    Ok(())
}

// Repeated forward sums turn scaled monomial coefficients into Bernstein poles.
fn accumulate(
    dim: i32,
    reflen: i32,
    poles: &mut Array1<'_, f64>,
    pair: &mut WeightPair,
) -> Result<(), Failure> {
    let lop: i32 = poles.lower();
    for i in 1..reflen {
        for j in (i..reflen).rev() {
            if let Some((_, target)) = pair {
                let lowp: i32 = target.lower();
                let previous: f64 = array_value(target, lowp + j - 1)?;
                *array_cell(target, lowp + j)? += previous;
            }
            for k in 0..dim {
                let previous: f64 = array_value(poles, lop + dim * (j - 1) + k)?;
                *array_cell(poles, lop + dim * j + k)? += previous;
            }
        }
    }
    Ok(())
}

fn divide_by_weights(
    dim: i32,
    reflen: i32,
    poles: &mut Array1<'_, f64>,
    pair: &WeightPair,
) -> Result<(), Failure> {
    let Some((_, target)) = pair else {
        return Ok(());
    };
    let (lop, lowp) = (poles.lower(), target.lower());
    for i in 1..=reflen {
        for j in 0..dim {
            *array_cell(poles, lop + dim * (i - 1) + j)? /= array_value(target, lowp + i - 1)?;
        }
    }
    Ok(())
}
