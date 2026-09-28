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
// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_Context.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_Context`: the fixed parameters of an approximation.
use super::approx_f2var::{mma2jmx_, mma2roo_, mmapptt_};
use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
use rustycad_tkernel::standard::Failure;

/// The per-subspace tolerances of the three dimensions, as the native handles
/// (`None` for a null handle, which a zero count never reads).
#[derive(Clone, Copy, Default)]
pub struct SubspaceTolerances<'a> {
    /// `tol1D`/`tof1D`.
    pub one_d: Option<&'a Array1<'a, f64>>,
    /// `tol2D`/`tof2D`.
    pub two_d: Option<&'a Array1<'a, f64>>,
    /// `tol3D`/`tof3D`.
    pub three_d: Option<&'a Array1<'a, f64>>,
}

/// The per-subspace frontier tolerances `(nbSS, 4)` of the three dimensions.
#[derive(Clone, Copy, Default)]
pub struct FrontierTolerances<'a> {
    /// `tof1D`.
    pub one_d: Option<&'a Array2<'a, f64>>,
    /// `tof2D`.
    pub two_d: Option<&'a Array2<'a, f64>>,
    /// `tof3D`.
    pub three_d: Option<&'a Array2<'a, f64>>,
}

/// The degrees, point counts, Jacobi tables and tolerances an approximation uses
/// (`AdvApp2Var_Context`). A default context has zero counts and no tables.
///
/// `AdvApp2VarContext::new(0, 0, 0, 2, 2, 0, [1, 1, 1], tol, fr)?.total_dimension()`.
#[derive(Clone, Default)]
pub struct AdvApp2VarContext {
    fav: i32,
    ord_u: i32,
    ord_v: i32,
    lim_u: i32,
    lim_v: i32,
    nb_dss: [i32; 3],
    jdeg_u: i32,
    jdeg_v: i32,
    jmax_u: Option<Array1<'static, f64>>,
    jmax_v: Option<Array1<'static, f64>>,
    u_roots: Option<Array1<'static, f64>>,
    v_roots: Option<Array1<'static, f64>>,
    u_gauss: Option<Array1<'static, f64>>,
    v_gauss: Option<Array1<'static, f64>>,
    internal_tol: Option<Array1<'static, f64>>,
    frontier_tol: Option<Array2<'static, f64>>,
    cutting_tol: Option<Array2<'static, f64>>,
}

/// `lesparam`: the Jacobi degree and the point count for the continuity order
/// `iordre`, the coefficient limit `ncflim` and the precision code `icodeo`;
/// `None` for a negative code.
fn lesparam(iordre: i32, ncflim: i32, icodeo: i32) -> Option<(i32, i32)> {
    // The Jacobi degree always keeps a reserve coefficient.
    let mut ndgjac: i32 = ncflim;
    if icodeo < 0 {
        return None;
    }
    if icodeo > 0 {
        ndgjac += 9 - (iordre + 1);
        ndgjac += (icodeo - 1) * 10;
    }
    // The counts 15, 25 and 61 are skipped natively ("Bug Uneven number").
    let mut nbpnts: i32 = match ndgjac {
        ..8 => 8,
        8..10 => 10,
        10..20 => 20,
        20..30 => 30,
        30..40 => 40,
        _ => 50,
    };
    if iordre > -1 {
        nbpnts += 2;
    }
    Some((nbpnts, ndgjac))
}

fn construction(message: &str) -> Failure {
    Failure::with_message("Standard_ConstructionError", message)
}

fn array_failure(error: ArrayError) -> Failure {
    error.into_failure("AdvApp2Var_Context")
}

fn one_based(size: i32) -> Result<Array1<'static, f64>, Failure> {
    Array1::with_bounds(1, size).map_err(array_failure)
}

/// The per-direction tables of the constructor: point count, Jacobi degree,
/// Jacobi maxima.
struct DirectionTables {
    nb_root: i32,
    jdeg: i32,
    points: i32,
    jmax: Array1<'static, f64>,
}

fn direction(order: i32, nlim: i32, iprecis: i32) -> Result<DirectionTables, Failure> {
    let limit: i32 = nlim.max(2 * order + 2);
    let (nb_root, jdeg) =
        lesparam(order, limit, iprecis).ok_or_else(|| construction("AdvApp2Var_Context"))?;
    let points: i32 = if order > -1 { nb_root - 2 } else { nb_root };
    let mut jmax = one_based(jdeg - 2 * order - 1)?;
    mma2jmx_(jdeg, order, jmax.as_mut_slice());
    Ok(DirectionTables {
        nb_root,
        jdeg,
        points,
        jmax,
    })
}

fn gauss_table(tables: &DirectionTables, order: i32) -> Result<Array1<'static, f64>, Failure> {
    let size: i32 = (tables.points / 2 + 1) * (tables.jdeg - 2 * order - 1);
    let mut gauss = one_based(size)?;
    let mut error: i32 = 0;
    mmapptt_(
        tables.jdeg,
        tables.points,
        order,
        gauss.as_mut_slice(),
        &mut error,
    );
    if error != 0 {
        return Err(construction("AdvApp2Var_Context : Error in FORTRAN"));
    }
    Ok(gauss)
}

