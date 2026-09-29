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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_Batten.cxx and
// FairCurve_MinimalVariation.cxx (the minimization of the private Compute) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The minimization of a homotopy step and the bookkeeping around it: the sliding
//! length, the Newton solve, the failure codes, and the knot refinement.
use super::FairCurveBatten;
use crate::fair_curve::{
    FairCurveAnalysisCode, FairCurveBattenLaw, FairCurveEnergyModel, FairCurveNewton,
};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, arrays::one_based, errors};
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tkmath::math::{MultipleVarFunctionWithHessian, Vector};

/// The chord and sliding of the deformed poles.
pub(in crate::fair_curve) struct SlidingFrame {
    /// `LReference`, the reference sliding of the deformed chord.
    pub(in crate::fair_curve) lreference: f64,
    /// `P1P2`, the deformed chord.
    pub(in crate::fair_curve) p1p2: Vector2,
    /// The end angles from the Ox axis.
    pub(in crate::fair_curve) angles: [f64; 2],
    /// The imposed or initial sliding length.
    pub(in crate::fair_curve) sliding: f64,
}

/// The Newton solver of a step and its variables.
pub(in crate::fair_curve) struct StepSolution {
    pub(in crate::fair_curve) newton: FairCurveNewton,
    pub(in crate::fair_curve) vinit: Vector<'static>,
}

