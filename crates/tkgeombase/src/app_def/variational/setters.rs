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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_Variational.cxx (the Set members)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The field setters of `AppDef_Variational`. Most rebuild the smoothing criterion
//! (InitSmoothCriterion), which resets the parameters to the chord lengths; none
//! clears the done flag.
use super::{Variational, construction, domain, niv_cont_of};
use crate::app_par_curves::Constraints;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::Shape;

impl Variational {
    /// `SetConstraints(aConstraint)`: rebuilds the tables (Init); false when the
    /// problem is (or was once) over-constrained; `smoothing.set_constraints(&c)?`.
    pub fn set_constraints(&mut self, a_constraint: &Constraints) -> Result<bool, Failure> {
        self.constraints = a_constraint.clone();
        self.init()?;
        Ok(!self.is_over_constr)
    }

    /// `SetParameters(param)`: copies the values and the bounds of `param` into the
    /// parameters (no rebuild); `smoothing.set_parameters(&param)`.
    pub fn set_parameters(&mut self, param: &Handle<HArray1<f64>>) -> Result<(), Failure> {
        let values: Array1<'static, f64> = param
            .get()
            .ok_or_else(|| construction("SetParameters: null parameters"))?
            .array1()
            .clone();
        self.parameters.change_array1().assign(&values);
        Ok(())
    }

    /// `SetKnots(knots)`: copies the values and the bounds of `knots`; always true;
    /// `smoothing.set_knots(&knots)?`.
    pub fn set_knots(&mut self, knots: &Handle<HArray1<f64>>) -> Result<bool, Failure> {
        let values: Array1<'static, f64> = knots
            .get()
            .ok_or_else(|| construction("SetKnots: null knots"))?
            .array1()
            .clone();
        self.knots.change_array1().assign(&values);
        Ok(true)
    }

    /// `SetMaxDegree(Degree)`: false when over-constrained with `degree`, else the
    /// degree (neither checked nor clamped as in the constructor) and a rebuild;
    /// `smoothing.set_max_degree(10)?`.
    pub fn set_max_degree(&mut self, degree: i32) -> Result<bool, Failure> {
        if self.over_constrained(degree, self.niv_cont, self.options.max_segment) {
            return Ok(false);
        }
        self.options.max_degree = degree;
        self.init_smooth_criterion()?;
        Ok(true)
    }

    /// `SetMaxSegment(NbSegment)`: false when cutting is allowed and `nb_segment`
    /// segments over-constrain the problem, else the count (no rebuild);
    /// `smoothing.set_max_segment(10)`.
    pub fn set_max_segment(&mut self, nb_segment: i32) -> bool {
        if self.options.with_cutting
            && self.over_constrained(self.options.max_degree, self.niv_cont, nb_segment)
        {
            return false;
        }
        self.options.max_segment = nb_segment;
        true
    }

    /// `SetContinuity(C)`: Standard_ConstructionError for a continuity other than C0,
    /// C1 or C2; false when it over-constrains, else the continuity and a rebuild;
    /// `smoothing.set_continuity(Shape::C1)?`.
    pub fn set_continuity(&mut self, c: Shape) -> Result<bool, Failure> {
        let niv_cont: i32 = niv_cont_of(c)?;
        if self.over_constrained(self.options.max_degree, niv_cont, self.options.max_segment) {
            return Ok(false);
        }
        self.options.continuity = c;
        self.niv_cont = niv_cont;
        self.init_smooth_criterion()?;
        Ok(true)
    }

    /// `SetWithMinMax(MinMax)` and a rebuild; `smoothing.set_with_min_max(true)?`.
    pub fn set_with_min_max(&mut self, min_max: bool) -> Result<(), Failure> {
        self.options.with_min_max = min_max;
        self.init_smooth_criterion()
    }

    /// `SetWithCutting(Cutting)`: false when it over-constrains (without cutting, with
    /// as many segments as knots — the native test), else the flag and a rebuild;
    /// `smoothing.set_with_cutting(false)?`.
    pub fn set_with_cutting(&mut self, cutting: bool) -> Result<bool, Failure> {
        let segments: i32 = if cutting {
            self.options.max_segment
        } else {
            self.knots.array1().length()
        };
        if self.over_constrained(self.options.max_degree, self.niv_cont, segments) {
            return Ok(false);
        }
        self.options.with_cutting = cutting;
        self.init_smooth_criterion()?;
        Ok(true)
    }

    /// `SetCriteriumWeight(Percent1, Percent2, Percent3)`: the normalized weights of
    /// the tension, flexion and jerk criteria and a rebuild; Standard_DomainError for
    /// a negative one; `smoothing.set_criterium_weight(1.0, 1.0, 2.0)?`.
    pub fn set_criterium_weight(
        &mut self,
        percent1: f64,
        percent2: f64,
        percent3: f64,
    ) -> Result<(), Failure> {
        if percent1 < 0. || percent2 < 0. || percent3 < 0. {
            return Err(domain(&format!(
                "SetCriteriumWeight: {percent1}, {percent2}, {percent3}; expected >= 0"
            )));
        }
        let total: f64 = percent1 + percent2 + percent3;
        self.percent = [percent1 / total, percent2 / total, percent3 / total];
        self.init_smooth_criterion()
    }

    /// `SetCriteriumWeight(Order, Percent)`: the weight of criterion `order`, then
    /// all three normalized and a rebuild; Standard_DomainError for a negative weight,
    /// Standard_ConstructionError for an order outside 1..=3;
    /// `smoothing.set_criterium_weight_of(2, 0.5)?`.
    pub fn set_criterium_weight_of(&mut self, order: i32, percent: f64) -> Result<(), Failure> {
        if percent < 0. {
            return Err(domain(&format!(
                "SetCriteriumWeight: percent {percent}; expected >= 0"
            )));
        }
        if !(1..=3).contains(&order) {
            return Err(construction(&format!(
                "SetCriteriumWeight: order {order}; expected 1..=3"
            )));
        }
        self.percent[(order - 1) as usize] = percent;
        let total: f64 = self.percent[0] + self.percent[1] + self.percent[2];
        self.percent = self.percent.map(|p| p / total);
        self.init_smooth_criterion()
    }

    /// `SetTolerance(Tol)` and a rebuild; `smoothing.set_tolerance(1e-3)?`.
    pub fn set_tolerance(&mut self, tol: f64) -> Result<(), Failure> {
        self.options.tolerance = tol;
        self.init_smooth_criterion()
    }

    /// `SetNbIterations(Iter)` (no check, no rebuild); `smoothing.set_nb_iterations(3)`.
    pub fn set_nb_iterations(&mut self, iter: i32) {
        self.options.nb_iterations = iter;
    }
}
