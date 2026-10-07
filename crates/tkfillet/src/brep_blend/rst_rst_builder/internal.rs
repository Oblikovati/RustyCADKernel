// Rust translation of BRepBlend_RstRstLineBuilder::InternalPerform and MakeExtremity
// (and Transition through brep_blend::arc_ends) (BRepBlend_RstRstLineBuilder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The marching loop of the restriction-restriction blend: the steps along the guide,
//! each one solved, compared with both restrictions, reframed where it leaves them, and
//! tested; and the ends of the line on the restrictions.
use super::grounding::Grounding;
use super::perform::RstRstInverses;
use super::{BRepBlendRstRstLineBuilder, CLASS, Sol, VertexFlag};
use crate::blend::{BlendRstRstFunction, DecrochStatus, Status};
use crate::brep_blend::BRepBlendExtremity;
use crate::brep_blend::arc_ends::ArcEnds;
use crate::brep_blend::marching::{math_vector, root_of, solver_for};
use crate::chfi_ds::lock;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::math::{FunctionSetRoot, Vector};

/// The state of the marching loop (the locals of InternalPerform, kept between steps).
struct RrMarch {
    inf: Vector<'static>,
    sup: Vector<'static>,
    solver: FunctionSetRoot,
    parinit: Sol,
    stepw: f64,
    parprec: f64,
    situ: [State; 2],
    g: Grounding,
    ext: [BRepBlendExtremity; 2],
    arrive: bool,
}

impl BRepBlendRstRstLineBuilder {
    /// `InternalPerform(Func, Finv1, FinvP1, Finv2, FinvP2, Bound)`: the marching from
    /// the current point up to the guide parameter `bound`, the restrictions, or a stop.
    pub(super) fn internal_perform(
        &mut self,
        func: &mut dyn BlendRstRstFunction,
        mut inv: RstRstInverses<'_>,
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
        // The last step is redone if it is not too small.
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
        func: &mut dyn BlendRstRstFunction,
        stepw: f64,
        parprec: f64,
    ) -> Result<RrMarch, Failure> {
        let (mut tolerance, mut inf, mut sup) = (
            math_vector(&[0.0; 2])?,
            math_vector(&[0.0; 2])?,
            math_vector(&[0.0; 2])?,
        );
        func.get_tolerance(&mut tolerance, self.tolpoint3d)?;
        func.get_bounds(&mut inf, &mut sup)?;
        let solver = solver_for(func, &tolerance, 30)?;
        Ok(RrMarch {
            inf,
            sup,
            solver,
            parinit: self.sol,
            stepw,
            parprec,
            situ: [State::Unknown; 2],
            g: Grounding::new(0.0),
            ext: [BRepBlendExtremity::new(), BRepBlendExtremity::new()],
            arrive: false,
        })
    }

    /// One step: the solve, the checks inside, the reframings and the test; the status.
    fn march_step(
        &mut self,
        func: &mut dyn BlendRstRstFunction,
        inv: &mut RstRstInverses<'_>,
        bound: f64,
        m: &mut RrMarch,
    ) -> Result<Status, Failure> {
        func.set_param(self.param)?;
        let root = root_of(&mut m.solver, func, &m.parinit, (&m.inf, &m.sup));
        func.take_failure()?;
        let Some(root) = root? else {
            return Ok(Status::StepTooLarge);
        };
        self.sol = root;
        let inside = self.check_inside(func)?;
        (m.situ, m.g.decroch) = (inside.situ, inside.decroch);
        if !inside.inside() && lock(self.line_handle()?).nb_points() == 1 {
            return Ok(Status::StepTooLarge);
        }
        m.g.reset(bound);
        let echecrecad = self.reframe_step(func, inv, m)?;
        m.g.decroch = DecrochStatus::NoDecroch;
        if echecrecad && !m.g.any() {
            // reframing failed. Leave with PointsConfondus
            return Ok(Status::SamePoints);
        }
        let state = self.ground(func, &mut m.g, true, Status::Ok)?;
        self.test_arret(func, true, state)
    }

    /// The reframings of the step: at the end points of the restrictions it left, then
    /// where it leaves them (the unhooking status); whether one failed.
    fn reframe_step(
        &mut self,
        func: &mut dyn BlendRstRstFunction,
        inv: &mut RstRstInverses<'_>,
        m: &mut RrMarch,
    ) -> Result<bool, Failure> {
        let mut echecrecad = false;
        for side in 0..2 {
            // pb inversion point/surf
            if m.situ[side] != State::In
                && !m
                    .g
                    .reframe_on_end_point(self, side, &mut *inv.finv_p[side])?
            {
                echecrecad = true;
            }
        }
        let decroch = m.g.decroch;
        let lost = [
            matches!(
                decroch,
                DecrochStatus::DecrochRst1 | DecrochStatus::DecrochBoth
            ),
            matches!(
                decroch,
                DecrochStatus::DecrochRst2 | DecrochStatus::DecrochBoth
            ),
        ];
        for (side, lost) in lost.into_iter().enumerate() {
            // pb inversion rst/surf
            if lost && !m.g.reframe_on_rst(self, side, func, &mut *inv.finv[side])? {
                echecrecad = true;
            }
        }
        Ok(echecrecad)
    }

