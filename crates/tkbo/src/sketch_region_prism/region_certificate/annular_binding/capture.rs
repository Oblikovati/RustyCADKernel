// SPDX-License-Identifier: GPL-2.0-only
//! Rebind the certified selected candidate to both current copy roots before chart admission.
use super::super::super::{
    contexts::body_contexts,
    surface_charts::{CapturedPrismSources, capture_sources},
};
use super::super::{boundary::same_support, identity_multiset::same_multiset};
use super::*;
use crate::sketch_region::ArrangementRelation;
use crate::sketch_region_source_conformance::{same_metric_curve, same_space_representation};

pub(super) fn copy_inputs(
    request: &TwoLoopRegionRequest<'_, '_>,
    region: &TwoLoopRegionCertificate,
    receipt: &PrismReceipt,
) -> AnnularCheck<'static, AnnularCopyCapture> {
    let selected = receipt
        .input
        .selected
        .as_ref()
        .ok_or_else(|| missing("actual selected candidate"))?;
    let copied = receipt
        .copied
        .as_ref()
        .ok_or_else(|| missing("actual copied base"))?;
    let names = selected_names(request, region)?;
    holds(
        "same certified actual selected face",
        names.actual == PartitionOccurrenceIdentity::capture(&selected.face),
    )?;
    let first = contexts(&selected.face)?;
    let second = contexts(&copied.base)?;
    let members = correspondence::pair_members(names, &first, &second, &copied.observations)?;
    copy_relations(selected, copied)?;
    let selected = capture_sources(&selected.face).map_err(snapshot_failure)?;
    let actual_copy = capture_sources(&copied.base).map_err(snapshot_failure)?;
    compare_sources(region, names, &members, &selected, &actual_copy)?;
    Ok(AnnularCopyCapture {
        members,
        selected,
        copied: actual_copy,
    })
}
pub(super) fn selected_names<'a>(
    request: &'a TwoLoopRegionRequest<'_, '_>,
    region: &TwoLoopRegionCertificate,
) -> AnnularCheck<'static, &'a TwoLoopCandidateBinding> {
    let matches: Vec<_> = request
        .identities
        .candidates
        .iter()
        .filter(|c| c.name == region.selected_candidate_id())
        .collect();
    require("unique certified selected candidate name", matches.len(), 1)?;
    Ok(matches[0])
}
fn contexts(
    shape: &rustycad_tkbrep::topods::Shape,
) -> AnnularCheck<'static, Vec<PrismBodyContext>> {
    body_contexts(shape).map_err(|cause| Box::new(AnnularBindingCause::Body(Box::new(cause))))
}
fn missing(required: &'static str) -> Box<AnnularBindingCause<'static>> {
    Box::new(AnnularBindingCause::Count {
        required,
        actual: 0,
        expected: 1,
    })
}
fn snapshot_failure(
    cause: std::sync::Arc<super::super::super::surface_charts::PrismSurfaceCause>,
) -> Box<AnnularBindingCause<'static>> {
    Box::new(AnnularBindingCause::Capture(
        super::super::super::surface_charts::PrismSurfaceCause::SourceSnapshot(cause),
    ))
}
fn copy_relations(
    selected: &crate::sketch_region_candidates::RegionCandidate,
    copied: &PrismCopiedSelection,
) -> AnnularCheck<'static> {
    require(
        "all selected copy observations",
        copied.observations.len(),
        selected.relations.len(),
    )?;
    require(
        "all copied relations",
        copied.relations.len(),
        selected.relations.len(),
    )?;
    for observed in &copied.observations {
        let before = relation(&selected.relations, &observed.selected.output)?;
        same_parents(before, &observed.selected)?;
        let after = relation(&copied.relations, &observed.copied_image)?;
        let expected =
            crate::sketch_region_candidates::composed_copy_relation(before, &observed.copied_image);
        same_parents(after, &expected)?;
        holds(
            "native copied definition and placement",
            observed.native_image.is_same(&observed.copied_image),
        )?;
        holds(
            "copied member orientation",
            observed.selected.output.orientation() == observed.copied_image.orientation(),
        )?;
    }
    Ok(())
}
fn relation<'a>(
    rows: &'a [ArrangementRelation],
    shape: &rustycad_tkbrep::topods::Shape,
) -> AnnularCheck<'static, &'a ArrangementRelation> {
    let matches: Vec<_> = rows.iter().filter(|r| r.output.is_equal(shape)).collect();
    require("one actual oriented copy relation", matches.len(), 1)?;
    Ok(matches[0])
}
fn same_parents(
    first: &ArrangementRelation,
    second: &ArrangementRelation,
) -> AnnularCheck<'static> {
    holds(
        "all preserved modified generated parent multiplicities",
        same_multiset(&first.preserved, &second.preserved)
            && same_multiset(&first.modified, &second.modified)
            && same_multiset(&first.generated, &second.generated),
    )
}
fn compare_sources(
    region: &TwoLoopRegionCertificate,
    names: &TwoLoopCandidateBinding,
    members: &[AnnularCopiedMember],
    selected: &CapturedPrismSources,
    copied: &CapturedPrismSources,
) -> AnnularCheck<'static> {
    let candidates: Vec<_> = region
        .candidates()
        .iter()
        .filter(|c| c.name == names.name)
        .collect();
    require(
        "unique certified selected candidate proof",
        candidates.len(),
        1,
    )?;
    let proof = candidates[0];
    require(
        "all selected certified boundary curves",
        selected.boundaries.len(),
        proof.wires.len(),
    )?;
    require(
        "all copied certified boundary curves",
        copied.boundaries.len(),
        proof.wires.len(),
    )?;
    holds(
        "selected certified plane",
        same_support(&proof.support, &selected.support),
    )?;
    holds(
        "copy certified plane",
        same_support(&proof.support, &copied.support),
    )?;
    for wire in &proof.wires {
        compare_wire(wire, members, selected, copied)?;
    }
    Ok(())
}
fn compare_wire(
    wire: &TwoLoopWireProof,
    members: &[AnnularCopiedMember],
    selected: &CapturedPrismSources,
    copied: &CapturedPrismSources,
) -> AnnularCheck<'static> {
    let rows: Vec<_> = members
        .iter()
        .filter(|m| m.key.wire == Some(wire.name) && m.key.role == TwoLoopMemberRole::Boundary)
        .collect();
    require("one copied boundary per certified wire", rows.len(), 1)?;
    for (sources, identity) in [
        (selected, &rows[0].selected.occurrence),
        (copied, &rows[0].copied.occurrence),
    ] {
        let found: Vec<_> = sources
            .boundaries
            .iter()
            .filter(|b| &b.occurrence == identity)
            .collect();
        require(
            "one actual captured boundary for directed copy role",
            found.len(),
            1,
        )?;
        let boundary = found[0];
        holds(
            "full certified copied boundary range",
            boundary.interval == [0., 1.],
        )?;
        holds(
            "certified copied space coefficients and placement",
            same_space_representation(&wire.space, &boundary.space),
        )?;
        holds(
            "certified copied pcurve coefficients",
            same_metric_curve(&wire.pcurve, &boundary.pcurve),
        )?;
    }
    Ok(())
}
