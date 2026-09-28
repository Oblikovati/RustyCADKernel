// Copyright (c) 2002-2023 OPEN CASCADE SAS
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

//! `malloc`/`calloc`/`realloc`/`free` and `_mm_malloc` semantics over Rust's global allocator.
//!
//! OCCT frees by address alone (`Standard::Free(void*)`), while Rust's global allocator needs the
//! original layout back. Every block therefore carries a small header, placed just before the
//! returned address, recording the payload size and alignment.
use std::{
    alloc::{self, Layout},
    ptr::NonNull,
};

/// Header bytes kept before every payload; also glibc's x86-64 `malloc` alignment.
pub(super) const MALLOC_ALIGNMENT: usize = 16;

/// Recorded shape of one payload, stored in the `MALLOC_ALIGNMENT` bytes preceding it.
#[derive(Clone, Copy)]
struct Header {
    size: usize,
    align: usize,
}

/// Payload offset from the underlying allocation: one header, grown to the alignment.
fn payload_offset(align: usize) -> usize {
    align.max(MALLOC_ALIGNMENT)
}

/// Layout of the underlying allocation holding a header and `size` payload bytes.
fn block_layout(size: usize, align: usize) -> Option<Layout> {
    let total = payload_offset(align).checked_add(size)?;
    Layout::from_size_align(total, align.max(MALLOC_ALIGNMENT)).ok()
}

/// Allocates `size` bytes aligned to `align` (a power of two), zeroed when `zeroed`.
///
/// Returns `None` where the C allocator would return NULL (exhaustion or an impossible layout).
/// Example: `heap_allocate(24, 16, true)` behaves like `calloc(24, 1)`.
pub(super) fn heap_allocate(size: usize, align: usize, zeroed: bool) -> Option<NonNull<u8>> {
    let layout = block_layout(size, align)?;
    // SAFETY: the layout has a nonzero size (it always includes the header).
    let base = unsafe {
        if zeroed {
            alloc::alloc_zeroed(layout)
        } else {
            alloc::alloc(layout)
        }
    };
    let base = NonNull::new(base)?;
    // SAFETY: the header offset lies inside the allocation of `layout.size()` bytes.
    Some(unsafe { write_header(base, Header { size, align }) })
}

/// Writes the header in front of the payload and returns the payload address.
///
/// # Safety
/// `base` must start an allocation of `block_layout(header.size, header.align)`.
unsafe fn write_header(base: NonNull<u8>, header: Header) -> NonNull<u8> {
    let offset = payload_offset(header.align);
    // SAFETY: `offset <= layout.size()` and `offset >= MALLOC_ALIGNMENT >= size_of::<Header>()`,
    // so the header fits in the bytes preceding the payload; unaligned writes need no alignment.
    unsafe {
        let payload = base.add(offset);
        payload
            .sub(MALLOC_ALIGNMENT)
            .cast::<Header>()
            .write_unaligned(header);
        payload
    }
}

/// Reads the header of a live payload and the base address of its underlying allocation.
///
/// # Safety
/// `payload` must come from `heap_allocate`/`heap_reallocate` and not be freed.
unsafe fn read_header(payload: NonNull<u8>) -> (Header, NonNull<u8>) {
    // SAFETY: guaranteed by the caller; see `write_header` for the header placement.
    unsafe {
        let header = payload
            .sub(MALLOC_ALIGNMENT)
            .cast::<Header>()
            .read_unaligned();
        (header, payload.sub(payload_offset(header.align)))
    }
}

/// Releases a payload, like `free`.
///
/// # Safety
/// `payload` must come from `heap_allocate`/`heap_reallocate` and not be freed yet.
pub(super) unsafe fn heap_free(payload: NonNull<u8>) {
    // SAFETY: guaranteed by the caller.
    unsafe {
        let (header, base) = read_header(payload);
        let layout =
            block_layout(header.size, header.align).expect("layout recorded at allocation");
        alloc::dealloc(base.as_ptr(), layout);
    }
}

/// Resizes a payload like `realloc`, keeping its alignment; `None` leaves the block untouched.
///
/// # Safety
/// `payload` must come from `heap_allocate`/`heap_reallocate` and not be freed yet.
pub(super) unsafe fn heap_reallocate(payload: NonNull<u8>, size: usize) -> Option<NonNull<u8>> {
    // SAFETY: guaranteed by the caller.
    unsafe {
        let (header, base) = read_header(payload);
        let old = block_layout(header.size, header.align).expect("layout recorded at allocation");
        let new = block_layout(size, header.align)?;
        let moved = NonNull::new(alloc::realloc(base.as_ptr(), old, new.size()))?;
        Some(write_header(
            moved,
            Header {
                size,
                align: header.align,
            },
        ))
    }
}

/// The alignment `_mm_malloc` (GCC, x86/x86-64: the recorded oracle profile) really requests.
///
/// `1` falls back to `malloc`; `2` and `4` are raised to the pointer size; any other value
/// must be a power of two, else `posix_memalign` fails with EINVAL and the result is NULL.
/// Example: `mm_malloc_alignment(3) == None`, `mm_malloc_alignment(4) == Some(8)`.
pub(super) fn mm_malloc_alignment(alignment: usize) -> Option<usize> {
    match alignment {
        1 => Some(MALLOC_ALIGNMENT),
        2 | 4 => Some(size_of::<usize>()),
        value if value.is_power_of_two() => Some(value),
        _ => None,
    }
}
