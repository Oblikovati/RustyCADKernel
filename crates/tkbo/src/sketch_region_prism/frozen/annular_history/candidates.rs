// SPDX-License-Identifier: GPL-2.0-only
//! Every authored candidate binds its sealed disposition, actual generated area and receipt history.
use super::super::complete_history::{
    candidate_copy, copied_contexts, generated, observed_values, relations,
};
use super::sources::bound_scope;
use super::*;
use crate::sketch_region_candidates::{
    GeneratedAreaSnapshot, GeneratedTopologyObservation, RegionCandidate,
};

/// Operation-level partition-to-candidate copy correspondence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnnularCandidateCopy {
    pub queries: Vec<PrismHistoricalCopyQuery>,
    pub links: Vec<PrismCandidateCopyLink>,
    pub tree: Vec<PrismCopiedTreeMember>,
    pub administrative_container: PrismHistoryUseKey,
}
/// One authored wire of one candidate; `wire.source` is the authored boundary role.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AnnularCandidateWire {
    pub name: u64,
    pub source: RegionSourceId,
    pub wire: PrismHistoryUseKey,
}
/// Complete history of one authored candidate and its sealed disposition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnnularCandidateHistory {
    pub name: u64,
    pub scope: AnnularHistoryScope,
    pub disposition: TwoLoopCandidateDisposition,
    pub face: PrismHistoryUseKey,
    pub wires: Vec<AnnularCandidateWire>,
    pub generated: Vec<PrismGeneratedMember>,
    pub relations: Vec<PrismHistoricalRelation>,
}
/// Project candidate copy queries, member links, physical tree and administrative container.
pub(super) fn copy(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    scopes: &[AnnularBoundScope<'_>],
    partition_relations: &[PrismHistoricalRelation],
) -> HistoryCheck<AnnularCandidateCopy> {
    let source = bound_scope(scopes, AnnularHistoryScope::Partition)?;
    let copied = bound_scope(scopes, AnnularHistoryScope::CandidateCopy)?;
    let observed = super::scopes::candidate_copy(request)?;
    let queries = copied_contexts::project_copier_queries(observed, source, copied)?;
    let native = request.native_inputs();
    let links = &candidate_copy::inspection(&native)?.links;
    let links = candidate_copy::project_copy_links(
        &native,
        links,
        source,
        copied,
        &queries,
        partition_relations,
    )?;
    Ok(AnnularCandidateCopy {
        queries,
        links,
        tree: candidate_copy::project_tree(&native, copied)?,
        administrative_container: container(copied)?,
    })
}
fn container(copied: &BoundHistoricalContexts<'_>) -> HistoryCheck<PrismHistoryUseKey> {
    let index = root_index(copied)?;
    let kind = copied.actual[index].kind;
    let container = kind == rustycad_tkg3d::top_abs::ShapeEnum::Compound;
    holds("copied administrative root is a container", container)?;
    Ok(copied.owned[index].key.clone())
}
/// Lookup-only index of the scope's unique root context.
fn root_index(bound: &BoundHistoricalContexts<'_>) -> HistoryCheck<usize> {
    let rows: Vec<_> = bound
        .actual
        .iter()
        .enumerate()
        .filter(|(_, r)| r.child_slots.is_empty())
        .map(|(i, _)| i)
        .collect();
    history_count("one root context per history scope", rows.len(), 1)?;
    Ok(rows[0])
}
/// Capture all authored candidates in name order and bind their receipts.
pub(super) fn capture(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    scopes: &[AnnularBoundScope<'_>],
    copy: &AnnularCandidateCopy,
) -> HistoryCheck<Vec<AnnularCandidateHistory>> {
    let areas = areas(request)?;
    let authored = request.region.identities.candidates.len();
    history_count(
        "one generated area per authored candidate",
        areas.len(),
        authored,
    )?;
    let mut result = Vec::new();
    for binding in &request.region.identities.candidates {
        result.push(candidate(request, scopes, copy, areas, binding)?);
    }
    result.sort_by_key(|c| c.name);
    super::receipts::admit(request, scopes, &result)?;
    Ok(result)
}
fn candidate(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    scopes: &[AnnularBoundScope<'_>],
    copy: &AnnularCandidateCopy,
    areas: &[GeneratedAreaSnapshot],
    binding: &TwoLoopCandidateBinding,
) -> HistoryCheck<AnnularCandidateHistory> {
    let scope = super::scopes::candidate_scope(request, binding.name);
    let bound = bound_scope(scopes, scope)?;
    let copied = bound_scope(scopes, AnnularHistoryScope::CandidateCopy)?;
    let actual = super::scopes::candidate(request, &binding.actual)?;
    let area = area(request, areas, bound, actual, scope)?;
    let generated = generated::project_area(&request.native_inputs(), bound, copied, area)?;
    let relations = candidate_relations(request, bound, scope, actual, &generated, copy)?;
    Ok(AnnularCandidateHistory {
        name: binding.name,
        scope,
        disposition: disposition(request, binding.name, scope)?,
        face: bound.owned[root_index(bound)?].key.clone(),
        wires: wires(request, bound, binding)?,
        generated,
        relations,
    })
}
fn candidate_relations(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    bound: &BoundHistoricalContexts<'_>,
    scope: AnnularHistoryScope,
    actual: &RegionCandidate,
    generated: &[PrismGeneratedMember],
    copy: &AnnularCandidateCopy,
) -> HistoryCheck<Vec<PrismHistoricalRelation>> {
    let (native, links) = (&actual.relations, &copy.links);
    let publication = request.publication;
    relations::generated_relations(publication, bound, scope.stage(), native, generated, links)
}
fn areas<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
) -> HistoryCheck<&'a [GeneratedAreaSnapshot]> {
    match &candidate_copy::inspection(&request.native_inputs())?.generated_topology {
        GeneratedTopologyObservation::Observed(rows) => Ok(rows),
        GeneratedTopologyObservation::NotReached => Err(PrismHistoryCause::Missing {
            required: "generated topology observation",
        }),
        GeneratedTopologyObservation::Incomplete { areas, cause_index } => {
            Err(PrismHistoryCause::UnsupportedDiagnostic {
                stage: PrismHistoryStage::CandidateCopy,
                kind: HistoricalDiagnosticKind::GeneratedObservation,
                observation: Some(*cause_index),
                retained_rows: areas.len(),
            })
        }
    }
}
/// The generated area whose actual contexts equal this scope; its ordinal must name the same face.
fn area<'a>(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    areas: &'a [GeneratedAreaSnapshot],
    bound: &BoundHistoricalContexts<'_>,
    actual: &RegionCandidate,
    scope: AnnularHistoryScope,
) -> HistoryCheck<&'a GeneratedAreaSnapshot> {
    let mut found = Vec::new();
    for row in areas {
        if retained_history_contexts(&row.contexts, scope.stage())? == bound.actual {
            found.push(row);
        }
    }
    let required = "one actual generated area for authored candidate";
    history_count(required, found.len(), 1)?;
    admit_ordinal(request, found[0], actual)?;
    Ok(found[0])
}
fn admit_ordinal(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    area: &GeneratedAreaSnapshot,
    actual: &RegionCandidate,
) -> HistoryCheck<()> {
    let batch = super::scopes::batch(request)?;
    let face = batch
        .candidates
        .get(area.ordinal)
        .ok_or(PrismHistoryCause::Count {
            required: "generated ordinal within actual retained batch",
            actual: area.ordinal,
            expected: batch.candidates.len(),
        })?;
    let same = face.face.is_equal(&actual.face);
    holds("generated ordinal identifies authored candidate", same)
}
/// Borrow the sealed proof row for one authored candidate name.
fn sealed_candidate<'a>(
    request: &'a AnnularHistoryRequest<'_, '_, '_, '_>,
    name: u64,
) -> HistoryCheck<&'a TwoLoopCandidateProof> {
    let candidates = request.binding().region().candidates();
    let proofs: Vec<_> = candidates.iter().filter(|c| c.name == name).collect();
    history_count("one sealed proof per authored candidate", proofs.len(), 1)?;
    Ok(proofs[0])
}
fn disposition(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    name: u64,
    scope: AnnularHistoryScope,
) -> HistoryCheck<TwoLoopCandidateDisposition> {
    let proof = sealed_candidate(request, name)?;
    let selected = proof.disposition == TwoLoopCandidateDisposition::SelectedAnnulus;
    let authored = matches!(scope, AnnularHistoryScope::Selected(_));
    let required = "sealed selected disposition equals authored selected scope";
    holds(required, selected == authored)?;
    Ok(proof.disposition)
}
fn wires(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    bound: &BoundHistoricalContexts<'_>,
    binding: &TwoLoopCandidateBinding,
) -> HistoryCheck<Vec<AnnularCandidateWire>> {
    admit_proof_wires(request, binding)?;
    let mut result = Vec::new();
    for wire in &binding.wires {
        result.push(AnnularCandidateWire {
            name: wire.name,
            source: wire.source,
            wire: wire_key(bound, wire)?,
        });
    }
    Ok(observed_values::sorted(result))
}
fn wire_key(
    bound: &BoundHistoricalContexts<'_>,
    wire: &TwoLoopWireBinding,
) -> HistoryCheck<PrismHistoryUseKey> {
    let is_wire = |r: &RegionObservedContext| {
        r.kind == rustycad_tkg3d::top_abs::ShapeEnum::Wire
            && same_definition(&r.occurrence, &wire.actual)
    };
    let rows: Vec<_> = bound
        .actual
        .iter()
        .enumerate()
        .filter(|(_, r)| is_wire(r))
        .collect();
    let required = "one actual directed wire per authored candidate wire";
    history_count(required, rows.len(), 1)?;
    Ok(bound.owned[rows[0].0].key.clone())
}
/// The authored wire roles must equal the sealed proof's wire names and source roles.
fn admit_proof_wires(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    binding: &TwoLoopCandidateBinding,
) -> HistoryCheck<()> {
    let proof = sealed_candidate(request, binding.name)?;
    let mut sealed: Vec<_> = proof.wires.iter().map(|w| (w.name, w.source)).collect();
    let mut authored: Vec<_> = binding.wires.iter().map(|w| (w.name, w.source)).collect();
    sealed.sort();
    authored.sort();
    let required = "authored candidate wire roles equal sealed proof";
    observed_values::equal(&authored, &sealed, required)
}
