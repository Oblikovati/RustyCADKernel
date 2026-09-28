// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKG3d/Geom/Geom_OffsetSurfaceUtils.pxx (ComputeDerivatives) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The derivatives of `N = D1U ^ D1V` from the surface derivatives, optionally
//! mixing the derivatives of an osculating surface along a degenerate boundary.
use super::{at, put, zero_table};
use crate::geom::{BasisSurfaceEvaluator, GeomSurface};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::cslib;
use rustycad_tkmath::gp::Vector3;

/// The orders of one ComputeDerivatives call: the extra order, the order below
/// which the caller already filled the surface table, and the requested (Nu, Nv).
#[derive(Clone, Copy, Debug)]
pub struct DerivativeOrders {
    /// `theMaxOrder`.
    pub max_order: i32,
    /// `theMinOrder`.
    pub min_order: i32,
    /// `theNU`.
    pub nu: i32,
    /// `theNV`.
    pub nv: i32,
}

/// `ComputeDerivatives`: fill `der_surf` with the basis derivatives above
/// `min_order` and `der_nuv` with the derivatives of the normal. With an
/// osculating surface (`along = (AlongU, AlongV)`) the extra order is forced to 0
/// and the normal mixes the osculating D1 with the basis one, as natively;
/// `compute_derivatives(orders, (u, v), &*basis, (false, false), None, &mut nuv, &mut surf)?`.
pub fn compute_derivatives<B: BasisSurfaceEvaluator + ?Sized>(
    orders: DerivativeOrders,
    uv: (f64, f64),
    basis: &B,
    along: (bool, bool),
    osculating: Option<&dyn GeomSurface>,
    der_nuv: &mut Array2<'_, Vector3>,
    der_surf: &mut Array2<'_, Vector3>,
) -> Result<bool, Failure> {
    if let (true, Some(osc)) = (along.0 || along.1, osculating) {
        let orders = DerivativeOrders {
            max_order: 0,
            ..orders
        };
        let mut der_surf_l = zero_table(orders.nu + 1, orders.nv + 1)?;
        first_osculating_orders(orders.min_order, uv, osc, &mut der_surf_l)?;
        fill_osculating(orders, uv, basis, osc, &mut der_surf_l, der_surf)?;
        for i in 0..=orders.nu {
            for j in 0..=orders.nv {
                if along.0 {
                    put(
                        der_nuv,
                        i,
                        j,
                        cslib::dnnuv_osculating(i, j, &der_surf_l, der_surf)?,
                    )?;
                }
                if along.1 {
                    put(
                        der_nuv,
                        i,
                        j,
                        cslib::dnnuv_osculating(i, j, der_surf, &der_surf_l)?,
                    )?;
                }
            }
        }
        return Ok(true);
    }
    fill_basis(orders, uv, basis, der_surf)?;
    for i in 0..=orders.max_order + orders.nu {
        for j in 0..=orders.max_order + orders.nv {
            put(der_nuv, i, j, cslib::dnnuv(i, j, der_surf)?)?;
        }
    }
    Ok(true)
}

// The osculating derivatives up to `min_order` (the switch on theMinOrder).
fn first_osculating_orders(
    min_order: i32,
    uv: (f64, f64),
    osc: &dyn GeomSurface,
    table: &mut Array2<'_, Vector3>,
) -> Result<(), Failure> {
    let (u, v) = uv;
    let values: Vec<(i32, i32, Vector3)> = match min_order {
        1 => {
            let r = osc.eval_d1(u, v)?;
            vec![(1, 0, r.d1u), (0, 1, r.d1v)]
        }
        2 => {
            let r = osc.eval_d2(u, v)?;
            vec![
                (1, 0, r.d1u),
                (0, 1, r.d1v),
                (1, 1, r.d2uv),
                (2, 0, r.d2u),
                (0, 2, r.d2v),
            ]
        }
        3 => {
            let r = osc.eval_d3(u, v)?;
            vec![
                (1, 0, r.d1u),
                (0, 1, r.d1v),
                (1, 1, r.d2uv),
                (2, 0, r.d2u),
                (0, 2, r.d2v),
                (3, 0, r.d3u),
                (2, 1, r.d3uuv),
                (1, 2, r.d3uvv),
                (0, 3, r.d3v),
            ]
        }
        _ => Vec::new(),
    };
    for (i, j, value) in values {
        put(table, i, j, value)?;
    }
    Ok(())
}

// The osculating branch: DN of both surfaces at (i, j) and, below the diagonal
// bound, at (j, i); the scan runs over the smaller of Nu and Nv first.
fn fill_osculating<B: BasisSurfaceEvaluator + ?Sized>(
    orders: DerivativeOrders,
    uv: (f64, f64),
    basis: &B,
    osc: &dyn GeomSurface,
    der_surf_l: &mut Array2<'_, Vector3>,
    der_surf: &mut Array2<'_, Vector3>,
) -> Result<(), Failure> {
    let (u, v) = uv;
    let u_first: bool = orders.nu <= orders.nv;
    let (outer_last, inner_last, mirror_bound) = if u_first {
        (
            orders.max_order + 1 + orders.nu,
            orders.max_order + orders.nv + 1,
            orders.nu + 1,
        )
    } else {
        (
            orders.max_order + 1 + orders.nv,
            orders.max_order + orders.nu + 1,
            orders.nv + 1,
        )
    };
    for a in 0..=outer_last {
        for b in a..=inner_last {
            let (i, j) = if u_first { (a, b) } else { (b, a) };
            if i + j <= orders.min_order {
                continue;
            }
            let osc_dn: Vector3 = osc.eval_dn(u, v, i, j)?;
            let basis_dn: Vector3 = basis.basis_dn(u, v, i, j)?;
            put(der_surf_l, i, j, osc_dn)?;
            put(der_surf, i, j, basis_dn)?;
            if i != j && b <= mirror_bound {
                let basis_ji: Vector3 = basis.basis_dn(u, v, j, i)?;
                let osc_ji: Vector3 = osc.eval_dn(u, v, j, i)?;
                put(der_surf, j, i, basis_ji)?;
                put(der_surf_l, j, i, osc_ji)?;
            }
        }
    }
    Ok(())
}

// The plain branch: basis DN at (i, j) for j >= i and at (j, i) inside the table.
fn fill_basis<B: BasisSurfaceEvaluator + ?Sized>(
    orders: DerivativeOrders,
    uv: (f64, f64),
    basis: &B,
    der_surf: &mut Array2<'_, Vector3>,
) -> Result<(), Failure> {
    let (u, v) = uv;
    for i in 0..=orders.max_order + orders.nu + 1 {
        for j in i..=orders.max_order + orders.nv + 1 {
            if i + j <= orders.min_order {
                continue;
            }
            put(der_surf, i, j, basis.basis_dn(u, v, i, j)?)?;
            if i != j && j <= der_surf.upper_row() && i <= der_surf.upper_col() {
                put(der_surf, j, i, basis.basis_dn(u, v, j, i)?)?;
            }
        }
    }
    Ok(())
}

/// Read one entry of a derivative table (for the evaluators); `entry(&t, 1, 0)?`.
pub(crate) fn entry(table: &Array2<'_, Vector3>, i: i32, j: i32) -> Result<Vector3, Failure> {
    at(table, i, j)
}
