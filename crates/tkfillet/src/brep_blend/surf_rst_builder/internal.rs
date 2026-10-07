// Rust translation of BRepBlend_SurfRstLineBuilder::InternalPerform
// (BRepBlend_SurfRstLineBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The marching loop of the surface-restriction blend: the steps along the guide, each
//! one solved, compared with both domains, reframed where it leaves them, and tested.
use super::grounding::Grounding;
use super::perform::SurfRstInverses;
use super::{BRepBlendSurfRstLineBuilder, Sol};
use crate::blend::{BlendSurfRstFunction, Status};
use crate::brep_blend::BRepBlendExtremity;
use crate::brep_blend::marching::{math_vector, root_of, solver_for};
use crate::chfi_ds::lock;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::math::{FunctionSetRoot, Vector};

/// The state of the marching loop (the locals of InternalPerform, kept between steps).
struct SrMarch {
    inf: Vector<'static>,
    sup: Vector<'static>,
    solver: FunctionSetRoot,
    parinit: Sol,
    stepw: f64,
    parprec: f64,
    situ: (State, State),
    g: Grounding,
    exts: BRepBlendExtremity,
    extrst: BRepBlendExtremity,
    arrive: bool,
}

impl BRepBlendSurfRstLineBuilder {
    /// `InternalPerform(Func, Finv, FinvP, FinvC, Bound)`: the marching from the current
    /// point up to the guide parameter `bound`, the restrictions, or a stop.
    pub(super) fn internal_perform(
        &mut self,
        func: &mut dyn BlendSurfRstFunction,
        mut inv: SurfRstInverses<'_>,
        bound: f64,
    ) -> Result<(), Failure> {
        let stepw = self.first_step()?;
        let parprec = self.param;
        if self.sens * (parprec - bound) >= -self.tolgui {
            return Ok(());
        }
        let mut m = self.start_march(func, stepw, parprec)?;
        self.param = parprec + self.sens * m.stepw;
        if self.sens * (self.param - bound) > 0.0 {
            m.stepw = self.sens * (bound - parprec) * 0.5;
            self.param = parprec + self.sens * m.stepw;
        }
        while !m.arrive {
            let state = self.march_step(func, &mut inv, bound, &mut m)?;
            self.apply_status(state, bound, &mut m)?;
            if m.arrive {
                self.set_arrival_ends(&m)?;
            }
        }
        Ok(())
    }

    /// The first step: the last one of the line when it is not too small, else the
    /// largest step.
    fn first_step(&self) -> Result<f64, Failure> {
        let line = lock(self.line_handle()?);
        let nbp = line.nb_points();
        if nbp < 2 {
            return Ok(self.pasmax);
        }
        // The last step is reproduced if it is not too small.
        let stepw = if self.sens < 0.0 {
            line.point(2)?.parameter() - line.point(1)?.parameter()
        } else {
            line.point(nbp)?.parameter() - line.point(nbp - 1)?.parameter()
        };
        Ok(std_max(stepw, 100.0 * self.tolgui))
    }

    /// The solver of the loop and its starting state.
    fn start_march(
        &self,
        func: &mut dyn BlendSurfRstFunction,
        stepw: f64,
        parprec: f64,
    ) -> Result<SrMarch, Failure> {
        let (mut tolerance, mut inf, mut sup) = (
            math_vector(&[0.0; 3])?,
            math_vector(&[0.0; 3])?,
            math_vector(&[0.0; 3])?,
        );
        func.get_tolerance(&mut tolerance, self.tolpoint3d)?;
        func.get_bounds(&mut inf, &mut sup)?;
        let solver = solver_for(func, &tolerance, 30)?;
        Ok(SrMarch {
            inf,
            sup,
            solver,
            parinit: self.sol,
            stepw,
            parprec,
            situ: (State::Unknown, State::Unknown),
            g: Grounding::new(0.0),
            exts: BRepBlendExtremity::new(),
            extrst: BRepBlendExtremity::new(),
            arrive: false,
        })
    }

    /// One step: the solve, the checks inside, the reframings and the test; the status.
    fn march_step(
        &mut self,
        func: &mut dyn BlendSurfRstFunction,
        inv: &mut SurfRstInverses<'_>,
        bound: f64,
        m: &mut SrMarch,
    ) -> Result<Status, Failure> {
        func.set_param(self.param)?;
        let root = root_of(&mut m.solver, func, &m.parinit, (&m.inf, &m.sup));
        func.take_failure()?;
        let Some(root) = root? else {
            return Ok(Status::StepTooLarge);
        };
        self.sol = root;
        let inside = self.check_inside(func)?;
        (m.situ, m.g.decroch) = ((inside.situ_on_c, inside.situ_on_s), inside.decroch);
        if !inside.inside() && lock(self.line_handle()?).nb_points() == 1 {
            return Ok(Status::StepTooLarge);
        }
        m.g.reset(bound);
        let echecrecad = self.reframe_step(func, inv, m)?;
        m.g.decroch = false;
        if echecrecad && !m.g.any() {
            // Failed reframing. Leave with PointsConfondus
            return Ok(Status::SamePoints);
        }
        let state = self.ground(func, &mut m.g, true, Status::Ok)?;
        self.test_arret(func, true, state)
    }

