// Rust translation of BRepBlend_Walking::TestArret and CheckDeflection
// (BRepBlend_Walking.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The test of a computed section: is it a solution, is the step right for the
//! deflection, and the transitions of the line on both surfaces.
use super::{BRepBlendWalking, live, math_vector, sol_of};
use crate::blend::{BlendFunction, BlendPoint, SectionTangents, Status};
use crate::chfi_ds::lock;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_surf::IntSurfTypeTrans;
use rustycad_tkmath::gp::{Point3, Vector2, Vector3};

/// `CosRef3D` of CheckDeflection: rule set by tests in U4, 11.478 degrees.
const COS_REF_3D: f64 = 0.98;
/// `CosRef2D` of CheckDeflection: 25 degrees.
const COS_REF_2D: f64 = 0.88;

/// The 3d and 2d data of one end of a section, for the deflection test.
struct EndData {
    point: Point3,
    tangent: Vector3,
    uv: (f64, f64),
    tangent2d: Vector2,
}

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
        let (mut state1, mut state2) = if test_defl && self.check {
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
        for s in [&mut state1, &mut state2] {
            if *s == Status::Backward {
                *s = Status::StepTooLarge;
                self.rebrou = true;
            }
        }
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
        if state1 == Status::Ok || state2 == Status::Ok {
            self.previous_p = curpoint;
            return Ok(state);
        }
        if state1 == Status::StepTooSmall && state2 == Status::StepTooSmall {
            self.previous_p = curpoint;
            return Ok(if state == Status::Ok {
                Status::StepTooSmall
            } else {
                state
            });
        }
        Ok(if state == Status::Ok {
            Status::SamePoints
        } else {
            state
        })
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
        let corde = Vector3::between(prev.point, cur.point);
        let norme = corde.square_magnitude();
        let toler3d = 0.01 * self.tolpoint3d;
        if norme <= toler3d * toler3d {
            // il faudra peut etre forcer meme point
            return Ok(Status::SamePoints);
        }
        if let Some(status) = self.check_angles_3d(corde, norme, (&cur, &prev), tangents) {
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
        self.check_fleche(&cur, &prev, norme)
    }

    /// The 3d angles of the chord with the previous and the current tangents: `None`
    /// when the step passes them.
    fn check_angles_3d(
        &self,
        corde: Vector3,
        norme: f64,
        (cur, prev): (&EndData, &EndData),
        (cur_tangent, prev_tangent): (bool, bool),
    ) -> Option<Status> {
        let toler3d = 0.01 * self.tolpoint3d;
        if prev_tangent {
            let prev_norme = prev.tangent.square_magnitude();
            if prev_norme <= toler3d * toler3d {
                return Some(Status::SamePoints);
            }
            let cosi = self.sens * corde.dot(prev.tangent);
            if cosi < 0.0 {
                // angle 3d > pi/2 --> retour arriere
                return Some(Status::Backward);
            }
            if cosi * cosi / prev_norme / norme < COS_REF_3D {
                return Some(Status::StepTooLarge);
            }
        }
        if cur_tangent {
            let cosi = self.sens * corde.dot(cur.tangent);
            let cosi2 = cosi * cosi / cur.tangent.square_magnitude() / norme;
            if cosi2 < COS_REF_3D || cosi < 0.0 {
                return Some(Status::StepTooLarge);
            }
        }
        None
    }

    /// Estimation de la fleche courante: the deflection of the step against the
    /// requested one.
    fn check_fleche(&self, cur: &EndData, prev: &EndData, norme: f64) -> Result<Status, Failure> {
        let gap = *prev.tangent.normalized()?.xyz() - *cur.tangent.normalized()?.xyz();
        let fleche_courante = gap.square_modulus() * norme / 64.0;
        if fleche_courante <= 0.25 * self.fleche * self.fleche {
            return Ok(Status::StepTooSmall);
        }
        if fleche_courante > self.fleche * self.fleche {
            // pas trop grand : commentaire interessant
            return Ok(Status::StepTooLarge);
        }
        Ok(Status::Ok)
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

/// The point, the tangents (when `tangent`) and the parameters (when `with_2d`) of one
/// end of a line point.
fn end_data(
    point: &BlendPoint,
    on_first: bool,
    tangent: bool,
    with_2d: bool,
) -> Result<EndData, Failure> {
    let mut data = EndData {
        point: if on_first {
            point.point_on_s1()
        } else {
            point.point_on_s2()
        },
        tangent: Vector3::default(),
        uv: (0.0, 0.0),
        tangent2d: Vector2::default(),
    };
    if tangent {
        data.tangent = if on_first {
            point.tangent_on_s1()?
        } else {
            point.tangent_on_s2()?
        };
    }
    if !with_2d {
        return Ok(data);
    }
    data.uv = if on_first {
        point.parameters_on_s1()?
    } else {
        point.parameters_on_s2()?
    };
    if tangent {
        data.tangent2d = if on_first {
            point.tangent2d_on_s1()?
        } else {
            point.tangent2d_on_s2()?
        };
    }
    Ok(data)
}
