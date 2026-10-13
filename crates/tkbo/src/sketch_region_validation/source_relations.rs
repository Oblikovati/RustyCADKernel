// SPDX-License-Identifier: GPL-2.0-only
//! Retain operation-local member keys and relation kinds; root IDs are only a convenience.
use super::outcome::AffectedOccurrence;
use crate::sketch_region::{ArrangementRelation, OriginalMemberIdentity};
use rustycad_tkbrep::topods::Shape;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SourceMemberRelations {
    pub(crate) preserved: Vec<OriginalMemberIdentity>,
    pub(crate) modified: Vec<OriginalMemberIdentity>,
    pub(crate) generated: Vec<OriginalMemberIdentity>,
}

pub(crate) struct SourceRelation {
    pub(crate) output: Shape,
    pub(crate) parents: SourceMemberRelations,
}

impl SourceMemberRelations {
    fn is_empty(&self) -> bool {
        let kinds = [&self.preserved, &self.modified, &self.generated];
        kinds.iter().all(|members| members.is_empty())
    }

    /// Root IDs summarize these actual member relations, not complete history.
    /// Example: `parents.source_ids()`.
    pub(crate) fn source_ids(&self) -> Vec<u64> {
        let mut ids: Vec<u64> = self
            .preserved
            .iter()
            .chain(&self.modified)
            .chain(&self.generated)
            .map(|member| member.source.0)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }
}

/// Copy attributed relation metadata from the already-owned arrangement only.
/// Example: `source_relations(&owned.relations)`.
pub(crate) fn source_relations(relations: &[ArrangementRelation]) -> Vec<SourceRelation> {
    relations
        .iter()
        .map(|relation| SourceRelation {
            output: relation.output.clone(),
            parents: SourceMemberRelations {
                preserved: relation.preserved.clone(),
                modified: relation.modified.clone(),
                generated: relation.generated.clone(),
            },
        })
        .collect()
}

pub(super) fn attributed_occurrence(
    shape: &Shape,
    sources: &[SourceRelation],
) -> AffectedOccurrence {
    let original_parents = matched_parents(shape, sources);
    let source_parents = original_parents
        .as_ref()
        .map(SourceMemberRelations::source_ids);
    AffectedOccurrence {
        shape: shape.clone(),
        original_parents,
        source_parents,
    }
}

fn matched_parents(shape: &Shape, sources: &[SourceRelation]) -> Option<SourceMemberRelations> {
    let mut parents = SourceMemberRelations::default();
    for relation in sources.iter().filter(|row| row.output.is_equal(shape)) {
        // A matching row with unknown attribution prevents a complete parent-set claim.
        if relation.parents.is_empty() {
            return None;
        }
        append_distinct(&mut parents.preserved, &relation.parents.preserved);
        append_distinct(&mut parents.modified, &relation.parents.modified);
        append_distinct(&mut parents.generated, &relation.parents.generated);
    }
    if parents.is_empty() {
        return None;
    }
    Some(parents)
}

fn append_distinct(target: &mut Vec<OriginalMemberIdentity>, members: &[OriginalMemberIdentity]) {
    for member in members {
        // Hashes and topology addresses alone do not identify an oriented occurrence.
        if !target.contains(member) {
            target.push(member.clone());
        }
    }
    target.sort_by_key(|member| member.source);
}