/// The weight of the border tolerances of the order: 0, 1, 1.5 or 1.75
/// (`hMaxFactor`).
fn h_max_factor(order: i32) -> f64 {
    match order {
        -1 => 0.0,
        0 => 1.0,
        1 => 1.5,
        _ => 1.75,
    }
}

/// Copies `count` values of `source` (1-based) after `offset` in `target`
/// (`appendInternalTolerance`).
fn append_internal(
    source: Option<&Array1<'_, f64>>,
    count: i32,
    offset: i32,
    target: &mut Array1<'static, f64>,
) -> Result<(), Failure> {
    for index in 1..=count {
        let source =
            source.ok_or_else(|| construction("AdvApp2Var_Context: null tolerance array"))?;
        let value: f64 = *source.value(index).map_err(array_failure)?;
        target
            .set_value(offset + index, value)
            .map_err(array_failure)?;
    }
    Ok(())
}

/// Copies the four frontier tolerances of `count` subspaces after `offset`, and
/// zero cutting tolerances (`appendFrontierTolerance`).
fn append_frontier(
    source: Option<&Array2<'_, f64>>,
    count: i32,
    offset: i32,
    frontier: &mut Array2<'static, f64>,
    cutting: &mut Array2<'static, f64>,
) -> Result<(), Failure> {
    for index in 1..=count {
        let source =
            source.ok_or_else(|| construction("AdvApp2Var_Context: null tolerance array"))?;
        for column in 1..=4 {
            let value: f64 = *source.value(index, column).map_err(array_failure)?;
            frontier
                .set_value(offset + index, column, value)
                .map_err(array_failure)?;
            cutting
                .set_value(offset + index, column, 0.0)
                .map_err(array_failure)?;
        }
    }
    Ok(())
}

impl AdvApp2VarContext {
    /// The context of an approximation (`AdvApp2Var_Context(ifav, iu, iv, nlimu,
    /// nlimv, iprecis, nb1Dss, nb2Dss, nb3Dss, tol1D, tol2D, tol3D, tof1D, tof2D,
    /// tof3D)`); `nb_dss` holds the three subspace counts. Standard_ConstructionError
    /// for a negative precision code or a failed Gauss table.
    /// `AdvApp2VarContext::new(0, 0, 0, 2, 2, 0, [1, 1, 1], tol, fr)?`.
    #[allow(clippy::too_many_arguments)] // The native member list, counts grouped.
    pub fn new(
        ifav: i32,
        iu: i32,
        iv: i32,
        nlimu: i32,
        nlimv: i32,
        iprecis: i32,
        nb_dss: [i32; 3],
        tol: SubspaceTolerances<'_>,
        tof: FrontierTolerances<'_>,
    ) -> Result<Self, Failure> {
        let u = direction(iu, nlimu, iprecis)?;
        let v = direction(iv, nlimv, iprecis)?;
        let mut u_roots = one_based(u.nb_root)?;
        let mut v_roots = one_based(v.nb_root)?;
        mma2roo_(
            u.points,
            v.points,
            u_roots.as_mut_slice(),
            v_roots.as_mut_slice(),
        );
        let u_gauss = gauss_table(&u, iu)?;
        let v_gauss = gauss_table(&v, iv)?;
        let mut context = Self {
            fav: ifav,
            ord_u: iu,
            ord_v: iv,
            lim_u: nlimu,
            lim_v: nlimv,
            nb_dss,
            jdeg_u: u.jdeg,
            jdeg_v: v.jdeg,
            jmax_u: Some(u.jmax),
            jmax_v: Some(v.jmax),
            u_roots: Some(u_roots),
            v_roots: Some(v_roots),
            u_gauss: Some(u_gauss),
            v_gauss: Some(v_gauss),
            ..Self::default()
        };
        context.set_tolerances(tol, tof)?;
        Ok(context)
    }

