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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BezierSurface.cxx (fields, init, Rational, evaluation, iso curves, Resolution) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fields of Geom_BezierSurface, their initialisation and the evaluators.
use super::grid::{at2, one_based_grid};
use crate::geom::GeomBezierCurve;
use crate::geom::GeomCurve;
use crate::geom::errors;
use crate::geom::{GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3};
use crate::geom_eval::{RepSurfaceDesc, rep_utils};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle, real};
use rustycad_tkg2d::geom2d::spline_data::{
    bezier_knot_sequence, bezier_knots, bezier_multiplicities,
};
use rustycad_tkmath::bsplslib::{self, ResolutionDirection, SurfaceSpan};
use rustycad_tkmath::gp::{Point3, Vector3};

pub(super) const CLASS: &str = "Geom_BezierSurface";

#[derive(Clone)]
pub(super) struct SurfaceState {
    pub(super) poles: Array2<'static, Point3>,
    pub(super) weights: Array2<'static, f64>,
    pub(super) eval_rep: Handle<dyn RepSurfaceDesc>,
    pub(super) u_rational: bool,
    pub(super) v_rational: bool,
    pub(super) u_max_deriv_inv: f64,
    pub(super) v_max_deriv_inv: f64,
    pub(super) max_deriv_inv_ok: bool,
}

// One weight step of the file-static `Rational`: `|W(a) - W(b)| > Epsilon(|W(a)|)`.
fn step(weights: &Array2<'_, f64>, a: (i32, i32), b: (i32, i32)) -> Result<bool, Failure> {
    let first: f64 = at2(weights, a.0, a.1)?;
    Ok((first - at2(weights, b.0, b.1)?).abs() > real::epsilon(first.abs()))
}

/// The file-static `Rational(Weights, Urational, Vrational)`: V-rational when a column
/// has two different consecutive weights, U-rational when a row has;
/// `rationality(&weights)?`.
pub(super) fn rationality(weights: &Array2<'_, f64>) -> Result<(bool, bool), Failure> {
    let (lr, ur, lc, uc) = (
        weights.lower_row(),
        weights.upper_row(),
        weights.lower_col(),
        weights.upper_col(),
    );
    let mut v_rational: bool = false;
    for j in lc..=uc {
        for i in lr..ur {
            if step(weights, (i, j), (i + 1, j))? {
                v_rational = true;
                break;
            }
        }
        if v_rational {
            break;
        }
    }
    let mut u_rational: bool = false;
    for i in lr..=ur {
        for j in lc..uc {
            if step(weights, (i, j), (i, j + 1))? {
                u_rational = true;
                break;
            }
        }
        if u_rational {
            break;
        }
    }
    Ok((u_rational, v_rational))
}

/// A one-based copy of `source` (`Resize(1, n, 1, m)` then `CopyValues`).
pub(super) fn rebased_grid<T: Default + Clone>(
    source: &Array2<'_, T>,
) -> Result<Array2<'static, T>, Failure> {
    let mut target: Array2<'static, T> =
        one_based_grid(source.nb_rows() as i32, source.nb_columns() as i32)?;
    target.as_mut_slice().clone_from_slice(source.as_slice());
    Ok(target)
}

