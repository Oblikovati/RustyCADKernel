// Rust translation of Perform of ModelingData/TKGeomBase/ExtremaPC2d/
// ExtremaPC2d_GridEvaluator.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC2d_GridEvaluator::Perform`, which never raises: every failure of its
//! search is caught and reported as NumericalError.
use super::{Domain1D, ExtremaResult, GridEvaluator, SearchMode, Status};
use crate::extrema_pc_core::{GridFailurePolicy, PcSpace, Space2d};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkmath::gp::Point2;

impl GridEvaluator {
    /// The extrema of `point` against `curve` over `domain` with the cached partition
    /// (`Perform`); InvalidInput when the partition does not cover the domain;
    /// `evaluator.perform(&adaptor, &point, &domain, 1e-9, SearchMode::MinMax)`.
    pub fn perform(
        &mut self,
        curve: &dyn Adaptor2dCurve2d,
        point: &Point2,
        domain: &Domain1D,
        tol: f64,
        mode: SearchMode,
    ) -> &ExtremaResult {
        // The 2D policy catches every failure inside the core: an error cannot escape it.
        debug_assert_eq!(Space2d::FAILURES, GridFailurePolicy::CatchAll);
        if self
            .perform_core(curve, point, domain, (tol, mode))
            .is_err()
        {
            let result: &mut ExtremaResult = self.result();
            result.clear();
            result.status = Status::NumericalError;
        }
        self.result()
    }
}
