// Created on: 2014-04-01
// Created by: Kirill Gavrilov
// Copyright (c) 2014 OPEN CASCADE SAS
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

//! Byte buffer whose storage retains the original allocator until released.
use super::{AllocatedBytes, BaseAllocator};
/// Owned buffer with explicit allocation state; `Buffer::new(Some(allocator), 16)`.
pub struct Buffer {
    allocator: Option<BaseAllocator>,
    storage: Option<AllocatedBytes>,
}
impl Buffer {
    /// Creates and allocates a buffer; `Buffer::new(Some(BaseAllocator::common()), 0)`.
    pub fn new(allocator: Option<BaseAllocator>, size: usize) -> Self {
        let mut buffer = Self {
            allocator,
            storage: None,
        };
        buffer.allocate(size);
        buffer
    }
    /// Replaces storage, returning false without an allocator; `buffer.allocate(32)`.
    pub fn allocate(&mut self, size: usize) -> bool {
        self.free();
        self.storage = self
            .allocator
            .as_ref()
            .map(|allocator| allocator.allocate(size));
        self.storage.is_some()
    }
    /// Releases storage while retaining allocator selection; `buffer.free()`.
    pub fn free(&mut self) {
        self.storage = None;
    }
    /// Reports missing allocation, independently of length; `buffer.free(); assert!(buffer.is_empty())`.
    pub fn is_empty(&self) -> bool {
        self.storage.is_none()
    }
    /// Returns the allocated byte count; `buffer.size() == 16`.
    pub fn size(&self) -> usize {
        self.as_slice().len()
    }
    /// Borrows buffer bytes; `buffer.as_slice().iter()`.
    pub fn as_slice(&self) -> &[u8] {
        self.storage.as_ref().map_or(&[], AllocatedBytes::as_slice)
    }
    /// Borrows mutable buffer bytes; `buffer.as_mut_slice().fill(42)`.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.storage
            .as_mut()
            .map_or(&mut [], AllocatedBytes::as_mut_slice)
    }
    /// Returns the selected allocator identity; `buffer.allocator().is_some()`.
    pub fn allocator(&self) -> Option<&BaseAllocator> {
        self.allocator.as_ref()
    }
    /// Frees storage before changing allocator; `buffer.set_allocator(None)`.
    pub fn set_allocator(&mut self, allocator: Option<BaseAllocator>) {
        self.free();
        self.allocator = allocator;
    }
}
