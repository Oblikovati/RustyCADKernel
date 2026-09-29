// Copyright (c) 1996-1999 Matra Datavision
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

//! `AdvApp2Var_Patch::AddConstraints` and `AddErrors`: the boundary interpolation
//! added back to the approximated patch, and the boundary errors added to its errors.
use super::approx_f2var::{
    CornerConstraints, IsoConstraints, mma1her_, mma2ac1_, mma2ac2_, mma2ac3_,
};
use super::arrays::required;
use super::context::AdvApp2VarContext;
use super::framework::AdvApp2VarFramework;
use super::iso::AdvApp2VarIso;
use super::math_base::mmdrc11_;
use super::patch::{AdvApp2VarPatch, PatchState, construction};
use super::patch_discretise::{corner_values, corners, scaled_iso_table};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, Handle};

const ADD_CONSTRAINTS: &str = "AdvApp2Var_Patch::AddConstraints";
const ADD_ERRORS: &str = "AdvApp2Var_Patch::AddErrors";

/// The polynomial of a border iso scaled per derivative block by `step^order`, and
/// its coefficient count repeated per derivative (`HIsoU0`, `HCFU0`...).
struct BorderIso {
    values: Vec<f64>,
    counts: Vec<i32>,
}

fn border_iso(
    iso: &AdvApp2VarIso,
    block: i32,
    orders: i32,
    step: f64,
) -> Result<BorderIso, Failure> {
    let values: Vec<f64> = scaled_iso_table(iso.polynom(), block, orders, step, ADD_CONSTRAINTS)?;
    Ok(BorderIso {
        values,
        counts: vec![iso.nb_coeff(); (orders + 1).max(0) as usize],
    })
}

/// The Hermite basis of one order, or a ConstructionError when `mma1her_` fails.
fn hermite(order: i32) -> Result<Vec<f64>, Failure> {
    let mut hermit: Vec<f64> = vec![0.0; ((2 * order + 2) * (2 * order + 2)).max(0) as usize];
    if order >= 0 {
        let mut iercod: i32 = 0;
        mma1her_(order, &mut hermit, &mut iercod);
        if iercod != 0 {
            return Err(construction(
                "AdvApp2Var_Patch::AddConstraints : Error in FORTRAN",
            ));
        }
    }
    Ok(hermit)
}

/// `std::max` for doubles: `b` when `a < b`, else `a`.
fn cpp_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// The layout of the corner table `(NDIMEN, IORDRU+2, IORDRV+2)` x 4.
#[derive(Clone, Copy)]
struct CornerLayout {
    ndimen: i32,
    iordru: i32,
    size: i32,
}

impl CornerLayout {
    /// The 0-based index of `(idim, order_u, order_v)` in corner block `block`.
    fn index(self, block: i32, idim: i32, order_u: i32, order_v: i32) -> usize {
        (block * self.size
            + self.ndimen * order_u
            + self.ndimen * (self.iordru + 2) * order_v
            + idim
            - 1) as usize
    }
}

/// The scratch of `mmdrc11_` shared by every border (`HEXTR`, `HFACT`): a call on an
/// empty curve leaves the previous values, which are then added again, as natively.
struct Extremities {
    extr: Vec<f64>,
    fact: Vec<f64>,
}

impl Extremities {
    /// The values at -1 and +1 of `curve`, up to order `order`, into `extr`.
    fn evaluate(&mut self, order: i32, ncoeff: i32, curve: &[f64]) {
        mmdrc11_(order, 1, ncoeff, curve, &mut self.extr, &mut self.fact);
    }

    /// `HEXTR->Value(1 + 2*(k-1))` then `+1`: the -1 and +1 values of order `k-1`.
    fn ends(&self, k: i32) -> (f64, f64) {
        (
            self.extr[(2 * (k - 1)) as usize],
            self.extr[(2 * (k - 1) + 1) as usize],
        )
    }
}

/// Adds the ends of the U border isos to the corner table (`add extremities of isos U`).
fn add_iso_u_ends(
    coins: &mut [f64],
    layout: CornerLayout,
    isos: [&BorderIso; 2],
    ncflmv: i32,
    iordrv: i32,
    scratch: &mut Extremities,
) {
    let ndimen: i32 = layout.ndimen;
    for iu in 1..=layout.iordru + 1 {
        for idim in 1..=ndimen {
            let start: usize = (ncflmv * (idim - 1) + ncflmv * ndimen * (iu - 1)) as usize;
            for (first_block, iso) in [0, 1].into_iter().zip(isos) {
                scratch.evaluate(iordrv, iso.counts[(iu - 1) as usize], &iso.values[start..]);
                for iv in 1..=iordrv + 1 {
                    let (minus, plus) = scratch.ends(iv);
                    coins[layout.index(first_block, idim, iu - 1, iv - 1)] += minus;
                    coins[layout.index(first_block + 2, idim, iu - 1, iv - 1)] += plus;
                }
            }
        }
    }
}

