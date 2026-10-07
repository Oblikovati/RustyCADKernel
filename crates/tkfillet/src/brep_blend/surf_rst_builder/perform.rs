// Rust translation of BRepBlend_SurfRstLineBuilder::Perform, PerformFirstSection and
// Complete (BRepBlend_SurfRstLineBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The start of the marching (the first section, in the domain or on a restriction) and
//! its completion in the other direction.
use super::grounding::Grounding;
use super::{BRepBlendSurfRstLineBuilder, CLASS, Sol};
use crate::blend::{
    BlendFuncInv, BlendSurfCurvFuncInv, BlendSurfPointFuncInv, BlendSurfRstFunction, Status,
};
use crate::brep_blend::BRepBlendExtremity;
use crate::brep_blend::marching::{self, math_vector, root_of, solver_for};
use crate::chfi_ds::lock;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;

/// The settings of `Perform`: `MaxStep`, `Tol3d`, `Tol2d`, `TolGuide` and `Fleche`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SurfRstWalkSettings {
    /// `MaxStep`: the largest step on the guide.
    pub max_step: f64,
    /// `Tol3d`: the 3d tolerance of the points.
    pub tol3d: f64,
    /// `Tol2d`: the 2d tolerance of the classification on the surface.
    pub tol2d: f64,
    /// `TolGuide`: the tolerance on the guide parameter.
    pub tol_guide: f64,
    /// `Fleche`: the deflection of the sections.
    pub fleche: f64,
}

/// The inverse functions of a surface-restriction marching: on the restriction
/// (`Finv`), at its end point (`FinvP`), and on an arc of the surface (`FinvC`).
pub struct SurfRstInverses<'a> {
    /// `Finv`: the section ending on the restriction (the function leaves it).
    pub finv: &'a mut dyn BlendFuncInv,
    /// `FinvP`: the section through the end point of the restriction.
    pub finv_p: &'a mut dyn BlendSurfPointFuncInv,
    /// `FinvC`: the section ending on an arc of the surface.
    pub finv_c: &'a mut dyn BlendSurfCurvFuncInv,
}

impl BRepBlendSurfRstLineBuilder {
    /// `Perform(Func, Finv, FinvP, FinvC, Pdep, Pmax, MaxStep, Tol3d, Tol2d, TolGuide,
    /// Soldep, Fleche, Appro)`: the marching from `pdep` to `pmax` from the solution
    /// `soldep` (solved first and checked inside when `appro`).
    /// `builder.perform(&mut f, inverses, (0.0, 1.0), settings, &soldep, false)?`.
    pub fn perform(
        &mut self,
        func: &mut dyn BlendSurfRstFunction,
        inv: SurfRstInverses<'_>,
        (pdep, pmax): (f64, f64),
        settings: SurfRstWalkSettings,
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
    fn start(&mut self, settings: SurfRstWalkSettings, (pdep, pmax): (f64, f64)) {
        self.iscomplete = false;
        self.comptra = false;
        self.new_line();
        self.tolpoint3d = settings.tol3d;
        self.tolpoint2d = settings.tol2d;
        self.tolgui = settings.tol_guide.abs();
        self.rebrou = false;
        self.sens = if pmax - pdep >= 0.0 { 1.0 } else { -1.0 };
    }

    /// The first section from `soldep` (solved and checked inside when `appro`): whether
    /// it is accepted.
    fn first_point(
        &mut self,
        func: &mut dyn BlendSurfRstFunction,
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
        func: &mut dyn BlendSurfRstFunction,
        start: &Sol,
    ) -> Result<bool, Failure> {
        let (mut tolerance, mut inf, mut sup) = (
            math_vector(&[0.0; 3])?,
            math_vector(&[0.0; 3])?,
            math_vector(&[0.0; 3])?,
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

    /// The extremities of the line at its first point. The point on the restriction
    /// gets the parameters of the point on the surface, as natively.
    fn set_first_extremities(&mut self) -> Result<(), Failure> {
        let previous = self.previous_p;
        let (u, v) = previous.parameters_on_s()?;
        let (w, tol) = (previous.parameter(), self.tolpoint3d);
        let mut ptf1 = BRepBlendExtremity::on_surface(previous.point_on_s(), u, v, w, tol);
        let mut ptf2 = BRepBlendExtremity::on_surface(previous.point_on_c(), u, v, w, tol);
        if !previous.is_tangency_point() {
            ptf1.set_tangent(previous.tangent_on_s()?);
            ptf2.set_tangent(previous.tangent_on_c()?);
        }
        let mut line = lock(self.line_handle()?);
        if self.sens > 0.0 {
            line.set_start_points(ptf1, ptf2);
        } else {
            line.set_end_points(ptf1, ptf2);
        }
        Ok(())
    }

    /// `PerformFirstSection(Func, Finv, FinvP, FinvC, Pdep, Pmax, Soldep, Tol3d, Tol2d,
    /// TolGuide, RecRst, RecP, RecS, Psol, ParSol)`: the first section on the
    /// restrictions asked for (the restriction, its end point, the surface domain): its
    /// guide parameter and its solution, or `None`.
    pub fn perform_first_section(
        &mut self,
        func: &mut dyn BlendSurfRstFunction,
        inv: SurfRstInverses<'_>,
        (pdep, pmax): (f64, f64),
        soldep: &Vector<'_>,
        settings: SurfRstWalkSettings,
        (rec_rst, rec_p, rec_s): (bool, bool, bool),
    ) -> Result<Option<(f64, Sol)>, Failure> {
        self.done = false;
        self.start(settings, (pdep, pmax));
        let mut g = Grounding::new(pmax);
        self.param = pdep;
        func.set_param(self.param)?;
        if !self.solve_first(func, &marching::values_of(soldep)?)? {
            return Ok(None);
        }
        if rec_s {
            g.reframe_on_surface(self, inv.finv_c)?;
        }
        if rec_p {
            g.reframe_on_end_point(self, inv.finv_p)?;
        }
        if rec_rst {
            g.reframe_on_restriction(self, func, inv.finv)?;
        }
        if !g.any() {
            return Ok(None);
        }
        let state = self.ground(func, &mut g, false, Status::OnRst12)?;
        self.test_arret(func, false, state)?;
        Ok(Some((self.param, self.sol)))
    }

    /// `Complete(Func, Finv, FinvP, FinvC, Pmin)`: the marching from the other end of the
    /// line down to `pmin`, once; StdFail_NotDone before a marching.
    pub fn complete(
        &mut self,
        func: &mut dyn BlendSurfRstFunction,
        inv: SurfRstInverses<'_>,
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
        self.previous_parameters()?;
        self.internal_perform(func, inv, pmin)?;
        self.iscomplete = true;
        Ok(true)
    }
}
