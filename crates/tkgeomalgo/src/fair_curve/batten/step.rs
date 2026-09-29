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
// FairCurve_MinimalVariation.cxx (the homotopy loops of Compute) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The homotopy loop shared by the batten and the minimal-variation curve: the
//! constraints move from the computed ("old") values to the requested ("new")
//! ones in steps small enough for the Newton minimization.
use super::FairCurveBatten;
use crate::fair_curve::FairCurveAnalysisCode;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Vector2;
use std::f64::consts::PI;

/// The parameter of the step function of the angles (about 40 degrees).
const ANGLE_MAX: f64 = 0.7;

/// One homotopy step: the moves of the end points and angles.
pub(in crate::fair_curve) struct HomotopyStep {
    pub(in crate::fair_curve) delta_p1: Vector2,
    pub(in crate::fair_curve) delta_p2: Vector2,
    pub(in crate::fair_curve) dangle1: f64,
    pub(in crate::fair_curve) dangle2: f64,
    /// The fraction of the remaining move this step makes (1 for the last step).
    pub(in crate::fair_curve) ratio: f64,
}

impl HomotopyStep {
    /// `delta` scaled as the native loop scales the remaining moves.
    pub(in crate::fair_curve) fn scaled(&self, delta: f64) -> f64 {
        if self.ratio == 1.0 {
            delta
        } else {
            delta * self.ratio
        }
    }
}

/// A curve computed by homotopy: the native virtual pieces of `Compute`.
pub(in crate::fair_curve) trait Homotopy {
    /// The batten part of the curve.
    fn batten(&self) -> &FairCurveBatten;
    /// The batten part, mutable.
    fn batten_mut(&mut self) -> &mut FairCurveBatten;
    /// Further limits on the step ratio (the curvature moves); `ratio` otherwise.
    fn limit_ratio(&self, _old_dist: f64, ratio: f64) -> f64 {
        ratio
    }
    /// One deformation and minimization (the private native `Compute`).
    fn step(
        &mut self,
        step: &HomotopyStep,
        code: &mut FairCurveAnalysisCode,
        nb_iterations: i32,
        tolerance: f64,
    ) -> Result<bool, Failure>;
    /// Drops the requests the computed curve does not constrain.
    fn after_step(&mut self) {
        self.batten_mut().after_step();
    }
}

/// `Ratio < 1 / Fraction ? Ratio : 1 / Fraction`.
fn lesser(ratio: f64, fraction: f64) -> f64 {
    let inverse: f64 = 1.0 / fraction;
    if ratio < inverse { ratio } else { inverse }
}

/// The native homotopy loop of `Compute(ACode, NbIterations, Tolerance)`.
pub(in crate::fair_curve) fn run_homotopy(
    curve: &mut impl Homotopy,
    code: &mut FairCurveAnalysisCode,
    nb_iterations: i32,
    tolerance: f64,
) -> Result<bool, Failure> {
    let (mut ok, mut end) = (true, false);
    while ok && !end {
        let step: HomotopyStep = next_step(curve);
        let toler: f64 = if step.ratio == 1.0 {
            end = true;
            tolerance
        } else {
            10.0 * tolerance
        };
        ok = curve.step(&step, code, nb_iterations, toler)?;
        if *code != FairCurveAnalysisCode::Ok {
            end = true;
        }
        curve.after_step();
    }
    curve.batten_mut().code = *code;
    Ok(ok)
}

/// The next step: the remaining moves, scaled down when too large.
fn next_step(curve: &impl Homotopy) -> HomotopyStep {
    let batten: &FairCurveBatten = curve.batten();
    let (old, new) = (&batten.old, &batten.new);
    let dangle1: f64 = new.angle1 - old.angle1;
    let dangle2: f64 = new.angle2 - old.angle2;
    let (ratio, old_dist) = batten.step_ratio(dangle1, dangle2);
    let mut step = HomotopyStep {
        delta_p1: Vector2::between(old.p1, new.p1),
        delta_p2: Vector2::between(old.p2, new.p2),
        dangle1,
        dangle2,
        ratio: curve.limit_ratio(old_dist, ratio),
    };
    if step.ratio != 1.0 {
        step.delta_p1 *= step.ratio;
        step.delta_p2 *= step.ratio;
        step.dangle1 *= step.ratio;
        step.dangle2 *= step.ratio;
    }
    step
}

impl FairCurveBatten {
    /// The step ratio of the angle and chord moves, and the old chord length.
    fn step_ratio(&self, dangle1: f64, dangle2: f64) -> (f64, f64) {
        let angle_min: f64 = 2.0 * PI / 100.0;
        let fraction_of = |dangle: f64, old_angle: f64| {
            dangle.abs() / (ANGLE_MAX * (-old_angle.abs() / ANGLE_MAX).exp() + angle_min)
        };
        let mut ratio: f64 = 1.0;
        if self.new.constraint_order1 > 0 {
            let fraction: f64 = fraction_of(dangle1, self.old.angle1);
            if fraction > 1.0 {
                ratio = 1.0 / fraction;
            }
        }
        if self.new.constraint_order2 > 0 {
            let fraction: f64 = fraction_of(dangle2, self.old.angle2);
            if fraction > 1.0 {
                ratio = lesser(ratio, fraction);
            }
        }
        let old_dist: f64 = self.old.p1.distance(self.old.p2);
        let new_dist: f64 = self.new.p1.distance(self.new.p2);
        let fraction: f64 = (old_dist - new_dist).abs() / (old_dist / 3.0);
        if fraction > 1.0 {
            ratio = lesser(ratio, fraction);
        }
        (ratio, old_dist)
    }
    /// The curvature limit of the minimal-variation steps:
    /// `|DRho| * OldDist / (2 + |OldAngle1| + |OldAngle2|)`.
    pub(in crate::fair_curve) fn limit_by_fraction(ratio: f64, fraction: f64) -> f64 {
        if fraction > 1.0 {
            lesser(ratio, fraction)
        } else {
            ratio
        }
    }
    /// The native tail of the loop body for the batten requests.
    pub(in crate::fair_curve) fn after_step(&mut self) {
        if self.new.free_sliding {
            self.new.sliding_factor = self.old.sliding_factor;
        }
        if self.new.constraint_order1 == 0 {
            self.new.angle1 = self.old.angle1;
        }
        if self.new.constraint_order2 == 0 {
            self.new.angle2 = self.old.angle2;
        }
    }
}
