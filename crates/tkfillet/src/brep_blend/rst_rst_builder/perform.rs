// Rust translation of BRepBlend_RstRstLineBuilder::Perform, PerformFirstSection and
// Complete (BRepBlend_RstRstLineBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The start of the restriction-restriction marching (the first section, between the
//! restrictions or on their ends) and its completion in the other direction.
use super::grounding::Grounding;
use super::{BRepBlendRstRstLineBuilder, CLASS, Sol};
use crate::blend::{BlendCurvPointFuncInv, BlendRstRstFunction, BlendSurfCurvFuncInv, Status};
use crate::brep_blend::BRepBlendExtremity;
use crate::brep_blend::marching::{self, math_vector, root_of, solver_for};
use crate::chfi_ds::lock;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;

/// The settings of `Perform`: `MaxStep`, `Tol3d`, `TolGuide` and `Fleche`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RstRstWalkSettings {
    /// `MaxStep`: the largest step on the guide.
    pub max_step: f64,
    /// `Tol3d`: the 3d tolerance of the points.
    pub tol3d: f64,
    /// `TolGuide`: the tolerance on the guide parameter.
    pub tol_guide: f64,
    /// `Fleche`: the deflection of the sections.
    pub fleche: f64,
}

/// The inverse functions of a restriction-restriction marching, per restriction: the
/// section where the function leaves it (`Finv1`, `Finv2`) and the section through its
/// end point (`FinvP1`, `FinvP2`).
pub struct RstRstInverses<'a> {
    /// `Finv1`, `Finv2`.
    pub finv: [&'a mut dyn BlendSurfCurvFuncInv; 2],
    /// `FinvP1`, `FinvP2`.
    pub finv_p: [&'a mut dyn BlendCurvPointFuncInv; 2],
}

impl BRepBlendRstRstLineBuilder {
    /// `Perform(Func, Finv1, FinvP1, Finv2, FinvP2, Pdep, Pmax, MaxStep, Tol3d, TolGuide,
    /// Soldep, Fleche, Appro)`: the marching from `pdep` to `pmax` from the solution
    /// `soldep` (solved first and checked inside when `appro`).
    /// `builder.perform(&mut f, inverses, (0.0, 1.0), settings, &soldep, false)?`.
    pub fn perform(
        &mut self,
        func: &mut dyn BlendRstRstFunction,
        inv: RstRstInverses<'_>,
        (pdep, pmax): (f64, f64),
        settings: RstRstWalkSettings,
        soldep: &Vector<'_>,
        appro: bool,
    ) -> Result<(), Failure> {
        self.done = false;
        self.start(settings, (pdep, pmax));
        self.fleche = settings.fleche.abs();
        self.pasmax = settings.max_step.abs();
        self.param = pdep;
        func.set_param(self.param)?;
        if !self.first_point(func, soldep, appro)? {
            return Ok(());
        }
        // Update the line.
        lock(self.line_handle()?).append(self.previous_p);
        self.set_first_extremities()?;
        self.internal_perform(func, inv, pmax)?;
        self.done = true;
        Ok(())
    }

    /// The common start of Perform and PerformFirstSection: a new line, the tolerances
    /// and the direction.
    fn start(&mut self, settings: RstRstWalkSettings, (pdep, pmax): (f64, f64)) {
        self.iscomplete = false;
        self.comptra = false;
        self.new_line();
        self.tolpoint3d = settings.tol3d;
        self.tolgui = settings.tol_guide.abs();
        self.rebrou = false;
        self.sens = if pmax - pdep >= 0.0 { 1.0 } else { -1.0 };
    }

    /// The first section from `soldep` (solved and checked inside when `appro`): whether
    /// it is accepted.
    fn first_point(
        &mut self,
        func: &mut dyn BlendRstRstFunction,
        soldep: &Vector<'_>,
        appro: bool,
    ) -> Result<bool, Failure> {
        let start: Sol = marching::values_of(soldep)?;
        if appro {
            if !self.solve_first(func, &start)? {
                return Ok(false);
            }
            if !self.check_inside(func)?.inside() {
                return Ok(false);
            }
        } else {
            self.sol = start;
        }
        Ok(self.test_arret(func, false, Status::Ok)? == Status::Ok)
    }

    /// The solve of a first section from `start` (30 iterations): `sol` gets the root;
    /// false when the solver fails.
    fn solve_first(
        &mut self,
        func: &mut dyn BlendRstRstFunction,
        start: &Sol,
    ) -> Result<bool, Failure> {
        let (mut tolerance, mut inf, mut sup) = (
            math_vector(&[0.0; 2])?,
            math_vector(&[0.0; 2])?,
            math_vector(&[0.0; 2])?,
        );
        func.get_tolerance(&mut tolerance, self.tolpoint3d)?;
        func.get_bounds(&mut inf, &mut sup)?;
        let mut solver = solver_for(func, &tolerance, 30)?;
        let root = root_of(&mut solver, func, start, (&inf, &sup));
        func.take_failure()?;
        let Some(root) = root? else {
            return Ok(false);
        };
        self.sol = root;
        Ok(true)
    }

    /// The extremities of the line at its first point, on both restrictions.
    fn set_first_extremities(&mut self) -> Result<(), Failure> {
        let previous = self.previous_p;
        let (u, v) = (previous.parameter_on_c1()?, previous.parameter_on_c2()?);
        let (w, tol) = (previous.parameter(), self.tolpoint3d);
        let mut ptf1 = BRepBlendExtremity::on_curve(previous.point_on_c1(), u, w, tol);
        let mut ptf2 = BRepBlendExtremity::on_curve(previous.point_on_c2(), v, w, tol);
        if !previous.is_tangency_point() {
            ptf1.set_tangent(previous.tangent_on_c1()?);
            ptf2.set_tangent(previous.tangent_on_c2()?);
        }
        let mut line = lock(self.line_handle()?);
        if self.sens > 0.0 {
            line.set_start_points(ptf1, ptf2);
        } else {
            line.set_end_points(ptf1, ptf2);
        }
        Ok(())
    }

    /// `PerformFirstSection(Func, Finv1, FinvP1, Finv2, FinvP2, Pdep, Pmax, Soldep,
    /// Tol3d, TolGuide, RecRst1, RecP1, RecRst2, RecP2, Psol, ParSol)`: the first section
    /// on the restrictions asked for (where the section leaves a restriction, the end
    /// point of a restriction): its guide parameter and its solution, or `None`.
    pub fn perform_first_section(
        &mut self,
        func: &mut dyn BlendRstRstFunction,
        inv: RstRstInverses<'_>,
        (pdep, pmax): (f64, f64),
        soldep: &Vector<'_>,
        settings: RstRstWalkSettings,
        rec: [(bool, bool); 2],
    ) -> Result<Option<(f64, Sol)>, Failure> {
        self.done = false;
        self.start(settings, (pdep, pmax));
        let mut g = Grounding::new(pmax);
        self.param = pdep;
        func.set_param(self.param)?;
        if !self.solve_first(func, &marching::values_of(soldep)?)? {
            return Ok(None);
        }
        let sides = rec.into_iter().zip(inv.finv).zip(inv.finv_p);
        for (side, (((rec_rst, rec_p), finv), finv_p)) in sides.enumerate() {
            if rec_rst {
                g.reframe_on_rst(self, side, func, finv)?;
            }
            if rec_p {
                g.reframe_on_end_point(self, side, finv_p)?;
            }
        }
        if !g.any() {
            return Ok(None);
        }
        let state = self.ground(func, &mut g, false, Status::OnRst12)?;
        self.test_arret(func, false, state)?;
        Ok(Some((self.param, self.sol)))
    }

    /// `Complete(Func, Finv1, FinvP1, Finv2, FinvP2, Pmin)`: the marching from the other
    /// end of the line down to `pmin`, once; StdFail_NotDone before a marching.
    pub fn complete(
        &mut self,
        func: &mut dyn BlendRstRstFunction,
        inv: RstRstInverses<'_>,
        pmin: f64,
    ) -> Result<bool, Failure> {
        if !self.done {
            return Err(marching::not_done(CLASS, "Complete"));
        }
        if self.iscomplete {
            return Ok(true);
        }
        self.previous_p = {
            let line = lock(self.line_handle()?);
            let index = if self.sens > 0.0 { 1 } else { line.nb_points() };
            *line.point(index)?
        };
        self.sens = -self.sens;
        self.param = self.previous_p.parameter();
        self.sol = [
            self.previous_p.parameter_on_c1()?,
            self.previous_p.parameter_on_c2()?,
        ];
        self.internal_perform(func, inv, pmin)?;
        self.iscomplete = true;
        Ok(true)
    }
}
