// SPDX-License-Identifier: GPL-2.0-only
//! Container provenance derives only from actual represented descendants, never every input.
use super::{generation::CandidateParents, outcome::*};
use crate::sketch_region::{ArrangementRelation, OriginalMemberIdentity, equal_occurrences};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;
pub(super) fn candidate_relations(
    root: &Shape,
    copied: &CandidateParents,
) -> Result<Vec<ArrangementRelation>, CandidateCause> {
    let mut relations = Vec::new();
    for output in equal_occurrences(root).map_err(CandidateCause::Mapped)? {
        relations.push(member_relation(&output, copied)?);
    }
    Ok(relations)
}
fn member_relation(
    output: &Shape,
    copied: &CandidateParents,
) -> Result<ArrangementRelation, CandidateCause> {
    let (modified, generated) = direct_parents(output, copied);
    if !modified.is_empty() || !generated.is_empty() {
        return Ok(owned_relation(output, modified, generated));
    }
    if !matches!(
        output.shape_type().map_err(CandidateCause::Mapped)?,
        ShapeEnum::Face | ShapeEnum::Wire
    ) {
        return Err(unattributed_member(output));
    }
    Ok(owned_relation(
        output,
        Vec::new(),
        descendant_parents(output, copied)?,
    ))
}
fn owned_relation(
    output: &Shape,
    modified: Vec<OriginalMemberIdentity>,
    generated: Vec<OriginalMemberIdentity>,
) -> ArrangementRelation {
    ArrangementRelation {
        output: output.clone(),
        preserved: Vec::new(),
        modified,
        generated,
    }
}
fn unattributed_member(output: &Shape) -> CandidateCause {
    CandidateCause::Incomplete(CandidateGap::UnattributedMember {
        member: output.clone(),
    })
}
fn direct_parents(
    output: &Shape,
    copied: &CandidateParents,
) -> (Vec<OriginalMemberIdentity>, Vec<OriginalMemberIdentity>) {
    let mut modified = Vec::new();
    let mut generated = Vec::new();
    for relation in copied
        .iter()
        .filter(|relation| relation.output.is_same(output))
    {
        append_unique(&mut modified, &relation.modified);
        append_unique(&mut generated, &relation.generated);
    }
    modified.sort_by_key(|key| key.source);
    generated.sort_by_key(|key| key.source);
    (modified, generated)
}
fn descendant_parents(
    output: &Shape,
    copied: &CandidateParents,
) -> Result<Vec<OriginalMemberIdentity>, CandidateCause> {
    let mut parents = Vec::new();
    for child in equal_occurrences(output)
        .map_err(CandidateCause::Mapped)?
        .iter()
        .skip(1)
    {
        let (modified, generated) = direct_parents(child, copied);
        append_unique(&mut parents, &modified);
        append_unique(&mut parents, &generated);
    }
    if parents.is_empty() {
        return Err(unattributed_member(output));
    }
    parents.sort_by_key(|key| key.source);
    Ok(parents)
}
fn append_unique(parents: &mut Vec<OriginalMemberIdentity>, candidates: &[OriginalMemberIdentity]) {
    for candidate in candidates {
        if !parents.contains(candidate) {
            parents.push(candidate.clone());
        }
    }
}
