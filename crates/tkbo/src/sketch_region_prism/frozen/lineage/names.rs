// SPDX-License-Identifier: GPL-2.0-only
//! Identity comes from explicit authored definitions and uses, never original-parent equivalence.
use super::*;
use crate::sketch_region::{ArrangementRelation, PartitionOccurrenceIdentity};

pub(super) fn member(
    inputs: &LineageInputs<'_>,
    shape: &Shape,
    stage: CopyLineageStage,
) -> LineageCheck<CopyMemberKey> {
    let member = definition_name(inputs, shape, stage)?;
    let kind = shape
        .shape_type()
        .map_err(|e| inputs.frozen(CopyLineagePhase::Members, PrismFreezeCause::Native(e)))?
        as u8;
    Ok(CopyMemberKey {
        operation: inputs.publication.operation,
        stage,
        member,
        kind,
    })
}
fn definition_name(
    inputs: &LineageInputs<'_>,
    shape: &Shape,
    stage: CopyLineageStage,
) -> LineageCheck<AuthoredPrismId> {
    let actual = PartitionOccurrenceIdentity::capture(shape);
    let bindings = match stage {
        CopyLineageStage::Selected => &inputs.authored.selected_members,
        CopyLineageStage::Copied => &inputs.publication.sources,
    };
    let names: Vec<_> = bindings
        .iter()
        .filter(|b| super::super::identities::same_definition(&actual, &b.actual))
        .map(|b| b.member)
        .collect();
    let name = unique_name(inputs, &names)?;
    inputs.require(
        CopyLineagePhase::Members,
        "one authored name per distinct member",
        usize::from(!conflicting(bindings, &actual, name)),
        1,
    )?;
    Ok(name)
}
fn unique_name(
    inputs: &LineageInputs<'_>,
    names: &[AuthoredPrismId],
) -> LineageCheck<AuthoredPrismId> {
    let Some(name) = names.first().filter(|n| n.0 != 0) else {
        return Err(inputs.count(
            CopyLineagePhase::Members,
            "nonzero authored member",
            names.len(),
            1,
        ));
    };
    inputs.require(
        CopyLineagePhase::Members,
        "one authored name for actual member",
        names.iter().filter(|other| *other != name).count(),
        0,
    )?;
    Ok(*name)
}

fn conflicting(
    bindings: &[PrismSourceBinding],
    actual: &PartitionOccurrenceIdentity,
    name: AuthoredPrismId,
) -> bool {
    bindings
        .iter()
        .any(|b| b.member == name && !super::super::identities::same_definition(actual, &b.actual))
}
pub(super) fn occurrence(
    inputs: &LineageInputs<'_>,
    shape: &Shape,
    stage: CopyLineageStage,
) -> LineageCheck<CopyOccurrence> {
    let key = member(inputs, shape, stage)?;
    let placement = super::super::history::freeze_placement(shape.location())
        .map_err(|e| attributed(inputs.frozen(CopyLineagePhase::Members, e), key))?;
    let values = placement
        .linear
        .into_iter()
        .flatten()
        .chain(placement.origin)
        .chain([placement.scale]);
    let mut placement_bits = [0; 13];
    for (slot, value) in placement_bits.iter_mut().zip(values) {
        *slot = value.to_bits();
    }
    Ok(CopyOccurrence {
        member: key,
        orientation: shape.orientation() as u8,
        placement_bits,
    })
}
pub(super) fn parents(
    inputs: &LineageInputs<'_>,
    row: &ArrangementRelation,
) -> LineageCheck<[Vec<FrozenOriginalKey>; 3]> {
    let mut result = [Vec::new(), Vec::new(), Vec::new()];
    for (out, original) in result
        .iter_mut()
        .zip([&row.preserved, &row.modified, &row.generated])
    {
        for parent in original {
            out.push(
                super::super::identities::original_key(inputs.publication, parent)
                    .map_err(|e| inputs.frozen(CopyLineagePhase::Members, e))?,
            );
        }
        out.sort(); // Preserve multiplicity; sorting only makes owned replay independent of row order.
    }
    Ok(result)
}
pub(super) fn attributed(
    mut finding: CopyLineageFinding,
    key: CopyMemberKey,
) -> CopyLineageFinding {
    finding.entity = Some(key);
    finding
}
pub(super) fn contexts(
    inputs: &LineageInputs<'_>,
    stage: CopyLineageStage,
) -> LineageCheck<Vec<PrismBodyContext>> {
    let root = match stage {
        CopyLineageStage::Selected => inputs.receipt.input.selected.as_ref().map(|s| &*s.face),
        CopyLineageStage::Copied => inputs.receipt.copied.as_ref().map(|s| &s.base),
    }
    .ok_or_else(|| {
        inputs.count(
            CopyLineagePhase::Directed,
            "actual selection/copy root",
            0,
            1,
        )
    })?;
    super::super::super::contexts::body_contexts(root).map_err(|e| {
        inputs.frozen(
            CopyLineagePhase::Directed,
            super::super::bindings::freeze_context_cause(e),
        )
    })
}
