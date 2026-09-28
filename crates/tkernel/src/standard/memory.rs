// Created on: 1991-09-05
// Created by: J.P. TIRAUlt
// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2023 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKernel/Standard/Standard.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Standard` package's global memory services, as the pinned kernel builds them.
//!
//! The oracle kernel is configured with `USE_MMGR_TYPE=NATIVE`, so none of
//! `OCCT_MMGT_OPT_FLEXIBLE`, `OCCT_MMGT_OPT_TBB` or `OCCT_MMGT_OPT_JEMALLOC` is defined:
//! `Allocate` is `calloc`, `AllocateOptimal` is `malloc`, `Reallocate` is `realloc`,
//! `Free` is `free` and `Purge` does nothing. Rust's global allocator is the same system
//! allocator, so the blocks below own ordinary heap storage; fallible reservation reports
//! exactly the requests the C runtime refuses.
use super::Failure;
mod aligned;
pub use aligned::{AlignedMemory, allocate_aligned, free_aligned, free_aligned_and_null};

/// The memory managers `Standard` can be built with; `AllocatorType::Native as i32 == 0`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AllocatorType {
    /// The C runtime allocator (`calloc`/`malloc`/`free`).
    Native = 0,
    /// OCCT's pooled `Standard_MMgrOpt`.
    Opt = 1,
    /// Intel TBB's scalable allocator.
    Tbb = 2,
    /// jemalloc.
    Jemalloc = 3,
}

/// Owned bytes from the `Standard` allocator; `allocate(16)?.as_slice().len() == 16`.
#[derive(Debug, Eq, PartialEq)]
pub struct StandardMemory {
    bytes: Vec<u8>,
}
impl StandardMemory {
    /// Borrows the block's bytes; `block.as_slice()[0]`.
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }
    /// Borrows the block's bytes for writing; `block.as_mut_slice().fill(7)`.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
    /// Returns the usable size requested for the block; `allocate(3)?.len() == 3`.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    /// Reports a zero-byte block (still a live, distinct allocation); `allocate(0)?.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

/// Reports the manager the kernel was built with; `get_allocator_type() == AllocatorType::Native`.
/// The native build selects NATIVE because it defines no `OCCT_MMGT_OPT_*` macro.
pub fn get_allocator_type() -> AllocatorType {
    AllocatorType::Native
}

/// Allocates zero-filled bytes like `calloc`; `allocate(4)?.as_slice() == [0; 4]`.
/// A refused request raises Standard_OutOfMemory with the native message.
pub fn allocate(size: usize) -> Result<StandardMemory, Failure> {
    zeroed_bytes(size)
        .map(|bytes| StandardMemory { bytes })
        .ok_or_else(|| out_of_memory("Standard_MMgrRaw::Allocate(): malloc failed"))
}

/// Allocates like `malloc`, returning `None` instead of raising; `allocate_optimal(8).is_some()`.
/// Native contents are unspecified; Rust hands out zeroed bytes, which is one valid outcome.
pub fn allocate_optimal(size: usize) -> Option<StandardMemory> {
    zeroed_bytes(size).map(|bytes| StandardMemory { bytes })
}

/// Releases a block; `None` is the null pointer, which `free` ignores; `free(Some(block))`.
pub fn free(memory: Option<StandardMemory>) {
    drop(memory);
}

/// The template `Free(T*&)`: releases the block and nulls the slot; `free_and_null(&mut slot)`.
pub fn free_and_null(slot: &mut Option<StandardMemory>) {
    free(slot.take());
}

/// Resizes a block like `realloc`, keeping the common prefix; `reallocate(&mut slot, 64)?`.
///
/// The slot is the native pointer: `None` behaves as `malloc`. On success it holds the new
/// block. A refused request raises Standard_OutOfMemory and leaves the old block in place.
/// glibc and the MSVC runtime free the block and return null for a zero size, so the port
/// empties the slot and raises Standard_OutOfMemory there too (a native quirk).
pub fn reallocate(slot: &mut Option<StandardMemory>, new_size: usize) -> Result<(), Failure> {
    let failure = || out_of_memory("Standard_MMgrRaw::Reallocate(): realloc failed");
    if new_size == 0 && REALLOC_ZERO_FREES && slot.is_some() {
        *slot = None;
        return Err(failure());
    }
    let Some(memory) = slot.as_mut() else {
        *slot = allocate_optimal(new_size);
        return slot.as_ref().map(|_| ()).ok_or_else(failure);
    };
    resize_bytes(&mut memory.bytes, new_size).ok_or_else(failure)
}

/// Releases cached memory; the NATIVE build caches nothing and returns true (1); `purge() == 1`.
pub fn purge() -> i32 {
    1
}

// realloc(ptr, 0) frees the block and returns NULL on glibc and MSVC; other C runtimes
// (macOS, musl) return a live minimum-size block instead.
const REALLOC_ZERO_FREES: bool = cfg!(any(
    all(target_os = "linux", target_env = "gnu"),
    target_os = "windows"
));

/// Reserves exactly `size` zeroed bytes, or `None` when the allocator refuses; `zeroed_bytes(3)`.
pub(crate) fn zeroed_bytes(size: usize) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    resize_bytes(&mut bytes, size)?;
    Some(bytes)
}

fn resize_bytes(bytes: &mut Vec<u8>, size: usize) -> Option<()> {
    if size > bytes.len() {
        bytes.try_reserve_exact(size - bytes.len()).ok()?;
    }
    bytes.resize(size, 0);
    Some(())
}

/// Standard_OutOfMemory (a Standard_ProgramError) with `message`; `out_of_memory("malloc failed")`.
pub(crate) fn out_of_memory(message: &str) -> Failure {
    // The class is registered in exceptions::exception_parent, so catch-by-base works.
    Failure::with_message("Standard_OutOfMemory", message)
}
