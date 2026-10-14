// SPDX-License-Identifier: GPL-2.0-only
//! Raw descriptor counts use the original polynomial and cumulative-limit policy before capture.
use super::{failure::TwoLoopRegionFailure, geometry, request::*, work::TwoLoopWork};
pub(in crate::sketch_region_prism) fn admit_two_loop_polynomial_counts(
    request: &TwoLoopRegionRequest<'_, '_>, counts: &[i32], reserved_units: usize,
) -> Result<usize, Box<TwoLoopRegionFailure<'static>>> {
    let mut progress = NoGeometryCallback;
    let mut work = TwoLoopWork::new(request.identities.operation, request.requested, &mut progress);
    work.stage = TwoLoopRegionStage::Membership;
    work.units("retained hierarchy and history rows before geometry", reserved_units)?;
    work.units("actual geometric descriptor", counts.len())?;
    for count in counts { geometry::poles(*count, &mut work)?; }
    Ok(work.retained_units)
}
struct NoGeometryCallback;
impl TwoLoopRegionProgress for NoGeometryCallback {
    fn checkpoint(&mut self, _: TwoLoopRegionStage) -> Result<bool, rustycad_tkernel::standard::Failure> { Ok(false) }
}
