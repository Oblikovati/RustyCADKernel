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
// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_Iso.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_Iso`: the constraints of the approximation on one iso-curve.
use super::iso_approx::IsoResult;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{SharedState, StandardTransient};
use rustycad_tkmath::geom_abs::IsoType;

/// The locked fields of an iso.
#[derive(Clone)]
pub(super) struct IsoState {
    pub(super) iso_type: IsoType,
    pub(super) const_par: f64,
    pub(super) u0: f64,
    pub(super) u1: f64,
    pub(super) v0: f64,
    pub(super) v1: f64,
    pub(super) position: i32,
    pub(super) extrem_order: i32,
    pub(super) deriv_order: i32,
    pub(super) nb_coeff: i32,
    pub(super) appr_is_done: bool,
    pub(super) has_result: bool,
    pub(super) equation: Option<Array1<'static, f64>>,
    pub(super) max_errors: Option<Array2<'static, f64>>,
    pub(super) moy_errors: Option<Array2<'static, f64>>,
    pub(super) som_tab: Option<Array1<'static, f64>>,
    pub(super) dif_tab: Option<Array1<'static, f64>>,
}

/// The iso-curve `U = cte` or `V = cte` of the domain `[U0,U1]x[V0,V1]`, with the
/// polynomial approximation of the function and of its cross derivatives on it
/// (`AdvApp2Var_Iso`). `position` is 0 for an inner cutting line and 1..4 for the
/// frontiers V0, V1, U0, U1.
///
/// Shared through handles; every member takes `&self`.
/// `AdvApp2VarIso::with_orders(IsoType::IsoU, 3, 1).u_order() == 3`.
pub struct AdvApp2VarIso {
    parent: StandardTransient,
    pub(super) state: SharedState<IsoState>,
}
impl_transient!(AdvApp2VarIso, "AdvApp2Var_Iso", StandardTransient, parent);

impl Default for AdvApp2VarIso {
    fn default() -> Self {
        Self::new()
    }
}

impl AdvApp2VarIso {
    /// The iso U = 0.5 of the unit square with orders (2, 2) (`AdvApp2Var_Iso()`);
    /// `AdvApp2VarIso::new()`.
    pub fn new() -> Self {
        Self::with_domain(IsoType::IsoU, 0.5, [0.0, 1.0, 0.0, 1.0], 0, 2, 2)
    }

    /// The iso of `iso_type` at 0.5 on the unit square, inner (`AdvApp2Var_Iso(type,
    /// iu, iv)`); `AdvApp2VarIso::with_orders(IsoType::IsoV, 1, 1)`.
    pub fn with_orders(iso_type: IsoType, iu: i32, iv: i32) -> Self {
        Self::with_domain(iso_type, 0.5, [0.0, 1.0, 0.0, 1.0], 0, iu, iv)
    }

    /// The iso of `iso_type` at `cte` on `domain = [Ufirst, Ulast, Vfirst, Vlast]` at
    /// `pos` (`AdvApp2Var_Iso(type, cte, Ufirst, Ulast, Vfirst, Vlast, pos, iu, iv)`):
    /// along a U-iso the derivative order is `iu` and the end order `iv`, and
    /// conversely. `AdvApp2VarIso::with_domain(IsoType::IsoV, 0.0, d, 1, 1, 1)`.
    pub fn with_domain(
        iso_type: IsoType,
        cte: f64,
        domain: [f64; 4],
        pos: i32,
        iu: i32,
        iv: i32,
    ) -> Self {
        let (extrem_order, deriv_order) = if iso_type == IsoType::IsoU {
            (iv, iu)
        } else {
            (iu, iv)
        };
        let state = IsoState {
            iso_type,
            const_par: cte,
            u0: domain[0],
            u1: domain[1],
            v0: domain[2],
            v1: domain[3],
            position: pos,
            extrem_order,
            deriv_order,
            nb_coeff: 0,
            appr_is_done: false,
            has_result: false,
            equation: None,
            max_errors: None,
            moy_errors: None,
            som_tab: None,
            dif_tab: None,
        };
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }

    /// Whether the approximation met its tolerance (`IsApproximated`); `iso.is_approximated()`.
    pub fn is_approximated(&self) -> bool {
        self.state.read().appr_is_done
    }

    /// Whether an approximation exists, within tolerance or not (`HasResult`);
    /// `iso.has_result()`.
    pub fn has_result(&self) -> bool {
        self.state.read().has_result
    }

    /// Restricts the iso along its direction to `[a, b]` (`ChangeDomain(a, b)`);
    /// `iso.change_domain(0.0, 0.5)`.
    pub fn change_domain(&self, a: f64, b: f64) {
        let mut state = self.state.write();
        if state.iso_type == IsoType::IsoU {
            state.v0 = a;
            state.v1 = b;
        } else {
            state.u0 = a;
            state.u1 = b;
        }
    }

    /// Replaces the whole domain (`ChangeDomain(a, b, c, d)`);
    /// `iso.change_domain_uv(0.0, 1.0, 0.0, 0.5)`.
    pub fn change_domain_uv(&self, a: f64, b: f64, c: f64, d: f64) {
        let mut state = self.state.write();
        (state.u0, state.u1, state.v0, state.v1) = (a, b, c, d);
    }

