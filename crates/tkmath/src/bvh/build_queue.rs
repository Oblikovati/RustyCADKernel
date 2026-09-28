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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_BuildQueue.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_BuildQueue`: FIFO of node indices shared by BVH build threads.
use std::collections::VecDeque;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// Command queue for parallel building of BVH nodes.
///
/// The sequence is mutex-guarded; the size and busy-thread counters are atomics
/// read without the lock, with the upstream acquire/release orderings.
/// Example: `queue.enqueue(0); assert_eq!(queue.fetch(&mut busy), 0);`.
#[derive(Debug, Default)]
pub struct BvhBuildQueue {
    queue: Mutex<VecDeque<i32>>,
    nb_threads: AtomicI32,
    size: AtomicI32,
}
impl BvhBuildQueue {
    /// Creates new empty BVH build queue; `BvhBuildQueue::new().size() == 0`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Returns current size of BVH build queue (acquire load); `queue.size()`.
    pub fn size(&self) -> i32 {
        self.size.load(Ordering::Acquire)
    }
    /// Enqueues new work-item onto BVH build queue; `queue.enqueue(42)`.
    pub fn enqueue(&self, work_item: i32) {
        let mut queue = self.lock_nodes();
        queue.push_back(work_item);
        self.size.fetch_add(1, Ordering::Release);
    }
    /// Fetches the first work-item, or -1 when the queue is empty.
    ///
    /// `was_busy` carries the calling thread's state between calls: a thread
    /// joins the busy count on its first successful fetch and leaves it on its
    /// first empty fetch. Example: `let node = queue.fetch(&mut was_busy);`.
    pub fn fetch(&self, was_busy: &mut bool) -> i32 {
        let query = self.pop_front().unwrap_or(-1);
        if query != -1 {
            if !*was_busy {
                self.nb_threads.fetch_add(1, Ordering::Release);
            }
        } else if *was_busy {
            self.nb_threads.fetch_sub(1, Ordering::Release);
        }
        *was_busy = query != -1;
        query
    }
    /// Checks if there are active build threads (acquire load); `queue.has_busy_threads()`.
    pub fn has_busy_threads(&self) -> bool {
        self.nb_threads.load(Ordering::Acquire) != 0
    }
    /// Locks the queue mutex. `BVH_QueueBuilder` (a friend upstream) holds it
    /// while appending child nodes so concurrent node creation is serialized.
    pub(crate) fn lock_nodes(&self) -> MutexGuard<'_, VecDeque<i32>> {
        self.queue.lock().unwrap_or_else(PoisonError::into_inner)
    }
    /// Leaves the busy count for a thread abandoning its work after a failure.
    ///
    /// Upstream an escaping exception ends the process; here the failing thread
    /// must not keep the other workers spinning. Example: `queue.abandon(was_busy)`.
    pub(crate) fn abandon(&self, was_busy: bool) {
        if was_busy {
            self.nb_threads.fetch_sub(1, Ordering::Release);
        }
    }
    fn pop_front(&self) -> Option<i32> {
        let mut queue = self.lock_nodes();
        let item = queue.pop_front()?;
        self.size.fetch_sub(1, Ordering::Release);
        Some(item)
    }
}
