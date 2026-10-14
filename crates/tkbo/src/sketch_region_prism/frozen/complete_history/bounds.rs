// SPDX-License-Identifier: GPL-2.0-only
//! Preflight before legacy recapture: detect TShape cycles even when placement changes.
use super::limits::*;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};

/// Bound an actual hierarchy before recapture; `preflight_history_root(&root, limits)`.
/// This grants no topology validity and excludes concurrent alias writes between reads.
pub(in crate::sketch_region_prism) fn preflight_history_root(
    root: &Shape,
    requested: PrismHistoryLimits,
) -> Result<usize, HistoryBoundCause> {
    preflight_history_census(root, requested).map(|census| census.directed_uses)
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct HistoryHierarchyCensus {
    pub directed_uses: usize,
    pub ancestor_elements: usize,
}
/// Count actual ancestry in the same bounded walk; `preflight_history_census(root, limits)`.
pub(in crate::sketch_region_prism) fn preflight_history_census(
    root: &Shape,
    requested: PrismHistoryLimits,
) -> Result<HistoryHierarchyCensus, HistoryBoundCause> {
    let effective = requested.admit()?;
    let mut census = HistoryHierarchyCensus::default();
    visit(root, effective, &mut Vec::new(), &mut census)?;
    Ok(census)
}
fn visit(
    root: &Shape,
    limits: PrismHistoryLimits,
    ancestors: &mut Vec<Shape>,
    census: &mut HistoryHierarchyCensus,
) -> Result<(), HistoryBoundCause> {
    limits.count("ancestor depth", ancestors.len(), limits.maximum_depth)?;
    check_cycle(root, ancestors)?;
    limits.count(
        "directed uses",
        census.directed_uses + 1,
        limits.maximum_uses,
    )?;
    // Admitted limits bound these sums by 16,384 uses times at most 64 ancestors.
    census.directed_uses += 1;
    census.ancestor_elements += ancestors.len();
    ancestors.push(root.clone());
    visit_children(root, limits, ancestors, census)?;
    ancestors.pop();
    Ok(())
}
fn check_cycle(root: &Shape, ancestors: &[Shape]) -> Result<(), HistoryBoundCause> {
    if let Some(ancestor_depth) = ancestors.iter().position(|parent| parent.is_partner(root)) {
        return Err(HistoryBoundCause::AncestorCycle {
            ancestor_depth,
            repeated_depth: ancestors.len(),
        });
    }
    Ok(())
}
fn visit_children(
    root: &Shape,
    limits: PrismHistoryLimits,
    ancestors: &mut Vec<Shape>,
    census: &mut HistoryHierarchyCensus,
) -> Result<(), HistoryBoundCause> {
    let mut children = ShapeIterator::new(root, true, true);
    while children.more() {
        let child = children.value().map_err(HistoryBoundCause::Native)?;
        visit(child, limits, ancestors, census)?;
        children.next();
    }
    Ok(())
}
