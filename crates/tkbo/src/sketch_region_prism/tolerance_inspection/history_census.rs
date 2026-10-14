// SPDX-License-Identifier: GPL-2.0-only
//! Attach existing shared-binding-validated relation dispositions to every actual directed use.
use super::super::PrismBodyContext;
use super::{census_contexts::*, census_types::*};
use crate::sketch_region::ArrangementRelation;

pub(super) fn capture_history(
    contexts: &[PrismBodyContext],
    relations: &[ArrangementRelation],
) -> CensusCheck<Vec<ToleranceUseHistory>> {
    let mut history = Vec::new();
    for context in contexts {
        let key = capture_context(context);
        history.push(attach_context(
            bind_relation(context, relations, key.clone()),
            &key,
        )?);
    }
    Ok(history)
}

fn bind_relation(
    context: &PrismBodyContext,
    relations: &[ArrangementRelation],
    key: ToleranceUseContext,
) -> CensusCheck<ToleranceUseHistory> {
    let matches: Vec<_> = relations
        .iter()
        .filter(|relation| relation.output.is_equal(&context.occurrence))
        .collect();
    count("current parent relation for directed use", matches.len(), 1)?;
    let relation = matches[0];
    Ok(ToleranceUseHistory {
        context: key,
        preserved: relation.preserved.clone(),
        modified: relation.modified.clone(),
        generated: relation.generated.clone(),
    })
}
