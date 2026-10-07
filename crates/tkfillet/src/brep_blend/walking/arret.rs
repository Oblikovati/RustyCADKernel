// Rust translation of BRepBlend_Walking::TestArret and CheckDeflection
// (BRepBlend_Walking.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The test of a computed section: is it a solution, is the step right for the
//! deflection, and the transitions of the line on both surfaces.
use super::{BRepBlendWalking, live, math_vector, sol_of};
use crate::blend::{BlendFunction, BlendPoint, SectionTangents, Status};
use crate::brep_blend::marching::{Deflection, EndData, end_data, step_outcome, without_backward};
use crate::chfi_ds::lock;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_surf::IntSurfTypeTrans;

/// `CosRef2D` of CheckDeflection: 25 degrees.
const COS_REF_2D: f64 = 0.88;

impl BRepBlendWalking {
    /// `TestArret(F, State, TestDeflection, TestSolution, TestLengthStep)`: whether the
    /// solution is a section; then the step check on both surfaces, the twists and the
    /// transitions of the line. It returns `State` when the step is accepted.
    pub(super) fn test_arret(
        &mut self,
        function: &mut dyn BlendFunction,
        state: Status,
        test_defl: bool,
        test_solu: bool,
        test_length_step: bool,
    ) -> Result<Status, Failure> {
        let mut tolsolu = self.tolpoint3d;
        if !test_solu {
            tolsolu *= 1000.0; // Ca doit toujours etre bon
        }
        if !function.is_solution(&math_vector(&self.sol)?, tolsolu)? {
            return Ok(Status::StepTooLarge);
        }
        let (curpoint, twists) = self.current_point(function)?;
        let states = self.step_states(function, &curpoint, test_defl, test_length_step)?;
        if states.0 == Status::StepTooLarge || states.1 == Status::StepTooLarge {
            return Ok(Status::StepTooLarge);
        }
        self.accept_point(function, curpoint, twists, state, states)
    }

    /// The step status on both surfaces: the deflection test, or only the length of the
    /// step; a backward step is too large and flagged (`rebrou`).
    fn step_states(
        &mut self,
        function: &mut dyn BlendFunction,
        curpoint: &BlendPoint,
        test_defl: bool,
        test_length_step: bool,
    ) -> Result<(Status, Status), Failure> {
        let raw = if test_defl && self.check {
            // Verification du critere de fleche sur chaque surface et sur la ligne guide
            (
                self.check_deflection(true, curpoint)?,
                self.check_deflection(false, curpoint)?,
            )
        } else if test_length_step {
            self.length_step_states(function, curpoint)?
        } else {
            (Status::Ok, Status::Ok)
        };
        let ((state1, state2), backward) = without_backward(raw);
        self.rebrou |= backward;
        Ok((state1, state2))
    }

    /// The end of TestArret once the step is not too large: the twists (decided only
    /// here, after the rejections, BUC60322), the transitions, and the status.
    fn accept_point(
        &mut self,
        function: &mut dyn BlendFunction,
        curpoint: BlendPoint,
        (loctwist1, loctwist2): (bool, bool),
        state: Status,
        (state1, state2): (Status, Status),
    ) -> Result<Status, Failure> {
        self.twistflag1 |= loctwist1;
        self.twistflag2 |= loctwist2;
        if !self.comptra && !curpoint.is_tangency_point() {
            self.compute_transitions(function, &curpoint, (loctwist1, loctwist2))?;
        }
        let (status, keep) = step_outcome((state1, state2), state);
        if keep {
            self.previous_p = curpoint;
        }
        Ok(status)
    }

    /// The point of the solution, with its tangents unless it is a tangency point, and
    /// the twists of the function.
    fn current_point(
        &self,
        function: &mut dyn BlendFunction,
    ) -> Result<(BlendPoint, (bool, bool)), Failure> {
        let (pt1, pt2) = (function.point_on_s1(), function.point_on_s2());
        if function.is_tangency_point() {
            let point = BlendPoint::on_surfaces(pt1, pt2, self.param, self.sol);
            return Ok((point, (false, false)));
        }
        let tangents = SectionTangents {
            tg1: function.tangent_on_s1()?,
            tg2: function.tangent_on_s2()?,
            tg12d: function.tangent2d_on_s1()?,
            tg22d: function.tangent2d_on_s2()?,
        };
        let point = BlendPoint::on_surfaces_with_tangents(pt1, pt2, self.param, self.sol, tangents);
        Ok((point, (function.twist_on_s1()?, function.twist_on_s2()?)))
    }

