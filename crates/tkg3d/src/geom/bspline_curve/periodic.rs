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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineCurve.cxx, Geom_BSplineCurve_1.cxx (periodicity, origin, Resolution) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Periodicity changes, origin shifts and `Resolution`.
use super::state::BSplineState;
use crate::geom::errors;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkg2d::geom2d::spline_data::rotated;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::Point3;

impl BSplineState {
    /// `SetPeriodic`: keep the knots between the first and last used ones, with equal
    /// end multiplicities, and the first `NbPoles` poles.
    pub(super) fn set_periodic(&mut self) -> Result<(), Failure> {
        self.clear_eval_rep();
        let (first, last) = (self.first_u_knot_index()?, self.last_u_knot_index()?);
        let count: i32 = last - first + 1;
        let mut knots: Array1<'static, f64> = one_based(count)?;
        let mut mults: Array1<'static, i32> = one_based(count)?;
        for k in first..=last {
            put(&mut knots, k - first + 1, at(&self.knots, k)?)?;
            put(&mut mults, k - first + 1, at(&self.mults, k)?)?;
        }
        let end: i32 = at(&mults, 1)?.max(at(&mults, count)?).min(self.deg);
        put(&mut mults, 1, end)?;
        put(&mut mults, count, end)?;
        self.knots = knots;
        self.mults = mults;
        let nbp: i32 = bsplclib::nb_poles(self.deg, true, &self.mults)?;
        self.poles
            .resize_bounds(1, nbp, true)
            .map_err(errors::array_bounds)?;
        if self.rational {
            self.weights
                .resize_bounds(1, nbp, true)
                .map_err(errors::array_bounds)?;
        } else {
            self.weights = bsplclib::unit_weights(nbp)?;
        }
        self.periodic = true;
        self.max_deriv_inv_ok = false;
        self.update_knots()
    }
    /// `SetOrigin(Index)`: make knot `index` the first one of a periodic curve; the
    /// evaluation representation is cleared before the index check, as natively.
    pub(super) fn set_origin(&mut self, index: i32) -> Result<(), Failure> {
        if !self.periodic {
            return Err(errors::no_such_object(
                "Geom_BSplineCurve::SetOrigin: the curve is not periodic",
            ));
        }
        self.clear_eval_rep();
        let (first, last) = (self.first_u_knot_index()?, self.last_u_knot_index()?);
        if index < first || index > last {
            return Err(errors::domain(&format!(
                "Geom_BSplineCurve::SetOrigin: Index = {index}; expected in [{first}, {last}]"
            )));
        }
        let (knots, mults) = self.rotated_knots(index, first, last)?;
        let mut pole_index: i32 = 1;
        for i in first + 1..=index {
            pole_index += at(&self.mults, i)?;
        }
        self.poles = rotated(&self.poles, pole_index)?;
        if self.rational {
            self.weights = rotated(&self.weights, pole_index)?;
        } else {
            self.weights = bsplclib::unit_weights(self.poles.len() as i32)?;
        }
        (self.knots, self.mults) = (knots, mults);
        self.max_deriv_inv_ok = false;
        self.update_knots()
    }
    // SetOrigin's knots: `index..last`, then `first+1..index` shifted by one period.
    fn rotated_knots(
        &self,
        index: i32,
        first: i32,
        last: i32,
    ) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
        let nbknots: i32 = self.knots.len() as i32;
        let mut knots: Array1<'static, f64> = one_based(nbknots)?;
        let mut mults: Array1<'static, i32> = one_based(nbknots)?;
        let period: f64 = at(&self.knots, last)? - at(&self.knots, first)?;
        let mut k: i32 = 1;
        for i in index..=last {
            put(&mut knots, k, at(&self.knots, i)?)?;
            put(&mut mults, k, at(&self.mults, i)?)?;
            k += 1;
        }
        for i in first + 1..=index {
            put(&mut knots, k, at(&self.knots, i)? + period)?;
            put(&mut mults, k, at(&self.mults, i)?)?;
            k += 1;
        }
        Ok((knots, mults))
    }
    /// `SetOrigin(U, Tol)`: shift the knots by whole periods so that `u` lies in the
    /// range, then make the knot at `u` (inserted when none is within `tol`) the origin.
    ///
    /// Like the native loops, an infinite `u` or period never terminates; callers pass
    /// finite values.
    pub(super) fn set_origin_parameter(&mut self, u: f64, tol: f64) -> Result<(), Failure> {
        if !self.periodic {
            return Err(errors::no_such_object(
                "Geom_BSplineCurve::SetOrigin: the curve is not periodic",
            ));
        }
        let (uf, ul) = self.shift_range_to(u, tol)?;
        if (u - uf).abs() < tol || (u - ul).abs() < tol {
            return Ok(());
        }
        let (mut ik, delta) = self.nearest_knot(u)?;
        if delta.abs() > tol {
            self.insert_knot_default(u)?;
            if delta < 0. {
                ik += 1;
            }
        }
        self.set_origin(ik)
    }
    // The reparametrization step of SetOrigin(U, Tol): returns the shifted (uf, ul).
    fn shift_range_to(&mut self, u: f64, tol: f64) -> Result<(f64, f64), Failure> {
        let (mut uf, mut ul) = (self.first_parameter(), self.last_parameter());
        let (mut shifted, period) = (u, ul - uf);
        while tol < (uf - shifted) {
            shifted += period;
        }
        while tol > (ul - shifted) {
            shifted -= period;
        }
        if (u - shifted).abs() > tol {
            self.clear_eval_rep();
            let delta: f64 = u - shifted;
            uf += delta;
            ul += delta;
            for knot in self.knots.as_mut_slice() {
                *knot += delta;
            }
            self.update_knots()?;
        }
        Ok((uf, ul))
    }
    // The first knot closest to `u` and its signed distance (`delta = RealLast()` start).
    fn nearest_knot(&self, u: f64) -> Result<(i32, f64), Failure> {
        let (mut ik, mut delta): (i32, f64) = (0, f64::MAX);
        for i in self.knots.lower()..=self.knots.upper() {
            let dki: f64 = at(&self.knots, i)? - u;
            if dki.abs() < delta.abs() {
                ik = i;
                delta = dki;
            }
        }
        Ok((ik, delta))
    }
    // `InsertKnot(U)` with its default arguments (M = 1, tolerance 0, Add = true).
    fn insert_knot_default(&mut self, u: f64) -> Result<(), Failure> {
        let mut knots: Array1<'static, f64> = one_based(1)?;
        put(&mut knots, 1, u)?;
        let mut mults: Array1<'static, i32> = one_based(1)?;
        put(&mut mults, 1, 1)?;
        self.insert_knots(&knots, &mults, 0.0, true)
    }
    /// `SetNotPeriodic`: unperiodize knots, poles and weights.
    pub(super) fn set_not_periodic(&mut self) -> Result<(), Failure> {
        if !self.periodic {
            return Ok(());
        }
        self.clear_eval_rep();
        let (nb_knots, nb_poles) = bsplclib::prepare_unperiodize(self.deg, &self.mults)?;
        let mut new_poles: Array1<'static, Point3> = one_based(nb_poles)?;
        let (mut new_knots, mut new_mults) =
            (one_based::<f64>(nb_knots)?, one_based::<i32>(nb_knots)?);
        let mut new_weights: Array1<'static, f64> = self.new_weights(nb_poles)?;
        let target_weights = self.rational.then_some(&mut new_weights);
        bsplclib::unperiodize(
            self.deg,
            &self.mults,
            &self.knots,
            &self.poles,
            self.weights_view(),
            &mut new_mults,
            &mut new_knots,
            &mut new_poles,
            target_weights,
        )?;
        self.poles = new_poles;
        self.weights = new_weights;
        if !self.rational {
            self.reset_unit_weights()?;
        }
        (self.mults, self.knots) = (new_mults, new_knots);
        self.periodic = false;
        self.max_deriv_inv_ok = false;
        self.update_knots()
    }
    /// `Resolution`: the cached inverse derivative bound times `tolerance_3d`; a periodic
    /// curve is measured on its unperiodized pole sequence.
    pub(super) fn resolution(&mut self, tolerance_3d: f64) -> Result<f64, Failure> {
        if !self.max_deriv_inv_ok {
            self.max_deriv_inv = if self.periodic {
                self.periodic_resolution()?
            } else {
                let count: i32 = self.poles.len() as i32;
                let weights = self.weights_view();
                bsplclib::resolution(&self.poles, weights, count, &self.flat_knots, self.deg, 1.)?
            };
            self.max_deriv_inv_ok = true;
        }
        Ok(tolerance_3d * self.max_deriv_inv)
    }
    fn periodic_resolution(&self) -> Result<f64, Failure> {
        let (_, nb_poles) = bsplclib::prepare_unperiodize(self.deg, &self.mults)?;
        let mut new_poles: Array1<'static, Point3> = one_based(nb_poles)?;
        let mut new_weights: Array1<'static, f64> = one_based(nb_poles)?;
        let length: i32 = self.poles.len() as i32;
        for ii in 1..=nb_poles {
            put(&mut new_poles, ii, at(&self.poles, (ii - 1) % length + 1)?)?;
        }
        if self.rational {
            for ii in 1..=nb_poles {
                put(
                    &mut new_weights,
                    ii,
                    at(&self.weights, (ii - 1) % length + 1)?,
                )?;
            }
        }
        let weights: Option<&Array1<'_, f64>> = self.rational.then_some(&new_weights);
        bsplclib::resolution(
            &new_poles,
            weights,
            nb_poles,
            &self.flat_knots,
            self.deg,
            1.,
        )
    }
}
