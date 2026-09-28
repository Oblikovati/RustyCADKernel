// Created on: 2015-05-28
// Created by: Denis BOGOLEPOV
// Copyright (c) 2015 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_QueueBuilder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_QueueBuilder::Build` and its `BVH_TypedBuildTool`.
use super::BvhQueueBuilder;
use crate::bvh::bounding_box::BvhBox;
use crate::bvh::build_queue::BvhBuildQueue;
use crate::bvh::build_thread::{BvhBuildThread, BvhBuildTool};
use crate::bvh::set::BvhSet;
use crate::bvh::tree::BvhTree;
use crate::bvh::vector::BvhVector;
use rustycad_tkernel::standard::Failure;
use std::sync::{Mutex, PoisonError};

/// The set and tree under construction, locked for each node split.
///
/// Upstream threads split disjoint nodes concurrently through raw pointers;
/// safe Rust serializes the splits instead. Numbering stays schedule-dependent
/// with several threads, exactly as upstream.
type BuildTarget<'a, V> = (&'a mut dyn BvhSet<V>, &'a mut BvhTree<V>);

/// `BVH_TypedBuildTool`: splits a node, then registers its children.
struct TypedBuildTool<'a, 'b, V: BvhVector, B: BvhQueueBuilder<V> + ?Sized> {
    target: Mutex<BuildTarget<'a, V>>,
    queue: &'b BvhBuildQueue,
    algorithm: &'b B,
}
impl<V: BvhVector, B: BvhQueueBuilder<V> + ?Sized> BvhBuildTool for TypedBuildTool<'_, '_, V, B> {
    fn perform(&self, node: i32) -> Result<(), Failure> {
        let mut target = self.target.lock().unwrap_or_else(PoisonError::into_inner);
        let (set, bvh) = &mut *target;
        let children = self.algorithm.build_node(&mut **set, bvh, node)?;
        self.algorithm
            .add_children(bvh, self.queue, node, &children)
    }
}

/// `BVH_QueueBuilder::Build`: resets the tree, adds the root leaf covering all
/// primitives and splits queued nodes on one or `num_of_threads` threads.
/// Example: `queue_build(self, set, bvh, bounding)` inside `BvhBuilder::build`.
pub fn queue_build<V: BvhVector, B: BvhQueueBuilder<V> + ?Sized>(
    algorithm: &B,
    set: &mut dyn BvhSet<V>,
    bvh: &mut BvhTree<V>,
    bounding: &BvhBox<V>,
) -> Result<(), Failure> {
    bvh.clear();
    let set_size = set.size();
    if set_size == 0 {
        return Ok(());
    }
    let root = bvh.add_leaf_node_box(bounding, 0, set_size - 1);
    if set.size() == 1 {
        return Ok(());
    }
    let queue = BvhBuildQueue::new();
    queue.enqueue(root);
    let threads = algorithm.num_of_threads();
    if threads <= 1 {
        let tool = typed_tool(algorithm, &queue, set, bvh);
        return BvhBuildThread::new(&tool, &queue).execute();
    }
    // Reserve the maximum possible number of nodes in the BVH.
    bvh.reserve(2 * set_size - 1)?;
    run_threads(algorithm, &queue, set, bvh, threads)?;
    // "Free unused memory": a no-op upstream too, Reserve never shrinks.
    let length = bvh.length();
    bvh.reserve(length)
}

fn typed_tool<'a, 'b, V: BvhVector, B: BvhQueueBuilder<V> + ?Sized>(
    algorithm: &'b B,
    queue: &'b BvhBuildQueue,
    set: &'a mut dyn BvhSet<V>,
    bvh: &'a mut BvhTree<V>,
) -> TypedBuildTool<'a, 'b, V, B> {
    TypedBuildTool {
        target: Mutex::new((set, bvh)),
        queue,
        algorithm,
    }
}

/// Starts every worker, then waits for all of them before reporting the first error.
fn run_threads<V: BvhVector, B: BvhQueueBuilder<V> + ?Sized>(
    algorithm: &B,
    queue: &BvhBuildQueue,
    set: &mut dyn BvhSet<V>,
    bvh: &mut BvhTree<V>,
    threads: i32,
) -> Result<(), Failure> {
    let tool = typed_tool(algorithm, queue, set, bvh);
    let workers: Vec<BvhBuildThread<'_>> = (0..threads)
        .map(|_| BvhBuildThread::new(&tool, queue))
        .collect();
    std::thread::scope(|scope| {
        let handles: Vec<_> = workers.iter().map(|worker| worker.run(scope)).collect();
        // Join every worker before reporting, so none is left running.
        let outcomes: Vec<Result<(), Failure>> =
            handles.into_iter().map(BvhBuildThread::wait).collect();
        outcomes.into_iter().collect::<Result<(), Failure>>()
    })
}
