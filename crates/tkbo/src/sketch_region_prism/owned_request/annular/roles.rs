// SPDX-License-Identifier: GPL-2.0-only
//! Authored region names attach to actual candidates by their native source parents only; the
//! two-loop proof later certifies the attachment. Batch order never assigns a name.
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_candidates::RegionCandidate;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use std::collections::BTreeSet;

/// Name every actual candidate and wire from the authored role whose source set it has.
pub(super) fn identities(
    request: &OwnedAnnularPrismRequest,
    bridge: &PairedContactTransaction,
) -> PreparationCheck<TwoLoopRegionIdentities> {
    let batch = crate::sketch_region_prism::region_certificate::borrowed_batch(bridge);
    let batch = local(batch, "actual completed candidate batch")?;
    let mut candidates = Vec::new();
    for candidate in &batch.candidates {
        candidates.push(candidate_binding(request, candidate)?);
    }
    let names: BTreeSet<_> = candidates.iter().map(|c| c.name).collect();
    let required = "one actual candidate per authored region role";
    role_count(required, names.len(), 3)?;
    role_count("exactly three actual candidates", candidates.len(), 3)?;
    let [outer, inner] = request.boundaries().map(|(_, id)| id);
    Ok(TwoLoopRegionIdentities {
        operation: request.operation.0,
        outer,
        inner,
        selected: request.regions.annulus,
        candidates,
    })
}
fn candidate_binding(
    request: &OwnedAnnularPrismRequest,
    candidate: &RegionCandidate,
) -> PreparationCheck<TwoLoopCandidateBinding> {
    let name = region_name(request, &candidate_sources(candidate))?;
    let mut wires = Vec::new();
    let mut iterator = ShapeIterator::new(&candidate.face, true, true);
    while iterator.more() {
        let wire = iterator.value().map_err(OwnedPreparationCause::Native)?;
        wires.push(wire_binding(request, candidate, name, wire)?);
        iterator.next();
    }
    let actual = PartitionOccurrenceIdentity::capture(&candidate.face);
    Ok(TwoLoopCandidateBinding {
        name,
        actual,
        wires,
    })
}
fn candidate_sources(candidate: &RegionCandidate) -> BTreeSet<RegionSourceId> {
    let segments = candidate.wires.iter().flat_map(|w| &w.segments);
    segments
        .flat_map(|s| &s.parents)
        .map(|p| p.source)
        .collect()
}
/// Annulus: both sources; inner domain: inner only; outer complement: outer only.
fn region_name(
    request: &OwnedAnnularPrismRequest,
    sources: &BTreeSet<RegionSourceId>,
) -> PreparationCheck<u64> {
    let [outer, inner] = request.boundaries().map(|(_, id)| id);
    let names = request.regions;
    let roles = [
        (BTreeSet::from([outer, inner]), names.annulus),
        (BTreeSet::from([inner]), names.inner_domain),
        (BTreeSet::from([outer]), names.outer_complement),
    ];
    let found: Vec<_> = roles.iter().filter(|(set, _)| set == sources).collect();
    role_count(
        "candidate source set matches one authored region role",
        found.len(),
        1,
    )?;
    Ok(found[0].1)
}
fn wire_binding(
    request: &OwnedAnnularPrismRequest,
    candidate: &RegionCandidate,
    region: u64,
    wire: &Shape,
) -> PreparationCheck<TwoLoopWireBinding> {
    let source = wire_source(candidate, wire)?;
    let name = wire_name(request, region, source)?;
    let actual = PartitionOccurrenceIdentity::capture(wire);
    Ok(TwoLoopWireBinding {
        name,
        actual,
        source,
    })
}
/// The single native source parent of every segment edge under this actual wire.
fn wire_source(candidate: &RegionCandidate, wire: &Shape) -> PreparationCheck<RegionSourceId> {
    let mut sources = BTreeSet::new();
    let mut edges = ShapeIterator::new(wire, true, true);
    while edges.more() {
        let edge = PartitionOccurrenceIdentity::capture(
            edges.value().map_err(OwnedPreparationCause::Native)?,
        );
        let segments = candidate.wires.iter().flat_map(|w| &w.segments);
        let same = segments.filter(|s| PartitionOccurrenceIdentity::capture(&s.edge) == edge);
        sources.extend(same.flat_map(|s| &s.parents).map(|p| p.source));
        edges.next();
    }
    role_count("one native source parent per actual wire", sources.len(), 1)?;
    Ok(*sources.first().expect("one checked source"))
}
fn wire_name(
    request: &OwnedAnnularPrismRequest,
    region: u64,
    source: RegionSourceId,
) -> PreparationCheck<u64> {
    let [outer, inner] = request.boundaries().map(|(_, id)| id);
    let names = request.regions;
    let roles = [
        (names.annulus, outer, names.annulus_outer_wire),
        (names.annulus, inner, names.annulus_inner_wire),
        (names.inner_domain, inner, names.inner_domain_wire),
        (names.outer_complement, outer, names.outer_complement_wire),
    ];
    let found: Vec<_> = roles
        .iter()
        .filter(|(r, s, _)| *r == region && *s == source)
        .collect();
    role_count("wire source matches one authored wire role", found.len(), 1)?;
    Ok(found[0].2)
}
fn role_count(required: &'static str, actual: usize, expected: usize) -> PreparationCheck<()> {
    if actual == expected {
        return Ok(());
    }
    Err(AnnularPreparationCause::Role {
        required,
        actual,
        expected,
    })
}
