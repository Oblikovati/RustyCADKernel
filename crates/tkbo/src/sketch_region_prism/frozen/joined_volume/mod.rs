// SPDX-License-Identifier: GPL-2.0-only
//! ADR0044: exact sign transport preserves the original representative's error bound.
mod access;
mod replay;
mod transport;
mod types;
use super::{AuthoredPrismId, joined_measures::*, joined_orientation::*, proof_join::*};
pub(crate) use types::*;
#[cfg(test)]
mod tests;

/// Interpret one bound integral; `interpret_joined_prism_volume(join, directions, measures, 1, cancel)`.
/// This conditional scalar grants no complete-operation or history authority.
pub(crate) fn interpret_joined_prism_volume(
    join: &JoinedPrismProof,
    directions: &JoinedPrismOrientationCertificate,
    measures: &JoinedPrismMeasures,
    version: u32,
    mut cancelled: impl FnMut() -> bool + Send,
) -> Result<JoinedPrismVolume, JoinedPrismVolumeFailure> {
    let operation = join.projection().operation;
    let run = VolumeRun {
        operation,
        version,
        entity: join.original().clone(),
    };
    run.admit(join, directions, measures)?;
    run.poll(VolumeCheckpoint::Begin, &mut cancelled)?;
    let result =
        transport::interpret(join, directions, measures).map_err(|cause| run.failure(cause))?;
    run.poll(VolumeCheckpoint::Finish, &mut cancelled)?;
    Ok(result)
}
struct VolumeRun {
    operation: AuthoredPrismId,
    version: u32,
    entity: super::FrozenOriginalKey,
}
impl VolumeRun {
    fn failure(&self, cause: JoinedPrismVolumeCause) -> JoinedPrismVolumeFailure {
        JoinedPrismVolumeFailure {
            operation: self.operation,
            requested_version: self.version,
            entity: self.entity.clone(),
            class: if matches!(cause, JoinedPrismVolumeCause::Cancelled(_)) {
                JoinedVolumeFailureClass::Cancelled
            } else {
                JoinedVolumeFailureClass::Incomplete
            },
            cause,
        }
    }
    fn admit(
        &self,
        join: &JoinedPrismProof,
        directions: &JoinedPrismOrientationCertificate,
        measures: &JoinedPrismMeasures,
    ) -> Result<(), JoinedPrismVolumeFailure> {
        if self.version != 1 {
            return Err(self.failure(JoinedPrismVolumeCause::Version { expected: 1 }));
        }
        if !directions.matches_join(join) {
            return Err(self.failure(JoinedPrismVolumeCause::Binding(VolumeOperand::Orientation)));
        }
        if !measures.matches_join(join) {
            return Err(self.failure(JoinedPrismVolumeCause::Binding(VolumeOperand::Measures)));
        }
        Ok(())
    }
    fn poll(
        &self,
        checkpoint: VolumeCheckpoint,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), JoinedPrismVolumeFailure> {
        if cancelled() {
            return Err(self.failure(JoinedPrismVolumeCause::Cancelled(checkpoint)));
        }
        Ok(())
    }
}
