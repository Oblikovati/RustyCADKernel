// Copyright (c) 2026 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_ParallelPolicy.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_ParallelPolicy`: whether a phase should launch parallel work.
use rustycad_tkernel::osd::Parallel;
use std::sync::OnceLock;

/// `BRepGraph_ParallelPolicy::Workload`: the visible work of a phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Workload {
    /// `PrimaryItems`: the main loop range.
    pub primary_items: u32,
    /// `AuxiliaryItems`: other independent items of the phase.
    pub auxiliary_items: u32,
    /// `InteractionCount`: pairwise or adjacency work of the phase.
    pub interaction_count: u32,
}

/// `BRepGraph_ParallelPolicy`: a workload-aware parallel launch decision.
///
/// ```
/// use rustycad_tkbrep::brep_graph::parallel_policy::{ParallelPolicy, Workload};
/// let workload = Workload { primary_items: 100, ..Workload::default() };
/// assert!(ParallelPolicy::should_run_with(true, 4, &workload));
/// assert!(!ParallelPolicy::should_run_with(false, 4, &workload));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ParallelPolicy;

impl ParallelPolicy {
    /// `WorkerCount()`: the logical processor count, at least 1, read once.
    pub fn worker_count() -> i32 {
        static WORKERS: OnceLock<i32> = OnceLock::new();
        *WORKERS.get_or_init(|| Parallel::nb_logical_processors().max(1))
    }

    /// `IsParallelAllowed(theAllowParallel)`: allowed and more than one worker.
    pub fn is_parallel_allowed(allow_parallel: bool) -> bool {
        allow_parallel && Self::worker_count() > 1
    }

    /// `ShouldRun(theAllowParallel, theWorkers, theWorkload)`: more primary items than
    /// workers and more total work than the squared worker count.
    pub fn should_run_with(allow_parallel: bool, workers: i32, workload: &Workload) -> bool {
        if !allow_parallel || workers <= 1 {
            return false;
        }
        // The native `std::max(x, 0u)` on an unsigned value is the value itself; the
        // cast to `int` of a count above INT_MAX wraps as natively.
        let primary = workload.primary_items as i32;
        if primary <= 1 || primary <= workers {
            return false;
        }
        let total = i64::from(primary)
            + i64::from(workload.auxiliary_items)
            + i64::from(workload.interaction_count);
        total > i64::from(workers) * i64::from(workers)
    }

    /// `ShouldRun(theAllowParallel, theWorkload)`: with [`worker_count`](Self::worker_count).
    pub fn should_run(allow_parallel: bool, workload: &Workload) -> bool {
        Self::should_run_with(allow_parallel, Self::worker_count(), workload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_workloads_stay_sequential() {
        let few = Workload {
            primary_items: 3,
            ..Workload::default()
        };
        assert!(!ParallelPolicy::should_run_with(true, 4, &few));
        let medium = Workload {
            primary_items: 10,
            auxiliary_items: 7,
            interaction_count: 0,
        };
        assert!(ParallelPolicy::should_run_with(true, 4, &medium));
        assert!(!ParallelPolicy::should_run_with(true, 1, &medium));
        assert!(ParallelPolicy::worker_count() >= 1);
        assert!(!ParallelPolicy::is_parallel_allowed(false));
        let _ = ParallelPolicy::should_run(true, &medium);
    }
}
