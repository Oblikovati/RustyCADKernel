// SPDX-License-Identifier: GPL-2.0-only
//! ADR0040: bounded scalar reports tied to one immutable, caller-named prism proof.
mod access;
mod admission;
mod error;
mod replay;
mod types;
use super::proof_join::{JoinedPrismFace, JoinedPrismProof};
use super::{AuthoredPrismId, FrozenUseKey};
use crate::sketch_region_prism::{PrismImageRole, region_certificate::*};
pub(crate) use error::*;
use rustycad_tkmath::predicates::roots::metric_bounds::*;
pub(crate) use types::*;
#[cfg(test)]
mod tests;

/// Measure one owned proof; `inspect_joined_prism_measures(join, options, cancel)` never reads live shapes.
/// Scalar completion grants no body validity; ordered validation/history and a final source rebind remain required.
pub(crate) fn inspect_joined_prism_measures(
    join: &JoinedPrismProof,
    requested: JoinedPrismMeasureOptions,
    mut cancelled: impl FnMut() -> bool + Send,
) -> Result<JoinedPrismMeasures, Box<JoinedPrismMeasureFailure>> {
    use JoinedMeasureCheckpoint::{Begin, Finish};
    let run = JoinedMeasureRun { join, requested };
    let names =
        admission::admit(join, requested).map_err(|cause| failure(join, requested, cause))?;
    run.poll(Begin, &mut cancelled)?;
    let measures = run.measure(&mut cancelled)?;
    run.poll(Finish, &mut cancelled)?;
    Ok(run.complete(names, measures))
}
struct JoinedMeasureRun<'a> {
    join: &'a JoinedPrismProof,
    requested: JoinedPrismMeasureOptions,
}
impl JoinedMeasureRun<'_> {
    fn measure(
        &self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<CompletePrismMeasures, Box<JoinedPrismMeasureFailure>> {
        measure_bound_prism(
            self.join.embedding().binding(),
            self.join.projection().limits().extrusion,
            self.requested.scalars,
            cancelled,
        )
        .map_err(|cause| {
            failure(
                self.join,
                self.requested,
                JoinedPrismMeasureCause::Arithmetic(Box::new(cause)),
            )
        })
    }
    fn complete(
        &self,
        names: [JoinedPrismFace; 3],
        measures: CompletePrismMeasures,
    ) -> JoinedPrismMeasures {
        JoinedPrismMeasures {
            requested: self.requested,
            names,
            measures,
            binding: self.join.replay_bytes().into(),
        }
    }
    fn poll(
        &self,
        checkpoint: JoinedMeasureCheckpoint,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), Box<JoinedPrismMeasureFailure>> {
        if cancelled() {
            return Err(failure(
                self.join,
                self.requested,
                JoinedPrismMeasureCause::Cancelled(checkpoint),
            ));
        }
        Ok(())
    }
}
