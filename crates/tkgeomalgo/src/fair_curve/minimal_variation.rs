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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_MinimalVariation.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FairCurve_MinimalVariation`: a batten that also minimizes the variation of
//! its curvature, with optional end curvatures.
use super::batten::deform::EndDerivative;
use super::batten::step::{Homotopy, HomotopyStep, run_homotopy};
use super::energy_mvc::check_physical_ratio;
use super::{FairCurveAnalysisCode, FairCurveBatten, FairCurveEnergyModel, FairCurveEnergyOfMvc};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2, Xy};
use std::f64::consts::FRAC_PI_2;
use std::ops::{Deref, DerefMut};

/// The minimal-variation curve; the batten setters and getters come through
/// `Deref`, and `compute` and `dump` replace the batten ones.
///
/// `let mut mvc = FairCurveMinimalVariation::new(p1, p2, 1.0, 0.0, 0.5)?;
/// mvc.set_constraint_order1(2); mvc.set_curvature1(0.1);
/// mvc.compute(&mut code, 50, 1e-3)?;`
#[derive(Clone)]
pub struct FairCurveMinimalVariation {
    batten: FairCurveBatten,
    pub(super) old_curvatures: [f64; 2],
    pub(super) old_physical_ratio: f64,
    pub(super) new_curvatures: [f64; 2],
    pub(super) new_physical_ratio: f64,
}

