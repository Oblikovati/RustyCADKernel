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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (Adjusting)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adjusting`: the smoothing repeated with heavier least squares and point
//! weights, and forced knots, until the maximum error is within the tolerance.
use super::super::linear_criteria::LinearCriteria;
use super::super::smooth_criterion::SmoothCriterion;
use super::cutting::near_index;
use super::{Variational, VariationalResults, criterion_ref};
use crate::fem_tool::Curve;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};

fn at(array: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    array
        .value(index)
        .copied()
        .map_err(|error| array_failure(error, "AppDef_Variational::Adjusting"))
}

fn put(array: &mut Array1<'_, f64>, index: i32, value: f64) -> Result<(), Failure> {
    array
        .set_value(index, value)
        .map_err(|error| array_failure(error, "AppDef_Variational::Adjusting"))
}

/// The results of the previous smoothing that a rejected pass restores.
struct Previous {
    vocri: [f64; 4],
    erold: f64,
    emold: f64,
}

impl Previous {
    // (1) The criteria, the maximal and the average error of `results`.
    fn of(results: &VariationalResults) -> Self {
        Self {
            vocri: results.criterium,
            erold: results.max_error,
            emold: results.average_error,
        }
    }

    // The saved criteria and errors put back into `results`.
    fn restore(self, results: &mut VariationalResults) {
        results.criterium = self.vocri;
        results.max_error = self.erold;
        results.average_error = self.emold;
    }
}

/// The state Adjusting carries across its passes: the tolerance of the new
/// curves, the point weights (`TbPoid`), the tension target (`J1cibl`) and the
/// pass count.
struct AdjustingRun {
    curv_tol: f64,
    tbpoid: Array1<'static, f64>,
    j1cibl: f64,
    iter: i32,
}

// (2) The quadratic weight scaled by J1cibl / (0.9 WQuality) when J1cibl is above
// 0.9 WQuality.
fn raise_quadratic_weight(w_quadratic: &mut f64, j1cibl: f64, w_quality: f64) {
    let vtest: f64 = w_quality * 0.9;
    if j1cibl > vtest {
        *w_quadratic *= j1cibl / vtest;
    }
}

// The curve of `handle`, refused when null.
fn curve_of(handle: &Handle<Curve>) -> Result<&Curve, Failure> {
    handle
        .get()
        .ok_or_else(|| Failure::new("AppDef_Variational::Adjusting: null curve"))
}

impl Variational {
    /// `Adjusting(J, WQuadratic, WQuality, TheCurve, Ecarts)`: at most two passes
    /// (more while the curve can still be cut) of the smoothing with a quadratic
    /// weight raised to the target error, the weights of the points beyond 0.9
    /// WQuality raised, and a knot forced near the worst point; a pass is kept (its
    /// criterion and curve replace `j` and `the_curve`) unless it worsens the error or
    /// the tension, in which case the previous errors and criteria are restored (the
    /// parameters, the distances and the worst point index of the rejected pass stay).
    pub(super) fn adjusting(
        &mut self,
        j: &mut Handle<dyn SmoothCriterion>,
        (w_quadratic, w_quality): (&mut f64, f64),
        the_curve: &mut Handle<Curve>,
        ecarts: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let nbr_pnt: i32 = self.last_point - self.first_point + 1;
        let mut run = AdjustingRun {
            curv_tol: 1.0e-6 * criterion_ref(j)?.est_length() / f64::from(nbr_pnt),
            tbpoid: self.unit_point_weights()?,
            j1cibl: self.tension_target(),
            iter: 0,
        };
        let mut loptim: bool = true;
        while loptim {
            loptim =
                self.adjusting_pass(j, (w_quadratic, w_quality), the_curve, ecarts, &mut run)?;
        }
        Ok(())
    }

    // One pass (1) to (7) of Adjusting; false when the loop stops (the pass was
    // rejected, or the error is reached, or the curve cannot be cut any more).
    fn adjusting_pass(
        &mut self,
        j: &mut Handle<dyn SmoothCriterion>,
        (w_quadratic, w_quality): (&mut f64, f64),
        the_curve: &mut Handle<Curve>,
        ecarts: &mut Array1<'_, f64>,
        run: &mut AdjustingRun,
    ) -> Result<bool, Failure> {
        run.iter += 1;
        // (1) Save the previous state
        let previous: Previous = Previous::of(&self.results);
        // (2) Increase the least squares weight
        raise_quadratic_weight(w_quadratic, run.j1cibl, w_quality);
        // (3) Increase the weight associated with problematic points
        self.raise_point_weights(&mut run.tbpoid, ecarts, w_quality)?;
        // (4) Forced splitting
        let mut c_new: Handle<Curve> = self.forced_split(curve_of(the_curve)?, run.curv_tol)?;
        let j_new = self.adjusted_criterion(j, (*w_quadratic, w_quality), &run.tbpoid, &c_new)?;
        // (5) Relissage
        self.the_motor(criterion_ref(&j_new)?, w_quality, &mut c_new, ecarts)?;
        // (6) Tests de rejet
        run.j1cibl = self.tension_target();
        let kept: bool =
            self.keep_or_reject(previous, w_quality, (j, j_new), (the_curve, &c_new))?;
        // (7) Test de convergence
        Ok(kept && !self.adjusting_converged(run.iter, &c_new, w_quality))
    }

    // The weight of every point of the line: 1.
    fn unit_point_weights(&self) -> Result<Array1<'static, f64>, Failure> {
        let mut tbpoid: Array1<'static, f64> =
            Array1::with_bounds(self.first_point, self.last_point)
                .map_err(|error| array_failure(error, "AppDef_Variational::Adjusting"))?;
        tbpoid.init(1.);
        Ok(tbpoid)
    }

    // J1cibl: the root of the quadratic criterion per unconstrained point.
    fn tension_target(&self) -> f64 {
        let nbr_pnt: i32 = self.last_point - self.first_point + 1;
        let nbr_constraint: i32 = self.tables.nb_constr();
        (self.results.criterium[0] / f64::from(nbr_pnt - nbr_constraint)).sqrt()
    }

    // (3) The weights of the points farther than 0.9 WQuality raised by their excess
    // over 0.88 WQuality.
    fn raise_point_weights(
        &self,
        tbpoid: &mut Array1<'_, f64>,
        ecarts: &Array1<'_, f64>,
        w_quality: f64,
    ) -> Result<(), Failure> {
        let (vtest, vseuil) = (w_quality * 0.9, w_quality * 0.88);
        for ipnt in self.first_point..=self.last_point {
            if at(ecarts, ipnt)? > vtest {
                let ecart: f64 = (at(ecarts, ipnt)? - vseuil) / w_quality;
                let weight: f64 = (ecart * 3. + 1.) * at(tbpoid, ipnt)?;
                put(tbpoid, ipnt, weight)?;
            }
        }
        Ok(())
    }

    // (6) The pass is rejected when it worsens the error or the tension (the
    // previous errors and criteria restored), else its criterion and curve are kept.
    fn keep_or_reject(
        &mut self,
        previous: Previous,
        w_quality: f64,
        (j, j_new): (
            &mut Handle<dyn SmoothCriterion>,
            Handle<dyn SmoothCriterion>,
        ),
        (the_curve, c_new): (&mut Handle<Curve>, &Handle<Curve>),
    ) -> Result<bool, Failure> {
        let vseuil: f64 = previous.vocri[1].sqrt() + (previous.erold - self.results.max_error) * 4.;
        let max_error: f64 = self.results.max_error;
        let lrejet: bool = (max_error > w_quality && max_error > previous.erold * 1.01)
            || self.results.criterium[1].sqrt() > vseuil * 1.05;
        if lrejet {
            previous.restore(&mut self.results);
            return Ok(false);
        }
        *j = j_new;
        *the_curve = c_new.clone();
        criterion_ref(j)?.set_curve(the_curve)?;
        Ok(true)
    }

    // (7) Converged: the error is within WQuality, or after MXITER passes the curve
    // has MaxSegment elements.
    fn adjusting_converged(&self, iter: i32, c_new: &Handle<Curve>, w_quality: f64) -> bool {
        let mxiter: i32 = 2;
        let nb_new: i32 = c_new.get().map_or(0, Curve::nb_elements);
        (iter >= mxiter && self.options.max_segment == nb_new) || self.results.max_error < w_quality
    }

    // A curve with a knot inserted near the parameter of the worst point, halfway to
    // the middle of its element, when the curve can still be cut; else a copy of the
    // knots.
    fn forced_split(&self, curve: &Curve, curv_tol: f64) -> Result<Handle<Curve>, Failure> {
        let knots: Array1<'static, f64> = curve.knots().clone();
        if curve.nb_elements() >= self.options.max_segment || !self.options.with_cutting {
            let c_new: Curve =
                Curve::new(self.dimension, curve.nb_elements(), curve.base(), curv_tol)?;
            c_new.knots().assign(&knots);
            return Ok(Handle::new(c_new));
        }
        let worst: f64 = self.parameter(self.results.max_error_index)?;
        let (numint, _) = near_index(worst, &knots, 0.)?;
        let tpara: f64 = (at(&knots, numint)? + at(&knots, numint + 1)? + worst * 2.) / 4.;
        let c_new: Curve = Curve::new(
            self.dimension,
            curve.nb_elements() + 1,
            curve.base(),
            curv_tol,
        )?;
        let mut new_knots = c_new.knots();
        for i in 1..=numint {
            put(&mut new_knots, i, at(&knots, i)?)?;
        }
        for i in numint + 1..=knots.length() {
            put(&mut new_knots, i + 1, at(&knots, i)?)?;
        }
        put(&mut new_knots, numint + 1, tpara)?;
        drop(new_knots);
        Ok(Handle::new(c_new))
    }

    // A new LinearCriteria with the estimations of `j`, the current parameters, the
    // weights and the point weights, on `c_new`.
    fn adjusted_criterion(
        &self,
        j: &Handle<dyn SmoothCriterion>,
        (w_quadratic, w_quality): (f64, f64),
        tbpoid: &Array1<'_, f64>,
        c_new: &Handle<Curve>,
    ) -> Result<Handle<dyn SmoothCriterion>, Failure> {
        let j_new: LinearCriteria =
            LinearCriteria::new(&self.ssp, self.first_point, self.last_point)?;
        let j_old: &dyn SmoothCriterion = criterion_ref(j)?;
        j_new.set_est_length(j_old.est_length());
        let [e1, e2, e3] = j_old.get_estimation();
        j_new.set_estimation(e1, e2, e3);
        j_new.set_parameters(&self.parameters);
        j_new.set_weight(w_quadratic, w_quality, self.percent)?;
        j_new.set_point_weights(tbpoid);
        j_new.set_curve(c_new)?;
        Ok(Handle::new(j_new).cast())
    }
}
