// SPDX-License-Identifier: GPL-2.0-only
//! Original and selected/sweep names reuse existing authored bindings without implicit IDs.
use super::*;
use crate::sketch_region::{
    OriginalMemberIdentity, PartitionOccurrenceIdentity, RegionObservedContext,
};

pub(super) fn original_names(
    request: &PrismHistoryRequest<'_>,
    rows: &[RegionObservedContext],
) -> HistoryCheck<PrismHistoricalStageBindings> {
    history_count(
        "one original authored binding per actual source use",
        request.publication.originals.len(),
        rows.len(),
    )?;
    let mut result = PrismHistoricalStageBindings {
        stage: PrismHistoryStage::Original,
        members: Vec::new(),
        uses: Vec::new(),
    };
    for row in rows {
        let actual = original_identity(request, &row.occurrence)?;
        let found: Vec<_> = request
            .publication
            .originals
            .iter()
            .filter(|b| b.actual == actual)
            .collect();
        history_count("unique actual original use binding", found.len(), 1)?;
        append_member(&mut result, &row.occurrence, found[0].member)?;
        result.uses.push(PrismHistoricalUseBinding {
            actual: row.clone(),
            authored_use: found[0].directed_use,
        });
    }
    Ok(result)
}
pub(super) fn original_identity(
    request: &PrismHistoryRequest<'_>,
    actual: &PartitionOccurrenceIdentity,
) -> HistoryCheck<OriginalMemberIdentity> {
    history_count("one authored Jordan input source", request.sources.len(), 1)?;
    Ok(OriginalMemberIdentity {
        source: request.sources[0].identity,
        topology_address: actual.topology_address,
        location: actual.location.clone(),
        orientation: actual.orientation,
    })
}
pub(super) fn legacy_names(
    request: &PrismHistoryRequest<'_>,
    stage: PrismHistoryStage,
    rows: &[RegionObservedContext],
) -> HistoryCheck<PrismHistoricalStageBindings> {
    let (members, uses) = match stage {
        PrismHistoryStage::Selected => (
            &request.copied.selected_members,
            &request.copied.selected_uses,
        ),
        _ => (&request.publication.sources, &request.copied.copied_uses),
    };
    history_count(
        "complete authored legacy use census",
        uses.len(),
        rows.len(),
    )?;
    let mut result = PrismHistoricalStageBindings {
        stage,
        members: Vec::new(),
        uses: Vec::new(),
    };
    for binding in members {
        append_member(&mut result, &binding.actual, binding.member)?;
    }
    for binding in uses {
        let actual = context_snapshot(&binding.actual)?;
        result.uses.push(PrismHistoricalUseBinding {
            actual,
            authored_use: binding.authored_use,
        });
    }
    Ok(result)
}
fn append_member(
    names: &mut PrismHistoricalStageBindings,
    actual: &PartitionOccurrenceIdentity,
    member: AuthoredPrismId,
) -> HistoryCheck<()> {
    let matches: Vec<_> = names
        .members
        .iter()
        .filter(|b| same_definition(actual, &b.actual))
        .collect();
    if let Some(known) = matches.first() {
        return history_count(
            "same authored name for shared historical definition",
            usize::from(known.member == member),
            1,
        );
    }
    names.members.push(PrismSourceBinding {
        actual: actual.clone(),
        member,
    });
    Ok(())
}
fn context_snapshot(
    actual: &crate::sketch_region::RegionDirectedContext,
) -> HistoryCheck<RegionObservedContext> {
    Ok(RegionObservedContext {
        occurrence: PartitionOccurrenceIdentity::capture(&actual.occurrence),
        ancestors: actual
            .ancestors
            .iter()
            .map(PartitionOccurrenceIdentity::capture)
            .collect(),
        child_slots: actual.child_slots.clone(),
        kind: actual
            .occurrence
            .shape_type()
            .map_err(|cause| PrismHistoryCause::Native(std::sync::Arc::new(cause)))?,
    })
}
