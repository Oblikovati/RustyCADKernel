// Rust translation of the end of GCPnts_TangentialDeflection::PerformCurve
// (GCPnts_TangentialDeflection.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The refinement after the march: the second-to-last point moves to the middle of
//! the last two spans, midpoints are added up to the minimum number of points, and
//! every span whose maximal deflection or angle exceeds the criteria is split there.
use super::super::estim_defl::estim_angl;
use crate::gcpnts::{TCurveTypes, TangentialDeflection};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

impl TangentialDeflection {
    /// The three refinement steps.
    pub(super) fn refine<C: TCurveTypes + ?Sized>(&mut self, curve: &C) -> Result<(), Failure> {
        self.readjust_second_to_last(curve)?;
        self.reach_minimum_points(curve)?;
        self.split_deflected_spans(curve)
    }

    // "Readjustment of the second to last point": with at least three points, the
    // second-to-last one moves to the middle of the third-to-last and the range end.
    fn readjust_second_to_last<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
    ) -> Result<(), Failure> {
        let i: i32 = self.my_points.length() - 1;
        if i < 2 {
            return Ok(());
        }
        let mut middle_u: f64 = self.my_points.parameter(i - 1)?;
        middle_u = (self.my_last_u + middle_u) * 0.5;
        let middle_point: Point3 = curve.pnts_curve().lifted_d0(middle_u)?;
        self.my_points.set(i, middle_u, middle_point)
    }

    // "Add points at midpoints of segments if the minimum number of points is not yet
    // reached": each pass splits every other span, its bound following the growing
    // count, as natively.
    fn reach_minimum_points<C: TCurveTypes + ?Sized>(&mut self, curve: &C) -> Result<(), Failure> {
        let mut nbp: i32 = self.my_points.length();
        while nbp < self.my_min_nb_pnts {
            let mut i: i32 = 2;
            while i <= nbp {
                let middle_u: f64 =
                    (self.my_points.parameter(i - 1)? + self.my_points.parameter(i)?) * 0.5;
                let middle_point: Point3 = curve.pnts_curve().lifted_d0(middle_u)?;
                self.my_points.insert_before(i, middle_u, middle_point)?;
                nbp += 1;
                i += 2;
            }
        }
        Ok(())
    }

    // "Additional check for intervals": a span is split at its point of maximal
    // deflection and checked again, up to ten times the initial count of points.
    fn split_deflected_spans<C: TCurveTypes + ?Sized>(&mut self, curve: &C) -> Result<(), Failure> {
        let min_len2: f64 = self.my_min_len * self.my_min_len;
        let mut nbp: i32 = self.my_points.length();
        let max_nbp: i32 = 10 * nbp;
        let mut i: i32 = 1;
        while i < nbp {
            if !self.split_span(curve, i, min_len2)? {
                i += 1;
                continue;
            }
            nbp += 1;
            // i must point to first part of split interval
            if nbp > max_nbp {
                break;
            }
        }
        Ok(())
    }

    // Span `i`: split at the maximum of EstimDefl when the deflection exceeds the linear
    // deflection or the angle half the angular one, the maximum is more than UTol inside
    // the span and more than MinLen from both ends; true when split.
    fn split_span<C: TCurveTypes + ?Sized>(
        &mut self,
        curve: &C,
        i: i32,
        min_len2: f64,
    ) -> Result<bool, Failure> {
        let u1: f64 = self.my_points.parameter(i)?;
        let u2: f64 = self.my_points.parameter(i + 1)?;
        if u2 - u1 <= self.my_u_tol {
            return Ok(false);
        }
        // Check maximal deflection on interval;
        let (dmax, umax) = self.estim_defl(curve, u1, u2)?;
        let p1: Point3 = self.my_points.point(i)?;
        let p2: Point3 = self.my_points.point(i + 1)?;
        let middle_point: Point3 = curve.pnts_curve().lifted_d0(umax)?;
        let amax: f64 = estim_angl(p1, middle_point, p2);
        if !(dmax > self.my_curvature_deflection || amax > self.my_angular_deflection * 0.5) {
            return Ok(false);
        }
        if !(umax - u1 > self.my_u_tol && u2 - umax > self.my_u_tol) {
            return Ok(false);
        }
        if p1.square_distance(&middle_point) > min_len2
            && p2.square_distance(&middle_point) > min_len2
        {
            self.my_points.insert_after(i, umax, middle_point)?;
            return Ok(true);
        }
        Ok(false)
    }
}
