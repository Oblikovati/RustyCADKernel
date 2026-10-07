// Rust translation of BRepBlend_SurfRstLineBuilder::TestArret, CheckDeflectionOnSurf,
// CheckDeflectionOnRst and CheckInside (BRepBlend_SurfRstLineBuilder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The test of a computed section of the surface-restriction marching: is it a
//! solution, is the step right for the deflection on both ends, the transitions of the
//! line, and where the section is compared with both domains.
use super::BRepBlendSurfRstLineBuilder;
use crate::blend::{BlendPoint, BlendSurfRstFunction, SectionTangents, Status};
use crate::brep_blend::marching::{
    Deflection, end_data, math_vector, step_outcome, without_backward,
};
use crate::chfi_ds::lock;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, State};
use rustycad_tkgeomalgo::int_surf::IntSurfTypeTrans;
use rustycad_tkmath::gp::Point2;

/// Where a section is: on the restriction (`SituOnC`), on the surface domain
/// (`SituOnS`), and whether it leaves the restriction (`Decroch`).
pub(super) struct Inside {
    pub(super) situ_on_c: State,
    pub(super) situ_on_s: State,
    pub(super) decroch: bool,
}

impl Inside {
    /// Inside the restriction and the surface domain, on the restriction.
    pub(super) fn inside(&self) -> bool {
        self.situ_on_c == State::In && self.situ_on_s == State::In && !self.decroch
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

impl BRepBlendSurfRstLineBuilder {
    /// `TestArret(Func, TestDeflection, State)`: whether the solution is a section; then
    /// the deflection test on both ends and the transitions of the line. It returns
    /// `State` when the step is accepted.
    pub(super) fn test_arret(
        &mut self,
        func: &mut dyn BlendSurfRstFunction,
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
        let ((state_s, state_rst), backward) = without_backward(raw);
        self.rebrou |= backward;
        if state_s == Status::StepTooLarge || state_rst == Status::StepTooLarge {
            return Ok(Status::StepTooLarge);
        }
        if !self.comptra && !curpoint.is_tangency_point() {
            self.compute_transitions(func, &curpoint)?;
        }
        let (status, keep) = step_outcome((state_s, state_rst), state);
        if keep {
            self.previous_p = curpoint;
        }
        Ok(status)
    }

    /// The point of the solution, with its tangents unless it is a tangency point.
    fn current_point(&self, func: &mut dyn BlendSurfRstFunction) -> Result<BlendPoint, Failure> {
        let (pts, ptrst, pt2drst) = (func.point_on_s(), func.point_on_rst(), func.pnt2d_on_rst());
        let [u, v, w] = self.sol;
        let uv = [u, v, pt2drst.x(), pt2drst.y()];
        if func.is_tangency_point() {
            return Ok(BlendPoint::on_surface_rst(pts, ptrst, self.param, uv, w));
        }
        let tangents = SectionTangents {
            tg1: func.tangent_on_s()?,
            tg2: func.tangent_on_rst()?,
            tg12d: func.tangent2d_on_s()?,
            tg22d: func.tangent2d_on_rst()?,
        };
        Ok(BlendPoint::on_surface_rst_with_tangents(
            pts, ptrst, self.param, uv, w, tangents,
        ))
    }

    /// `CheckDeflectionOnSurf` (`on_surf`) or `CheckDeflectionOnRst`: the controls 3d of
    /// Blend_CSWalking on one end of the step.
    fn check_deflection(&self, on_surf: bool, cur_point: &BlendPoint) -> Result<Status, Failure> {
        let tangents = (
            !cur_point.is_tangency_point(),
            !self.previous_p.is_tangency_point(),
        );
        let cur = end_data(cur_point, on_surf, tangents.0, false)?;
        let prev = end_data(&self.previous_p, on_surf, tangents.1, false)?;
        let deflection = Deflection {
            tolpoint3d: self.tolpoint3d,
            sens: self.sens,
            fleche: self.fleche,
        };
        deflection.check_3d(&cur, &prev, tangents)
    }

    /// The transitions of the line on the surface (from the unhooking normal) and on the
    /// restriction (from its orientation in its domain), once they are decided.
    fn compute_transitions(
        &mut self,
        func: &mut dyn BlendSurfRstFunction,
        curpoint: &BlendPoint,
    ) -> Result<(), Failure> {
        let (_, nors, tgsecs) = func.decroch(&math_vector(&self.sol)?)?;
        let testra = tgsecs.dot(nors.normalized()?.crossed(curpoint.tangent_on_s()?));
        if testra.abs() <= self.tolpoint3d {
            return Ok(());
        }
        let tras = if testra < 0.0 {
            IntSurfTypeTrans::In
        } else if testra > 0.0 {
            IntSurfTypeTrans::Out
        } else {
            IntSurfTypeTrans::Undecided
        };
        let (_, tg2drstref) = self.live_rst()?.d1(self.sol[2])?;
        let testra = curpoint.tangent2d_on_s2()?.dot(tg2drstref);
        let or = self.domain2.arc_orientation(&self.rst)?;
        if testra.abs() <= 1.0e-8 {
            return Ok(());
        }
        let trarst = if testra < 0.0 {
            conv_or_to_tra(top_abs::reverse(or))
        } else if testra > 0.0 {
            conv_or_to_tra(or)
        } else {
            IntSurfTypeTrans::Undecided
        };
        self.comptra = true;
        lock(self.line_handle()?).set_transitions(tras, trarst);
        Ok(())
    }

    /// `CheckInside(Func, SituOnC, SituOnS, Decroch)`: the restriction parameter against
    /// the restriction (its tolerance aside), the surface point against the domain, and
    /// the unhooking of the section.
    pub(super) fn check_inside(
        &self,
        func: &mut dyn BlendSurfRstFunction,
    ) -> Result<Inside, Failure> {
        let mut tolerance = math_vector(&[0.0; 3])?;
        func.get_tolerance(&mut tolerance, self.tolpoint3d)?;
        let tol3 = crate::blend_func::linalg::vget(&tolerance, 3)?;
        // face pcurve.
        let w = self.sol[2];
        let rst = self.live_rst()?;
        let (first, last) = (rst.first_parameter()?, rst.last_parameter()?);
        let situ_on_c = if w < first - tol3 || w > last + tol3 {
            State::Out
        } else if w > first && w < last {
            State::In
        } else {
            State::On
        };
        // face surface
        let situ_on_s = self.domain1.classify(
            Point2::new(self.sol[0], self.sol[1]),
            self.tolpoint2d,
            false,
        )?;
        // lost contact
        let (decroch, _, _) = func.decroch(&math_vector(&self.sol)?)?;
        Ok(Inside {
            situ_on_c,
            situ_on_s,
            decroch,
        })
    }
}