    /// The internal, frontier and cutting tolerances of the constructor.
    fn set_tolerances(
        &mut self,
        tol: SubspaceTolerances<'_>,
        tof: FrontierTolerances<'_>,
    ) -> Result<(), Failure> {
        let [nb1, nb2, nb3] = self.nb_dss;
        let nb_ssp: i32 = nb1 + nb2 + nb3;
        let mut itol = one_based(nb_ssp)?;
        append_internal(tol.one_d, nb1, 0, &mut itol)?;
        append_internal(tol.two_d, nb2, nb1, &mut itol)?;
        append_internal(tol.three_d, nb3, nb1 + nb2, &mut itol)?;
        let constrained: bool = self.ord_u > -1 || self.ord_v > -1;
        if constrained {
            itol.iter_mut().for_each(|value| *value /= 2.0);
        }
        let mut ftol: Array2<'static, f64> =
            Array2::with_bounds(1, nb_ssp, 1, 4).map_err(array_failure)?;
        let mut ctol: Array2<'static, f64> =
            Array2::with_bounds(1, nb_ssp, 1, 4).map_err(array_failure)?;
        append_frontier(tof.one_d, nb1, 0, &mut ftol, &mut ctol)?;
        append_frontier(tof.two_d, nb2, nb1, &mut ftol, &mut ctol)?;
        append_frontier(tof.three_d, nb3, nb1 + nb2, &mut ftol, &mut ctol)?;
        if constrained {
            let (hu, hv) = (h_max_factor(self.ord_u), h_max_factor(self.ord_v));
            let weight: f64 = hu * hv + hu + hv;
            for ssp in 1..=nb_ssp {
                for column in 1..=4 {
                    let tol_min: f64 = *itol.value(ssp).map_err(array_failure)? / weight;
                    let frontier = ftol.change_value(ssp, column).map_err(array_failure)?;
                    if tol_min < *frontier {
                        *frontier = tol_min;
                    }
                    ctol.set_value(ssp, column, tol_min)
                        .map_err(array_failure)?;
                }
            }
        }
        self.internal_tol = Some(itol);
        self.frontier_tol = Some(ftol);
        self.cutting_tol = Some(ctol);
        Ok(())
    }
}

impl AdvApp2VarContext {
    /// The total dimension `nb1 + 2 nb2 + 3 nb3` (`TotalDimension`); `ctx.total_dimension()`.
    pub fn total_dimension(&self) -> i32 {
        self.nb_dss[0] + 2 * self.nb_dss[1] + 3 * self.nb_dss[2]
    }

    /// The number of subspaces (`TotalNumberSSP`); `ctx.total_number_ssp()`.
    pub fn total_number_ssp(&self) -> i32 {
        self.nb_dss.iter().sum()
    }

    /// 1 to evaluate U-isos first, 2 for V-isos (`FavorIso`); `ctx.favor_iso()`.
    pub fn favor_iso(&self) -> i32 {
        self.fav
    }

    /// The continuity order in U (`UOrder`); `ctx.u_order()`.
    pub fn u_order(&self) -> i32 {
        self.ord_u
    }

    /// The continuity order in V (`VOrder`); `ctx.v_order()`.
    pub fn v_order(&self) -> i32 {
        self.ord_v
    }

    /// The coefficient limit in U (`ULimit`); `ctx.u_limit()`.
    pub fn u_limit(&self) -> i32 {
        self.lim_u
    }

    /// The coefficient limit in V (`VLimit`); `ctx.v_limit()`.
    pub fn v_limit(&self) -> i32 {
        self.lim_v
    }

    /// The Jacobi degree in U (`UJacDeg`); `ctx.u_jac_deg()`.
    pub fn u_jac_deg(&self) -> i32 {
        self.jdeg_u
    }

    /// The Jacobi degree in V (`VJacDeg`); `ctx.v_jac_deg()`.
    pub fn v_jac_deg(&self) -> i32 {
        self.jdeg_v
    }

    /// The Jacobi maxima in U, `None` for a default context (`UJacMax`); `ctx.u_jac_max()`.
    pub fn u_jac_max(&self) -> Option<&Array1<'static, f64>> {
        self.jmax_u.as_ref()
    }

    /// The Jacobi maxima in V (`VJacMax`); `ctx.v_jac_max()`.
    pub fn v_jac_max(&self) -> Option<&Array1<'static, f64>> {
        self.jmax_v.as_ref()
    }

    /// The U Legendre roots (`URoots`): the roots, then two unused slots when the
    /// order is at least 0 (uninitialised natively, zero here). `ctx.u_roots()`.
    pub fn u_roots(&self) -> Option<&Array1<'static, f64>> {
        self.u_roots.as_ref()
    }

    /// The V Legendre roots (`VRoots`); `ctx.v_roots()`.
    pub fn v_roots(&self) -> Option<&Array1<'static, f64>> {
        self.v_roots.as_ref()
    }

    /// The U Gauss projection table (`UGauss`); `ctx.u_gauss()`.
    pub fn u_gauss(&self) -> Option<&Array1<'static, f64>> {
        self.u_gauss.as_ref()
    }

    /// The V Gauss projection table (`VGauss`); `ctx.v_gauss()`.
    pub fn v_gauss(&self) -> Option<&Array1<'static, f64>> {
        self.v_gauss.as_ref()
    }

    /// The internal tolerance of each subspace, halved when an order is at least 0
    /// (`IToler`); `ctx.i_toler()`.
    pub fn i_toler(&self) -> Option<&Array1<'static, f64>> {
        self.internal_tol.as_ref()
    }

    /// The frontier tolerances `(nbSSP, 4)` (`FToler`); `ctx.f_toler()`.
    pub fn f_toler(&self) -> Option<&Array2<'static, f64>> {
        self.frontier_tol.as_ref()
    }

    /// The cutting tolerances `(nbSSP, 4)` (`CToler`); `ctx.c_toler()`.
    pub fn c_toler(&self) -> Option<&Array2<'static, f64>> {
        self.cutting_tol.as_ref()
    }
}
