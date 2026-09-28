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

//! `Standard::AllocateAligned` / `FreeAligned` with the C runtime's alignment rules.
use super::zeroed_bytes;

/// Owned bytes whose first byte sits on the requested boundary; `block.address() % 16 == 0`.
///
/// The storage over-allocates `alignment - 1` bytes and starts the block at the first
/// aligned address inside it, so no unsafe allocator call is needed. The heap buffer never
/// moves while the block lives, which keeps the start address stable.
#[derive(Debug)]
pub struct AlignedMemory {
    storage: Vec<u8>,
    offset: usize,
    length: usize,
}
impl AlignedMemory {
    /// Borrows the aligned bytes; `block.as_slice().len()`.
    pub fn as_slice(&self) -> &[u8] {
        &self.storage[self.offset..self.offset + self.length]
    }
    /// Borrows the aligned bytes for writing; `block.as_mut_slice().fill(1)`.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.storage[self.offset..self.offset + self.length]
    }
    /// Address of the first byte, the value the native pointer holds; `block.address()`.
    pub fn address(&self) -> usize {
        self.as_slice().as_ptr() as usize
    }
    /// Returns the requested size; `allocate_aligned(24, 16).unwrap().len() == 24`.
    pub fn len(&self) -> usize {
        self.length
    }
    /// Reports a zero-byte block; `allocate_aligned(0, 16).unwrap().is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

/// Allocates `size` bytes on an `alignment` boundary; `allocate_aligned(64, 32)`.
///
/// Returns `None` where the native call returns null: an alignment the C runtime rejects
/// (see [`effective_alignment`]) or a request it cannot satisfy. It never raises.
pub fn allocate_aligned(size: usize, alignment: usize) -> Option<AlignedMemory> {
    let alignment = effective_alignment(alignment)?;
    let storage = zeroed_bytes(size.checked_add(alignment - 1)?)?;
    let address = storage.as_ptr() as usize;
    let offset = (alignment - address % alignment) % alignment;
    Some(AlignedMemory {
        storage,
        offset,
        length: size,
    })
}

/// Releases an aligned block; `None` is null, which the runtime ignores; `free_aligned(Some(b))`.
pub fn free_aligned(memory: Option<AlignedMemory>) {
    drop(memory);
}

/// The template `FreeAligned(T*&)`: releases and nulls the slot; `free_aligned_and_null(&mut s)`.
pub fn free_aligned_and_null(slot: &mut Option<AlignedMemory>) {
    free_aligned(slot.take());
}

/// GCC's `_mm_malloc` on x86: alignment 1 is plain `malloc`, 2 and 4 (on 64-bit) widen to
/// a pointer, and everything else goes to `posix_memalign`; `effective_alignment(2) == Some(8)`.
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(target_env = "msvc")
))]
fn effective_alignment(alignment: usize) -> Option<usize> {
    const POINTER: usize = size_of::<usize>();
    match alignment {
        1 => Some(1),
        2 => Some(POINTER),
        4 if POINTER == 8 => Some(POINTER),
        other => posix_alignment(other),
    }
}

/// `_aligned_malloc` accepts any power of two; `effective_alignment(3) == None`.
#[cfg(target_env = "msvc")]
fn effective_alignment(alignment: usize) -> Option<usize> {
    alignment.is_power_of_two().then_some(alignment)
}

/// Other platforms call `posix_memalign` directly; `effective_alignment(4) == None`.
#[cfg(not(any(
    target_env = "msvc",
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        not(target_env = "msvc")
    )
)))]
fn effective_alignment(alignment: usize) -> Option<usize> {
    posix_alignment(alignment)
}

/// `posix_memalign` rejects (EINVAL) anything but a power-of-two multiple of a pointer.
#[cfg(not(target_env = "msvc"))]
fn posix_alignment(alignment: usize) -> Option<usize> {
    (alignment.is_power_of_two() && alignment.is_multiple_of(size_of::<usize>()))
        .then_some(alignment)
}