impl FairCurveBatten {
    /// The chord, reference sliding, end angles and sliding length of `npoles`.
    pub(in crate::fair_curve) fn sliding_frame(
        &self,
        npoles: &Handle<HArray1<Point2>>,
        [alph1, alph2]: [f64; 2],
    ) -> Result<SlidingFrame, Failure> {
        let poles = npoles.get().expect("deformed poles").array1();
        let (first, last) = (poles[poles.lower()], poles[poles.upper()]);
        let dist: f64 = last.distance(first);
        let lreference: f64 = self.sliding_of_reference_for(dist, alph1, alph2);
        let p1p2 = Vector2::from(last.xy() - first.xy());
        let ox_angle: f64 = Vector2::new(1.0, 0.0).angle(p1p2)?;
        let sliding: f64 = if !self.new.free_sliding {
            self.new.sliding_factor * lreference
        } else if self.old.free_sliding {
            self.old.sliding_factor * lreference
        } else {
            self.sliding_of_reference_for(dist, alph1, alph2)
        };
        Ok(SlidingFrame {
            lreference,
            p1p2,
            angles: [ox_angle + alph1, -ox_angle + alph2],
            sliding,
        })
    }
    /// The requested height law of the sliding length.
    pub(in crate::fair_curve) fn step_law(&self, sliding: f64) -> FairCurveBattenLaw {
        FairCurveBattenLaw::new(self.new.height, self.new.slope, sliding)
    }
    /// The Newton minimization of `energy` from its current variables; the
    /// convexity bound is the smallest flexion criterion `0.01 (H / L)^3`.
    pub(in crate::fair_curve) fn minimize<E>(
        &self,
        energy: &mut E,
        sliding: f64,
        [spatial_tolerance, tolerance]: [f64; 2],
        nb_iterations: i32,
    ) -> Result<StepSolution, Failure>
    where
        E: FairCurveEnergyModel + MultipleVarFunctionWithHessian,
    {
        let mut vinit: Vector<'static> =
            Vector::with_bounds(1, FairCurveEnergyModel::nb_variables(energy))?;
        let mut vconvex: f64 = 0.01 * (self.new.height / sliding).powf(3.0);
        if vconvex < 1.0e-12 {
            vconvex = 1.0e-12;
        }
        FairCurveEnergyModel::variable(energy, &mut vinit);
        let mut newton = FairCurveNewton::new(
            energy,
            spatial_tolerance,
            tolerance,
            nb_iterations,
            vconvex,
            true,
        )?;
        newton.perform(energy, &vinit)?;
        Ok(StepSolution { newton, vinit })
    }
    /// The code of a failed minimization: `NullHeight` (a step result of true) when
    /// the law vanishes at an end, the energy status (false) otherwise.
    pub(in crate::fair_curve) fn failed_step(
        status: FairCurveAnalysisCode,
        law: &FairCurveBattenLaw,
        code: &mut FairCurveAnalysisCode,
    ) -> bool {
        *code = status;
        if !law.height(0.0).0 || !law.height(1.0).0 {
            *code = FairCurveAnalysisCode::NullHeight;
            return true;
        }
        false
    }
    /// Keeps the sliding factor of the solution `vinit` (its last variable when free).
    pub(in crate::fair_curve) fn accept_sliding(&mut self, vinit: &Vector<'_>, lreference: f64) {
        self.old.sliding_factor = if self.new.free_sliding {
            vinit[vinit.upper()] / lreference
        } else {
            self.new.sliding_factor
        };
    }
    /// The requests the solved curve now satisfies, besides its angles.
    pub(in crate::fair_curve) fn commit(&mut self) {
        let poles = self.poles.get().expect("batten poles").array1();
        self.old.p1 = poles[poles.lower()];
        self.old.p2 = poles[poles.upper()];
        drop(poles);
        self.old.constraint_order1 = self.new.constraint_order1;
        self.old.constraint_order2 = self.new.constraint_order2;
        self.old.free_sliding = self.new.free_sliding;
        self.old.slope = self.new.slope;
        self.old.height = self.new.height;
    }
    /// The end of a solved step: the convergence and sliding codes from the final
    /// variables `vinit`, then the knot refinement.
    pub(in crate::fair_curve) fn finish(
        &mut self,
        (vinit, converged): (&Vector<'_>, bool),
        lreference: f64,
        code: &mut FairCurveAnalysisCode,
    ) -> Result<(), Failure> {
        if !converged {
            *code = FairCurveAnalysisCode::NotConverged;
        }
        if self.new.free_sliding && vinit[vinit.upper()] > 2.0 * lreference {
            *code = FairCurveAnalysisCode::InfiniteSliding;
        }
        self.refine_knots()
    }
    /// Inserts uniform knots of multiplicity `Degree - 3` when the angles grow.
    fn refine_knots(&mut self) -> Result<(), Failure> {
        let knot_count: i32 = self.knots.get().expect("batten knots").array1().len() as i32;
        let val_angles: f64 = self.old.angle1.abs()
            + self.old.angle2.abs()
            + 2.0 * (self.old.angle2 - self.old.angle1).abs();
        // i64 keeps the native int products from overflowing (undefined natively).
        let mut nb_knots: i64 = i64::from(knot_count);
        let mut new_knots: bool = false;
        while val_angles > ((2 * (nb_knots - 2) + 1) * (1 + 2 * nb_knots)) as f64 {
            new_knots = true;
            nb_knots += nb_knots - 1;
        }
        if !new_knots {
            return Ok(());
        }
        let nb_knots: i32 = i32::try_from(nb_knots).map_err(|_| {
            errors::out_of_range(&format!(
                "FairCurve: {nb_knots} knots; expected at most {}",
                i32::MAX
            ))
        })?;
        self.insert_uniform_knots(nb_knots)
    }
    fn insert_uniform_knots(&mut self, nb_knots: i32) -> Result<(), Failure> {
        let curve: Geom2dBSplineCurve = {
            let poles = self.poles.get().expect("batten poles").array1();
            let knots = self.knots.get().expect("batten knots").array1();
            let mults = self.mults.get().expect("batten mults").array1();
            Geom2dBSplineCurve::new(&poles, &knots, &mults, self.degree, false)?
        };
        let mut nmults: Array1<'static, i32> = one_based(nb_knots)?;
        nmults.init(self.degree - 3);
        let mut nknots: Array1<'static, f64> = one_based(nb_knots)?;
        for ii in 1..=nb_knots {
            nknots[ii] = f64::from(ii - 1) / f64::from(nb_knots - 1);
        }
        curve.insert_knots(&nknots, &nmults, 1.0e-10, false)?;
        self.poles = Handle::new(HArray1::from_owned(curve.poles()));
        self.mults = Handle::new(HArray1::from_owned(curve.multiplicities()));
        self.knots = Handle::new(HArray1::from_owned(curve.knots()));
        self.flatknots = Handle::new(HArray1::from_owned(curve.knot_sequence()));
        Ok(())
    }
}