impl FairCurveMinimalVariation {
    /// A curve from `p1` to `p2` of middle height `heigth` and `slope`, weighting
    /// flexion against jerk by `physical_ratio` (checked by `compute`). The batten
    /// errors apply; `FairCurveMinimalVariation::new(p1, p2, 1.0, 0.0, 0.0)?`.
    pub fn new(
        p1: Point2,
        p2: Point2,
        heigth: f64,
        slope: f64,
        physical_ratio: f64,
    ) -> Result<Self, Failure> {
        Ok(Self {
            batten: FairCurveBatten::new(p1, p2, heigth, slope)?,
            old_curvatures: [0.0; 2],
            old_physical_ratio: physical_ratio,
            new_curvatures: [0.0; 2],
            new_physical_ratio: physical_ratio,
        })
    }
    /// The curvature at P1 (used with constraint order 2); `mvc.set_curvature1(0.1)`.
    pub fn set_curvature1(&mut self, curvature: f64) {
        self.new_curvatures[0] = curvature;
    }
    /// The curvature at P2; `mvc.set_curvature2(0.1)`.
    pub fn set_curvature2(&mut self, curvature: f64) {
        self.new_curvatures[1] = curvature;
    }
    /// The weight of the flexion; outside [0, 1] raises `Standard_DomainError`;
    /// `mvc.set_physical_ratio(0.5)?`.
    pub fn set_physical_ratio(&mut self, ratio: f64) -> Result<(), Failure> {
        check_physical_ratio("FairCurve_MinimalVariation::SetPhysicalRatio", ratio)?;
        self.new_physical_ratio = ratio;
        Ok(())
    }
    /// The curvature at P1 of the computed curve; `mvc.get_curvature1()`.
    pub fn get_curvature1(&self) -> f64 {
        self.old_curvatures[0]
    }
    /// The curvature at P2 of the computed curve; `mvc.get_curvature2()`.
    pub fn get_curvature2(&self) -> f64 {
        self.old_curvatures[1]
    }
    /// The physical ratio of the computed curve; `mvc.get_physical_ratio()`.
    pub fn get_physical_ratio(&self) -> f64 {
        self.old_physical_ratio
    }
    /// Computes the curve of the requested constraints like
    /// `FairCurveBatten::compute`; `mvc.compute(&mut code, 50, 1.0e-3)?`.
    pub fn compute(
        &mut self,
        code: &mut FairCurveAnalysisCode,
        nb_iterations: i32,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        run_homotopy(self, code, nb_iterations, tolerance)
    }
    /// The second derivative of the current curve at an end (`OldSeconde`):
    /// `P(end) + P(end +- 2) - 2 P(end +- 1)` scaled by `Degree (Degree - 1) / gap^2`.
    fn old_seconde(&self, start: bool, knot_gap: f64) -> Vector2 {
        let poles = self.batten.poles.get().expect("batten poles").array1();
        let (end, next, far) = if start {
            (poles.lower(), poles.lower() + 1, poles.lower() + 2)
        } else {
            (poles.upper(), poles.upper() - 1, poles.upper() - 2)
        };
        let mut seconde = Vector2::from(poles[end].xy() + poles[far].xy() - 2.0 * poles[next].xy());
        let gap_sq: f64 = knot_gap * knot_gap;
        if gap_sq.abs() > precision::square_computational() {
            let degree: i32 = self.batten.degree;
            seconde *= f64::from(degree * (degree - 1)) / gap_sq;
        }
        seconde
    }
    /// The tangent and second-derivative deltas of one end.
    fn end_deltas(&self, start: bool, (dangle, drho): (f64, f64), adelta: &mut Vec<Xy>) {
        let order: i32 = if start {
            self.batten.new.constraint_order1
        } else {
            self.batten.new.constraint_order2
        };
        if order <= 0 {
            return;
        }
        let EndDerivative {
            derivative,
            knot_gap,
        } = if start {
            self.batten.start_derivative()
        } else {
            self.batten.end_derivative()
        };
        adelta.push((derivative.rotated(dangle) - derivative).xy());
        if order > 1 {
            let seconde: Vector2 = self.old_seconde(start, knot_gap);
            let cprim: f64 = derivative.magnitude();
            let turned: Vector2 = derivative.rotated(FRAC_PI_2 + dangle);
            adelta.push((seconde.rotated(dangle) - seconde + drho * cprim * turned).xy());
        }
    }
    /// The tangent `P(end +- 1) - P(end)` (oriented from P1 to P2) and the second
    /// difference of the solved curve at an end.
    fn end_frame(&self, start: bool) -> (Vector2, Vector2) {
        let poles = self.batten.poles.get().expect("batten poles").array1();
        let (lower, upper) = (poles.lower(), poles.upper());
        let (t, p) = if start {
            let t = poles[lower + 1].xy() - poles[lower].xy();
            (
                t,
                poles[lower].xy() - 2.0 * poles[lower + 1].xy() + poles[lower + 2].xy(),
            )
        } else {
            let t = poles[upper].xy() - poles[upper - 1].xy();
            (
                t,
                poles[upper].xy() - 2.0 * poles[upper - 1].xy() + poles[upper - 2].xy(),
            )
        };
        (Vector2::from(t), Vector2::from(p))
    }
    fn set_old_angle(&mut self, side: usize, angle: f64) {
        if side == 0 {
            self.batten.old.angle1 = angle;
        } else {
            self.batten.old.angle2 = angle;
        }
    }
    /// The angles and curvatures of the solved curve, in the native order: an end
    /// not constrained to G2 gets its angle, then its curvature
    /// `((D - 1) / D) (T/|T| ^ N) / |T|^2`.
    fn solved_ends(&mut self, p1p2: Vector2, alph: [f64; 2], rho: [f64; 2]) -> Result<(), Failure> {
        let orders = [
            self.batten.new.constraint_order1,
            self.batten.new.constraint_order2,
        ];
        let degree: f64 = f64::from(self.batten.degree);
        for (side, &order) in orders.iter().enumerate() {
            if order >= 2 {
                self.old_curvatures[side] = rho[side];
                self.set_old_angle(side, alph[side]);
                continue;
            }
            let (tangente, pseudo) = self.end_frame(side == 0);
            let angle: f64 = match (order, side) {
                (0, 0) => p1p2.angle(tangente)?,
                (0, _) => (-tangente).angle(-p1p2)?,
                _ => alph[side],
            };
            self.set_old_angle(side, angle);
            self.old_curvatures[side] = ((degree - 1.0) / degree)
                * tangente.normalized()?.crossed(pseudo)
                / tangente.square_magnitude();
        }
        Ok(())
    }
}