    /// The line ends where the marching arrives, and the restrictions it leaves there.
    fn set_arrival_ends(&mut self, m: &RrMarch) -> Result<(), Failure> {
        let line = self.line_handle()?.clone();
        let (e1, e2) = (m.ext[0].clone(), m.ext[1].clone());
        if self.sens > 0.0 {
            lock(&line).set_end_points(e1, e2);
            self.decrochfin = m.g.decroch;
        } else {
            lock(&line).set_start_points(e1, e2);
            self.decrochdeb = m.g.decroch;
        }
        Ok(())
    }

    /// The switch on the status of a step.
    fn apply_status(&mut self, state: Status, bound: f64, m: &mut RrMarch) -> Result<(), Failure> {
        match state {
            Status::Ok => self.advance(bound, m, false),
            Status::StepTooSmall => self.advance(bound, m, true),
            Status::StepTooLarge => self.shorten(m),
            Status::OnRst1 | Status::OnRst2 | Status::OnRst12 => self.stop_on_restrictions(m),
            Status::SamePoints => self.stop_at_previous(m),
            Status::Backward => Ok(()),
        }
    }

    /// Update the line, then the next parameter (the step grown by half when it was too
    /// small), or the end on the bound.
    fn advance(&mut self, bound: f64, m: &mut RrMarch, grow: bool) -> Result<(), Failure> {
        self.push_previous()?;
        m.parinit = self.sol;
        m.parprec = self.param;
        if grow {
            m.stepw = std_min(1.5 * m.stepw, self.pasmax);
        }
        if self.param == bound {
            // Show that end is on Bound.
            m.arrive = true;
            let p = self.previous_p;
            m.ext[0].set_value_on_curve(
                p.point_on_c1(),
                p.parameter_on_c1()?,
                p.parameter(),
                self.tolpoint3d,
            );
            let vtx = m.g.side[1].vtxrst.clone();
            return self.make_extremity(&mut m.ext[1], 1, self.sol[1], &vtx);
        }
        self.param += self.sens * m.stepw;
        if self.sens * (self.param - bound) > -self.tolgui {
            self.param = bound;
        }
        Ok(())
    }

    /// The step halved, or the stop at the previous point below the guide tolerance.
    fn shorten(&mut self, m: &mut RrMarch) -> Result<(), Failure> {
        m.stepw /= 2.0;
        if m.stepw.abs() < self.tolgui {
            return self.stop_at_previous(m);
        }
        self.param = m.parprec + self.sens * m.stepw; // there is no risk to exceed Bound.
        Ok(())
    }

    /// The stop at the previous point on both restrictions.
    fn stop_at_previous(&mut self, m: &mut RrMarch) -> Result<(), Failure> {
        let p = self.previous_p;
        let (w, tol) = (p.parameter(), self.tolpoint3d);
        m.ext[0].set_value_on_curve(p.point_on_c1(), p.parameter_on_c1()?, w, tol);
        m.ext[1].set_value_on_curve(p.point_on_c2(), p.parameter_on_c2()?, w, tol);
        m.arrive = true;
        Ok(())
    }

    /// The stop on one or both restrictions: the ends on both.
    fn stop_on_restrictions(&mut self, m: &mut RrMarch) -> Result<(), Failure> {
        self.push_previous()?;
        let (v1, v2) = (m.g.side[0].vtxrst.clone(), m.g.side[1].vtxrst.clone());
        self.make_extremity(&mut m.ext[0], 0, self.sol[0], &v1)?;
        self.make_extremity(&mut m.ext[1], 1, self.sol[1], &v2)?;
        m.arrive = true;
        Ok(())
    }

    /// `MakeExtremity(Extrem, OnFirst, Arc, Param, IsVtx, Vtx)`: the end at the previous
    /// point on the restriction of `side` at `param`, and on every arc through its
    /// vertex. The end on the second restriction takes the tangent on the first one, as
    /// natively.
    fn make_extremity(
        &self,
        extrem: &mut BRepBlendExtremity,
        side: usize,
        param: f64,
        vtx: &VertexFlag,
    ) -> Result<(), Failure> {
        let previous = &self.previous_p;
        let point = if side == 0 {
            previous.point_on_c1()
        } else {
            previous.point_on_c2()
        };
        extrem.set_value_on_curve(point, self.sol[side], previous.parameter(), self.tolpoint3d);
        if !previous.is_tangency_point() {
            extrem.set_tangent(previous.tangent_on_c1()?);
        }
        let line = lock(self.line_handle()?);
        let ends = ArcEnds {
            class: CLASS,
            line: &line,
            previous,
            sens: self.sens,
            on_first: side == 0,
            surface: self.surface(side)?,
            domain: &self.domain[side],
        };
        ends.add_arcs(extrem, (&self.rst[side], param), vtx.vertex())
    }
}
