// SPDX-License-Identifier: GPL-2.0-only
//! Reuse the reviewed copy/sweep lineage projector against this exact already-frozen V2 body.
use super::super::{bindings::BoundProjection, lineage::*};
use super::*;
pub(super) fn capture(
    request: &PrismHistoryRequest<'_, '_>,
    bound: &BoundProjection<'_>,
) -> HistoryCheck<PrismLineageInspection> {
    capture_projection(bound, request.joined.projection(), request.copied)
}
/// Project reviewed copy/sweep lineage over one frozen body; `capture_projection(bound, frozen, names)`.
pub(in crate::sketch_region_prism::frozen) fn capture_projection(
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
    copied: &PrismCopyBindings,
) -> HistoryCheck<PrismLineageInspection> {
    match super::super::lineage::project_prism_lineage(bound, projection, copied) {
        PrismLineageOutcome::Inspected(rows) => Ok(*rows),
        PrismLineageOutcome::Incomplete(rows) | PrismLineageOutcome::Failed(rows) => {
            Err(PrismHistoryCause::Lineage(rows))
        }
        PrismLineageOutcome::Cancelled { phase, .. } => {
            Err(PrismHistoryCause::LineageCancelled(phase))
        }
    }
}
pub(in crate::sketch_region_prism::frozen) fn historical(key: &CopyUseKey) -> PrismHistoryUseKey {
    PrismHistoryUseKey(
        key.0
            .iter()
            .map(|(member, use_name)| {
                (
                    PrismHistoryMemberKey {
                        operation: member.operation,
                        stage: match member.stage {
                            CopyLineageStage::Selected => PrismHistoryStage::Selected,
                            CopyLineageStage::Copied => PrismHistoryStage::SweepBase,
                        },
                        member: member.member,
                        kind: member.kind,
                    },
                    *use_name,
                )
            })
            .collect(),
    )
}
pub(in crate::sketch_region_prism::frozen) fn copied_use(
    lineage: &PrismLineageInspection,
    selected: &PrismHistoryUseKey,
) -> HistoryCheck<PrismHistoryUseKey> {
    let found: Vec<_> = lineage
        .directed()
        .iter()
        .filter(|r| &historical(&r.selected) == selected)
        .collect();
    history_count(
        "one proven selected-to-sweep directed correspondence",
        found.len(),
        1,
    )?;
    Ok(historical(&found[0].copied))
}