impl SurfaceState {
    /// `init(thePoles, theWeights)` on a fresh state; `SurfaceState::initialized(&p, None)?`.
    pub(super) fn initialized(
        poles: &Array2<'_, Point3>,
        weights: Option<&Array2<'_, f64>>,
    ) -> Result<Self, Failure> {
        let mut state = SurfaceState {
            poles: Array2::default(),
            weights: Array2::default(),
            eval_rep: Handle::default(),
            u_rational: false,
            v_rational: false,
            u_max_deriv_inv: 0.0,
            v_max_deriv_inv: 0.0,
            max_deriv_inv_ok: false,
        };
        state.init(poles, weights)?;
        Ok(state)
    }
    // Geom_BezierSurface::init: one-based copies, rationality, unit weights otherwise,
    // cache flag and evaluation representation reset.
    fn init(
        &mut self,
        poles: &Array2<'_, Point3>,
        weights: Option<&Array2<'_, f64>>,
    ) -> Result<(), Failure> {
        self.poles = rebased_grid(poles)?;
        (self.u_rational, self.v_rational) = (false, false);
        self.weights = self.unit_weights()?;
        if let Some(weights) = weights {
            self.weights = rebased_grid(weights)?;
            self.update_rationality()?;
        }
        self.max_deriv_inv_ok = false;
        self.eval_rep = Handle::default();
        Ok(())
    }
    /// `Rational(myWeights, ...)` then unit weights when neither direction is rational,
    /// the tail of most edits.
    pub(super) fn update_rationality(&mut self) -> Result<(), Failure> {
        (self.u_rational, self.v_rational) = rationality(&self.weights)?;
        if !self.is_rational() {
            self.weights = self.unit_weights()?;
        }
        Ok(())
    }
    /// `BSplSLib::UnitWeights(NbUPoles, NbVPoles)`.
    pub(super) fn unit_weights(&self) -> Result<Array2<'static, f64>, Failure> {
        bsplslib::unit_weights(self.nb_u_poles(), self.nb_v_poles())
    }
    pub(super) fn is_rational(&self) -> bool {
        self.u_rational || self.v_rational
    }
    /// `Weights()`: the weights of a rational surface, `None` otherwise.
    pub(super) fn weights_view(&self) -> Option<&Array2<'static, f64>> {
        self.is_rational().then_some(&self.weights)
    }
    /// `myPoles.ColLength()`.
    pub(super) fn nb_u_poles(&self) -> i32 {
        self.poles.nb_rows() as i32
    }
    /// `myPoles.RowLength()`.
    pub(super) fn nb_v_poles(&self) -> i32 {
        self.poles.nb_columns() as i32
    }
    pub(super) fn u_degree(&self) -> i32 {
        self.nb_u_poles() - 1
    }
    pub(super) fn v_degree(&self) -> i32 {
        self.nb_v_poles() - 1
    }
    /// `UMultiplicities()`: `{UDegree+1, UDegree+1}`.
    pub(super) fn u_mults(&self) -> Result<Array1<'static, i32>, Failure> {
        bezier_multiplicities(self.u_degree(), CLASS)
    }
    /// `VMultiplicities()`: `{VDegree+1, VDegree+1}`.
    pub(super) fn v_mults(&self) -> Result<Array1<'static, i32>, Failure> {
        bezier_multiplicities(self.v_degree(), CLASS)
    }
    /// `UKnotSequence()`.
    pub(super) fn u_knot_sequence(&self) -> Result<Array1<'static, f64>, Failure> {
        bezier_knot_sequence(self.u_degree(), CLASS)
    }
    /// `VKnotSequence()`.
    pub(super) fn v_knot_sequence(&self) -> Result<Array1<'static, f64>, Failure> {
        bezier_knot_sequence(self.v_degree(), CLASS)
    }
}

// The knots, multiplicities and flags of the BSplSLib evaluation calls. The native
// code passes UKnots() for both directions (the Bezier knots are {0, 1} either way).
struct EvalData {
    knots: Array1<'static, f64>,
    u_mults: Array1<'static, i32>,
    v_mults: Array1<'static, i32>,
}

