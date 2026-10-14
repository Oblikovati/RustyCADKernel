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
    let effective = requested.admit()?;
    let mut count = 0;
    visit(root, effective, &mut Vec::new(), &mut count)?;
    Ok(count)
}
fn visit(
    root: &Shape,
    limits: PrismHistoryLimits,
    ancestors: &mut Vec<Shape>,
    count: &mut usize,
) -> Result<(), HistoryBoundCause> {
    limits.count("ancestor depth", ancestors.len(), limits.maximum_depth)?;
    check_cycle(root, ancestors)?;
    limits.count("directed uses", *count + 1, limits.maximum_uses)?;
    *count += 1; // The admitted bound keeps this addition far below usize overflow.
    ancestors.push(root.clone());
    visit_children(root, limits, ancestors, count)?;
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
    count: &mut usize,
) -> Result<(), HistoryBoundCause> {
    let mut children = ShapeIterator::new(root, true, true);
    while children.more() {
        let child = children.value().map_err(HistoryBoundCause::Native)?;
        visit(child, limits, ancestors, count)?;
        children.next();
    }
    Ok(())
}
