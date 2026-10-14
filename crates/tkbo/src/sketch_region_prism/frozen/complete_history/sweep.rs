// SPDX-License-Identifier: GPL-2.0-only
//! Exhaustive actual query answers retain empty Generated, null caps and false retirement explicitly.
use super::super::bindings::BoundProjection;
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_prism::{PrismImageRole, PrismSweepQueries, PrismSweepQueryObservation};
use rustycad_tkbrep::topods::Shape;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismNativeSweepAnswer {
    pub definition: FrozenPrismKey,
    pub parents: [Vec<FrozenOriginalKey>; 3],
    pub native_orientation: u8,
    pub native_placement_bits: [u64; 13],
    pub outputs: Vec<FrozenUseKey>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismCompleteSweepQuery {
    pub copied: PrismHistoryUseKey,
    pub parents: [Vec<FrozenOriginalKey>; 3],
    pub generated: Vec<PrismNativeSweepAnswer>,
    pub first: Option<PrismNativeSweepAnswer>,
    pub last: Option<PrismNativeSweepAnswer>,
    pub retired: bool,
}
pub(super) fn capture(
    request: &PrismHistoryRequest<'_>,
    bound: &BoundProjection<'_>,
    copied: &BoundHistoricalContexts<'_>,
) -> HistoryCheck<Vec<PrismCompleteSweepQuery>> {
    let PrismSweepQueries::Captured(queries) = &bound.diagnostic.sweep_queries else {
        return Err(PrismHistoryCause::Missing {
            required: "exhaustive completed sweep query outcomes",
        });
    };
    history_count(
        "one exhaustive sweep query per copied context",
        queries.len(),
        copied.actual.len(),
    )?;
    let mut result = Vec::new();
    for query in queries {
        result.push(query_row(request, bound, copied, query)?);
    }
    history_count(
        "unique copied query contexts",
        result
            .iter()
            .map(|r| &r.copied)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        copied.actual.len(),
    )?;
    sweep_coverage::compare(request, bound, copied, &result)?;
    cover_retirement(request, bound, copied, &result)?;
    Ok(observed_values::sorted(result))
}
fn query_row(
    request: &PrismHistoryRequest<'_>,
    bound: &BoundProjection<'_>,
    copied: &BoundHistoricalContexts<'_>,
    query: &PrismSweepQueryObservation,
) -> HistoryCheck<PrismCompleteSweepQuery> {
    let index = parent_rows::occurrence_index(
        copied,
        &PartitionOccurrenceIdentity::capture(&query.query.output),
    )?;
    let parents = parent_rows::relation_parents(request, &query.query)?;
    let relations = &request
        .receipt
        .copied
        .as_ref()
        .ok_or(PrismHistoryCause::Missing {
            required: "sweep copied base",
        })?
        .relations;
    let matches: Vec<_> = relations
        .iter()
        .filter(|r| r.output.is_equal(&query.query.output))
        .collect();
    history_count("one actual copied relation per query", matches.len(), 1)?;
    relations::same_parents(
        PrismHistoryStage::SweepBase,
        &parents,
        &parent_rows::relation_parents(request, matches[0])?,
    )?;
    let generated = query
        .generated
        .iter()
        .map(|s| required_answer(request, bound, query, PrismImageRole::Generated, s))
        .collect::<HistoryCheck<_>>()?;
    Ok(PrismCompleteSweepQuery {
        copied: copied.owned[index].key.clone(),
        parents,
        generated,
        first: answer(request, bound, query, PrismImageRole::First, &query.first)?,
        last: answer(request, bound, query, PrismImageRole::Last, &query.last)?,
        retired: query.retired,
    })
}
fn answer(
    request: &PrismHistoryRequest<'_>,
    bound: &BoundProjection<'_>,
    query: &PrismSweepQueryObservation,
    role: PrismImageRole,
    native: &Shape,
) -> HistoryCheck<Option<PrismNativeSweepAnswer>> {
    if native.is_null() {
        return Ok(None);
    }
    required_answer(request, bound, query, role, native).map(Some)
}
fn required_answer(
    request: &PrismHistoryRequest<'_>,
    bound: &BoundProjection<'_>,
    query: &PrismSweepQueryObservation,
    role: PrismImageRole,
    native: &Shape,
) -> HistoryCheck<PrismNativeSweepAnswer> {
    let bindings: Vec<_> = bound
        .diagnostic
        .bindings
        .iter()
        .filter(|b| {
            b.native.query.is_equal(&query.query.output)
                && FrozenSweepRole::from(b.native.role) == FrozenSweepRole::from(role)
                && b.native.returned.output.is_equal(native)
        })
        .collect();
    if bindings.is_empty() {
        return Err(PrismHistoryCause::Missing {
            required: "actual transported context for native query answer",
        });
    }
    let parents = parent_rows::relation_parents(request, &bindings[0].native.returned)?;
    let mut outputs = Vec::new();
    let mut definitions = std::collections::BTreeSet::new();
    for binding in bindings {
        relations::same_parents(
            PrismHistoryStage::SweepBase,
            &parent_rows::relation_parents(request, &binding.native.returned)?,
            &parents,
        )?;
        let indices: Vec<_> = bound
            .contexts
            .iter()
            .enumerate()
            .filter(|(_, c)| super::super::identities::same_context(c, &binding.context))
            .map(|(i, _)| i)
            .collect();
        history_count(
            "one actual current output context for bound native answer",
            indices.len(),
            1,
        )?;
        definitions.insert(bound.names[indices[0]]);
        outputs.push(bound.use_names[indices[0]].clone());
    }
    history_count(
        "one physical output definition across transported uses",
        definitions.len(),
        1,
    )?;
    history_count(
        "distinct actual transported use bindings",
        outputs
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        outputs.len(),
    )?;
    let placement = super::super::history::freeze_placement(native.location())
        .map_err(|e| PrismHistoryCause::Frozen(Box::new(e)))?;
    let mut bits = [0; 13];
    let scalar = placement
        .linear
        .into_iter()
        .flatten()
        .chain(placement.origin)
        .chain([placement.scale]);
    for (out, value) in bits.iter_mut().zip(scalar) {
        *out = value.to_bits();
    }
    Ok(PrismNativeSweepAnswer {
        parents,
        definition: *definitions.first().expect("one bound definition"),
        native_orientation: native.orientation() as u8,
        native_placement_bits: bits,
        outputs: observed_values::sorted(outputs),
    })
}
fn cover_retirement(
    request: &PrismHistoryRequest<'_>,
    bound: &BoundProjection<'_>,
    copied: &BoundHistoricalContexts<'_>,
    queries: &[PrismCompleteSweepQuery],
) -> HistoryCheck<()> {
    let retired =
        parent_rows::relation_rows_subset(request, copied, &bound.diagnostic.native_retired)?;
    let expected: Vec<_> = queries
        .iter()
        .filter(|q| q.retired)
        .map(|q| PrismHistoricalRelation {
            output: q.copied.clone(),
            parents: q.parents.clone(),
        })
        .collect();
    observed_values::equal(
        &retired,
        &observed_values::sorted(expected),
        "native retirement rows exactly match exhaustive true answers",
    )
}
