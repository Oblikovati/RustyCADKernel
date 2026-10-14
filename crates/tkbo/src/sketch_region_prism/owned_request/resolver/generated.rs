// SPDX-License-Identifier: GPL-2.0-only
//! A fresh certified face supplies semantic selection; native reused boundaries supply directed roles.
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_prism::region_certificate::current_binding::CurrentPrismBinding;
use rustycad_tkg3d::top_abs::ShapeEnum;
pub(super) fn generated_contexts(
    prepared: &PreparedPolynomialPrism,
    binding: &CurrentPrismBinding,
    copied: &[NamedSourceContext],
    stage: PrismHistoryStage,
) -> ResolveCheck<Vec<NamedSourceContext>> {
    let face = certified_face(binding, stage)?;
    let candidate = resolve_certified_candidate(current::candidates(prepared)?, face)?;
    let limits = prepared.retained.request.policy.publication.history.limits;
    let actual = capture_history_contexts(&candidate.face, stage, limits)?;
    admit_history_stage_contexts(stage, &actual, limits)?;
    let mut named = region_and_loop(&actual, face, stage)?;
    append_generated_boundaries(&mut named, copied, &actual)?;
    complete_context_coverage(&named, &actual)?;
    Ok(named)
}
pub(super) fn certified_face(
    binding: &CurrentPrismBinding,
    stage: PrismHistoryStage,
) -> ResolveCheck<&PartitionOccurrenceIdentity> {
    match stage {
        PrismHistoryStage::Selected => Ok(&binding.region.source_candidate.face),
        PrismHistoryStage::Complement => Ok(&binding.region.complementary_candidate.face),
        _ => Err(ResolutionCause::Missing {
            required: "selected or complement semantic stage",
        }),
    }
}
fn region_and_loop(
    actual: &[RegionObservedContext],
    face: &PartitionOccurrenceIdentity,
    stage: PrismHistoryStage,
) -> ResolveCheck<Vec<NamedSourceContext>> {
    let roles = region_roles(stage);
    Ok(vec![
        root_or_loop(actual, face, ShapeEnum::Face, roles[0])?,
        root_or_loop(actual, face, ShapeEnum::Wire, roles[1])?,
    ])
}
fn root_or_loop(
    actual: &[RegionObservedContext],
    face: &PartitionOccurrenceIdentity,
    kind: ShapeEnum,
    roles: (SemanticNameRole, SemanticNameRole),
) -> ResolveCheck<NamedSourceContext> {
    let found: Vec<_> = actual
        .iter()
        .filter(|row| row.kind == kind && is_region_ancestor(row, face))
        .collect();
    require_count(
        "unique certified region root or its sole Jordan wire",
        found.len(),
        1,
    )?;
    Ok(named_region_context(found[0], roles))
}
fn append_generated_boundaries(
    result: &mut Vec<NamedSourceContext>,
    copied: &[NamedSourceContext],
    actual: &[RegionObservedContext],
) -> ResolveCheck<()> {
    admit_candidate_aggregate(copied)?;
    for row in copied
        .iter()
        .filter(|row| row.member != SemanticNameRole::Aggregate)
    {
        let index = resolve_generated_context(&row.actual, actual)?;
        result.push(NamedSourceContext {
            actual: actual[index].clone(),
            member: row.member,
            directed_use: row.directed_use,
        });
    }
    Ok(())
}

fn region_roles(stage: PrismHistoryStage) -> [(SemanticNameRole, SemanticNameRole); 2] {
    use SemanticNameRole::*;
    if stage == PrismHistoryStage::Selected {
        [
            (BoundedRegion, BoundedRegionUse),
            (BoundedLoop, BoundedLoopUse),
        ]
    } else {
        [
            (ComplementRegion, ComplementRegionUse),
            (ComplementLoop, ComplementLoopUse),
        ]
    }
}

fn is_region_ancestor(row: &RegionObservedContext, face: &PartitionOccurrenceIdentity) -> bool {
    match row.kind {
        ShapeEnum::Face => &row.occurrence == face && row.ancestors.is_empty(),
        ShapeEnum::Wire => row.ancestors.as_slice() == std::slice::from_ref(face),
        _ => false,
    }
}

fn admit_candidate_aggregate(copied: &[NamedSourceContext]) -> ResolveCheck<()> {
    let count = copied
        .iter()
        .filter(|row| row.member == SemanticNameRole::Aggregate)
        .count();
    require_count("one candidate-copy aggregate context", count, 1)
}

fn named_region_context(
    actual: &RegionObservedContext,
    (member, directed_use): (SemanticNameRole, SemanticNameRole),
) -> NamedSourceContext {
    NamedSourceContext {
        actual: actual.clone(),
        member,
        directed_use,
    }
}
