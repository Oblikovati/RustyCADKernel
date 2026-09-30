// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve.cxx and ModelingData/TKG3d/Geom/Geom_BSplineCurve.cxx (Segment) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Segment`: restrict the curve to `[U1, U2]`.
use super::{BSplineCurveCore, BSplineCurveKind};
use crate::geom2d::arrays::{at, one_based, put};
use crate::geom2d::errors;
use crate::geom2d::native::{std_max, std_min};
use crate::geom2d::spline_data::slice;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, real};
use rustycad_tkmath::bsplclib;
// The parameters that the steps of Segment pass to each other.
struct SegmentPlan {
    u1: f64,
    u2: f64,
    new_u1: f64,
    new_u2: f64,
    located_u2: f64,
    du: f64,
    ddu: f64,
    eps: f64,
    was_periodic: bool,
}

impl<K: BSplineCurveKind> BSplineCurveCore<K> {
    fn locate_in_knots(&self, u: f64) -> Result<(i32, f64), Failure> {
        let (mut index, mut new_u): (i32, f64) = (0, 0.0);
        let (lower, upper) = (self.knots.lower(), self.knots.upper());
        bsplclib::locate_parameter(
            self.deg,
            &self.knots,
            &self.mults,
            u,
            self.periodic,
            lower,
            upper,
            &mut index,
            &mut new_u,
        )?;
        Ok((index, new_u))
    }
    /// `Segment(U1, U2, theTolerance)`; Standard_DomainError when `U2 < U1` or, on a
    /// periodic curve, when `U2 - U1` exceeds the period by more than PConfusion;
    /// `core.segment(0.2, 0.8, precision::p_confusion())?`.
    pub fn segment(&mut self, u1: f64, u2: f64, tolerance: f64) -> Result<(), Failure> {
        if u2 < u1 {
            return Err(errors::domain(&format!(
                "{}::Segment: U1 = {u1}, U2 = {u2}; expected U1 <= U2",
                K::BSPLINE_CLASS
            )));
        }
        self.clear_eval_rep_on_edit();
        let mut plan: SegmentPlan = self.segment_plan(u1, u2, tolerance)?;
        let mut knots: Array1<'static, f64> = one_based(2)?;
        put(&mut knots, 1, std_min(plan.new_u1, plan.new_u2))?;
        put(&mut knots, 2, std_max(plan.new_u1, plan.new_u2))?;
        let mut mults: Array1<'static, i32> = one_based(2)?;
        mults.init(self.deg);
        self.insert_knots(&knots, &mults, plan.eps, false)?;
        if self.periodic {
            self.set_segment_origin(&mut plan)?;
        }
        self.keep_segment(&plan)
    }
    fn segment_plan(&self, u1: f64, u2: f64, tolerance: f64) -> Result<SegmentPlan, Failure> {
        let (mut du, mut ddu): (f64, f64) = (0.0, 0.0);
        if self.periodic {
            let period: f64 = self.last_parameter() - self.first_parameter();
            du = u2 - u1;
            if du - period > precision::p_confusion() {
                return Err(errors::domain(&format!(
                    "{}::Segment: U2 - U1 = {du}; expected <= the period {period}",
                    K::BSPLINE_CLASS
                )));
            }
            if du > period {
                du = period;
            }
            ddu = du;
        }
        let (_, new_u1) = self.locate_in_knots(u1)?;
        let (_, new_u2) = self.locate_in_knots(u2)?;
        let mut abs_u_max: f64 = std_max(new_u1.abs(), new_u2.abs());
        let range_max: f64 = std_max(self.first_parameter().abs(), self.last_parameter().abs());
        abs_u_max = std_max(abs_u_max, range_max);
        let eps: f64 = std_max(real::epsilon(abs_u_max), tolerance);
        Ok(SegmentPlan {
            u1,
            u2,
            new_u1,
            new_u2,
            located_u2: new_u2,
            du,
            ddu,
            eps,
            was_periodic: self.periodic,
        })
    }
    // A periodic curve gets its origin at NewU1 and becomes non-periodic.
    fn set_segment_origin(&mut self, plan: &mut SegmentPlan) -> Result<(), Failure> {
        let (mut index, u) = self.locate_in_knots(plan.u1)?;
        if (at(&self.knots, index + 1)? - u).abs() <= plan.eps {
            index += 1;
        }
        self.set_origin(index)?;
        self.set_not_periodic()?;
        plan.new_u2 = plan.new_u1 + plan.du;
        Ok(())
    }
    // Keep the knots, multiplicities, poles and weights between NewU1 and NewU2.
    fn keep_segment(&mut self, plan: &SegmentPlan) -> Result<(), Failure> {
        let (index1, index2) = self.segment_knot_range(plan)?;
        let shift: f64 = if plan.du > 0.0 {
            plan.new_u1 - plan.u1
        } else {
            plan.du
        };
        let (mut knots, mults) = self.segment_knots(index1, index2, shift)?;
        let pindex1: i32 = bsplclib::pole_index(self.deg, index1, self.periodic, &self.mults)? + 1;
        let pindex2: i32 = (bsplclib::pole_index(self.deg, index2, self.periodic, &self.mults)?
            + 1)
        .min(self.poles.len() as i32);
        let poles: Array1<'static, K::Point> = slice(&self.poles, pindex1, pindex2)?;
        let weights: Option<Array1<'static, f64>> = match self.rational {
            true => Some(slice(&self.weights, pindex1, pindex2)?),
            false => None,
        };
        if plan.was_periodic {
            let (lower, upper) = (knots.lower(), knots.upper());
            put(&mut knots, lower, plan.u1)?;
            if plan.located_u2 < plan.u2 {
                put(&mut knots, upper, plan.u1 + plan.ddu)?;
            }
        }
        (self.knots, self.mults, self.poles) = (knots, mults, poles);
        match weights {
            Some(weights) => self.weights = weights,
            None => self.reset_unit_weights()?,
        }
        self.max_deriv_inv_ok = false;
        self.update_knots()
    }
    fn segment_knot_range(&self, plan: &SegmentPlan) -> Result<(i32, i32), Failure> {
        let (mut index1, u) = self.locate_in_knots(plan.new_u1)?;
        if (at(&self.knots, index1 + 1)? - u).abs() <= plan.eps {
            index1 += 1;
        }
        let (mut index2, u) = self.locate_in_knots(plan.new_u2)?;
        if (at(&self.knots, index2 + 1)? - u).abs() <= plan.eps || index2 == index1 {
            index2 += 1;
        }
        Ok((index1, index2))
    }
    fn segment_knots(
        &self,
        index1: i32,
        index2: i32,
        shift: f64,
    ) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
        let count: i32 = index2 - index1 + 1;
        let mut knots: Array1<'static, f64> = one_based(count)?;
        let mut mults: Array1<'static, i32> = one_based(count)?;
        for (k, i) in (index1..=index2).enumerate() {
            put(&mut knots, k as i32 + 1, at(&self.knots, i)? - shift)?;
            put(&mut mults, k as i32 + 1, at(&self.mults, i)?)?;
        }
        put(&mut mults, 1, self.deg + 1)?;
        put(&mut mults, count, self.deg + 1)?;
        Ok((knots, mults))
    }
}
