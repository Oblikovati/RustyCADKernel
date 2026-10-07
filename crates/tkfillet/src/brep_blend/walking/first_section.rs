// Rust translation of BRepBlend_Walking::Perform and PerformFirstSection(F, Pdep, ParDep,
// Tol3d, TolGuide, Pos1, Pos2) (BRepBlend_Walking.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The start of a walking: the first section in the domains.
use super::{BRepBlendWalking, Sol, math_vector, root_of, sol_of, solver_for};
use crate::blend::{BlendFuncInv, BlendFunction, Status};
use crate::brep_blend::BRepBlendExtremity;
use crate::chfi_ds::lock;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::Vector;

/// The settings of `Perform`: `MaxStep`, `Tol3d`, `TolGuide` and `Fleche`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WalkingTolerances {
    /// `MaxStep`: the largest step on the guide.
    pub max_step: f64,
    /// `Tol3d`: the 3d tolerance of the points.
    pub tol3d: f64,
    /// `TolGuide`: the tolerance on the guide parameter.
    pub tol_guide: f64,
    /// `Fleche`: the deflection of the sections.
    pub fleche: f64,
}

/// The function bounds and tolerances of a solve (`GetTolerance`, `GetBounds`).
pub(super) struct SolveSetup {
    pub(super) tolerance: Vector<'static>,
    pub(super) inf: Vector<'static>,
    pub(super) sup: Vector<'static>,
}

impl BRepBlendWalking {
    /// `Perform(F, FInv, Pdep, Pmax, MaxStep, Tol3d, TolGuide, Soldep, Fleche, Appro)`:
    /// the walking from `pdep` to `pmax`, from the solution `soldep` (solved first when
    /// `appro`). `walking.perform(&mut f, &mut finv, (0.0, 1.0), settings, &soldep, false)?`.
    pub fn perform(
        &mut self,
        func: &mut dyn BlendFunction,
        func_inv: &mut dyn BlendFuncInv,
        (pdep, pmax): (f64, f64),
        settings: WalkingTolerances,
        soldep: &Vector<'_>,
        appro: bool,
    ) -> Result<(), Failure> {
        let doextremities = self.start_walking(settings, (pdep, pmax));
        func.set_param(self.param)?;
        if !self.first_point(func, soldep, appro)? {
            return Ok(());
        }
        // Mettre a jour la ligne. Correct first parameter if needed
        if self.to_correct_on_rst1 || self.to_correct_on_rst2 {
            self.previous_p.set_parameter(self.corrected_param);
        }
        lock(self.line_handle()?).append(self.previous_p);
        if doextremities {
            self.set_first_extremities()?;
        }
        self.internal_perform(func, func_inv, pmax)?;
        self.done = true;
        Ok(())
    }

    /// The settings and the direction of a new walking from `pdep`; whether the
    /// extremities of the line are to be set.
    fn start_walking(&mut self, settings: WalkingTolerances, (pdep, pmax): (f64, f64)) -> bool {
        self.done = false;
        self.iscomplete = false;
        self.comptra = false;
        let doextremities = self.reset_line();
        self.tolpoint3d = settings.tol3d;
        self.tolgui = settings.tol_guide.abs();
        self.fleche = settings.fleche.abs();
        self.rebrou = false;
        self.pasmax = settings.max_step.abs();
        self.sens = if pmax - pdep >= 0.0 { 1.0 } else { -1.0 };
        self.param = pdep;
        doextremities
    }

    /// The first section from `soldep` (solved first and classified when `appro`):
    /// whether it is a section inside the domains.
    fn first_point(
        &mut self,
        func: &mut dyn BlendFunction,
        soldep: &Vector<'_>,
        appro: bool,
    ) -> Result<bool, Failure> {
        if appro {
            if !self.solve_in_domains(func, &sol_of(soldep)?)? {
                return Ok(false);
            }
        } else {
            self.sol = sol_of(soldep)?;
        }
        Ok(self.test_arret(func, Status::Ok, false, true, false)? == Status::Ok)
    }

    /// A new line, or the old one cleared; whether the extremities are to be set (only
    /// on a new line).
    fn reset_line(&mut self) -> bool {
        match &self.line {
            None => {
                self.new_line();
                true
            }
            Some(line) => {
                lock(line).clear();
                false
            }
        }
    }

    /// The solve of the first section from `start`, accepted only inside both domains
    /// (when classified).
    fn solve_in_domains(
        &mut self,
        func: &mut dyn BlendFunction,
        start: &Sol,
    ) -> Result<bool, Failure> {
        let Some(setup) = self.solve_first(func, start)? else {
            return Ok(false);
        };
        let situ1 = self.situation(true, &setup.tolerance)?;
        let situ2 = self.situation(false, &setup.tolerance)?;
        Ok(situ1 == State::In && situ2 == State::In)
    }

