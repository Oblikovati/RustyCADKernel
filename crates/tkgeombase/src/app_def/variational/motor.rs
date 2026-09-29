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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (TheMotor)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TheMotor`: the smoothing engine, alternating curve optimization, criteria
//! estimation, parameter optimization (projection and arc length) and knot
//! insertion.
use super::super::smooth_criterion::SmoothCriterion;
use super::Variational;
use super::motor_state::{
    MotorState, SMALL_VALUE, accept_estimation, accept_iteration, invalidate_new_knots,
    progression, quality_criteria, restore_previous_iteration, save_before_cutting,
};
use super::motor_support::{at, copy_of, curve_of, fresh, put, raise_estimations, stable_sort};
use crate::fem_tool::{Assembly, Curve};
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bvh::std_min;

impl Variational {
    /// `TheMotor(J, WQuadratic, WQuality, TheCurve, Ecarts)`: smooths from `the_curve`
    /// and the current parameters, then stores the best curve in `the_curve`, the
    /// projection parameters in myParameters, the point distances in `ecarts` and
    /// the errors and criteria (the quadratic weight is unused natively).
    pub(super) fn the_motor(
        &mut self,
        j: &dyn SmoothCriterion,
        w_quality: f64,
        the_curve: &mut Handle<Curve>,
        ecarts: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let mut st: MotorState = self.motor_init(j, w_quality, the_curve)?;
        let dependence: Array2<'static, i32> = j.dependence_table()?.array2().clone();
        j.set_curve(&st.c_current)?;
        let mut assembly: Assembly = Assembly::new(&dependence, &j.assembly_table()?)?;
        let mut again: bool = true;
        while again {
            if self.estimation_loop(j, &mut st, &mut assembly, ecarts)? {
                break; // goto L8000
            }
            self.parametric_loop(j, &mut st, &mut assembly, ecarts)?;
            again = self.optional_splitting(j, &mut st, &mut assembly, &dependence)?;
        }
        self.motor_finish(j, st, the_curve, ecarts)
    }

    // (0) Init: the estimations raised to their minima, the working copies of the
    // parameters and the curve.
    fn motor_init(
        &self,
        j: &dyn SmoothCriterion,
        w_quality: f64,
        the_curve: &Handle<Curve>,
    ) -> Result<MotorState, Failure> {
        let j_min: [f64; 3] = raise_estimations(j);
        let current_ti: Handle<HArray1<f64>> = copy_of(&self.parameters)?;
        let cblong: f64 = j.est_length();
        let mut st: MotorState = MotorState::start(the_curve, current_ti, cblong)?;
        (st.j_min, st.w_quality) = (j_min, w_quality);
        Ok(st)
    }

    // A new curve on the knots of the current one, optimized for the current
    // parameters (the (1.1) and (2.3) steps).
    fn optimize_new_curve(
        &self,
        j: &dyn SmoothCriterion,
        st: &mut MotorState,
        a: &mut Assembly,
    ) -> Result<(), Failure> {
        let nbr_pnt: f64 = f64::from(self.last_point - self.first_point + 1);
        st.eps_length = SMALL_VALUE * st.cblong / nbr_pnt;
        let current: &Curve = curve_of(&st.c_current)?;
        let c_new: Curve = Curve::new(
            current.dimension(),
            current.nb_elements(),
            current.base(),
            st.eps_length,
        )?;
        let knots: Array1<'static, f64> = current.knots().clone();
        c_new.knots().assign(&knots);
        st.c_new = Handle::new(c_new);
        j.set_parameters(&st.current_ti);
        let eps_deg: f64 = std_min(st.w_quality * 0.1, st.cblong * 0.001);
        let parameters: Array1<'static, f64> = st.current_ti.array1().clone();
        self.optimization(j, a, (st.lconst, eps_deg), &st.c_new, &parameters)
    }

    // Project(C, CurrentTi, NewTi, Ecarts, NumPnt, ERRMAX, ERRQUA, ERRMOY, iterations)
    // into a new NewTi.
    fn project_new_ti(
        &self,
        curve: &Handle<Curve>,
        st: &mut MotorState,
        ecarts: &mut Array1<'_, f64>,
        iterations: i32,
    ) -> Result<(), Failure> {
        let ti: Array1<'static, f64> = st.current_ti.array1().clone();
        st.new_ti = fresh(ti.length())?;
        let mut proj: Array1<'static, f64> = st.new_ti.array1().clone();
        st.errors = self.project(
            curve_of(curve)?,
            (&ti, &mut proj),
            ecarts,
            st.errors.num_points,
            iterations,
        )?;
        st.new_ti.change_array1().assign(&proj);
        Ok(())
    }

    // The arc-length reparametrization of the (2.2) and (1.7) steps.
    fn arc_length_step(&self, j: &dyn SmoothCriterion, st: &mut MotorState) -> Result<(), Failure> {
        stable_sort(&st.current_ti);
        let decima: i32 = 4;
        let current: &Curve = curve_of(&st.c_current)?;
        st.cblong = current.length(0., 1.)?;
        j.set_est_length(st.cblong);
        let mut ti = st.current_ti.change_array1();
        self.acr(current, &mut ti, decima)?;
        st.lconst = true;
        Ok(())
    }

    /// (1) Loop Optimization / Estimation; true for the native `goto L8000` when the
    /// first estimation brings too little progress.
    fn estimation_loop(
        &self,
        j: &dyn SmoothCriterion,
        st: &mut MotorState,
        a: &mut Assembly,
        ecarts: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        let (mut lestim, mut nb_est) = (true, 0);
        st.lconst = true;
        j.set_curve(&st.c_current)?;
        while lestim {
            let (iprog, icdana) = self.estimate_once(j, st, a, ecarts)?;
            if iprog < 2 && nb_est == 0 {
                // (1.5) Invalidation of new knots.
                invalidate_new_knots(st);
                return Ok(true);
            }
            accept_estimation(st);
            // (1.6) Test if the Estimations seems OK, else repeat
            nb_est += 1;
            lestim = nb_est < 5 && icdana == 2 && iprog > 0;
            if lestim && st.isnear {
                // (1.7) Optimization of ti by ACR.
                self.arc_length_step(j, st)?;
            }
        }
        Ok(false)
    }

    // The (1.1) to (1.4) steps of one estimation: returns the progression IPROG and
    // the ICDANA of the quality criteria.
    fn estimate_once(
        &self,
        j: &dyn SmoothCriterion,
        st: &mut MotorState,
        a: &mut Assembly,
        ecarts: &mut Array1<'_, f64>,
    ) -> Result<(i32, i32), Failure> {
        // (1.1) Curve's Optimization.
        self.optimize_new_curve(j, st, a)?;
        st.lconst = false;
        // (1.2) calculation of quality criteria and improvement of estimation.
        let icdana: i32 = self.estimation_quality(j, st)?;
        // (1.3) Optimization of ti by orthogonal projection.
        if st.isnear {
            let c_new: Handle<Curve> = st.c_new.clone();
            self.project_new_ti(&c_new, st, ecarts, 2)?;
        } else {
            st.new_ti = st.current_ti.clone();
        }
        // (1.4) Progression's test
        Ok((progression(st), icdana))
    }

    // (1.2): the quality criteria, the errors and whether the curve is near the
    // points; returns ICDANA.
    fn estimation_quality(
        &self,
        j: &dyn SmoothCriterion,
        st: &mut MotorState,
    ) -> Result<i32, Failure> {
        let nbr_pnt: i32 = self.last_point - self.first_point + 1;
        let icdana: i32 = quality_criteria(j, st)?;
        let errors = j.error_values()?;
        (st.errors.max_err, st.errors.qua_err, st.errors.ave_err) = (
            errors.max_error,
            errors.quadratic_error,
            errors.average_error,
        );
        st.isnear = (st.errors.qua_err / f64::from(nbr_pnt)).sqrt() < 2. * st.w_quality
            && self.options.nb_iterations > 1;
        Ok(icdana)
    }

    // (2.1) Save current results.
    fn save_state(&self, st: &mut MotorState) -> Result<(), Failure> {
        st.vocri = st.valcri;
        st.erold = st.errors.max_err;
        st.lnold = st.cblong;
        st.c_old = st.c_current.clone();
        let values: Array1<'static, f64> = st.current_ti.array1().clone();
        st.old_ti.change_array1().assign(&values);
        Ok(())
    }

    /// (2) loop of parametric / geometric optimization.
    fn parametric_loop(
        &self,
        j: &dyn SmoothCriterion,
        st: &mut MotorState,
        a: &mut Assembly,
        ecarts: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let mut iter: i32 = 1;
        let mut to_optim: bool = iter < self.options.nb_iterations && st.isnear;
        while to_optim {
            iter += 1;
            to_optim = self.parametric_iteration(j, st, a, ecarts)?;
            if iter >= self.options.nb_iterations {
                to_optim = false;
            }
        }
        Ok(())
    }

    // The (2.1) to (2.7) steps of one parametric iteration; false when it regressed
    // and the previous iteration was restored (the loop stops).
    fn parametric_iteration(
        &self,
        j: &dyn SmoothCriterion,
        st: &mut MotorState,
        a: &mut Assembly,
        ecarts: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        self.save_state(st)?;
        // (2.2) Optimization of ti by ACR.
        self.arc_length_step(j, st)?;
        // (2.3) Optimization of curves
        self.optimize_new_curve(j, st, a)?;
        st.c_current = st.c_new.clone();
        // (2.4) calculation of quality criteria and improvement of estimation.
        quality_criteria(j, st)?;
        // (2.5) Optimization of ti by orthogonal projection
        let c_current: Handle<Curve> = st.c_current.clone();
        self.project_new_ti(&c_current, st, ecarts, 2)?;
        // (2.6) Test de non regression
        if self.regression(j, st) >= 2 {
            restore_previous_iteration(st);
            return Ok(false);
        }
        accept_iteration(st);
        Ok(true)
    }

    // The regression count of (2.6).
    fn regression(&self, j: &dyn SmoothCriterion, st: &MotorState) -> i32 {
        let nbr_pnt: i32 = self.last_point - self.first_point + 1;
        let (errmax, erold, wq) = (st.errors.max_err, st.erold, st.w_quality);
        let mut iregre: i32 = 0;
        if self.tables.nb_constr() < nbr_pnt {
            iregre += i32::from(errmax > wq && errmax > 1.05 * erold);
            iregre += i32::from(errmax > wq && errmax > 2. * erold);
            iregre -= i32::from(erold > wq && errmax <= 0.5 * erold);
        }
        let estimation: [f64; 3] = j.get_estimation();
        for ((valcri, vocri), estimation) in st.valcri.iter().zip(st.vocri).zip(estimation) {
            iregre += i32::from(*valcri > estimation && *valcri > 1.1 * vocri);
        }
        iregre
    }

    /// (3) Optional splitting: new knots when below MaxSegment with cutting;
    /// true when the curve was cut (the smoothing starts again on it).
    fn optional_splitting(
        &self,
        j: &dyn SmoothCriterion,
        st: &mut MotorState,
        a: &mut Assembly,
        dependence: &Array2<'_, i32>,
    ) -> Result<bool, Failure> {
        if curve_of(&st.c_current)?.nb_elements() >= self.options.max_segment
            || !self.options.with_cutting
        {
            return Ok(false);
        }
        // (3.1) Sauvgarde de l'etat precedent
        save_before_cutting(st);
        // (3.2) Arrange the ti: Sort + rescale to (0,1)
        stable_sort(&st.current_ti);
        self.rescale_ti(&st.current_ti)?;
        self.insert_new_knots(j, st, a, dependence)
    }

    // (3.3) Insert new Knots; true when the curve was cut (its new assembly built).
    fn insert_new_knots(
        &self,
        j: &dyn SmoothCriterion,
        st: &mut MotorState,
        a: &mut Assembly,
        dependence: &Array2<'_, i32>,
    ) -> Result<bool, Failure> {
        let ti: Array1<'static, f64> = st.current_ti.array1().clone();
        let Some(c_new) = self.split_curve(curve_of(&st.c_current)?, &ti, st.eps_length)? else {
            return Ok(false);
        };
        st.c_new = c_new;
        st.c_current = st.c_new.clone();
        // New Knots => New Assembly.
        j.set_curve(&st.c_new)?;
        *a = Assembly::new(dependence, &j.assembly_table()?)?;
        Ok(true)
    }

    // The sorted parameters mapped onto [0, 1] unless they already span it.
    fn rescale_ti(&self, ti: &Handle<HArray1<f64>>) -> Result<(), Failure> {
        let nbr_pnt: i32 = self.last_point - self.first_point + 1;
        if at(ti, 1)? == 0. && at(ti, nbr_pnt)? == 1. {
            return Ok(());
        }
        let delat_t: f64 = 1.0 / (at(ti, nbr_pnt)? - at(ti, 1)?);
        for ii in 2..nbr_pnt {
            let t: f64 = (at(ti, ii)? - at(ti, 1)?) * delat_t;
            put(ti, ii, t)?;
        }
        put(ti, 1, 0.)?;
        put(ti, nbr_pnt, 1.)
    }

    // (4) Compute the best Error, (5) field's update.
    fn motor_finish(
        &mut self,
        j: &dyn SmoothCriterion,
        mut st: MotorState,
        the_curve: &mut Handle<Curve>,
        ecarts: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let c_current: Handle<Curve> = st.c_current.clone();
        self.project_new_ti(&c_current, &mut st, ecarts, 10)?;
        *the_curve = c_current;
        j.set_est_length(st.cblong);
        let values: Array1<'static, f64> = st.new_ti.array1().clone();
        self.parameters.change_array1().assign(&values);
        self.store_motor_results(&st);
        Ok(())
    }

    // (5) The criteria, the maximal error and its point, and the average error over
    // the unconstrained points (over the constraints when every point is one).
    fn store_motor_results(&mut self, st: &MotorState) {
        let r = &mut self.results;
        r.criterium = [
            st.errors.qua_err,
            st.valcri[0].sqrt(),
            st.valcri[1].sqrt(),
            st.valcri[2].sqrt(),
        ];
        r.max_error = st.errors.max_err;
        r.max_error_index = st.errors.num_points;
        let (nbr_pnt, nbr_constraint) = (
            self.last_point - self.first_point + 1,
            self.tables.nb_constr(),
        );
        r.average_error = if nbr_pnt > nbr_constraint {
            st.errors.ave_err / f64::from(nbr_pnt - nbr_constraint)
        } else {
            st.errors.ave_err / f64::from(nbr_constraint)
        };
    }
}