impl Homotopy for FairCurveMinimalVariation {
    fn batten(&self) -> &FairCurveBatten {
        &self.batten
    }
    fn batten_mut(&mut self) -> &mut FairCurveBatten {
        &mut self.batten
    }
    fn limit_ratio(&self, old_dist: f64, mut ratio: f64) -> f64 {
        let (old, new) = (&self.batten.old, &self.batten.new);
        let orders = [new.constraint_order1, new.constraint_order2];
        for (side, &order) in orders.iter().enumerate() {
            if order > 1 {
                let drho: f64 = self.new_curvatures[side] - self.old_curvatures[side];
                let fraction: f64 =
                    drho.abs() * old_dist / (2.0 + old.angle1.abs() + old.angle2.abs());
                ratio = FairCurveBatten::limit_by_fraction(ratio, fraction);
            }
        }
        ratio
    }
    fn step(
        &mut self,
        step: &HomotopyStep,
        code: &mut FairCurveAnalysisCode,
        nb_iterations: i32,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        *code = FairCurveAnalysisCode::Ok;
        let Some(hermite) = self.batten.hermite()? else {
            return Ok(false);
        };
        let dangle_ref: f64 = self.batten.reference_angle(step)?;
        let drho: [f64; 2] =
            [0, 1].map(|side| step.scaled(self.new_curvatures[side] - self.old_curvatures[side]));
        let mut adelta: Vec<Xy> = vec![step.delta_p1.xy()];
        self.end_deltas(true, (step.dangle1 - dangle_ref, drho[0]), &mut adelta);
        adelta.push(step.delta_p2.xy());
        self.end_deltas(false, (dangle_ref - step.dangle2, drho[1]), &mut adelta);
        let npoles = self.batten.deformed_poles(&hermite, &adelta)?;
        let old = self.batten.old;
        let alph: [f64; 2] = [old.angle1 + step.dangle1, old.angle2 + step.dangle2];
        let rho: [f64; 2] = [
            self.old_curvatures[0] + drho[0],
            self.old_curvatures[1] + drho[1],
        ];
        let frame = self.batten.sliding_frame(&npoles, alph)?;
        let law = self.batten.step_law(frame.sliding);
        let new = self.batten.new;
        let mut energy = FairCurveEnergyOfMvc::new(
            self.batten.degree + 1,
            &self.batten.flatknots,
            &npoles,
            [new.constraint_order1, new.constraint_order2],
            &law,
            self.new_physical_ratio,
            frame.sliding,
            new.free_sliding,
            frame.angles,
            rho,
        )?;
        let spatial: f64 = tolerance * (frame.p1p2.magnitude() / 10.0);
        let mut solution = self.batten.minimize(
            &mut energy,
            frame.sliding,
            [spatial, tolerance],
            nb_iterations,
        )?;
        if !solution.newton.is_done() {
            return Ok(FairCurveBatten::failed_step(energy.status(), &law, code));
        }
        self.batten.poles = npoles;
        solution.newton.copy_location(&mut solution.vinit)?;
        self.batten
            .accept_sliding(&solution.vinit, frame.lreference);
        self.solved_ends(frame.p1p2, alph, rho)?;
        self.batten.commit();
        self.old_physical_ratio = self.new_physical_ratio;
        FairCurveEnergyModel::variable(&energy, &mut solution.vinit);
        let converged: bool = solution.newton.is_converged();
        self.batten
            .finish((&solution.vinit, converged), frame.lreference, code)?;
        Ok(true)
    }
    fn after_step(&mut self) {
        self.batten.after_step();
        let orders = [
            self.batten.new.constraint_order1,
            self.batten.new.constraint_order2,
        ];
        for (side, &order) in orders.iter().enumerate() {
            if order < 2 {
                self.new_curvatures[side] = self.old_curvatures[side];
            }
        }
    }
}

impl Deref for FairCurveMinimalVariation {
    type Target = FairCurveBatten;
    fn deref(&self) -> &FairCurveBatten {
        &self.batten
    }
}

impl DerefMut for FairCurveMinimalVariation {
    fn deref_mut(&mut self) -> &mut FairCurveBatten {
        &mut self.batten
    }
}
