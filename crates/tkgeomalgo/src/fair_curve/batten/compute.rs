// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_Batten.cxx (Compute) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FairCurve_Batten::Compute`: the homotopy to the requested constraints, each
//! step minimizing the batten energy.
use super::FairCurveBatten;
use super::step::{Homotopy, HomotopyStep, run_homotopy};
use crate::fair_curve::{FairCurveAnalysisCode, FairCurveEnergyModel, FairCurveEnergyOfBatten};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Vector2, Xy};

impl FairCurveBatten {
    /// Computes the curve of the requested constraints with at most
    /// `nb_iterations` Newton iterations per step (native default 50) and
    /// `tolerance` (native default 1e-3). Returns false when a minimization fails
    /// without a known cause; `code` reports non-convergence, infinite sliding
    /// or a null height. `batten.compute(&mut code, 50, 1.0e-3)?`.
    pub fn compute(
        &mut self,
        code: &mut FairCurveAnalysisCode,
        nb_iterations: i32,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        run_homotopy(self, code, nb_iterations, tolerance)
    }
    /// The angle deltas at the ends for a step (`ADelta` of the tangents).
    fn tangent_deltas(&self, step: &HomotopyStep, dangle_ref: f64) -> Vec<Xy> {
        let mut adelta: Vec<Xy> = vec![step.delta_p1.xy()];
        if self.new.constraint_order1 > 0 {
            let old = self.start_derivative().derivative;
            adelta.push((old.rotated(step.dangle1 - dangle_ref) - old).xy());
        }
        adelta.push(step.delta_p2.xy());
        if self.new.constraint_order2 > 0 {
            let old = self.end_derivative().derivative;
            adelta.push((old.rotated(dangle_ref - step.dangle2) - old).xy());
        }
        adelta
    }
    /// The angles of the solved curve at unconstrained ends, from its end tangents.
    fn solved_angles(&mut self, p1p2: Vector2, [alph1, alph2]: [f64; 2]) -> Result<(), Failure> {
        let poles = self.poles.get().expect("batten poles").array1();
        let (lower, upper) = (poles.lower(), poles.upper());
        self.old.angle1 = if self.new.constraint_order1 == 0 {
            p1p2.angle(Vector2::from(poles[lower + 1].xy() - poles[lower].xy()))?
        } else {
            alph1
        };
        self.old.angle2 = if self.new.constraint_order2 == 0 {
            let tangente = Vector2::from(poles[upper].xy() - poles[upper - 1].xy());
            (-tangente).angle(-p1p2)?
        } else {
            alph2
        };
        Ok(())
    }
}

impl Homotopy for FairCurveBatten {
    fn batten(&self) -> &FairCurveBatten {
        self
    }
    fn batten_mut(&mut self) -> &mut FairCurveBatten {
        self
    }
    fn step(
        &mut self,
        step: &HomotopyStep,
        code: &mut FairCurveAnalysisCode,
        nb_iterations: i32,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        *code = FairCurveAnalysisCode::Ok;
        let Some(hermite) = self.hermite()? else {
            return Ok(false);
        };
        let dangle_ref: f64 = self.reference_angle(step)?;
        let npoles = self.deformed_poles(&hermite, &self.tangent_deltas(step, dangle_ref))?;
        let alph: [f64; 2] = [
            self.old.angle1 + step.dangle1,
            self.old.angle2 + step.dangle2,
        ];
        let frame = self.sliding_frame(&npoles, alph)?;
        let law = self.step_law(frame.sliding);
        let orders = [self.new.constraint_order1, self.new.constraint_order2];
        let mut energy = FairCurveEnergyOfBatten::new(
            self.degree + 1,
            &self.flatknots,
            &npoles,
            orders,
            &law,
            frame.sliding,
            self.new.free_sliding,
            frame.angles,
        );
        let spatial: f64 = tolerance * frame.p1p2.magnitude() / 10.0;
        let mut solution = self.minimize(
            &mut energy,
            frame.sliding,
            [spatial, tolerance],
            nb_iterations,
        )?;
        if !solution.newton.is_done() {
            return Ok(Self::failed_step(energy.status(), &law, code));
        }
        self.poles = npoles;
        solution.newton.copy_location(&mut solution.vinit)?;
        self.accept_sliding(&solution.vinit, frame.lreference);
        self.solved_angles(frame.p1p2, alph)?;
        self.commit();
        FairCurveEnergyModel::variable(&energy, &mut solution.vinit);
        let converged: bool = solution.newton.is_converged();
        self.finish((&solution.vinit, converged), frame.lreference, code)?;
        Ok(true)
    }
}
