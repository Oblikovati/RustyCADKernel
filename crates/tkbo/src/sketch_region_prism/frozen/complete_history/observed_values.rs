// SPDX-License-Identifier: GPL-2.0-only
//! Native orientation remains an operand even when physical lookup uses a common member key.
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
pub(in crate::sketch_region_prism::frozen) fn occurrence(
    bound: &BoundHistoricalContexts<'_>,
    actual: &PartitionOccurrenceIdentity,
) -> HistoryCheck<PrismHistoricalOccurrence> {
    let index = parent_rows::physical_index(bound, actual)?;
    context_names::historical_occurrence(bound.owned[index].occurrence.member, actual)
}
pub(in crate::sketch_region_prism::frozen) fn point(
    value: &crate::sketch_region_vertex_observation::ObservedVertexPoint,
) -> HistoryCheck<Option<[u64; 3]>> {
    value
        .as_ref()
        .map(|v| {
            let point = partition_queries::native_result(v)?;
            history_count(
                "finite historical vertex point",
                usize::from(point.iter().all(|v| v.is_finite())),
                1,
            )?;
            Ok(point.map(f64::to_bits))
        })
        .transpose()
}
pub(in crate::sketch_region_prism::frozen) fn equal<T: PartialEq>(
    actual: &T,
    expected: &T,
    operand: &'static str,
) -> HistoryCheck<()> {
    if actual != expected {
        return Err(PrismHistoryCause::Changed { operand });
    }
    Ok(())
}
pub(in crate::sketch_region_prism::frozen) fn sorted<T: Ord>(mut values: Vec<T>) -> Vec<T> {
    values.sort();
    values
}
