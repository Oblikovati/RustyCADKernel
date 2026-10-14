// SPDX-License-Identifier: GPL-2.0-only
//! Per-source unsplit partition metadata and source-keyed closure witnesses from the sealed proof.
use super::super::complete_history::{observed_values, partition_metadata};
use super::sources::AnnularClosureWitness;
use super::*;
use crate::sketch_region::{
    OwnedPartitionSnapshot, PartitionOccurrenceIdentity, SourcePartitionObservation,
};

/// Capture this source's unsplit native metadata through its own InputCopy scope.
pub(super) fn capture(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    source: RegionSourceId,
    original: &BoundHistoricalContexts<'_>,
    copied: &BoundHistoricalContexts<'_>,
) -> HistoryCheck<PrismPartitionMetadata> {
    let snapshot = super::scopes::partition_snapshot(request)?;
    admit_operation_census(request, snapshot)?;
    let edge = source_row(snapshot, copied)?;
    admit_full_range(edge)?;
    let vertices = vertices(request, snapshot, copied)?;
    let missing = PrismHistoryCause::Missing {
        required: "native source shape count",
    };
    let count = snapshot.source_shape_count.ok_or(missing)?;
    let native = request.native_inputs();
    let result = partition_metadata::metadata(&native, copied, edge, count, vertices)?;
    admit_parents(source, original, &result)?;
    Ok(result)
}
fn admit_operation_census(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    snapshot: &OwnedPartitionSnapshot,
) -> HistoryCheck<()> {
    let sources = request.region.sources.len();
    let expected = i32::try_from(2 * sources).ok();
    let required = "two physical source definitions per closed source edge";
    observed_values::equal(&snapshot.source_shape_count, &expected, required)?;
    let required = "one native partition source per authored source";
    history_count(required, snapshot.sources.len(), sources)?;
    let required = "one native closure vertex per authored source";
    history_count(required, snapshot.vertices.len(), sources)
}
fn source_row<'a>(
    snapshot: &'a OwnedPartitionSnapshot,
    copied: &BoundHistoricalContexts<'_>,
) -> HistoryCheck<&'a SourcePartitionObservation> {
    let rows = snapshot
        .sources
        .iter()
        .filter(|s| in_scope(copied, &s.occurrence));
    let rows: Vec<_> = rows.collect();
    let required = "one native partition source in this source's copy scope";
    history_count(required, rows.len(), 1)?;
    Ok(rows[0])
}
fn vertices(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    snapshot: &OwnedPartitionSnapshot,
    copied: &BoundHistoricalContexts<'_>,
) -> HistoryCheck<Vec<PrismPartitionVertex>> {
    let native = request.native_inputs();
    let rows = snapshot
        .vertices
        .iter()
        .filter(|v| in_scope(copied, &v.occurrence));
    let project = |v| partition_metadata::vertex(&native, copied, v);
    let result = rows.map(project).collect::<HistoryCheck<Vec<_>>>()?;
    let required = "one native physical closure vertex per source";
    history_count(required, result.len(), 1)?;
    Ok(result)
}
fn in_scope(scope: &BoundHistoricalContexts<'_>, actual: &PartitionOccurrenceIdentity) -> bool {
    scope
        .actual
        .iter()
        .any(|r| same_definition(&r.occurrence, actual))
}
fn admit_full_range(source: &SourcePartitionObservation) -> HistoryCheck<()> {
    let range = source.range.map(f64::to_bits);
    let full = [0.0f64.to_bits(), 1.0f64.to_bits()];
    observed_values::equal(&range, &full, "full polynomial native interval")?;
    partition_metadata::admit_unsplit(source)
}
/// Edge parent is the source root use; the vertex retains every other original use of that source.
fn admit_parents(
    source: RegionSourceId,
    original: &BoundHistoricalContexts<'_>,
    metadata: &PrismPartitionMetadata,
) -> HistoryCheck<()> {
    let root = original_keys(source, original, true)?;
    let required = "partition edge retains its source root original";
    observed_values::equal(&metadata.parents, &root, required)?;
    let others = original_keys(source, original, false)?;
    let required = "partition vertex retains both original closure use parents";
    observed_values::equal(&metadata.vertices[0].parents, &others, required)
}
fn original_keys(
    source: RegionSourceId,
    original: &BoundHistoricalContexts<'_>,
    root: bool,
) -> HistoryCheck<Vec<FrozenOriginalKey>> {
    let rows = original.owned.iter().zip(original.actual);
    let rows = rows.filter(|(_, actual)| actual.child_slots.is_empty() == root);
    let keys = rows.map(|(c, _)| original_key(source, c));
    Ok(observed_values::sorted(keys.collect::<HistoryCheck<_>>()?))
}
/// Recover the authored original key of one bound Original-scope context.
pub(super) fn original_key(
    source: RegionSourceId,
    context: &PrismHistoricalContext,
) -> HistoryCheck<FrozenOriginalKey> {
    let (_, directed_use) = context.key.0.last().ok_or(PrismHistoryCause::Missing {
        required: "original directed use key",
    })?;
    Ok(FrozenOriginalKey {
        source: source.0,
        member: context.occurrence.member.member,
        directed_use: *directed_use,
    })
}
/// Source-key the sealed proof's original and copied closure witnesses; their parameters must
/// equal this source's actual native endpoint parameters.
pub(super) fn closure(
    request: &AnnularHistoryRequest<'_, '_, '_, '_>,
    source: RegionSourceId,
    metadata: &PrismPartitionMetadata,
) -> HistoryCheck<Vec<AnnularClosureWitness>> {
    let sources = request.binding().region().sources();
    let proofs: Vec<_> = sources.iter().filter(|s| s.name == source).collect();
    let required = "one sealed two-loop proof per authored source";
    history_count(required, proofs.len(), 1)?;
    let native = metadata.endpoints.iter().map(|e| e.parameter_bits);
    let native = observed_values::sorted(native.collect());
    let (original, copied) = (&proofs[0].closure, &proofs[0].copied_closure);
    let mut result = witnesses(false, original, &native)?;
    result.extend(witnesses(true, copied, &native)?);
    Ok(observed_values::sorted(result))
}
fn witnesses(
    copied: bool,
    rows: &[TwoLoopEndpointProof],
    native: &Vec<u64>,
) -> HistoryCheck<Vec<AnnularClosureWitness>> {
    history_count("both directed closure witnesses", rows.len(), 2)?;
    let witness = |r: &TwoLoopEndpointProof| AnnularClosureWitness {
        copied,
        parameter_bits: r.parameter.to_bits(),
        orientation: r.orientation as u8,
    };
    let result: Vec<_> = rows.iter().map(witness).collect();
    let parameters = observed_values::sorted(result.iter().map(|w| w.parameter_bits).collect());
    let required = "sealed closure witnesses equal native endpoint parameters";
    observed_values::equal(&parameters, native, required)?;
    Ok(result)
}
