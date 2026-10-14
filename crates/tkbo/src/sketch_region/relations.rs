// SPDX-License-Identifier: GPL-2.0-only
//! Missing attribution is incomplete; new containers inherit only actual descendant provenance.
use super::{copy_inputs::CopiedSources, occurrences::equal_occurrences, outcome::*};
use crate::bop_algo::BOPAlgoBuilder;
use rustycad_tkbrep::{brep_tools::History, topods::Shape};

pub(super) fn compose_relations(
    copied: &CopiedSources,
    builder: &BOPAlgoBuilder,
) -> Result<ArrangementCompletion, ArrangementCause> {
    if !builder.has_history() || builder.shape().is_null() {
        return Err(ArrangementCause::MissingHistory { source: None });
    }
    let deleted = deleted_members(copied, builder)?;
    let outputs = equal_occurrences(builder.shape()).map_err(history_failure)?;
    let mut relations = Vec::new();
    for output in outputs.iter().skip(1) {
        relations.push(entity_relation(copied, builder, output)?);
    }
    finish_relations(copied, builder.shape(), relations, deleted)
}
fn finish_relations(
    copied: &CopiedSources,
    root: &Shape,
    mut relations: Vec<ArrangementRelation>,
    deleted: Vec<OriginalMemberIdentity>,
) -> Result<ArrangementCompletion, ArrangementCause> {
    if relations.is_empty() {
        if deleted.len() == copied.members.len() {
            return Ok(ArrangementCompletion::CompletedEmpty { deleted });
        }
        return Err(ArrangementCause::UnattributedOutput);
    }
    relations.push(container_relation(root, &relations));
    Ok(completed_arrangement(root, relations, deleted))
}
fn completed_arrangement(
    root: &Shape,
    relations: Vec<ArrangementRelation>,
    deleted: Vec<OriginalMemberIdentity>,
) -> ArrangementCompletion {
    ArrangementCompletion::Arranged(OwnedArrangement {
        arrangement: root.clone(),
        relations,
        deleted,
        achieved_error: None,
        incidences_certified: false,
    })
}

fn entity_relation(
    copied: &CopiedSources,
    builder: &BOPAlgoBuilder,
    output: &Shape,
) -> Result<ArrangementRelation, ArrangementCause> {
    let mut relation = empty_relation(output);
    for (identity, member) in &copied.members {
        append_member_relation(&mut relation, identity, member, builder)?;
    }
    normalize_parents(&mut relation);
    if relation.preserved.is_empty()
        && relation.modified.is_empty()
        && relation.generated.is_empty()
    {
        return Err(ArrangementCause::UnattributedOutput);
    }
    Ok(relation)
}
fn empty_relation(output: &Shape) -> ArrangementRelation {
    ArrangementRelation {
        output: output.clone(),
        preserved: Vec::new(),
        modified: Vec::new(),
        generated: Vec::new(),
    }
}
fn container_relation(root: &Shape, descendants: &[ArrangementRelation]) -> ArrangementRelation {
    let mut root_relation = empty_relation(root);
    for child in descendants {
        root_relation.generated.extend(
            child
                .preserved
                .iter()
                .chain(&child.modified)
                .chain(&child.generated)
                .cloned(),
        );
    }
    normalize_parents(&mut root_relation);
    root_relation
}
fn append_member_relation(
    relation: &mut ArrangementRelation,
    identity: &OriginalMemberIdentity,
    member: &Shape,
    builder: &BOPAlgoBuilder,
) -> Result<(), ArrangementCause> {
    let matched = member_relation_flags(&relation.output, member, builder)?;
    for (parents, matches) in [
        (&mut relation.preserved, matched[0]),
        (&mut relation.modified, matched[1]),
        (&mut relation.generated, matched[2]),
    ] {
        if matches {
            parents.push(identity.clone());
        }
    }
    Ok(())
}
fn member_relation_flags(
    output: &Shape,
    member: &Shape,
    builder: &BOPAlgoBuilder,
) -> Result<[bool; 3], ArrangementCause> {
    // Copy's Modified relation composes with an arrangement-preserved copied member as Modified.
    let modified = output.is_same(member) || history_contains(builder.modified(member), output)?;
    let generated = history_contains(builder.generated(member), output)?;
    Ok([false, modified, generated])
}
fn history_contains(
    images: Result<Vec<Shape>, rustycad_tkernel::standard::Failure>,
    output: &Shape,
) -> Result<bool, ArrangementCause> {
    Ok(images
        .map_err(history_failure)?
        .iter()
        .any(|shape| shape.is_same(output)))
}