    /// Without the deflection test: only checks that the step is not too large (bad
    /// extensions), at most 5% of the domain of each variable.
    fn length_step_states(
        &self,
        function: &mut dyn BlendFunction,
        curpoint: &BlendPoint,
    ) -> Result<(Status, Status), Failure> {
        let (mut inf, mut sup) = (math_vector(&[0.0; 4])?, math_vector(&[0.0; 4])?);
        function.get_bounds(&mut inf, &mut sup)?;
        let (inf, sup) = (sol_of(&inf)?, sol_of(&sup)?);
        let max_step: Vec<f64> = (0..4).map(|i| (sup[i] - inf[i]) * 0.05).collect();
        let too_large = |cur: (f64, f64), prev: (f64, f64), first: usize| {
            (cur.0 - prev.0).abs() > max_step[first] || (cur.1 - prev.1).abs() > max_step[first + 1]
        };
        let pick = |large: bool| {
            if large {
                Status::StepTooLarge
            } else {
                Status::Ok
            }
        };
        let on1 = too_large(
            curpoint.parameters_on_s1()?,
            self.previous_p.parameters_on_s1()?,
            0,
        );
        let on2 = too_large(
            curpoint.parameters_on_s2()?,
            self.previous_p.parameters_on_s2()?,
            2,
        );
        Ok((pick(on1), pick(on2)))
    }

    /// The transitions of the line on both surfaces, from the tangent of the section and
    /// the normals, once they are decided.
    fn compute_transitions(
        &mut self,
        function: &mut dyn BlendFunction,
        curpoint: &BlendPoint,
        (loctwist1, loctwist2): (bool, bool),
    ) -> Result<(), Failure> {
        let [u1, v1, u2, v2] = self.sol;
        let frames = function.tangent(u1, v1, u2, v2)?;
        let (nor1, nor2) = (
            frames.norm_first.normalized()?,
            frames.norm_last.normalized()?,
        );
        let testra = frames.tg_first.dot(nor1.crossed(curpoint.tangent_on_s1()?));
        if testra.abs() <= precision::confusion() {
            return Ok(());
        }
        let out1 = (testra > 0.0 && !loctwist1) || (testra < 0.0 && loctwist1);
        let tras1 = if out1 {
            IntSurfTypeTrans::Out
        } else {
            IntSurfTypeTrans::In
        };
        let testra = frames.tg_last.dot(nor2.crossed(curpoint.tangent_on_s2()?));
        if testra.abs() <= precision::confusion() {
            return Ok(());
        }
        let in2 = (testra > 0.0 && !loctwist2) || (testra < 0.0 && loctwist2);
        let tras2 = if in2 {
            IntSurfTypeTrans::In
        } else {
            IntSurfTypeTrans::Out
        };
        self.comptra = true;
        lock(self.line_handle()?).set_transitions(tras1, tras2);
        Ok(())
    }

    /// `CheckDeflection(OnFirst, CurPoint)`: the step from the previous point on one
    /// surface, against the 3d angles, the 2d angles and the deflection.
    pub(super) fn check_deflection(
        &self,
        on_first: bool,
        cur_point: &BlendPoint,
    ) -> Result<Status, Failure> {
        let tangents = (
            !cur_point.is_tangency_point(),
            !self.previous_p.is_tangency_point(),
        );
        let cur = end_data(cur_point, on_first, tangents.0, self.check2d)?;
        let prev = end_data(&self.previous_p, on_first, tangents.1, self.check2d)?;
        let surface = live(self.surface(on_first))?;
        let tolu = surface.u_resolution(self.tolpoint3d)?;
        let tolv = surface.v_resolution(self.tolpoint3d)?;
        let deflection = Deflection {
            tolpoint3d: self.tolpoint3d,
            sens: self.sens,
            fleche: self.fleche,
        };
        let (corde, norme) = match deflection.chord(&cur, &prev) {
            Ok(chord) => chord,
            Err(status) => return Ok(status), // il faudra peut etre forcer meme point
        };
        if let Some(status) = deflection.check_angles((corde, norme), (&cur, &prev), tangents) {
            return Ok(status);
        }
        let check_2d = || self.check_deflection_2d(&cur, &prev, tangents, (tolu, tolv));
        if self.check2d
            && let Some(status) = check_2d()
        {
            return Ok(status);
        }
        if !tangents.0 || !tangents.1 {
            return Ok(Status::Ok);
        }
        deflection.check_fleche(&cur, &prev, norme)
    }

    /// The 2d part of CheckDeflection: `None` when the step passes it.
    fn check_deflection_2d(
        &self,
        cur: &EndData,
        prev: &EndData,
        (cur_tangent, prev_tangent): (bool, bool),
        (tolu, tolv): (f64, f64),
    ) -> Option<Status> {
        let (du, dv) = (cur.uv.0 - prev.uv.0, cur.uv.1 - prev.uv.1);
        let duv = du * du + dv * dv;
        if du.abs() < tolu && dv.abs() < tolv {
            // il faudra peut etre forcer meme point
            return Some(Status::SamePoints); // point confondu 2d
        }
        if prev_tangent {
            let d2d = prev.tangent2d;
            if d2d.x().abs() < tolu && d2d.y().abs() < tolv {
                return Some(Status::SamePoints); // point confondu 2d
            }
            if self.sens * (du * d2d.x() + dv * d2d.y()) < 0.0 {
                return Some(Status::Backward);
            }
        }
        if cur_tangent {
            let t2d = cur.tangent2d;
            let cosi = self.sens * (du * t2d.x() + dv * t2d.y()) / t2d.magnitude();
            if cosi * cosi / duv < COS_REF_2D || cosi < 0.0 {
                return Some(Status::StepTooLarge);
            }
        }
        None
    }
}
