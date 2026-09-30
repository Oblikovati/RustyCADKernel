// Rust translation of the default case of Extrema_GGExtPC::Perform at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The offset and other curves of Extrema_GGExtPC: each C2 interval (or, for a C2
//! curve, each interval of `DeflCurvIntervals`) that meets the range is searched with
//! a sample count proportional to its length; a sign change of `(C - P).C'` across an
//! interval bound is an extremum at that bound.
use super::GGExtPC;
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::gp_space::{GpPoint, GpVector};
use crate::extrema::pon::POnCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::Shape;

const MAX_SAMPLE: i32 = 17;

impl<'a, T: ExtremaCurveTool> GGExtPC<'a, T> {
    // The bounds of the C2 intervals, or of the deflection intervals of a C2 curve.
    fn interval_bounds(curve: &T::Curve) -> Result<Vec<f64>, Failure> {
        let n: i32 = T::nb_intervals(curve, Shape::C2)?;
        if n <= 1 {
            return T::defl_curv_intervals(curve);
        }
        let mut bounds: Array1<'static, f64> =
            Array1::with_bounds(1, n + 1).map_err(|error| error.into_failure("Intervals"))?;
        T::intervals(curve, &mut bounds, Shape::C2)?;
        Ok((1..=n + 1).map(|i| bounds[i]).collect())
    }

    /// The default case of `Perform` (offset and other curves).
    pub(super) fn perform_general(
        &mut self,
        point: T::Point,
        curve: &'a T::Curve,
    ) -> Result<(), Failure> {
        let bounds: Vec<f64> = Self::interval_bounds(curve)?;
        let n: usize = bounds.len() - 1;
        self.sample = (self.sample / n as i32).max(MAX_SAMPLE);
        let mut max_int: f64 = 0.;
        for i in 0..n {
            let dt: f64 = bounds[i + 1] - bounds[i];
            if max_int < dt {
                max_int = dt;
            }
        }
        let is_periodic: bool = T::is_periodic(curve)?;
        let mut s2: f64 = 0.0;
        let mut int_ext_done: bool = false;
        self.ext_pc.initialize_curve(curve)?;
        for i in 1..=n {
            self.int_uinf = bounds[i - 1];
            self.int_usup = bounds[i];
            let ratio: f64 = f64::from(MAX_SAMPLE) * (self.int_usup - self.int_uinf) / max_int;
            self.sample = real_to_int(ratio).max(3);
            if !self.interval_meets_range(curve, is_periodic)? {
                continue;
            }
            if i != 1 {
                let (pp, v1) = T::d1(curve, self.int_uinf)?;
                let s1: f64 = point.vector_to(&pp).dot_with(&v1);
                if s1 * s2 < 0.0 {
                    let sq_dist: f64 = pp.square_distance_to(&point);
                    self.push_extremum(sq_dist, s1 < 0.0, POnCurve::new(self.int_uinf, pp));
                }
            }
            if i != n {
                let (pp, v1) = T::d1(curve, self.int_usup)?;
                s2 = point.vector_to(&pp).dot_with(&v1);
            }
            self.interval_perform(point)?;
            int_ext_done = int_ext_done || self.done;
        }
        self.done = int_ext_done;
        Ok(())
    }

    // Whether the current interval (moved into the period starting at `myuinf` for a
    // periodic curve) meets the range over more than the parameter tolerance (the
    // native `continue` tests, negated as written so that NaN behaves the same).
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn interval_meets_range(&self, curve: &T::Curve, is_periodic: bool) -> Result<bool, Failure> {
        let (mut inf_to_check, mut sup_to_check) = (self.int_uinf, self.int_usup);
        if is_periodic {
            let period: f64 = T::period(curve)?;
            inf_to_check = elclib::in_period(self.int_uinf, self.uinf, self.uinf + period);
            sup_to_check = self.int_usup + (inf_to_check - self.int_uinf);
        }
        if (self.uinf > sup_to_check) || (self.usup < inf_to_check) {
            return Ok(false);
        }
        if self.uinf >= inf_to_check {
            inf_to_check = self.uinf;
        }
        if self.usup <= sup_to_check {
            sup_to_check = self.usup;
        }
        Ok(!((sup_to_check - inf_to_check) <= self.tol_u))
    }
}
