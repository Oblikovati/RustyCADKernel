// SPDX-License-Identifier: GPL-2.0-only
use super::{descriptor_bits::*, types::*};
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision;
pub(super) fn complete_stages(
    row: &OriginalConstructionEvidence,
    policy: SourceConformancePolicy,
) -> bool {
    let expected = [
        ConstructionStage::OriginalPlane,
        ConstructionStage::CopiedPlane,
        ConstructionStage::CapturedBasis,
    ];
    row.stages.len() == 3
        && row.stages.iter().zip(expected).all(|(stage, kind)| {
            stage.stage == kind
                && matches!(stage.decision, Ok(ResidualDecision::Within { .. }))
                && valid_stage_limit(stage, policy)
        })
}
fn valid_stage_limit(stage: &ConstructionStageEvidence, policy: SourceConformancePolicy) -> bool {
    let expected = if stage.stage == ConstructionStage::CapturedBasis {
        policy.construction_limit
    } else {
        policy.original_plane_limit
    };
    stage.requested_limit.to_bits() == expected.to_bits()
        && stage.effective_limit.to_bits() == expected.to_bits()
}
pub(super) fn work_matches(report: &SourceConformanceInspection) -> bool {
    let mut spent = 0u64;
    for stage in report.entries.iter().flat_map(|row| &row.stages) {
        let mut expected = report.requested.roots;
        expected.max_bisections = expected
            .max_bisections
            .saturating_sub(spent.min(u16::MAX as u64) as u16);
        if !same_root_policy(stage.effective_roots, expected)
            || stage.bisections > u64::from(expected.max_bisections)
        {
            return false;
        }
        spent += stage.bisections;
    }
    spent == report.bisections
}
