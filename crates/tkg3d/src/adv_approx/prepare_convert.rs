// Rust translation of ModelingData/TKG3d/AdvApprox/AdvApprox_ApproxAFunction.cxx (file-static PrepareConvert) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The local continuity of each junction of the polynomial pieces, before they are
//! converted to one B-spline.
use crate::geom::errors;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkmath::plib;

/// Subspace counts `(Num1DSS, Num2DSS, Num3DSS)`.
pub(super) type SubSpaces = (i32, i32, i32);

/// The polynomial pieces the continuity check reads.
pub(super) struct Pieces<'r, 'a> {
    pub num_coeff_per_curve: &'r Array1<'a, i32>,
    pub coefficients: &'r Array1<'a, f64>,
    pub polynomial_intervals: &'r Array2<'a, f64>,
    pub true_intervals: &'r Array1<'a, f64>,
}

// Scale factors of one derivative order at one junction (native facteur and normal).
struct Factors {
    facteur1: f64,
    facteur2: f64,
    normal1: f64,
    normal2: f64,
}

// The per-subspace jump bounds of one order, or None when the jump is too large.
type Jumps = Option<(Vec<f64>, Vec<f64>)>;

/// `PrepareConvert`: for each junction, the highest order up to `continuity_order`
/// whose derivative jump stays below 1% of the tolerance, the jump bounds being
/// added to `error_max`; `prepare_convert(2, 13, 1, (0, 0, 1), &pieces, &tol, &mut e, &mut c)?`.
#[allow(clippy::too_many_arguments)] // The native argument list.
pub(super) fn prepare_convert(
    num_curves: i32,
    max_degree: i32,
    continuity_order: i32,
    spaces: SubSpaces,
    pieces: &Pieces<'_, '_>,
    local_tolerance: &Array1<'_, f64>,
    error_max: &mut Array1<'_, f64>,
    continuity: &mut Array1<'_, i32>,
) -> Result<(), Failure> {
    continuity.init(0);
    if continuity_order == 0 {
        return Ok(());
    }
    for icurve in 1..num_curves {
        let (res1, res2) = junction_values(icurve, max_degree, continuity_order, spaces, pieces)?;
        for iordre in 1..=continuity_order {
            let factors: Factors = factors(icurve, iordre, pieces)?;
            let Some((prec, suivant)) =
                jumps(&res1, &res2, iordre, spaces, &factors, local_tolerance)?
            else {
                break;
            };
            put(continuity, icurve + 1, iordre)?;
            accumulate(error_max, icurve, &prec, &suivant)?;
        }
    }
    Ok(())
}

// Derivatives 0..order at the end of piece `icurve` and the start of the next one.
fn junction_values(
    icurve: i32,
    max_degree: i32,
    order: i32,
    (n1, n2, n3): SubSpaces,
    pieces: &Pieces<'_, '_>,
) -> Result<(Vec<f64>, Vec<f64>), Failure> {
    let dimension: i32 = n1 + 2 * n2 + 3 * n3;
    let real_degree: i32 = (max_degree + 1).max(2 * order + 2);
    let stride: usize = (dimension * real_degree) as usize;
    let coefficients: &[f64] = pieces.coefficients.as_slice();
    let start1: usize = (icurve - 1) as usize * stride;
    let lower: i32 = pieces.num_coeff_per_curve.lower();
    let mut res1: Vec<f64> = vec![0.0; ((order + 1) * dimension) as usize];
    let mut res2: Vec<f64> = res1.clone();
    let deg1: i32 = at(pieces.num_coeff_per_curve, lower + icurve - 1)? - 1;
    let u1: f64 = interval(pieces.polynomial_intervals, icurve, 2)?;
    let coef1: &[f64] = coefficients.get(start1..).unwrap_or(&[]);
    plib::eval_polynomial(u1, order, deg1, dimension, coef1, &mut res1)?;
    let deg2: i32 = at(pieces.num_coeff_per_curve, lower + icurve)? - 1;
    let u2: f64 = interval(pieces.polynomial_intervals, icurve + 1, 1)?;
    let coef2: &[f64] = coefficients.get(start1 + stride..).unwrap_or(&[]);
    plib::eval_polynomial(u2, order, deg2, dimension, coef2, &mut res2)?;
    Ok((res1, res2))
}

