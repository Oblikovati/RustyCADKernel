// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    PrismBodyContext, PrismReceipt, contexts::body_contexts,
    region_certificate::current_binding::CurrentPrismBinding,
};
use super::endpoint_census::capture_endpoints;
use super::{
    census_contexts::*, census_types::*, history_census::capture_history,
    trim_census::capture_trim_census,
};
use crate::sketch_region::ArrangementRelation;

/// Preserve current incidence obligations; example: `capture_census(&receipt, &binding)`.
pub(super) fn capture_census(
    receipt: &PrismReceipt,
    binding: &CurrentPrismBinding,
) -> CensusCheck<PrismToleranceCensus> {
    let present = usize::from(receipt.diagnostic.is_some());
    count("actual prism body", present, 1)?;
    let contexts = body_contexts(&receipt.diagnostic.as_ref().unwrap().body)?;
    capture_census_from_contexts(
        &contexts,
        binding,
        &receipt.diagnostic.as_ref().unwrap().relations,
    )
}

pub(super) fn capture_census_from_contexts(
    contexts: &[PrismBodyContext],
    binding: &CurrentPrismBinding,
    relations: &[ArrangementRelation],
) -> CensusCheck<PrismToleranceCensus> {
    let keys = checked_contexts(contexts)?;
    let entities = capture_entities(contexts)?;
    let trims = capture_trim_census(contexts, binding)?;
    let history = capture_history(contexts, relations)?;
    let endpoints = capture_endpoints(contexts, &entities, binding)?;
    Ok(PrismToleranceCensus {
        contexts: keys,
        entities,
        trims,
        history,
        endpoints,
    })
}

fn checked_contexts(contexts: &[PrismBodyContext]) -> CensusCheck<Vec<ToleranceUseContext>> {
    for context in contexts {
        attach_context(check_parent_kinds(context), &capture_context(context))?;
    }
    capture_all_contexts(contexts)
}
