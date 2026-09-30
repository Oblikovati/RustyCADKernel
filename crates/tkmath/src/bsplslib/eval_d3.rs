// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib.cxx (D3, DN) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Third and arbitrary-order derivatives of B-spline surfaces.
use super::eval::{ZERO, first_direction, oriented, point, second_direction, vector};
use super::prepare::{Patch, SurfaceSpan, prepare_eval, validate_degrees};
use super::rational::rational_derivative;
use crate::bsplclib::bohm;
use crate::gp::{Point3, Vector3};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;

/// Point and derivatives up to the third order (`BSplSLib::D3` outputs, in native order).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SurfaceD3 {
    /// Point.
    pub p: Point3,
    /// dP/dU.
    pub vu: Vector3,
    /// dP/dV.
    pub vv: Vector3,
    /// d2P/dU2.
    pub vuu: Vector3,
    /// d2P/dV2.
    pub vvv: Vector3,
    /// d2P/dUdV.
    pub vuv: Vector3,
    /// d3P/dU3.
    pub vuuu: Vector3,
    /// d3P/dV3.
    pub vvvv: Vector3,
    /// d3P/dU2dV.
    pub vuuv: Vector3,
    /// d3P/dUdV2.
    pub vuvv: Vector3,
}

/// Point and derivatives up to the third order (`BSplSLib::D3`). A non-rational patch
/// of degree <= 1 (resp. <= 2) in a direction gives zero second (third) derivatives
/// in it; `bsplslib::d3(&span, &poles, None)?`.
pub fn d3(
    span: &SurfaceSpan<'_, '_>,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<SurfaceD3, Failure> {
    validate_degrees(span.u_degree, span.v_degree)?;
    let mut patch: Patch = prepare_eval(span, poles, weights)?;
    let row: usize = patch.dim() * (patch.d2 as usize + 1);
    first_direction(&mut patch, Some(3))?;
    second_direction(&mut patch, Some(3), 0)?;
    second_direction(&mut patch, Some(2), row)?;
    if patch.d1 > 1 {
        second_direction(&mut patch, Some(1), 2 * row)?;
    }
    if patch.d1 > 2 {
        second_direction(&mut patch, None, 3 * row)?;
    }
    if patch.rational {
        let mut ders: Vec<f64> = vec![0.0; 48];
        rational_derivative(patch.d1, patch.d2, 3, 3, &patch.poles, &mut ders, true)?;
        return Ok(rational_d3(&ders, patch.u_first));
    }
    Ok(polynomial_d3(
        &patch.poles,
        patch.u_first,
        row,
        patch.d2 as usize,
        [span.u_degree, span.v_degree],
    ))
}

// (i, j) of the first/second direction sits at 12*i + 3*j.
fn rational_d3(ders: &[f64], u_first: bool) -> SurfaceD3 {
    let [vu, vv, vuu, vvv] = oriented(u_first, [12, 3, 24, 6]);
    let [vuuu, vvvv, vuuv, vuvv] = if u_first {
        [36, 9, 27, 18]
    } else {
        [9, 36, 18, 27]
    };
    SurfaceD3 {
        p: point(ders),
        vu: vector(&ders[vu..]),
        vv: vector(&ders[vv..]),
        vuu: vector(&ders[vuu..]),
        vvv: vector(&ders[vvv..]),
        vuv: vector(&ders[15..]),
        vuuu: vector(&ders[vuuu..]),
        vvvv: vector(&ders[vvvv..]),
        vuuv: vector(&ders[vuuv..]),
        vuvv: vector(&ders[vuvv..]),
    }
}

// Offsets into the patch rows; None stands for BSplSLib_zero.
fn polynomial_d3(
    values: &[f64],
    u_first: bool,
    row: usize,
    d2: usize,
    [u_degree, v_degree]: [i32; 2],
) -> SurfaceD3 {
    let pick = |offset: Option<usize>| -> Vector3 {
        match offset {
            Some(offset) => vector(&values[offset..]),
            None => vector(&ZERO),
        }
    };
    let when = |keep: bool, offset: usize| if keep { Some(offset) } else { None };
    let (uu, vv3) = (u_degree > 1, v_degree > 1);
    let (uuu, vvvv) = (u_degree > 2, v_degree > 2);
    let [vu, vv, vuu, vvv] = oriented(u_first, [row, 3, 2 * row, 6]);
    let (vuuv, vuvv, vuuu, vvvv_offset) = if u_first {
        (2 * row + 3, row + 6, 3 * row, 9)
    } else {
        (row + 6, 2 * row + 3, 9, 3 * row)
    };
    SurfaceD3 {
        p: point(values),
        vu: vector(&values[vu..]),
        vv: vector(&values[vv..]),
        vuu: pick(when(uu, vuu)),
        vvv: pick(when(vv3, vvv)),
        vuv: vector(&values[3 * d2 + 6..]),
        vuuu: pick(when(uuu, vuuu)),
        vvvv: pick(when(vvvv, vvvv_offset)),
        vuuv: pick(when(uu, vuuv)),
        vuvv: pick(when(vv3, vuvv)),
    }
}

/// Derivative of order (Nu, Nv) (`BSplSLib::DN`). A non-rational patch gives zero
/// when an order exceeds its degree; `bsplslib::dn(&span, 1, 2, &poles, None)?`.
pub fn dn(
    span: &SurfaceSpan<'_, '_>,
    nu: i32,
    nv: i32,
    poles: &Array2<'_, Point3>,
    weights: Option<&Array2<'_, f64>>,
) -> Result<Vector3, Failure> {
    validate_degrees(span.u_degree, span.v_degree)?;
    let mut patch: Patch = prepare_eval(span, poles, weights)?;
    let dim: usize = patch.dim();
    if !patch.rational && (nu > span.u_degree || nv > span.v_degree) {
        return Ok(vector(&ZERO));
    }
    let (n1, n2): (i32, i32) = if patch.u_first { (nu, nv) } else { (nv, nu) };
    first_direction(&mut patch, Some(n1))?;
    let row: usize = dim * (patch.d2 as usize + 1);
    for k in 0..=n1.min(patch.d1).max(-1) {
        let offset: usize = k as usize * row;
        bohm(
            patch.u2,
            patch.d2,
            n2,
            &patch.knots2,
            dim as i32,
            &mut patch.poles[offset..],
        )?;
    }
    if patch.rational {
        let mut ders: Vec<f64> = vec![0.0; 48];
        rational_derivative(patch.d1, patch.d2, n1, n2, &patch.poles, &mut ders, false)?;
        return Ok(vector(&ders));
    }
    let offset: i64 = (i64::from(n1) * (i64::from(patch.d2) + 1) + i64::from(n2)) * dim as i64;
    let offset: usize = usize::try_from(offset).map_err(|_| {
        Failure::out_of_range(&format!("BSplSLib::DN: orders ({nu}, {nv}); expected >= 0"))
    })?;
    Ok(vector(&patch.poles[offset..]))
}
