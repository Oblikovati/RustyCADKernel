// Created by: Eugeny MALTCHIKOV
// Created on: 2019-04-17
// Copyright (c) 2019 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of FoundationClasses/TKMath/BVH/BVH_Traverse.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_BaseTraverse`, `BVH_Traverse` and `BVH_PairTraverse`: rule-driven
//! descent of one binary tree or of two trees in parallel.
//!
//! The selector supplies `RejectNode` and `Accept`; the base rules decide the
//! branch order (`IsMetricBetter`), prune stacked nodes (`RejectMetric`) and end
//! the descent (`Stop`). Example: `selector.select_tree(Some(&tree))?`.
use rustycad_tkernel::standard::Failure;
mod pair;
mod single;
pub use pair::{BvhPairNodesInStack, BvhPairSetTraverse, BvhPairTraverse, select_pairs};
pub use single::{BvhNodeInStack, BvhSetTraverse, BvhTraverse, select_elements};

/// The metric rules shared by single and pair traversals (`BVH_BaseTraverse`).
///
/// The defaults keep the left-to-right descent, never reject by metric and
/// never stop, as upstream. Example: `impl BvhBaseTraverse<f64> for Selector {}`.
pub trait BvhBaseTraverse<M> {
    /// True when the first metric is better than the second (default: true).
    fn is_metric_better(&self, _left: &M, _right: &M) -> bool {
        true
    }
    /// True when a node must be rejected by its metric (default: false).
    fn reject_metric(&self, _metric: &M) -> bool {
        false
    }
    /// True when the descent must end now (default: false).
    fn stop(&self) -> bool {
        false
    }
}

/// The fixed-size `aStack[]` with its `aHead` index.
struct TraverseStack<E> {
    entries: Vec<E>,
    capacity: usize,
    overflow: &'static str,
}
impl<E: Copy> TraverseStack<E> {
    fn new(capacity: usize, overflow: &'static str) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            capacity,
            overflow,
        }
    }
    /// `Standard_ASSERT_RAISE(aHead < capacity - 1, ...)`, which stays active in
    /// release builds and throws `Standard_ProgramError`.
    fn ensure_room(&self) -> Result<(), Failure> {
        if self.entries.len() < self.capacity {
            return Ok(());
        }
        Err(Failure::with_type(
            "Standard_ProgramError",
            &format!(
                "*** ERROR: ASSERT in file 'BVH_Traverse.lxx': \n{} (head {}, expected < {})",
                self.overflow,
                self.entries.len() as i32 - 1,
                self.capacity - 1
            ),
            "",
        ))
    }
    fn push(&mut self, entry: E) {
        self.entries.push(entry);
    }
    fn pop(&mut self) -> Option<E> {
        self.entries.pop()
    }
}

/// Pops stacked entries until one survives `RejectMetric`; `None` ends the descent.
fn pop_surviving<E: Copy, M>(
    stack: &mut TraverseStack<E>,
    rules: &(impl BvhBaseTraverse<M> + ?Sized),
    metric: impl Fn(&E) -> &M,
) -> Option<E> {
    loop {
        let entry = stack.pop()?;
        if !rules.reject_metric(metric(&entry)) {
            return Some(entry);
        }
    }
}