impl SurfaceState {
    fn eval_data(&self) -> Result<EvalData, Failure> {
        Ok(EvalData {
            knots: bezier_knots(),
            u_mults: self.u_mults()?,
            v_mults: self.v_mults()?,
        })
    }
    fn span<'r>(&self, data: &'r EvalData, u: f64, v: f64, index: i32) -> SurfaceSpan<'r, 'static> {
        SurfaceSpan {
            u,
            v,
            u_index: index,
            v_index: index,
            u_knots: &data.knots,
            v_knots: &data.knots,
            u_mults: Some(&data.u_mults),
            v_mults: Some(&data.v_mults),
            u_degree: self.u_degree(),
            v_degree: self.v_degree(),
            u_rat: self.u_rational,
            v_rat: self.v_rational,
            u_per: false,
            v_per: false,
        }
    }
    // Quirk kept: EvalD0 to EvalD2 pass the span hints (1, 1), EvalD3 and EvalDN (0, 0).
    pub(super) fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        if let Some(point) = rep_utils::try_eval_surface_d0(&self.eval_rep, u, v)? {
            return Ok(point);
        }
        let data = self.eval_data()?;
        bsplslib::d0(&self.span(&data, u, v, 1), &self.poles, self.weights_view())
    }
    pub(super) fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        if let Some(result) = rep_utils::try_eval_surface_d1(&self.eval_rep, u, v)? {
            return Ok(result);
        }
        let data = self.eval_data()?;
        let (point, d1u, d1v) =
            bsplslib::d1(&self.span(&data, u, v, 1), &self.poles, self.weights_view())?;
        Ok(GeomSurfaceD1 { point, d1u, d1v })
    }
    pub(super) fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        if let Some(result) = rep_utils::try_eval_surface_d2(&self.eval_rep, u, v)? {
            return Ok(result);
        }
        let data = self.eval_data()?;
        let r = bsplslib::d2(&self.span(&data, u, v, 1), &self.poles, self.weights_view())?;
        Ok(GeomSurfaceD2 {
            point: r.p,
            d1u: r.vu,
            d1v: r.vv,
            d2u: r.vuu,
            d2v: r.vvv,
            d2uv: r.vuv,
        })
    }
    pub(super) fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        if let Some(result) = rep_utils::try_eval_surface_d3(&self.eval_rep, u, v)? {
            return Ok(result);
        }
        let data = self.eval_data()?;
        let r = bsplslib::d3(&self.span(&data, u, v, 0), &self.poles, self.weights_view())?;
        Ok(GeomSurfaceD3 {
            point: r.p,
            d1u: r.vu,
            d1v: r.vv,
            d2u: r.vuu,
            d2v: r.vvv,
            d2uv: r.vuv,
            d3u: r.vuuu,
            d3v: r.vvvv,
            d3uuv: r.vuuv,
            d3uvv: r.vuvv,
        })
    }
    // Quirk kept: the evaluation representation is tried before the order check.
    pub(super) fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        if let Some(vector) = rep_utils::try_eval_surface_dn(&self.eval_rep, u, v, nu, nv)? {
            return Ok(vector);
        }
        crate::geom::check_dn_orders(CLASS, nu, nv)?;
        let data = self.eval_data()?;
        bsplslib::dn(
            &self.span(&data, u, v, 0),
            nu,
            nv,
            &self.poles,
            self.weights_view(),
        )
    }
    /// `UIso` (`is_u`) or `VIso`: the iso poles by BSplSLib::Iso; the curve is rational
    /// only when the surface is rational in the direction along the curve (U for UIso,
    /// V for VIso), as natively.
    pub(super) fn iso(&self, param: f64, is_u: bool) -> Result<Handle<dyn GeomCurve>, Failure> {
        let count: i32 = if is_u {
            self.nb_v_poles()
        } else {
            self.nb_u_poles()
        };
        let mut cpoles: Array1<'static, Point3> =
            Array1::with_bounds(1, count).map_err(errors::array_bounds)?;
        let mut cweights: Array1<'static, f64> =
            Array1::with_bounds(1, count).map_err(errors::array_bounds)?;
        let (mults, degree) = if is_u {
            (self.u_mults()?, self.u_degree())
        } else {
            (self.v_mults()?, self.v_degree())
        };
        let rational: bool = self.is_rational();
        let target: Option<&mut Array1<'static, f64>> = rational.then_some(&mut cweights);
        let knots = bezier_knots();
        bsplslib::iso(
            param,
            is_u,
            &self.poles,
            self.weights_view(),
            &knots,
            Some(&mults),
            degree,
            false,
            &mut cpoles,
            target,
        )?;
        let weighted: bool = if is_u {
            self.u_rational
        } else {
            self.v_rational
        };
        let curve = if weighted {
            GeomBezierCurve::with_weights(&cpoles, &cweights)?
        } else {
            GeomBezierCurve::new(&cpoles)?
        };
        Ok(Handle::new(curve).cast())
    }
    /// `Resolution`: fills the cached inverse derivative bounds once, then scales them.
    pub(super) fn resolution(&mut self, tolerance_3d: f64) -> Result<(f64, f64), Failure> {
        if !self.max_deriv_inv_ok {
            let (knots, u_mults, v_mults) = (bezier_knots(), self.u_mults()?, self.v_mults()?);
            let direction = |mults, degree, rational| ResolutionDirection {
                knots: &knots,
                mults,
                degree,
                rational,
                periodic: false,
            };
            let u = direction(&u_mults, self.u_degree(), self.u_rational);
            let v = direction(&v_mults, self.v_degree(), self.v_rational);
            (self.u_max_deriv_inv, self.v_max_deriv_inv) =
                bsplslib::resolution(&self.poles, self.weights_view(), u, v, 1.)?;
            self.max_deriv_inv_ok = true;
        }
        Ok((
            tolerance_3d * self.u_max_deriv_inv,
            tolerance_3d * self.v_max_deriv_inv,
        ))
    }
}
