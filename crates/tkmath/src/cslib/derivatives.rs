//! CSLib::DNNUV and CSLib::DNNormal: derivatives of the raw and unit surface normal.
mod unit_normal;

use crate::gp::Vector3;
use crate::plib::bin as plib_bin;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;

pub use unit_normal::dn_normal;

/// Reads `array(row, col)` with NCollection_Array2's native range failure.
pub(super) fn derivative_at(
    array: &Array2<'_, Vector3>,
    row: i32,
    col: i32,
) -> Result<Vector3, Failure> {
    array.value(row, col).copied().map_err(|error| {
        Failure::with_type(
            "Standard_OutOfRange",
            &format!("CSLib derivative ({row}, {col}): {error}"),
            "",
        )
    })
}

/// CSLib::DNNUV(Nu, Nv, DerSurf): `d^(Nu+Nv) (dS/du ^ dS/dv) / (du^Nu dv^Nv)`.
///
/// `der_surf(i, j)` holds `d^(i+j)S / (du^i dv^j)` for `i <= nu+1`, `j <= nv+1`.
/// `dnnuv(0, 0, &der_surf)?` is `D1U ^ D1V`.
pub fn dnnuv(nu: i32, nv: i32, der_surf: &Array2<'_, Vector3>) -> Result<Vector3, Failure> {
    dnnuv_osculating(nu, nv, der_surf, der_surf)
}

/// CSLib::DNNUV(Nu, Nv, DerSurf1, DerSurf2): derivative of `dS1/du ^ dS2/dv`.
///
/// With both arrays equal this is exactly [`dnnuv`], as in OCCT where the two
/// overloads share one loop body. `dnnuv_osculating(1, 0, &first, &second)?`.
pub fn dnnuv_osculating(
    nu: i32,
    nv: i32,
    der_surf1: &Array2<'_, Vector3>,
    der_surf2: &Array2<'_, Vector3>,
) -> Result<Vector3, Failure> {
    let mut result: Vector3 = Vector3::new(0.0, 0.0, 0.0);
    for i in 0..=nu {
        for j in 0..=nv {
            let left: Vector3 = derivative_at(der_surf1, i + 1, j)?;
            let right: Vector3 = derivative_at(der_surf2, nu - i, nv + 1 - j)?;
            let binomial: f64 = plib_bin(nu, i)? * plib_bin(nv, j)?;
            result += left.crossed(right) * binomial;
        }
    }
    Ok(result)
}