fn normalize_parents(relation: &mut ArrangementRelation) {
    for parents in [
        &mut relation.preserved,
        &mut relation.modified,
        &mut relation.generated,
    ] {
        *parents = unique_parents(std::mem::take(parents));
    }
}
fn unique_parents(candidates: Vec<OriginalMemberIdentity>) -> Vec<OriginalMemberIdentity> {
    let mut unique = Vec::new();
    for candidate in candidates {
        if !unique.iter().any(|known| known == &candidate) {
            unique.push(candidate);
        }
    }
    // Stable source-ID ordering retains declared occurrence traversal order within each source.
    unique.sort_by_key(|key| key.source);
    unique
}

fn deleted_members(
    copied: &CopiedSources,
    builder: &BOPAlgoBuilder,
) -> Result<Vec<OriginalMemberIdentity>, ArrangementCause> {
    let mut deleted = Vec::new();
    for (identity, member) in &copied.members {
        if !History::is_supported_type(member).map_err(history_failure)? {
            return Err(ArrangementCause::UnsupportedHistoryCoverage {
                member: identity.clone(),
            });
        }
        if builder.is_deleted(member).map_err(history_failure)? {
            deleted.push(identity.clone());
        }
    }
    Ok(deleted)
}
fn history_failure(cause: rustycad_tkernel::standard::Failure) -> ArrangementCause {
    ArrangementCause::Failure {
        phase: ArrangementPhase::History,
        cause,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkg3d::top_abs::Orientation;
    fn original_key(source: u64) -> OriginalMemberIdentity {
        OriginalMemberIdentity {
            source: RegionSourceId(source),
            topology_address: source as usize,
            location: Default::default(),
            orientation: Orientation::Forward,
        }
    }
    #[test]
    fn root_provenance_comes_only_from_represented_output_descendants() {
        let mut child = empty_relation(&Shape::default());
        child.modified.push(original_key(7));
        let root = container_relation(&Shape::default(), &[child]);
        assert_eq!(root.generated.len(), 1);
        assert_eq!(root.generated[0].source, RegionSourceId(7));
    }
    fn empty_test_member() -> CopiedSources {
        CopiedSources {
            contexts: Vec::new(),
            arguments: Default::default(),
            roots: Vec::new(),
            members: vec![(original_key(7), Shape::default())],
        }
    }
    #[test]
    fn completed_empty_stage_keeps_deletions_and_missing_coverage_is_not_success() {
        let copied = empty_test_member();
        let complete = finish_relations(
            &copied,
            &Shape::default(),
            Vec::new(),
            vec![original_key(7)],
        )
        .unwrap_or_else(|_| panic!("expected empty stage"));
        assert!(
            matches!(complete,ArrangementCompletion::CompletedEmpty{deleted} if deleted.len()==1)
        );
        let incomplete = finish_relations(&copied, &Shape::default(), Vec::new(), Vec::new());
        assert!(matches!(
            incomplete,
            Err(ArrangementCause::UnattributedOutput)
        ));
    }
    #[test]
    fn one_physical_output_retains_all_original_forward_and_reversed_parents() {
        let copied = forward_reverse_copy();
        let output = first_copied_edge(&copied);
        let relation = entity_relation(&copied, &BOPAlgoBuilder::new(), &output)
            .unwrap_or_else(|_| panic!("expected complete physicalparents"));
        assert_eq!(relation.modified.len(), 2);
        assert_eq!(
            [
                relation.modified[0].orientation,
                relation.modified[1].orientation
            ],
            [Orientation::Forward, Orientation::Reversed]
        );
        assert!(relation.output.is_equal(&output));
    }
    fn forward_reverse_copy() -> CopiedSources {
        use super::super::tests::{closed_cubic, sources};
        use rustycad_tkbrep::topods::borrowed_shape_batch;
        let original = closed_cubic();
        let reversed = original.oriented(Orientation::Reversed);
        let compound = borrowed_shape_batch(&[&original, &reversed]).unwrap();
        super::super::copy_inputs::copy_sources(&sources(&compound))
            .unwrap_or_else(|_| panic!("expected aggregatecopy"))
    }
    fn first_copied_edge(copied: &CopiedSources) -> Shape {
        copied
            .members
            .iter()
            .find(|(_, shape)| {
                shape.shape_type().unwrap() == rustycad_tkg3d::top_abs::ShapeEnum::Edge
            })
            .unwrap()
            .1
            .clone()
    }
    #[test]
    fn full_equality_dedup_keeps_first_occurrence_order_without_hash_or_address_ordering() {
        let mut first = original_key(2);
        first.source = RegionSourceId(7);
        let mut second = original_key(1);
        second.source = RegionSourceId(7);
        let normalized = unique_parents(vec![
            first.clone(),
            second.clone(),
            first.clone(),
            second.clone(),
        ]);
        let addresses: Vec<usize> = normalized.iter().map(|key| key.topology_address).collect();
        assert_eq!(addresses, [2, 1]);
        assert!(normalized[0] == first && normalized[1] == second);
        assert!(unique_parents(normalized.clone()) == normalized);
    }
}
