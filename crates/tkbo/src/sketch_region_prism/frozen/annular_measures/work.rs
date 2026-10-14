// SPDX-License-Identifier: GPL-2.0-only
use super::*;

pub(super) struct MeasureWork<'a> {
    pub join: &'a AnnularFrozenDomain,
    pub requested: AnnularMeasureOptions,
    pub boundaries: [AnnularBoundaryDirection; 2],
    pub phase: AnnularMeasurePhase,
    pub entity: Option<FrozenUseKey>,
    pub partial: AnnularMeasurePartial,
}
impl MeasureWork<'_> {
    pub(super) fn failure(self, cause: AnnularMeasureCause) -> Box<AnnularMeasureFailure> {
        Box::new(AnnularMeasureFailure {
            operation: self.join.projection().operation,
            phase: self.phase,
            entity: self.entity,
            requested: self.requested,
            binding: self.join.replay_bytes().into(),
            partial: self.partial,
            cause,
        })
    }
    pub(super) fn complete(self) -> AnnularPrismMeasures {
        let mut result = AnnularPrismMeasures {
            requested: self.requested,
            binding: self.join.replay_bytes().into(),
            boundaries: self.boundaries,
            faces: self.partial.faces,
            signed_sweep: self.partial.signed_sweep.expect("completed sweep stage"),
            total: self.partial.total.expect("completed total stage"),
            side_subdivisions: self.partial.side_subdivisions,
            replay: Vec::new(),
        };
        result.replay = replay::encode(&result);
        result
    }
}
