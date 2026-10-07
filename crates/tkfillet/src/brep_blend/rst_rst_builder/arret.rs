// Rust translation of BRepBlend_RstRstLineBuilder::TestArret, CheckDeflectionOnRst1,
// CheckDeflectionOnRst2 and CheckInside (BRepBlend_RstRstLineBuilder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The test of a computed section of the restriction-restriction marching: is it a
//! solution, is the step right for the deflection on both restrictions, the transitions
//! of the line, and where the section is compared with both restrictions.
use super::BRepBlendRstRstLineBuilder;
use crate::blend::{BlendPoint, BlendRstRstFunction, DecrochStatus, SectionTangents, Status};
use crate::brep_blend::marching::{
    Deflection, end_data, math_vector, step_outcome, without_backward,
};
use crate::chfi_ds::lock;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, State};
use rustycad_tkgeomalgo::int_surf::IntSurfTypeTrans;
use rustycad_tkmath::gp::Vector2;

/// Where a section is on both restrictions (`SituOnC1`, `SituOnC2`), and which one it
/// leaves (`Decroch`).
pub(super) struct Inside {
    pub(super) situ: [State; 2],
    pub(super) decroch: DecrochStatus,
}

impl Inside {
    /// Inside both restrictions, on both.
    pub(super) fn inside(&self) -> bool {
        self.situ == [State::In, State::In] && self.decroch == DecrochStatus::NoDecroch
    }
}

/// `ConvOrToTra(O)`: IN for FORWARD, else OUT.
fn conv_or_to_tra(o: Orientation) -> IntSurfTypeTrans {
    if o == Orientation::Forward {
        IntSurfTypeTrans::In
    } else {
        IntSurfTypeTrans::Out
    }
}

impl BRepBlendRstRstLineBuilder {
    /// `TestArret(Func, TestDeflection, State)`: whether the solution is a section; then
    /// the deflection test on both restrictions and the transitions of the line. It
    /// returns `State` when the step is accepted.
    pub(super) fn test_arret(
        &mut self,
        func: &mut dyn BlendRstRstFunction,
        test_deflection: bool,
        state: Status,
    ) -> Result<Status, Failure> {
        if !func.is_solution(&math_vector(&self.sol)?, self.tolpoint3d)? {
            return Ok(Status::StepTooLarge);
        }
        let curpoint = self.current_point(func)?;
        let raw = if test_deflection {
            (
                self.check_deflection(true, &curpoint)?,
                self.check_deflection(false, &curpoint)?,
            )
        } else {
            (Status::Ok, Status::Ok)
        };
        let ((state1, state2), backward) = without_backward(raw);
        self.rebrou |= backward;
        if state1 == Status::StepTooLarge || state2 == Status::StepTooLarge {
            return Ok(Status::StepTooLarge);
        }
        if !self.comptra && !curpoint.is_tangency_point() {
            self.compute_transitions(&curpoint)?;
        }
        let (status, keep) = step_outcome((state1, state2), state);
        if keep {
            self.previous_p = curpoint;
        }
        Ok(status)
    }

    /// The point of the solution, with its tangents unless it is a tangency point.
    fn current_point(&self, func: &mut dyn BlendRstRstFunction) -> Result<BlendPoint, Failure> {
        let (pt1, pt2) = (func.point_on_rst1(), func.point_on_rst2());
        let (p1, p2) = (func.pnt2d_on_rst1(), func.pnt2d_on_rst2());
        let uv = [p1.x(), p1.y(), p2.x(), p2.y()];
        if func.is_tangency_point() {
            return Ok(BlendPoint::on_rsts(pt1, pt2, self.param, uv, self.sol));
        }
        let tangents = SectionTangents {
            tg1: func.tangent_on_rst1()?,
            tg2: func.tangent_on_rst2()?,
            tg12d: func.tangent2d_on_rst1()?,
            tg22d: func.tangent2d_on_rst2()?,
        };
        Ok(BlendPoint::on_rsts_with_tangents(
            pt1, pt2, self.param, uv, self.sol, tangents,
        ))
    }

    /// `CheckDeflectionOnRst1` (`first`) or `CheckDeflectionOnRst2`: the controls 3d of
    /// Blend_CSWalking on one restriction.
    fn check_deflection(&self, first: bool, cur_point: &BlendPoint) -> Result<Status, Failure> {
        let tangents = (
            !cur_point.is_tangency_point(),
            !self.previous_p.is_tangency_point(),
        );
        let cur = end_data(cur_point, first, tangents.0, false)?;
        let prev = end_data(&self.previous_p, first, tangents.1, false)?;
        let deflection = Deflection {
            tolpoint3d: self.tolpoint3d,
            sens: self.sens,
            fleche: self.fleche,
        };
        deflection.check_3d(&cur, &prev, tangents)
    }

    /// The transitions of the line on both restrictions, from their orientations in their
    /// domains, once they are decided.
    fn compute_transitions(&mut self, curpoint: &BlendPoint) -> Result<(), Failure> {
        let Some(trarst1) = self.rst_transition(0, curpoint.tangent2d_on_s1()?)? else {
            return Ok(());
        };
        let Some(trarst2) = self.rst_transition(1, curpoint.tangent2d_on_s2()?)? else {
            return Ok(());
        };
        self.comptra = true;
        lock(self.line_handle()?).set_transitions(trarst1, trarst2);
        Ok(())
    }

    /// The transition on the restriction of `side` from the 2d tangent of the section
    /// there; `None` when it is not decided (the tangents nearly normal).
    fn rst_transition(
        &self,
        side: usize,
        tg2drst: Vector2,
    ) -> Result<Option<IntSurfTypeTrans>, Failure> {
        let (_, tg2drstref) = self.live_rst(side)?.d1(self.sol[side])?;
        let testra = tg2drst.dot(tg2drstref);
        let or = self.domain[side].arc_orientation(&self.rst[side])?;
        if testra.abs() <= self.tolpoint3d {
            return Ok(None);
        }
        Ok(Some(if testra < 0.0 {
            conv_or_to_tra(top_abs::reverse(or))
        } else if testra > 0.0 {
            conv_or_to_tra(or)
        } else {
            IntSurfTypeTrans::Undecided
        }))
    }

    /// `CheckInside(Func, SituOnC1, SituOnC2, Decroch)`: both restriction parameters
    /// against their restrictions (the second tolerance aside), and the unhooking of the
    /// section.
    pub(super) fn check_inside(
        &self,
        func: &mut dyn BlendRstRstFunction,
    ) -> Result<Inside, Failure> {
        let mut tolerance = math_vector(&[0.0; 2])?;
        func.get_tolerance(&mut tolerance, self.tolpoint3d)?;
        let tol2 = crate::blend_func::linalg::vget(&tolerance, 2)?;
        let mut situ = [State::Unknown; 2];
        for (side, s) in situ.iter_mut().enumerate() {
            // face pcurve
            let v = self.sol[side];
            let rst = self.live_rst(side)?;
            let (first, last) = (rst.first_parameter()?, rst.last_parameter()?);
            *s = if v < first - tol2 || v > last + tol2 {
                State::Out
            } else if v > first && v < last {
                State::In
            } else {
                State::On
            };
        }
        // lost contact
        let (decroch, _) = func.decroch(&math_vector(&self.sol)?)?;
        Ok(Inside { situ, decroch })
    }
}