/// Adds the ends of the V border isos to the corner table (`add extremities of isos V`).
fn add_iso_v_ends(
    coins: &mut [f64],
    layout: CornerLayout,
    isos: [&BorderIso; 2],
    ncflmu: i32,
    iordrv: i32,
    scratch: &mut Extremities,
) {
    let ndimen: i32 = layout.ndimen;
    for iv in 1..=iordrv + 1 {
        for idim in 1..=ndimen {
            let start: usize = (ncflmu * (idim - 1) + ncflmu * ndimen * (iv - 1)) as usize;
            for (first_block, iso) in [0, 2].into_iter().zip(isos) {
                scratch.evaluate(
                    layout.iordru,
                    iso.counts[(iv - 1) as usize],
                    &iso.values[start..],
                );
                for iu in 1..=layout.iordru + 1 {
                    let (minus, plus) = scratch.ends(iu);
                    coins[layout.index(first_block, idim, iu - 1, iv - 1)] += minus;
                    coins[layout.index(first_block + 1, idim, iu - 1, iv - 1)] += plus;
                }
            }
        }
    }
}

/// The largest error of the orders `1..=orders+1` of subspace 1 on two border isos,
/// the first iso first at each order (`errU`, `errV`).
fn border_max(tables: [&Array2<'static, f64>; 2], orders: i32) -> Result<f64, Failure> {
    let mut err: f64 = 0.;
    for order in 1..=orders + 1 {
        for table in tables {
            err = cpp_max(
                err,
                *table
                    .value(1, order)
                    .map_err(|e| e.into_failure(ADD_ERRORS))?,
            );
        }
    }
    Ok(err)
}

/// The largest node error of orders `(0..=iu, 0..=iv)` at each patch corner.
fn corner_errors(s: &PatchState, constraints: &AdvApp2VarFramework) -> Result<[f64; 4], Failure> {
    let mut emax: [f64; 4] = [0.; 4];
    for iu in 0..=s.ord_in_u {
        for iv in 0..=s.ord_in_v {
            for (e, (u, v)) in emax.iter_mut().zip(corners(s)) {
                *e = cpp_max(*e, constraints.node_at(u, v)?.error(iu, iv)?);
            }
        }
    }
    Ok(emax)
}

impl AdvApp2VarPatch {
    /// Adds to the canonical patch the Hermite interpolation of the border isos and
    /// of the corner nodes of `constraints`, scaled to [-1,1] (`AddConstraints`);
    /// Standard_ConstructionError when a Hermite basis fails. Only 3D functions.
    /// `patch.add_constraints(&ctx, &frame)?`.
    pub fn add_constraints(
        &self,
        conditions: &AdvApp2VarContext,
        constraints: &AdvApp2VarFramework,
    ) -> Result<(), Failure> {
        let s: PatchState = self.state.read().clone();
        let mut patcan: Vec<f64> = required(s.equation.clone(), ADD_CONSTRAINTS)?
            .as_slice()
            .to_vec();
        let ndimen: i32 = conditions.total_dimension();
        let (ncflmu, ncflmv) = (conditions.u_limit(), conditions.v_limit());
        let (ndegu, ndegv, iordru, iordrv) = (ncflmu - 1, ncflmv - 1, s.ord_in_u, s.ord_in_v);
        let (du, dv) = ((s.u1 - s.u0) / 2., (s.v1 - s.v0) / 2.);
        let isou0 = border_iso(
            constraints.iso_u(s.u0, s.v0, s.v1)?,
            ncflmv * ndimen,
            iordru,
            du,
        )?;
        let isou1 = border_iso(
            constraints.iso_u(s.u1, s.v0, s.v1)?,
            ncflmv * ndimen,
            iordru,
            du,
        )?;
        let isov0 = border_iso(
            constraints.iso_v(s.u0, s.u1, s.v0)?,
            ncflmu * ndimen,
            iordrv,
            dv,
        )?;
        let isov1 = border_iso(
            constraints.iso_v(s.u0, s.u1, s.v1)?,
            ncflmu * ndimen,
            iordrv,
            dv,
        )?;
        let herm_v: Vec<f64> = hermite(iordrv)?;
        if iordrv >= 0 {
            let isos = IsoConstraints {
                ncf1: &isov0.counts,
                crb1: &isov0.values,
                ncf2: &isov1.counts,
                crb2: &isov1.values,
            };
            mma2ac2_(
                ndimen,
                ndegu,
                ndegv,
                iordrv,
                ncflmu,
                &isos,
                &herm_v,
                &mut patcan,
            );
        }
        let herm_u: Vec<f64> = hermite(iordru)?;
        if iordru >= 0 {
            let isos = IsoConstraints {
                ncf1: &isou0.counts,
                crb1: &isou0.values,
                ncf2: &isou1.counts,
                crb2: &isou1.values,
            };
            mma2ac3_(
                ndimen,
                ndegu,
                ndegv,
                iordru,
                ncflmv,
                &isos,
                &herm_u,
                &mut patcan,
            );
        }
        let mut coins: Vec<f64> = corner_values(&s, constraints, ndimen, -1.0)?;
        let layout = CornerLayout {
            ndimen,
            iordru,
            size: ndimen * (iordru + 2) * (iordrv + 2),
        };
        let iordmx: i32 = iordru.max(iordrv);
        let mut scratch = Extremities {
            extr: vec![0.; (2 * iordmx + 2) as usize],
            fact: vec![0.; (iordmx + 1) as usize],
        };
        add_iso_u_ends(
            &mut coins,
            layout,
            [&isou0, &isou1],
            ncflmv,
            iordrv,
            &mut scratch,
        );
        add_iso_v_ends(
            &mut coins,
            layout,
            [&isov0, &isov1],
            ncflmu,
            iordrv,
            &mut scratch,
        );
        if iordru >= 0 && iordrv >= 0 {
            let size: usize = layout.size as usize;
            let (c1, rest) = coins.split_at(size);
            let (c2, rest) = rest.split_at(size);
            let (c3, c4) = rest.split_at(size);
            let corners = CornerConstraints {
                contr1: c1,
                contr2: c2,
                contr3: c3,
                contr4: c4,
            };
            mma2ac1_(
                ndimen,
                ndegu,
                ndegv,
                iordru,
                iordrv,
                &corners,
                &herm_u,
                &herm_v,
                &mut patcan,
            );
        }
        if let Some(equation) = self.state.write().equation.as_mut() {
            equation.as_mut_slice().copy_from_slice(&patcan);
        }
        Ok(())
    }

    /// Adds to the patch errors the errors of its border isos weighted by the
    /// Hermite bounds 0, 1, 1.5, 1.75 of the orders, and records the border errors
    /// plus the corner node errors on inner borders (`AddErrors`). Only one
    /// subspace (`NBSESP = 1`), as natively. `patch.add_errors(&frame)?`.
    pub fn add_errors(&self, constraints: &AdvApp2VarFramework) -> Result<(), Failure> {
        let s: PatchState = self.state.read().clone();
        let hmax: [f64; 4] = [0.0, 1.0, 1.5, 1.75];
        let (hu, hv) = (
            hmax[(s.ord_in_u + 1) as usize],
            hmax[(s.ord_in_v + 1) as usize],
        );
        let isos: [&Handle<AdvApp2VarIso>; 4] = [
            constraints.iso_v(s.u0, s.u1, s.v0)?,
            constraints.iso_v(s.u0, s.u1, s.v1)?,
            constraints.iso_u(s.u0, s.v0, s.v1)?,
            constraints.iso_u(s.u1, s.v0, s.v1)?,
        ];
        let maxs: Vec<Array2<'static, f64>> = isos
            .iter()
            .map(|iso| required(iso.max_errors(), ADD_ERRORS))
            .collect::<Result<_, _>>()?;
        let moys: Vec<Array2<'static, f64>> = isos
            .iter()
            .map(|iso| required(iso.moy_errors(), ADD_ERRORS))
            .collect::<Result<_, _>>()?;
        let (err_u, err_v) = (
            border_max([&maxs[0], &maxs[1]], s.ord_in_v)?,
            border_max([&maxs[2], &maxs[3]], s.ord_in_u)?,
        );
        let (moy_u, moy_v) = (
            border_max([&moys[0], &moys[1]], s.ord_in_v)?,
            border_max([&moys[2], &moys[3]], s.ord_in_u)?,
        );
        let emax: [f64; 4] = corner_errors(&s, constraints)?;
        let borders: [(f64, f64); 4] = [
            (cpp_max(emax[0], emax[1]), hu),
            (cpp_max(emax[2], emax[3]), hu),
            (cpp_max(emax[0], emax[2]), hv),
            (cpp_max(emax[1], emax[3]), hv),
        ];
        let mut heriso: Array2<'static, f64> =
            Array2::with_bounds(1, 1, 1, 4).map_err(|e| e.into_failure(ADD_ERRORS))?;
        for (column, ((iso, table), (err, h))) in (1..).zip(isos.iter().zip(&maxs).zip(borders)) {
            let mut value: f64 = *table.value(1, 1).map_err(|e| e.into_failure(ADD_ERRORS))?;
            if iso.position() == 0 {
                value += err * h;
            }
            heriso
                .set_value(1, column, value)
                .map_err(|e| e.into_failure(ADD_ERRORS))?;
        }
        let mut state = self.state.write();
        let max_errors = state
            .max_errors
            .as_mut()
            .ok_or_else(|| construction("AdvApp2Var_Patch::AddErrors: no max errors"))?;
        *max_errors
            .change_value(1)
            .map_err(|e| e.into_failure(ADD_ERRORS))? += err_u * hv + err_v * hu;
        let moy_errors = state
            .moy_errors
            .as_mut()
            .ok_or_else(|| construction("AdvApp2Var_Patch::AddErrors: no average errors"))?;
        let moy = moy_errors
            .change_value(1)
            .map_err(|e| e.into_failure(ADD_ERRORS))?;
        let mut error: f64 = *moy;
        error *= error;
        error += moy_u * hv * moy_u * hv + moy_v * hu * moy_v * hu;
        *moy = error.sqrt();
        state.iso_errors = Some(heriso);
        Ok(())
    }
}