fn interval(table: &Array2<'_, f64>, row: i32, col: i32) -> Result<f64, Failure> {
    table.value(row, col).copied().map_err(errors::array_bounds)
}

// The factors of order `iordre`; the "PRO18577" guard zeroes a factor whose true
// interval is shorter than 1e-5 instead of dividing by it.
fn factors(icurve: i32, iordre: i32, pieces: &Pieces<'_, '_>) -> Result<Factors, Failure> {
    let toler: f64 = 1.0e-5;
    let poly = |row: i32| -> Result<f64, Failure> {
        Ok(interval(pieces.polynomial_intervals, row, 2)?
            - interval(pieces.polynomial_intervals, row, 1)?)
    };
    let (f1_dividend, f2_dividend) = (poly(icurve)?, poly(icurve + 1)?);
    let true_at = |index: i32| at(pieces.true_intervals, index);
    let f1_divizor: f64 = true_at(icurve + 1)? - true_at(icurve)?;
    let f2_divizor: f64 = true_at(icurve + 2)? - true_at(icurve + 1)?;
    let order: f64 = f64::from(iordre);
    let scaled = |dividend: f64, divizor: f64| {
        if divizor.abs() < toler {
            return 0.0;
        }
        (dividend / divizor).powf(order)
    };
    Ok(Factors {
        facteur1: scaled(f1_dividend, f1_divizor),
        facteur2: scaled(f2_dividend, f2_divizor),
        normal1: f1_divizor.powf(order),
        normal2: f2_divizor.powf(order),
    })
}

// The jump bounds of every subspace at order `iordre`, None as soon as one exceeds
// its tolerance (the native isCi becoming false).
fn jumps(
    res1: &[f64],
    res2: &[f64],
    iordre: i32,
    (n1, n2, n3): SubSpaces,
    factors: &Factors,
    local_tolerance: &Array1<'_, f64>,
) -> Result<Jumps, Failure> {
    let dimension: i32 = n1 + 2 * n2 + 3 * n3;
    let mut offset: usize = (iordre * dimension) as usize;
    let (mut prec, mut suivant) = (Vec::new(), Vec::new());
    let widths = std::iter::repeat_n(1, n1 as usize)
        .chain(std::iter::repeat_n(2, n2 as usize))
        .chain(std::iter::repeat_n(3, n3 as usize));
    for (idim, width) in (1..).zip(widths) {
        let eps: f64 = at(local_tolerance, idim)? * 0.01;
        let (diff, moy) = difference(
            &res1[offset..offset + width],
            &res2[offset..offset + width],
            factors,
        );
        offset += width;
        // Only a relative jump is checked against the tolerance, as natively.
        let (p, s) = if diff > moy * 1.0e-9 {
            (diff * factors.normal1, diff * factors.normal2)
        } else {
            prec.push(0.);
            suivant.push(0.);
            continue;
        };
        if p > eps || s > eps {
            return Ok(None);
        }
        prec.push(p);
        suivant.push(s);
    }
    Ok(Some((prec, suivant)))
}

// |a*f1 - b*f2| and |a*f1 + b*f2| summed over the coordinates, in native order.
fn difference(val1: &[f64], val2: &[f64], factors: &Factors) -> (f64, f64) {
    let (mut diff, mut moy) = (0.0, 0.0);
    for (index, (&a, &b)) in val1.iter().zip(val2).enumerate() {
        let (x1, x2) = (a * factors.facteur1, b * factors.facteur2);
        if index == 0 {
            diff = (x1 - x2).abs();
            moy = (x1 + x2).abs();
        } else {
            diff += (x1 - x2).abs();
            moy += (x1 + x2).abs();
        }
    }
    (diff, moy)
}

fn accumulate(
    error_max: &mut Array1<'_, f64>,
    icurve: i32,
    prec: &[f64],
    suivant: &[f64],
) -> Result<(), Failure> {
    let nb_space: i32 = prec.len() as i32;
    let index: i32 = (icurve - 1) * nb_space + 1;
    for (offset, (&p, &s)) in (0..).zip(prec.iter().zip(suivant)) {
        let ii: i32 = index + offset;
        put(error_max, ii, at(error_max, ii)? + p)?;
        put(error_max, ii + nb_space, at(error_max, ii + nb_space)? + s)?;
    }
    Ok(())
}
