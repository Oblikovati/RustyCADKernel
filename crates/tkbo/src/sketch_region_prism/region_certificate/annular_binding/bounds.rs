// SPDX-License-Identifier: GPL-2.0-only
//! Fixed domain envelopes bound traversal; the caller's cap remains an independent cumulative limit.
use super::super::super::frozen::complete_history::{
    HistoryBoundCause, HistoryHierarchyCensus, PrismHistoryCause, PrismHistoryLimits,
    preflight_history_census,
};
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularHierarchyBudget {
    pub maximum_retained_units: usize,
    pub roots: [HistoryHierarchyCensus; 3],
    pub retained_units: usize,
}
pub(super) fn inspect(
    receipt: &PrismReceipt,
    maximum_retained_units: usize,
) -> AnnularCheck<'static, AnnularHierarchyBudget> {
    let roots = inspect_roots(receipt)?;
    // Each canonical directed context owns one occurrence plus complete ancestor and child-slot rosters.
    // Reused views of those contexts are not separate logical operands or a byte-allocation measure.
    let retained_units = roots
        .iter()
        .map(|r| r.directed_uses + 2 * r.ancestor_elements)
        .sum();
    limit(retained_units, maximum_retained_units)?;
    Ok(AnnularHierarchyBudget {
        maximum_retained_units,
        roots,
        retained_units,
    })
}
fn root(
    shape: &rustycad_tkbrep::topods::Shape,
    maximum_depth: usize,
    maximum_uses: usize,
) -> AnnularCheck<'static, HistoryHierarchyCensus> {
    preflight_history_census(
        shape,
        PrismHistoryLimits {
            maximum_depth,
            maximum_uses,
            maximum_rows: 1,
        },
    )
    .map_err(bound_failure)
}
pub(super) fn limit(actual: usize, maximum: usize) -> AnnularCheck<'static> {
    if actual > maximum {
        return Err(bound_failure(HistoryBoundCause::Exhausted {
            quantity: "annular retained structural units",
            actual,
            maximum,
        }));
    }
    Ok(())
}
fn bound_failure(cause: HistoryBoundCause) -> Box<AnnularBindingCause<'static>> {
    Box::new(AnnularBindingCause::History(Box::new(
        PrismHistoryCause::Bounds(cause),
    )))
}
fn absent(required: &'static str) -> Box<AnnularBindingCause<'static>> {
    Box::new(AnnularBindingCause::Count {
        required,
        actual: 0,
        expected: 1,
    })
}

fn inspect_roots(receipt: &PrismReceipt) -> AnnularCheck<'static, [HistoryHierarchyCensus; 3]> {
    let selected = receipt
        .input
        .selected
        .as_ref()
        .ok_or_else(|| absent("selected hierarchy"))?;
    let copied = receipt
        .copied
        .as_ref()
        .ok_or_else(|| absent("copied hierarchy"))?;
    let body = receipt
        .diagnostic
        .as_ref()
        .ok_or_else(|| absent("swept hierarchy"))?;
    Ok([
        root(&selected.face, 3, 9)?,
        root(&copied.base, 3, 9)?,
        root(&body.body, 5, 48)?,
    ])
}