    /// The solve of a first section from `start` (30 iterations): `sol` gets the root;
    /// `None` when the solver fails.
    pub(super) fn solve_first(
        &mut self,
        func: &mut dyn BlendFunction,
        start: &Sol,
    ) -> Result<Option<SolveSetup>, Failure> {
        let setup = self.solve_setup(func)?;
        let mut solver = solver_for(func, &setup.tolerance, 30)?;
        let root = root_of(&mut solver, func, start, (&setup.inf, &setup.sup));
        func.take_failure()?;
        let Some(root) = root? else {
            return Ok(None);
        };
        self.sol = root;
        Ok(Some(setup))
    }

    /// The common start of both PerformFirstSection: a new line and the tolerances.
    pub(super) fn start_first_section(&mut self, (tol3d, tol_guide): (f64, f64)) {
        self.iscomplete = false;
        self.comptra = false;
        self.new_line();
        self.tolpoint3d = tol3d;
        self.tolgui = tol_guide.abs();
    }

    /// `GetTolerance(tolerance, tolpoint3d)` and `GetBounds(infbound, supbound)`.
    pub(super) fn solve_setup(&self, func: &mut dyn BlendFunction) -> Result<SolveSetup, Failure> {
        let mut setup = SolveSetup {
            tolerance: math_vector(&[0.0; 4])?,
            inf: math_vector(&[0.0; 4])?,
            sup: math_vector(&[0.0; 4])?,
        };
        func.get_tolerance(&mut setup.tolerance, self.tolpoint3d)?;
        func.get_bounds(&mut setup.inf, &mut setup.sup)?;
        Ok(setup)
    }

    /// The classification of the solution on a domain, IN when not classified on it.
    pub(super) fn situation(
        &self,
        on_first: bool,
        tolerance: &Vector<'_>,
    ) -> Result<State, Failure> {
        let classify = if on_first {
            self.clason_s1
        } else {
            self.clason_s2
        };
        if !classify {
            return Ok(State::In);
        }
        self.classify_on(on_first, tolerance)
    }

    /// `domain->Classify(gp_Pnt2d(sol), min(tolerances), false)` on one domain.
    fn classify_on(&self, on_first: bool, tolerance: &Vector<'_>) -> Result<State, Failure> {
        let t = sol_of(tolerance)?;
        let (domain, point, tol) = if on_first {
            (
                &self.domain1,
                Point2::new(self.sol[0], self.sol[1]),
                std_min(t[0], t[1]),
            )
        } else {
            (
                &self.domain2,
                Point2::new(self.sol[2], self.sol[3]),
                std_min(t[2], t[3]),
            )
        };
        domain.classify(point, tol, false)
    }

    /// The extremities of a new line at its first point, on both surfaces.
    fn set_first_extremities(&mut self) -> Result<(), Failure> {
        let previous = &self.previous_p;
        let [u1, v1, u2, v2] = self.sol;
        // The 4-argument constructor is BRepBlend_Extremity(P, W, Param, Tol).
        let mut ptf1 =
            BRepBlendExtremity::on_curve(previous.point_on_s1(), u1, v1, self.tolpoint3d);
        let mut ptf2 =
            BRepBlendExtremity::on_curve(previous.point_on_s2(), u2, v2, self.tolpoint3d);
        if !previous.is_tangency_point() {
            ptf1.set_tangent(previous.tangent_on_s1()?);
            ptf2.set_tangent(previous.tangent_on_s2()?);
        }
        self.set_line_ends(ptf1, ptf2, self.sens > 0.0)
    }

    /// The start points (`at_start`) or the end points of the line.
    pub(super) fn set_line_ends(
        &self,
        ext1: BRepBlendExtremity,
        ext2: BRepBlendExtremity,
        at_start: bool,
    ) -> Result<(), Failure> {
        let mut line = lock(self.line_handle()?);
        if at_start {
            line.set_start_points(ext1, ext2);
        } else {
            line.set_end_points(ext1, ext2);
        }
        Ok(())
    }

    /// `PerformFirstSection(F, Pdep, ParDep, Tol3d, TolGuide, Pos1, Pos2)`: the first
    /// section from `par_dep` (replaced by the solution), and its classification on both
    /// domains; true when it is inside both.
    /// `let (ok, pos1, pos2) = walking.perform_first_section(&mut f, 0.0, &mut start, (1e-7, 1e-6))?;`.
    pub fn perform_first_section(
        &mut self,
        func: &mut dyn BlendFunction,
        pdep: f64,
        par_dep: &mut Vector<'_>,
        (tol3d, tol_guide): (f64, f64),
    ) -> Result<(bool, State, State), Failure> {
        self.start_first_section((tol3d, tol_guide));
        self.param = pdep;
        func.set_param(self.param)?;
        let Some(setup) = self.solve_first(func, &sol_of(par_dep)?)? else {
            return Ok((false, State::Unknown, State::Unknown));
        };
        for (i, value) in (1..).zip(self.sol) {
            crate::blend_func::linalg::vset(par_dep, i, value)?;
        }
        let pos1 = self.classify_on(true, &setup.tolerance)?;
        let pos2 = self.classify_on(false, &setup.tolerance)?;
        if pos1 != State::In || pos2 != State::In {
            return Ok((false, pos1, pos2));
        }
        self.test_arret(func, Status::Ok, false, true, false)?;
        Ok((true, pos1, pos2))
    }
}
