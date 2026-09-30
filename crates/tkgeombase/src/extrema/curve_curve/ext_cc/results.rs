// Rust translation of the two Extrema_ExtCC::PrepareResults at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The results of Extrema_ExtCC: the extrema whose parameters, moved into the period
//! of a periodic curve, lie in the ranges within RealEpsilon().
use super::ExtCC;
use crate::extrema::ExtElC;
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::curve_tools::CurveTool;
use crate::extrema::pon::POnCurv;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;

impl ExtCC<'_> {
    // `U` moved into `[start, start + Period)` of a periodic curve (`rank` 0 or 1).
    fn periodic_parameter(&self, rank: usize, u: f64, start: f64) -> Result<f64, Failure> {
        let curve = self.curve_pair()?[rank];
        if CurveTool::is_periodic(curve)? {
            return Ok(elclib::in_period(
                u,
                start,
                start + CurveTool::period(curve)?,
            ));
        }
        Ok(u)
    }

    // Whether the parameters lie in the ranges within RealEpsilon().
    fn in_ranges(u: f64, u2: f64, [ut11, ut12, ut21, ut22]: [f64; 4]) -> bool {
        let eps: f64 = f64::EPSILON;
        (u >= ut11 - eps) && (u <= ut12 + eps) && (u2 >= ut21 - eps) && (u2 <= ut22 + eps)
    }

    /// `PrepareResults(Extrema_ExtElC, theIsInverse, Ut11, Ut12, Ut21, Ut22)` (protected).
    pub(super) fn prepare_results_el(
        &mut self,
        alg: &ExtElC,
        is_inverse: bool,
        ranges: [f64; 4],
    ) -> Result<(), Failure> {
        self.done = alg.is_done();
        if !self.done {
            return Ok(());
        }
        self.is_parallel = alg.is_parallel()?;
        if self.is_parallel {
            return self.prepare_parallel_result(ranges, alg.square_distance(1)?);
        }
        for i in 1..=alg.nb_ext()? {
            // Verification of the validity of parameters
            let (mut p1, mut p2) = alg.points(i)?;
            let (u, u2) = if is_inverse {
                (p2.parameter(), p1.parameter())
            } else {
                (p1.parameter(), p2.parameter())
            };
            let u: f64 = self.periodic_parameter(0, u, ranges[0])?;
            let u2: f64 = self.periodic_parameter(1, u2, ranges[2])?;
            if !Self::in_ranges(u, u2, ranges) {
                continue;
            }
            self.sq_dist.push(alg.square_distance(i)?);
            if is_inverse {
                p1.set_values(u2, *p1.value());
                p2.set_values(u, *p2.value());
                self.points.extend([p2, p1]);
            } else {
                p1.set_values(u, *p1.value());
                p2.set_values(u2, *p2.value());
                self.points.extend([p1, p2]);
            }
        }
        Ok(())
    }

    /// `PrepareResults(Extrema_ECC, Ut11, Ut12, Ut21, Ut22)` (protected).
    pub(super) fn prepare_results_ecc(&mut self, ranges: [f64; 4]) -> Result<(), Failure> {
        self.done = self.ecc.is_done();
        if !self.done {
            return Ok(());
        }
        self.is_parallel = self.ecc.is_parallel()?;
        if self.is_parallel {
            let sq_dist: f64 = self.ecc.square_distance(1)?;
            return self.prepare_parallel_result(ranges, sq_dist);
        }
        for i in 1..=self.ecc.nb_ext()? {
            let (p1, p2) = self.ecc.points(i)?;
            // Check points to be into param space.
            let u: f64 = self.periodic_parameter(0, p1.parameter(), ranges[0])?;
            let u2: f64 = self.periodic_parameter(1, p2.parameter(), ranges[2])?;
            if Self::in_ranges(u, u2, ranges) {
                self.sq_dist.push(self.ecc.square_distance(i)?);
                self.points
                    .extend([POnCurv::new(u, *p1.value()), POnCurv::new(u2, *p2.value())]);
            }
        }
        Ok(())
    }
}
