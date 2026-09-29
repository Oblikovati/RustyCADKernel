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
use super::project::ProjectErrors;
use crate::fem_tool::{Assembly, Curve};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bvh::std_min;

const BIG_VALUE: f64 = 1.0e37;
const SMALL_VALUE: f64 = 1.0e-6;
const SMALLEST_VALUE: f64 = 1.0e-9;

/// The local variables of TheMotor that live across its phases. The native
/// `VALCRI`, `ERRQUA`, `ERRMOY` and `NumPnt` start uninitialized; the port's at 0.
#[derive(Default)]
pub(super) struct MotorState {
    vocri: [f64; 3],
    valcri: [f64; 3],
    erold: f64,
    errors: ProjectErrors,
    cblong: f64,
    lnold: f64,
    c_current: Handle<Curve>,
    c_old: Handle<Curve>,
    c_new: Handle<Curve>,
    current_ti: Handle<HArray1<f64>>,
    new_ti: Handle<HArray1<f64>>,
    old_ti: Handle<HArray1<f64>>,
    eps_length: f64,
    isnear: bool,
    lconst: bool,
    j_min: [f64; 3],
    w_quality: f64,
}

// A new HArray1(1, n) taking the values and the bounds of `source` (the native
// `new HArray1(1, n)` followed by an Array1 assignment).
fn copy_of(source: &Handle<HArray1<f64>>) -> Result<Handle<HArray1<f64>>, Failure> {
    let values: Array1<'static, f64> = source.array1().clone();
    let copy: HArray1<f64> = HArray1::with_bounds(1, values.length())
        .map_err(|error| array_failure(error, "AppDef_Variational::TheMotor"))?;
    copy.change_array1().assign(&values);
    Ok(Handle::new(copy))
}

fn fresh(length: i32) -> Result<Handle<HArray1<f64>>, Failure> {
    let values: HArray1<f64> = HArray1::with_bounds(1, length)
        .map_err(|error| array_failure(error, "AppDef_Variational::TheMotor"))?;
    Ok(Handle::new(values))
}

