// SPDX-License-Identifier: GPL-2.0-only
//! Full actual candidate and wire bijections precede semantic disposition; counts alone prove none.
use super::{
    candidate_wires, failure::*, geometry, nesting::TwoLoopNestingProof, outcome::*, request::*,
    work::*,
};
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_candidates::RegionCandidate;
use crate::sketch_region_source_conformance::capture_support;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn capture<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    sources: &[TwoLoopSourceProof],
    nesting: &TwoLoopNestingProof,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Vec<TwoLoopCandidateProof>> {
    let batch = super::super::borrowed_batch(request.bridge).ok_or_else(|| {
        work.failure(TwoLoopRegionCause::Missing {
            hypothesis: "actual candidate batch",
        })
    })?;
    work.count(
        "complete nested-loop candidate partition",
        batch.candidates.len(),
        3,
    )?;
    work.count(
        "all actual candidate bindings",
        request.identities.candidates.len(),
        3,
    )?;
    let originals = super::member_parents::originals(request, work)?;
    let mut result = Vec::new();
    for binding in &request.identities.candidates {
        let actual: Vec<_> = batch
            .candidates
            .iter()
            .filter(|c| PartitionOccurrenceIdentity::capture(&c.face) == binding.actual)
            .collect();
        work.count("one actual candidate per authored binding", actual.len(), 1)?;
        result.push(one(actual[0], binding, sources, nesting, &originals, work)?);
    }
    dispositions(&result, request.identities.selected, work)?;
    result.sort_by_key(|row| row.name);
    Ok(result)
}
fn one<'a>(
    candidate: &RegionCandidate,
    binding: &TwoLoopCandidateBinding,
    sources: &[TwoLoopSourceProof],
    nesting: &TwoLoopNestingProof,
    originals: &[super::member_parents::OriginalUseBinding],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopCandidateProof> {
    geometry::kind(&candidate.face, ShapeEnum::Face, work)?;
    geometry::reversed(candidate.face.orientation(), work)?;
    let actual = geometry::children(&candidate.face, ShapeEnum::Wire, 2, work)?;
    work.count(
        "all actual wire bindings",
        binding.wires.len(),
        actual.len(),
    )?;
    work.count(
        "all actual metadata wires",
        candidate.wires.len(),
        actual.len(),
    )?;
    for row in &candidate.wires {
        work.count("one full metadata edge per wire", row.segments.len(), 1)?;
    }
    work.count(
        "complete actual member relation roster",
        candidate.relations.len(),
        1 + 4 * actual.len(),
    )?;
    let support = capture_support(&candidate.face)
        .map_err(|e| work.failure(TwoLoopRegionCause::Geometry(std::sync::Arc::new(e))))?;
    work.holds(
        "candidate and source exact plane chart",
        super::super::boundary::same_support(&support, &sources[0].support),
    )?;
    let mut wires = Vec::new();
    for named in &binding.wires {
        let rows: Vec<_> = actual
            .iter()
            .filter(|w| PartitionOccurrenceIdentity::capture(w) == named.actual)
            .collect();
        work.count("one actual wire per authored binding", rows.len(), 1)?;
        wires.push(candidate_wires::capture(
            candidate, rows[0], named, sources, originals, work,
        )?);
    }
    let disposition = disposition(&wires, candidate.face.orientation(), sources, nesting, work)?;
    wires.sort_by_key(|row| row.name);
    work.units("named candidate", 1)?;
    Ok(TwoLoopCandidateProof {
        name: binding.name,
        orientation: candidate.face.orientation(),
        support,
        wires,
        disposition,
        members: super::member_capture::capture(candidate, binding, originals, work)?,
    })
}
fn disposition<'a>(
    wires: &[TwoLoopWireProof],
    face_orientation: rustycad_tkg3d::top_abs::Orientation,
    sources: &[TwoLoopSourceProof],
    nesting: &TwoLoopNestingProof,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopCandidateDisposition> {
    let mut signature = [0_i64; 2];
    for wire in wires {
        let source = sources
            .iter()
            .position(|s| s.name == wire.source)
            .expect("bound source");
        let changed = wire.edge_orientation != sources[source].orientation;
        signature[source] += nesting.native_signs[source] * if changed { -1 } else { 1 };
    }
    if geometry::reversed(face_orientation, work)? {
        signature = signature.map(|v| -v);
    }
    match (wires.len(), signature) {
        (2, [1, -1]) => Ok(TwoLoopCandidateDisposition::SelectedAnnulus),
        (1, [0, 1]) => Ok(TwoLoopCandidateDisposition::RejectedInnerDomain),
        (1, [-1, 0]) => Ok(TwoLoopCandidateDisposition::RejectedOuterComplement),
        _ => Err(work.failure(TwoLoopRegionCause::CandidateSignature {
            actual: signature,
            wires: wires.len(),
        })),
    }
}
fn dispositions<'a>(
    rows: &[TwoLoopCandidateProof],
    selected: u64,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    use TwoLoopCandidateDisposition::*;
    for disposition in [
        SelectedAnnulus,
        RejectedInnerDomain,
        RejectedOuterComplement,
    ] {
        work.count(
            "exact complete semantic candidate disposition",
            rows.iter().filter(|r| r.disposition == disposition).count(),
            1,
        )?;
    }
    work.holds(
        "selected authored candidate is the proved annulus",
        rows.iter()
            .any(|r| r.name == selected && r.disposition == SelectedAnnulus),
    )
}
