// SPDX-License-Identifier: GPL-2.0-only
//! Review d9daef1bc: cardinality cannot prove coverage; compare complete directed binding multisets.
use super::super::bindings::BoundProjection;
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SweepBindingOperand {
    copied: PrismHistoryUseKey,
    role: FrozenSweepRole,
    output: FrozenUseKey,
    parents: [Vec<FrozenOriginalKey>; 3],
}
pub(super) fn compare(
    request: &PrismHistoryRequest<'_, '_>,
    bound: &BoundProjection<'_>,
    copied: &BoundHistoricalContexts<'_>,
    queries: &[PrismCompleteSweepQuery],
) -> HistoryCheck<()> {
    let mut expected = Vec::new();
    for binding in &bound.diagnostic.bindings {
        expected.push(actual_binding(request, bound, copied, binding)?);
    }
    let mut actual = Vec::new();
    for query in queries {
        append_query(query, &mut actual);
    }
    observed_values::equal(
        &observed_values::sorted(actual),
        &observed_values::sorted(expected),
        "exact exhaustive query/role/directed-output/parent multiset",
    )
}
fn actual_binding(
    request: &PrismHistoryRequest<'_, '_>,
    bound: &BoundProjection<'_>,
    copied: &BoundHistoricalContexts<'_>,
    binding: &crate::sketch_region_prism::PrismImageBinding,
) -> HistoryCheck<SweepBindingOperand> {
    let query = parent_rows::occurrence_index(
        copied,
        &PartitionOccurrenceIdentity::capture(&binding.native.query),
    )?;
    let output = coverage_output(bound, binding)?;
    Ok(SweepBindingOperand {
        copied: copied.owned[query].key.clone(),
        role: binding.native.role.into(),
        output,
        parents: parent_rows::relation_parents(request, &binding.native.returned)?,
    })
}
fn append_query(query: &PrismCompleteSweepQuery, rows: &mut Vec<SweepBindingOperand>) {
    for (role, answers) in [
        (FrozenSweepRole::Generated, query.generated.as_slice()),
        (FrozenSweepRole::First, query.first.as_slice()),
        (FrozenSweepRole::Last, query.last.as_slice()),
    ] {
        for answer in answers {
            append_answer(query, role, answer, rows);
        }
    }
}
fn append_answer(
    query: &PrismCompleteSweepQuery,
    role: FrozenSweepRole,
    answer: &PrismNativeSweepAnswer,
    rows: &mut Vec<SweepBindingOperand>,
) {
    for output in &answer.outputs {
        rows.push(SweepBindingOperand {
            copied: query.copied.clone(),
            role,
            output: output.clone(),
            parents: answer.parents.clone(),
        });
    }
}

fn coverage_output(
    bound: &BoundProjection<'_>,
    binding: &crate::sketch_region_prism::PrismImageBinding,
) -> HistoryCheck<FrozenUseKey> {
    let rows: Vec<_> = bound
        .contexts
        .iter()
        .enumerate()
        .filter(|(_, c)| super::super::identities::same_context(c, &binding.context))
        .collect();
    history_count(
        "one actual output context for exhaustive binding coverage",
        rows.len(),
        1,
    )?;
    Ok(bound.use_names[rows[0].0].clone())
}
