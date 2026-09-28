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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineCurve.cxx (degree, knots and multiplicities) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Degree elevation, knot insertion and removal, knot setters and `LocateU`.
use super::state::{BSplineState, check_curve_data};
use crate::geom::errors;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, real};
use rustycad_tkg2d::geom2d::arrays::{at, check_one_based, one_based, put};
use rustycad_tkg2d::geom2d::spline_data::{bounded, locate_u_in};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::Point3;

impl BSplineState {
    /// `IncreaseDegree`; the evaluation representation is cleared before the degree
    /// check, so a rejected degree clears it too, as natively.
    pub(super) fn increase_degree(&mut self, degree: i32) -> Result<(), Failure> {
        if degree == self.deg {
            return Ok(());
        }
        self.clear_eval_rep();
        if degree < self.deg || degree > bsplclib::max_degree() {
            return Err(errors::construction(&format!(
                "BSpline curve: IncreaseDegree: bad degree value {degree}; expected in [{}, {}]",
                self.deg,
                bsplclib::max_degree()
            )));
        }
        let (from_k1, to_k2) = (self.first_u_knot_index()?, self.last_u_knot_index()?);
        let step: i32 = degree - self.deg;
        let nbpoles: i32 = self.poles.len() as i32 + step * (to_k2 - from_k1);
        let mut new_poles: Array1<'static, Point3> = one_based(nbpoles)?;
        let nbknots: i32 =
            bsplclib::increase_degree_count_knots(self.deg, degree, self.periodic, &self.mults)?;
        let (mut new_knots, mut new_mults) =
            (one_based::<f64>(nbknots)?, one_based::<i32>(nbknots)?);
        let mut new_weights: Array1<'static, f64> = self.new_weights(nbpoles)?;
        let target_weights = self.rational.then_some(&mut new_weights);
        bsplclib::increase_degree(
            self.deg,
            degree,
            self.periodic,
            &self.poles,
            self.weights_view(),
            &self.knots,
            &self.mults,
            &mut new_poles,
            target_weights,
            &mut new_knots,
            &mut new_mults,
        )?;
        self.deg = degree;
        self.poles = new_poles;
        self.weights = new_weights;
        if !self.rational {
            self.reset_unit_weights()?;
        }
        self.knots = new_knots;
        self.mults = new_mults;
        self.update_knots()
    }
    /// `IncreaseMultiplicity(Index, M)`.
    pub(super) fn increase_multiplicity(&mut self, index: i32, mult: i32) -> Result<(), Failure> {
        let mut knots: Array1<'static, f64> = one_based(1)?;
        put(&mut knots, 1, at(&self.knots, index)?)?;
        let mut mults: Array1<'static, i32> = one_based(1)?;
        put(&mut mults, 1, mult - at(&self.mults, index)?)?;
        self.insert_knots(&knots, &mults, real::epsilon(1.), true)
    }
    /// `IncreaseMultiplicity(I1, I2, M)` (`step` false) and `IncrementMultiplicity(I1, I2, Step)`
    /// (`step` true), on knots `i1..=i2`.
    pub(super) fn raise_multiplicities(
        &mut self,
        i1: i32,
        i2: i32,
        value: i32,
        step: bool,
    ) -> Result<(), Failure> {
        // Native `NCollection_Array1<double> k(myKnots(I1), I1, I2)` views the stored
        // knots from myKnots(I1), whose access is checked even for an empty range;
        // reading past myKnots is undefined natively and Standard_OutOfRange here.
        at(&self.knots, i1)?;
        let mut knots: Array1<'static, f64> = bounded(i1, i2)?;
        let mut mults: Array1<'static, i32> = bounded(i1, i2)?;
        for i in i1..=i2 {
            put(&mut knots, i, at(&self.knots, i)?)?;
            let added: i32 = if step {
                value
            } else {
                value - at(&self.mults, i)?
            };
            put(&mut mults, i, added)?;
        }
        self.insert_knots(&knots, &mults, real::epsilon(1.), true)
    }
    /// `InsertKnots`: Standard_ConstructionError when BSplCLib rejects the knots; the
    /// evaluation representation is cleared only when poles are added.
    pub(super) fn insert_knots(
        &mut self,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        epsilon: f64,
        add: bool,
    ) -> Result<(), Failure> {
        let (mut nbpoles, mut nbknots): (i32, i32) = (0, 0);
        let (deg, periodic) = (self.deg, self.periodic);
        let accepted: bool = bsplclib::prepare_insert_knots(
            deg,
            periodic,
            &self.knots,
            &self.mults,
            knots,
            Some(mults),
            &mut nbpoles,
            &mut nbknots,
            epsilon,
            add,
        )?;
        if !accepted {
            return Err(errors::construction(&format!(
                "Geom_BSplineCurve::InsertKnots: {} knots rejected by BSplCLib::PrepareInsertKnots \
                 (tolerance {epsilon}); expected knots inside the curve range with valid multiplicities",
                knots.len()
            )));
        }
        if nbpoles == self.poles.len() as i32 {
            return Ok(());
        }
        self.clear_eval_rep();
        let mut new_poles: Array1<'static, Point3> = one_based(nbpoles)?;
        let (mut new_knots, mut new_mults) =
            (one_based::<f64>(nbknots)?, one_based::<i32>(nbknots)?);
        let mut new_weights: Array1<'static, f64> = self.new_weights(nbpoles)?;
        let target_weights = self.rational.then_some(&mut new_weights);
        bsplclib::insert_knots(
            deg,
            periodic,
            &self.poles,
            self.weights_view(),
            &self.knots,
            &self.mults,
            knots,
            Some(mults),
            &mut new_poles,
            target_weights,
            &mut new_knots,
            &mut new_mults,
            epsilon,
            add,
        )?;
        self.weights = new_weights;
        self.poles = new_poles;
        if !self.rational {
            self.reset_unit_weights()?;
        }
        self.knots = new_knots;
        self.mults = new_mults;
        self.update_knots()
    }
    // The index check of RemoveKnot: strictly interior for a non-periodic curve.
    fn check_removable_index(&self, index: i32) -> Result<(), Failure> {
        let (i1, i2) = (self.first_u_knot_index()?, self.last_u_knot_index()?);
        let outside: bool = if self.periodic {
            index < i1 || index > i2
        } else {
            index <= i1 || index >= i2
        };
        if outside {
            return Err(errors::out_of_range(&format!(
                "BSpline curve: RemoveKnot: index {index} out of range; expected in \
                 ({i1}, {i2}) or [{i1}, {i2}] when periodic"
            )));
        }
        Ok(())
    }
    /// `RemoveKnot`: false when the knot cannot be removed within `tolerance`.
    pub(super) fn remove_knot(
        &mut self,
        index: i32,
        mult: i32,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        if mult < 0 {
            return Ok(true);
        }
        self.check_removable_index(index)?;
        let step: i32 = at(&self.mults, index)? - mult;
        if step <= 0 {
            return Ok(true);
        }
        let removed: i32 = i32::from(mult == 0);
        let nbpoles: i32 = self.poles.len() as i32 - step;
        let mut new_poles: Array1<'static, Point3> = one_based(nbpoles)?;
        let mut new_knots: Array1<'static, f64> = one_based(self.knots.len() as i32 - removed)?;
        let mut new_mults: Array1<'static, i32> = one_based(self.mults.len() as i32 - removed)?;
        let mut new_weights: Array1<'static, f64> = self.new_weights(nbpoles)?;
        let target_weights = self.rational.then_some(&mut new_weights);
        let removable: bool = bsplclib::remove_knot(
            index,
            mult,
            self.deg,
            self.periodic,
            &self.poles,
            self.weights_view(),
            &self.knots,
            &self.mults,
            &mut new_poles,
            target_weights,
            &mut new_knots,
            &mut new_mults,
            tolerance,
        )?;
        if !removable {
            return Ok(false);
        }
        self.clear_eval_rep();
        self.commit_removal(new_poles, new_weights, new_knots, new_mults)?;
        Ok(true)
    }
    fn commit_removal(
        &mut self,
        poles: Array1<'static, Point3>,
        weights: Array1<'static, f64>,
        knots: Array1<'static, f64>,
        mults: Array1<'static, i32>,
    ) -> Result<(), Failure> {
        self.weights = weights;
        self.poles = poles;
        if !self.rational {
            self.reset_unit_weights()?;
        }
        self.knots = knots;
        self.mults = mults;
        self.update_knots()?;
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `SetKnot(Index, K)`: the knot must stay strictly between its neighbours.
    pub(super) fn set_knot(&mut self, index: i32, knot: f64) -> Result<(), Failure> {
        let nbknots: i32 = self.knots.len() as i32;
        check_one_based(index, self.knots.len(), "BSpline curve: SetKnot")?;
        let dk: f64 = real::epsilon(knot).abs();
        let below: bool = index > 1 && knot <= at(&self.knots, index - 1)? + dk;
        let above: bool = index < nbknots && knot >= at(&self.knots, index + 1)? - dk;
        if below || above {
            return Err(errors::construction(&format!(
                "BSpline curve: SetKnot: K = {knot} out of range at Index = {index}; \
                 expected strictly between the neighbouring knots"
            )));
        }
        if knot != at(&self.knots, index)? {
            self.clear_eval_rep();
            put(&mut self.knots, index, knot)?;
            self.max_deriv_inv_ok = false;
            self.update_knots()?;
        }
        Ok(())
    }
    /// `SetKnots`: checked like the constructor, then copied in place.
    pub(super) fn set_knots(&mut self, knots: &Array1<'_, f64>) -> Result<(), Failure> {
        check_curve_data(&self.poles, knots, &self.mults, self.deg, self.periodic)?;
        self.clear_eval_rep();
        self.knots
            .copy_values(knots)
            .map_err(errors::array_bounds)?;
        self.max_deriv_inv_ok = false;
        self.update_knots()
    }
    /// `LocateU`: the knot interval `(I1, I2)` of `u`, with flat knots when `with_repetition`.
    pub(super) fn locate_u(
        &self,
        u: f64,
        parametric_tolerance: f64,
        with_repetition: bool,
    ) -> Result<(i32, i32), Failure> {
        let knots: &Array1<'static, f64> = if with_repetition {
            &self.flat_knots
        } else {
            &self.knots
        };
        let new_u: f64 = self.periodic_normalization(u)?;
        locate_u_in(knots, new_u, parametric_tolerance)
    }
}
