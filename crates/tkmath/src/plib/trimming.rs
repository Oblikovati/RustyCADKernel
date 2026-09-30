// Rust translation of FoundationClasses/TKMath/PLib/PLib.cxx (Trimming, UTrimming, VTrimming) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Reparameterization of canonical coefficients from [U1, U2] onto [0, 1].
//!
//! Substitutes u = U1 + v (U2 - U1) and expands with Horner's scheme in place.
use super::access::{array_cell, array_value, grid_cell, grid_value};
use super::coefficients_poles::flat_array;
use super::poles::{PolePoint, get_poles, set_poles};
use crate::gp::Point3;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;

/// Trims point coefficients through a flat temporary (`PLib::Trimming` on gp_Pnt/gp_Pnt2d);
/// `plib::trimming(0.25, 0.75, &mut coefficients, None)?`.
pub fn trimming<P: PolePoint>(
    u1: f64,
    u2: f64,
    coeffs: &mut Array1<'_, P>,
    wcoeffs: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    let mut temp: Array1<'static, f64> = flat_array(P::DIMENSION * coeffs.len())?;
    set_poles(coeffs, &mut temp)?;
    trimming_flat(u1, u2, P::DIMENSION as i32, &mut temp, wcoeffs)?;
    get_poles(&temp, coeffs)
}

/// Trims scalar coefficients (`PLib::Trimming` on real arrays, dimension 1);
/// `plib::trimming_scalar(0.0, 2.0, &mut coefficients, None)?`.
pub fn trimming_scalar(
    u1: f64,
    u2: f64,
    coeffs: &mut Array1<'_, f64>,
    wcoeffs: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    trimming_flat(u1, u2, 1, coeffs, wcoeffs)
}

/// Trims interleaved `dim`-component coefficients (`PLib::Trimming(U1,U2,dim,...)`).
///
/// Weighted input must hold one weight per coefficient row, else the native
/// Standard_Failure is returned; `plib::trimming_flat(0.0, 2.0, 3, &mut coefficients, None)?`.
pub fn trimming_flat(
    u1: f64,
    u2: f64,
    dim: i32,
    coeffs: &mut Array1<'_, f64>,
    mut wcoeffs: Option<&mut Array1<'_, f64>>,
) -> Result<(), Failure> {
    if dim <= 0 {
        return Err(Failure::out_of_range(&format!(
            "PLib::Trimming: dim = {dim}; expected > 0"
        )));
    }
    let lsp: f64 = u2 - u1;
    let upc: i32 = coeffs.upper() - dim + 1;
    let len: i32 = coeffs.len() as i32 / dim;
    if let Some(weights) = wcoeffs.as_deref()
        && len != weights.len() as i32
    {
        return Err(Failure::new(&format!(
            "PLib::Trimming : nbcoefs/dim != nbweights !!! ({len} rows, {} weights)",
            weights.len()
        )));
    }
    for i in 1..len {
        let indc: i32 = upc - dim * (i - 1);
        horner_pass(u1, lsp, dim, upc, indc, coeffs)?;
        if let Some(weights) = wcoeffs.as_deref_mut() {
            let upw: i32 = weights.upper();
            horner_pass(u1, lsp, 1, upw, upw - i + 1, weights)?;
        }
    }
    Ok(())
}

// One Horner step: lowest coefficient, intermediate coefficients, then the leading one.
fn horner_pass(
    u1: f64,
    lsp: f64,
    dim: i32,
    upper: i32,
    start: i32,
    values: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    for j in 0..dim {
        let lifted: f64 = u1 * array_value(values, start + j)?;
        *array_cell(values, start - dim + j)? += lifted;
    }
    let mut index: i32 = start;
    while index < upper {
        index += dim;
        for k in 0..dim {
            let value: f64 =
                u1 * array_value(values, index + k)? + lsp * array_value(values, index - dim + k)?;
            *array_cell(values, index - dim + k)? = value;
        }
    }
    for j in 0..dim {
        *array_cell(values, upper + j)? *= lsp;
    }
    Ok(())
}

/// Trims each column of a coefficient grid in U (`PLib::UTrimming`);
/// `plib::u_trimming(0.0, 0.5, &mut grid, None)?`.
pub fn u_trimming(
    u1: f64,
    u2: f64,
    coeffs: &mut Array2<'_, Point3>,
    mut wcoeffs: Option<&mut Array2<'_, f64>>,
) -> Result<(), Failure> {
    let (lr, ur) = (coeffs.lower_row(), coeffs.upper_row());
    for icol in coeffs.lower_col()..=coeffs.upper_col() {
        let cells: Vec<(i32, i32)> = (lr..=ur).map(|irow| (irow, icol)).collect();
        trim_line(u1, u2, lr, &cells, coeffs, wcoeffs.as_deref_mut())?;
    }
    Ok(())
}

/// Trims each row of a coefficient grid in V (`PLib::VTrimming`);
/// `plib::v_trimming(0.0, 0.5, &mut grid, None)?`.
pub fn v_trimming(
    v1: f64,
    v2: f64,
    coeffs: &mut Array2<'_, Point3>,
    mut wcoeffs: Option<&mut Array2<'_, f64>>,
) -> Result<(), Failure> {
    let (lc, uc) = (coeffs.lower_col(), coeffs.upper_col());
    for irow in coeffs.lower_row()..=coeffs.upper_row() {
        let cells: Vec<(i32, i32)> = (lc..=uc).map(|icol| (irow, icol)).collect();
        trim_line(v1, v2, lc, &cells, coeffs, wcoeffs.as_deref_mut())?;
    }
    Ok(())
}

// Copies one grid line into bounded temporaries, trims them, and writes them back.
fn trim_line(
    first: f64,
    last: f64,
    lower: i32,
    cells: &[(i32, i32)],
    coeffs: &mut Array2<'_, Point3>,
    mut wcoeffs: Option<&mut Array2<'_, f64>>,
) -> Result<(), Failure> {
    let upper: i32 = lower + cells.len() as i32 - 1;
    let mut temp: Array1<'static, Point3> = bounded(lower, upper)?;
    let mut temw: Array1<'static, f64> = bounded(lower, upper)?;
    for (offset, &(row, col)) in cells.iter().enumerate() {
        *array_cell(&mut temp, lower + offset as i32)? = grid_value(coeffs, row, col)?;
        if let Some(weights) = wcoeffs.as_deref() {
            *array_cell(&mut temw, lower + offset as i32)? = grid_value(weights, row, col)?;
        }
    }
    let rational: bool = wcoeffs.is_some();
    trimming(first, last, &mut temp, rational.then_some(&mut temw))?;
    for (offset, &(row, col)) in cells.iter().enumerate() {
        *grid_cell(coeffs, row, col)? = array_value(&temp, lower + offset as i32)?;
        if let Some(weights) = wcoeffs.as_deref_mut() {
            *grid_cell(weights, row, col)? = array_value(&temw, lower + offset as i32)?;
        }
    }
    Ok(())
}

fn bounded<T: Default>(lower: i32, upper: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(lower, upper).map_err(|error| Failure::out_of_range(&error.to_string()))
}
