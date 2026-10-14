// SPDX-License-Identifier: GPL-2.0-only
//! Semantic stages replace temporary candidate/report ranks in persistent proof names.
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_contact_transport::PairedContactTransaction;
use crate::sketch_region_prism::region_certificate::{ClosurePointStage, RegionCertificate};

pub(super) fn first_is_selected(
    bridge: &PairedContactTransaction,
    region: &RegionCertificate,
) -> CaptureResult<bool> {
    let batch = bridge
        .mapped
        .copied
        .as_ref()
        .and_then(|copied| copied.completion.as_ref().ok())
        .ok_or(missing("current paired candidate universe"))?;
    let matches: Vec<_> = batch
        .candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| {
            PartitionOccurrenceIdentity::capture(&candidate.face) == region.source_candidate.face
        })
        .map(|(i, _)| i)
        .collect();
    let selected = *only_bound(&matches, "selected actual region face in paired universe")?;
    if batch.candidates.len() != 2 {
        return Err(identity(
            "two actual paired region candidates",
            batch.candidates.len(),
        ));
    }
    Ok(selected == 0)
}
pub(super) fn sources(
    proof: &PrismEmbeddingEvidence,
    links: &JoinedProofLinks,
    bound: &BoundProjection<'_>,
    bytes: &mut Vec<u8>,
) -> CaptureResult<()> {
    let region = &proof.binding().region;
    for (stage, snapshot) in [
        ("selected", &region.source_candidate),
        ("complement", &region.complementary_candidate),
        ("copied", &region.copied),
    ] {
        replay::text(
            &format!(
                "{stage}/{:?}/{:?}",
                bound.bindings.selected_region, links.original
            ),
            bytes,
        );
        replay::block(&encoding::sources(snapshot), bytes);
        occurrence(&snapshot.face, bytes)?;
        for boundary in &snapshot.boundaries {
            occurrence(&boundary.occurrence, bytes)?;
            let mut endpoints = Vec::new();
            for (key, parameter) in &boundary.endpoints {
                let mut row = Vec::new();
                encoding::numbers([*parameter], &mut row);
                occurrence(key, &mut row)?;
                endpoints.push(row);
            }
            sorted_blocks(endpoints, bytes);
        }
    }
    Ok(())
}
pub(super) fn closure(
    region: &RegionCertificate,
    bindings: &PrismPublicationBindings,
    selected_first: bool,
    bytes: &mut Vec<u8>,
) -> CaptureResult<()> {
    let mut rows = Vec::new();
    for point in &region.closure_points {
        for context in &point.contexts {
            let mut row = Vec::new();
            replay::text(
                &format!(
                    "{}/{:?}",
                    semantic_stage(context.stage, selected_first),
                    original_key(bindings, &point.original)?
                ),
                &mut row,
            );
            occurrence(&context.occurrence, &mut row)?;
            encoding::numbers(point.point, &mut row);
            encoding::numbers(
                [point.requested_world_limit, point.exact_certificate_limit],
                &mut row,
            );
            replay::text(
                &format!(
                    "{:?}/{:?}/{}",
                    point.roots, point.decision, point.bisections
                ),
                &mut row,
            );
            rows.push(row);
        }
    }
    sorted_blocks(rows, bytes);
    Ok(())
}
pub(super) fn semantic_stage(stage: ClosurePointStage, selected_first: bool) -> &'static str {
    use ClosurePointStage::*;
    match stage {
        Original => "original",
        Census => "census",
        Partition => "partition",
        PrismCopy => "copied",
        CandidateFirst if selected_first => "selected",
        CandidateSecond if !selected_first => "selected",
        CandidateFirst | CandidateSecond => "complement",
    }
}
pub(super) fn occurrence(
    actual: &PartitionOccurrenceIdentity,
    bytes: &mut Vec<u8>,
) -> CaptureResult<()> {
    replay::text(&format!("{:?}", actual.orientation), bytes);
    encoding::placement(
        super::super::history::freeze_placement(&actual.location)?,
        bytes,
    );
    Ok(())
}
pub(super) fn sorted_blocks(mut rows: Vec<Vec<u8>>, bytes: &mut Vec<u8>) {
    // Sorting canonical records never supplies an identity, and deliberately keeps duplicates.
    rows.sort();
    bytes.extend((rows.len() as u64).to_le_bytes());
    for row in rows {
        replay::block(&row, bytes);
    }
}
