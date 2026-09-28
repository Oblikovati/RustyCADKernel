// Created on: 2002-04-12
// Created by: Alexander KARTOMIN (akm)
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

// Purpose:     Basic class for memory allocation wizards.
//              Defines  the  interface  for devising  different  allocators
//              firstly to be used  by collections of NCollection, though it
//              it is not  deferred. It allocates/frees  the memory  through
//              Standard procedures, thus it is  unnecessary (and  sometimes
//              injurious) to have  more than one such  allocator.  To avoid
//              creation  of multiple  objects the  constructors  were  maid
//              inaccessible.  To  create the  BaseAllocator use  the method
//              CommonBaseAllocator.
//              Note that this object is managed by Handle.

//! Incremental blocks; UnsafeCell preserves live allocation provenance during growth.
use crate::standard::Failure;
use allocator_api2::alloc::{AllocError, Layout};
use std::{cell::UnsafeCell, ptr::NonNull};
pub(super) struct Arena {
    chunks: Vec<Chunk>,
    block_size: usize,
    live: usize,
}
struct Chunk {
    bytes: Box<[UnsafeCell<u8>]>,
    used: usize,
}
impl Arena {
    pub(super) fn new(block_size: usize) -> Self {
        Self {
            chunks: Vec::new(),
            block_size: if block_size < 49 {
                12 * 1024
            } else {
                block_size
            },
            live: 0,
        }
    }
    pub(super) fn allocate(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        for chunk in &mut self.chunks {
            if let Some(pointer) = chunk.allocate(layout) {
                self.live += 1;
                return Ok(pointer);
            }
        }
        let size = layout
            .size()
            .max(1)
            .checked_add(layout.align() - 1)
            .ok_or(AllocError)?
            .max(self.block_size);
        let mut chunk = Chunk::new(size)?;
        let pointer = chunk.allocate(layout).ok_or(AllocError)?;
        self.chunks.try_reserve(1).map_err(|_| AllocError)?;
        self.chunks.push(chunk);
        self.live += 1;
        Ok(pointer)
    }
    pub(super) fn deallocate(&mut self) {
        self.live = self
            .live
            .checked_sub(1)
            .expect("deallocation requires a live arena block");
    }
    pub(super) fn reset(&mut self, release: bool) -> Result<(), Failure> {
        if self.live != 0 {
            return Err(Failure::new(&format!(
                "{} live allocations: expected 0 before arena reset",
                self.live
            )));
        }
        if release {
            self.chunks.clear();
            return Ok(());
        }
        for chunk in &mut self.chunks {
            chunk.used = 0;
        }
        Ok(())
    }
}
impl Chunk {
    fn new(size: usize) -> Result<Self, AllocError> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(size).map_err(|_| AllocError)?;
        bytes.resize_with(size, || UnsafeCell::new(0));
        Ok(Self {
            bytes: bytes.into_boxed_slice(),
            used: 0,
        })
    }
    fn allocate(&mut self, layout: Layout) -> Option<NonNull<[u8]>> {
        // Borrowing the cells as shared does not borrow their payloads; previously
        // issued mutable allocations remain valid while this chunk is extended.
        let base = self.bytes.as_ptr().cast::<u8>().cast_mut();
        let candidate = base.wrapping_add(self.used);
        let padding = candidate.align_offset(layout.align());
        let start = self.used.checked_add(padding)?;
        let end = start.checked_add(layout.size().max(1))?;
        if end > self.bytes.len() {
            return None;
        }
        self.used = end;
        let pointer = NonNull::new(base.wrapping_add(start))?;
        Some(NonNull::slice_from_raw_parts(pointer, layout.size()))
    }
}