// std::stable_sort with operator<.
fn stable_sort(ti: &Handle<HArray1<f64>>) {
    ti.change_array1().as_mut_slice().sort_by(|a, b| {
        if a < b {
            std::cmp::Ordering::Less
        } else if b < a {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });
}

// Like in MOTLIS: the estimations of J raised to their minima; returns the minima
// `[J1min, J2min, J2min]`.
fn raise_estimations(j: &dyn SmoothCriterion) -> [f64; 3] {
    let [mut e1, mut e2, mut e3] = j.get_estimation();
    let j1min: f64 = 1.0e-8;
    let j2min: f64 = (e1 + 1.0e-8) * 1.0e-6;
    if e1 < j1min {
        e1 = j1min;
    }
    if e2 < j2min {
        e2 = j2min;
    }
    if e3 < j2min {
        e3 = j2min;
    }
    j.set_estimation(e1, e2, e3);
    [j1min, j2min, j2min]
}

fn curve_of(curve: &Handle<Curve>) -> Result<&Curve, Failure> {
    curve
        .get()
        .ok_or_else(|| Failure::new("AppDef_Variational::TheMotor: null curve"))
}

fn at(ti: &Handle<HArray1<f64>>, index: i32) -> Result<f64, Failure> {
    ti.array1()
        .value(index)
        .copied()
        .map_err(|error| array_failure(error, "AppDef_Variational::TheMotor"))
}

fn put(ti: &Handle<HArray1<f64>>, index: i32, value: f64) -> Result<(), Failure> {
    ti.change_array1()
        .set_value(index, value)
        .map_err(|error| array_failure(error, "AppDef_Variational::TheMotor"))
}

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
        // The fields left to the default: VALCRI at 0, no new curve or parameters yet,
        // and not near.
        Ok(MotorState {
            vocri: [BIG_VALUE; 3],
            erold: BIG_VALUE,
            errors: ProjectErrors {
                max_err: BIG_VALUE,
                ..ProjectErrors::default()
            },
            cblong,
            lnold: cblong,
            c_current: the_curve.clone(),
            c_old: the_curve.clone(),
            old_ti: copy_of(&current_ti)?,
            current_ti,
            eps_length: SMALL_VALUE,
            lconst: true,
            j_min,
            w_quality,
            ..MotorState::default()
        })
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
        let nbr_pnt: i32 = self.last_point - self.first_point + 1;
        let (mut lestim, mut nb_est) = (true, 0);
        st.lconst = true;
        j.set_curve(&st.c_current)?;
        while lestim {
            // (1.1) Curve's Optimization.
            self.optimize_new_curve(j, st, a)?;
            st.lconst = false;
            // (1.2) calculation of quality criteria and improvement of estimation.
            let quality = j.quality_values(st.j_min[0], st.j_min[1], st.j_min[2])?;
            st.valcri = quality.criteria;
            if quality.icdana > 0 {
                st.lconst = true;
            }
            let errors = j.error_values()?;
            (st.errors.max_err, st.errors.qua_err, st.errors.ave_err) = (
                errors.max_error,
                errors.quadratic_error,
                errors.average_error,
            );
            st.isnear = (st.errors.qua_err / f64::from(nbr_pnt)).sqrt() < 2. * st.w_quality
                && self.options.nb_iterations > 1;
            // (1.3) Optimization of ti by orthogonal projection.
            if st.isnear {
                let c_new: Handle<Curve> = st.c_new.clone();
                self.project_new_ti(&c_new, st, ecarts, 2)?;
            } else {
                st.new_ti = st.current_ti.clone();
            }
            // (1.4) Progression's test
            let iprog: i32 = progression(st);
            if iprog < 2 && nb_est == 0 {
                // (1.5) Invalidation of new knots.
                st.valcri = st.vocri;
                st.errors.max_err = st.erold;
                st.cblong = st.lnold;
                st.c_current = st.c_old.clone();
                st.current_ti = st.old_ti.clone();
                return Ok(true);
            }
            (st.vocri, st.lnold, st.erold) = (st.valcri, st.cblong, st.errors.max_err);
            st.c_current = st.c_new.clone();
            st.current_ti = st.new_ti.clone();
            // (1.6) Test if the Estimations seems OK, else repeat
            nb_est += 1;
            lestim = nb_est < 5 && quality.icdana == 2 && iprog > 0;
            if lestim && st.isnear {
                // (1.7) Optimization of ti by ACR.
                self.arc_length_step(j, st)?;
            }
        }
        Ok(false)
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
            self.save_state(st)?;
            // (2.2) Optimization of ti by ACR.
            self.arc_length_step(j, st)?;
            // (2.3) Optimization of curves
            self.optimize_new_curve(j, st, a)?;
            st.c_current = st.c_new.clone();
            // (2.4) calculation of quality criteria and improvement of estimation.
            let quality = j.quality_values(st.j_min[0], st.j_min[1], st.j_min[2])?;
            st.valcri = quality.criteria;
            if quality.icdana > 0 {
                st.lconst = true;
            }
            // (2.5) Optimization of ti by orthogonal projection
            let c_current: Handle<Curve> = st.c_current.clone();
            self.project_new_ti(&c_current, st, ecarts, 2)?;
            // (2.6) Test de non regression
            if self.regression(j, st) >= 2 {
                // (2.7) restore the previous iteration
                st.valcri = st.vocri;
                st.errors.max_err = st.erold;
                st.cblong = st.lnold;
                st.c_current = st.c_old.clone();
                let values: Array1<'static, f64> = st.old_ti.array1().clone();
                st.current_ti.change_array1().assign(&values);
                to_optim = false;
            } else {
                // Iteration is Ok.
                st.c_current = st.c_new.clone();
                st.current_ti = st.new_ti.clone();
            }
            if iter >= self.options.nb_iterations {
                to_optim = false;
            }
        }
        Ok(())
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
        st.vocri = st.valcri;
        st.erold = st.errors.max_err;
        st.c_old = st.c_current.clone();
        let values: Array1<'static, f64> = st.current_ti.array1().clone();
        st.old_ti.change_array1().assign(&values);
        // (3.2) Arrange the ti: Sort + rescale to (0,1)
        stable_sort(&st.current_ti);
        self.rescale_ti(&st.current_ti)?;
        // (3.3) Insert new Knots
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
        Ok(())
    }
}

// (1.4) Progression's test.
fn progression(st: &MotorState) -> i32 {
    let (erold, errmax, wq) = (st.erold, st.errors.max_err, st.w_quality);
    let (valcri, vocri) = (st.valcri, st.vocri);
    let mut iprog: i32 = 0;
    iprog += i32::from(erold > wq && errmax < 0.95 * erold);
    iprog += i32::from(erold > wq && errmax < 0.8 * erold);
    iprog += i32::from(erold > wq && errmax < wq);
    iprog += i32::from(erold > wq && errmax < 0.99 * erold && errmax < 1.1 * wq);
    iprog += i32::from(valcri[0] < 0.975 * vocri[0]);
    iprog += i32::from(valcri[0] < 0.9 * vocri[0]);
    iprog += i32::from(valcri[1] < 0.95 * vocri[1]);
    iprog += i32::from(valcri[1] < 0.8 * vocri[1]);
    iprog += i32::from(valcri[2] < 0.95 * vocri[2]);
    iprog += i32::from(valcri[2] < 0.8 * vocri[2]);
    if vocri[1] > SMALLEST_VALUE
        && vocri[2] > SMALLEST_VALUE
        && valcri[1] / vocri[1] + 2. * valcri[2] / vocri[2] < 2.8
    {
        iprog += 1;
    }
    iprog
}