    /// Moves the iso to the constant `newcte` (`SetConstante`); `iso.set_constante(0.25)`.
    pub fn set_constante(&self, newcte: f64) {
        self.state.write().const_par = newcte;
    }

    /// Changes the position (0 inner, 1..4 frontier) (`SetPosition`); `iso.set_position(1)`.
    pub fn set_position(&self, newpos: i32) {
        self.state.write().position = newpos;
    }

    /// Forgets the approximation status (`ResetApprox`); `iso.reset_approx()`.
    pub fn reset_approx(&self) {
        let mut state = self.state.write();
        state.appr_is_done = false;
        state.has_result = false;
    }

    /// Accepts an existing result as approximated (`OverwriteApprox`);
    /// `iso.overwrite_approx()`.
    pub fn overwrite_approx(&self) {
        let mut state = self.state.write();
        if state.has_result {
            state.appr_is_done = true;
        }
    }

    /// U-iso or V-iso (`Type`); `iso.iso_type()`.
    pub fn iso_type(&self) -> IsoType {
        self.state.read().iso_type
    }

    /// The constant parameter (`Constante`); `iso.constante()`.
    pub fn constante(&self) -> f64 {
        self.state.read().const_par
    }

    /// The start of the iso along its direction (`T0`); `iso.t0()`.
    pub fn t0(&self) -> f64 {
        let state = self.state.read();
        if state.iso_type == IsoType::IsoU {
            state.v0
        } else {
            state.u0
        }
    }

    /// The end of the iso along its direction (`T1`); `iso.t1()`.
    pub fn t1(&self) -> f64 {
        let state = self.state.read();
        if state.iso_type == IsoType::IsoU {
            state.v1
        } else {
            state.u1
        }
    }

    /// `U0`; `iso.u0()`.
    pub fn u0(&self) -> f64 {
        self.state.read().u0
    }

    /// `U1`; `iso.u1()`.
    pub fn u1(&self) -> f64 {
        self.state.read().u1
    }

    /// `V0`; `iso.v0()`.
    pub fn v0(&self) -> f64 {
        self.state.read().v0
    }

    /// `V1`; `iso.v1()`.
    pub fn v1(&self) -> f64 {
        self.state.read().v1
    }

    /// The order in U: the derivative order of a U-iso, the end order of a V-iso
    /// (`UOrder`); `iso.u_order()`.
    pub fn u_order(&self) -> i32 {
        let state = self.state.read();
        if state.iso_type == IsoType::IsoU {
            state.deriv_order
        } else {
            state.extrem_order
        }
    }

    /// The order in V (`VOrder`); `iso.v_order()`.
    pub fn v_order(&self) -> i32 {
        let state = self.state.read();
        if state.iso_type == IsoType::IsoV {
            state.deriv_order
        } else {
            state.extrem_order
        }
    }

    /// 0 for an inner line, 1..4 for a frontier (`Position`); `iso.position()`.
    pub fn position(&self) -> i32 {
        self.state.read().position
    }

    /// The largest coefficient count of the approximations (`NbCoeff`); `iso.nb_coeff()`.
    pub fn nb_coeff(&self) -> i32 {
        self.state.read().nb_coeff
    }

    /// The canonical coefficients `(NCFLIM, NDIMEN, deriv+1)`, `None` without a
    /// result (`Polynom`); `iso.polynom()`.
    pub fn polynom(&self) -> Option<Array1<'static, f64>> {
        self.state.read().equation.clone()
    }

    /// The folded sums of the samples per derivative (`SomTab`); `iso.som_tab()`.
    pub fn som_tab(&self) -> Option<Array1<'static, f64>> {
        self.state.read().som_tab.clone()
    }

    /// The folded differences of the samples per derivative (`DifTab`); `iso.dif_tab()`.
    pub fn dif_tab(&self) -> Option<Array1<'static, f64>> {
        self.state.read().dif_tab.clone()
    }

    /// The maximum errors `(NBSESP, deriv+1)` (`MaxErrors`); `iso.max_errors()`.
    pub fn max_errors(&self) -> Option<Array2<'static, f64>> {
        self.state.read().max_errors.clone()
    }

    /// The average errors `(NBSESP, deriv+1)` (`MoyErrors`); `iso.moy_errors()`.
    pub fn moy_errors(&self) -> Option<Array2<'static, f64>> {
        self.state.read().moy_errors.clone()
    }

    /// Stores the outcome of `MakeApprox`: the flags, and the tables when there is
    /// a result.
    pub(super) fn store(&self, result: IsoResult) {
        let mut state = self.state.write();
        state.appr_is_done = result.done;
        state.has_result = result.has_result;
        if let Some(tables) = result.tables {
            state.equation = Some(tables.equation);
            state.nb_coeff = tables.nb_coeff;
            state.max_errors = Some(tables.max_errors);
            state.moy_errors = Some(tables.moy_errors);
            state.som_tab = Some(tables.som_tab);
            state.dif_tab = Some(tables.dif_tab);
        }
    }
}