    /// The reframings of the ends that left their domains: on the surface (with the
    /// unhooking reevaluated there, BUC60360), on the end point of the restriction, on
    /// the restriction where the section leaves it; whether one failed.
    fn reframe_step(
        &mut self,
        func: &mut dyn BlendSurfRstFunction,
        inv: &mut SurfRstInverses<'_>,
        m: &mut SrMarch,
    ) -> Result<bool, Failure> {
        let mut echecrecad = false;
        let (situonc, situons) = m.situ;
        if situons == State::Out || situons == State::On {
            // pb inverse rst/rst
            if m.g.reframe_on_surface(self, inv.finv_c)? {
                self.reevaluate_decroch(func, &mut m.g)?;
            } else {
                echecrecad = true;
            }
        }
        if (situonc == State::Out || situonc == State::On)
            && !m.g.reframe_on_end_point(self, inv.finv_p)?
        {
            echecrecad = true; // pb inverse point/surf
        }
        if m.g.decroch && !m.g.reframe_on_restriction(self, func, inv.finv)? {
            echecrecad = true; // pb inverse rst/surf
        }
        Ok(echecrecad)
    }

    /// It is necessary to reevaluate the deviation (BUC60360): the section on the arc of
    /// the surface reframing, and whether it leaves the restriction there.
    fn reevaluate_decroch(
        &mut self,
        func: &mut dyn BlendSurfRstFunction,
        g: &mut Grounding,
    ) -> Result<(), Failure> {
        func.set_param(g.ws)?;
        let p2d = crate::brep_blend::BRepBlendHCurve2dTool::value(&g.arc, g.solinvs[2])?;
        (g.u, g.v) = (p2d.x(), p2d.y());
        self.sol = [g.u, g.v, g.solinvs[1]];
        g.decroch = func.decroch(&math_vector(&self.sol)?)?.0;
        Ok(())
    }

    /// The line ends where the marching arrives, and whether they leave the restriction.
    fn set_arrival_ends(&mut self, m: &SrMarch) -> Result<(), Failure> {
        let line = self.line_handle()?.clone();
        if self.sens > 0.0 {
            lock(&line).set_end_points(m.exts.clone(), m.extrst.clone());
            self.decrochfin = m.g.decroch;
        } else {
            lock(&line).set_start_points(m.exts.clone(), m.extrst.clone());
            self.decrochdeb = m.g.decroch;
        }
        Ok(())
    }

    /// The switch on the status of a step.
    fn apply_status(&mut self, state: Status, bound: f64, m: &mut SrMarch) -> Result<(), Failure> {
        match state {
            Status::Ok => self.advance(bound, m, false),
            Status::StepTooSmall => self.advance(bound, m, true),
            Status::StepTooLarge => self.shorten(m),
            Status::OnRst1 | Status::OnRst2 | Status::OnRst12 => {
                self.stop_on_restrictions(state, m)
            }
            Status::SamePoints => self.stop_at_previous(m),
            Status::Backward => Ok(()),
        }
    }

    /// Update the line, then the next parameter (the step grown by half when it was too
    /// small), or the end on the bound.
    fn advance(&mut self, bound: f64, m: &mut SrMarch, grow: bool) -> Result<(), Failure> {
        self.push_previous()?;
        m.parinit = self.sol;
        m.parprec = self.param;
        if grow {
            m.stepw = std_min(1.5 * m.stepw, self.pasmax);
        }
        if self.param == bound {
            // Indicate end on Bound.
            m.arrive = true;
            let w = self.previous_p.parameter();
            m.exts.set_value(
                self.previous_p.point_on_s(),
                self.sol[0],
                self.sol[1],
                w,
                self.tolpoint3d,
            );
            let (rst, w3) = (self.rst.clone(), self.sol[2]);
            self.make_extremity(&mut m.extrst, false, (&rst, w3), &m.g.vtxrst)?;
            return Ok(());
        }
        self.param += self.sens * m.stepw;
        if self.sens * (self.param - bound) > -self.tolgui {
            self.param = bound;
        }
        Ok(())
    }

    /// The step halved, or the stop at the previous point below the guide tolerance.
    fn shorten(&mut self, m: &mut SrMarch) -> Result<(), Failure> {
        m.stepw /= 2.0;
        if m.stepw.abs() < self.tolgui {
            return self.stop_at_previous(m);
        }
        self.param = m.parprec + self.sens * m.stepw; // no risk to exceed Bound.
        Ok(())
    }

    /// The stop at the previous point (on the surface and on the restriction).
    fn stop_at_previous(&mut self, m: &mut SrMarch) -> Result<(), Failure> {
        let previous = self.previous_p;
        let (u, v) = previous.parameters_on_s()?;
        let (w, tol) = (previous.parameter(), self.tolpoint3d);
        m.exts.set_value(previous.point_on_s(), u, v, w, tol);
        m.extrst
            .set_value_on_curve(previous.point_on_c(), previous.parameter_on_c()?, w, tol);
        m.arrive = true;
        Ok(())
    }

    /// The stop on the surface domain (OnRst1), on the restriction (OnRst2) or both. The
    /// end on both takes the guide parameter of the surface reframing on its arc, as
    /// natively.
    fn stop_on_restrictions(&mut self, state: Status, m: &mut SrMarch) -> Result<(), Failure> {
        self.push_previous()?;
        let (arc, rst) = (m.g.arc.clone(), self.rst.clone());
        match state {
            Status::OnRst1 => {
                self.make_extremity(&mut m.exts, true, (&arc, m.g.solinvs[2]), &m.g.vtxs)?
            }
            Status::OnRst12 => {
                self.make_extremity(&mut m.exts, true, (&arc, m.g.solinvs[0]), &m.g.vtxs)?
            }
            _ => {
                let w = self.previous_p.parameter();
                m.exts.set_value(
                    self.previous_p.point_on_s(),
                    self.sol[0],
                    self.sol[1],
                    w,
                    self.tolpoint3d,
                );
            }
        }
        self.make_extremity(&mut m.extrst, false, (&rst, self.sol[2]), &m.g.vtxrst)?;
        m.arrive = true;
        Ok(())
    }
}
