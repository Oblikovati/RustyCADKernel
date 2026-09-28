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
use crate::standard::{
    Failure,
    dump::{DumpOutput, add_values_separator},
};
use std::io;

/// Owned buffer with explicit allocation state; `Buffer::new(Some(allocator), 16)`.
///
/// `None` stands for the native null allocator handle.
pub struct Buffer {
    allocator: Option<BaseAllocator>,
    storage: Option<AllocatedBytes>,
}

impl Buffer {
    /// Creates and allocates a buffer (`NCollection_Buffer(theAlloc, theSize)`).
    ///
    /// Construction starts with `Allocate`, whose `Free(nullptr)` raises Standard_ProgramError
    /// for an accumulating allocator, so upstream cannot build such a buffer either.
    /// Example: `Buffer::new(Some(BaseAllocator::common()), 0)?.is_empty() == false`.
    pub fn new(allocator: Option<BaseAllocator>, size: usize) -> Result<Self, Failure> {
        let mut buffer = Self {
            allocator,
            storage: None,
        };
        buffer.allocate(size)?;
        Ok(buffer)
    }

    /// Adopts data allocated by `allocator` (`NCollection_Buffer(theAlloc, theSize, theData)`).
    ///
    /// The owned allocation carries its own length, so `theSize` is `data.as_slice().len()`;
    /// it is freed by the allocator that made it even when `allocator` is `None`, where
    /// upstream would leak it. Example: `Buffer::with_data(Some(a.clone()), a.allocate(4).unwrap())`.
    pub fn with_data(allocator: Option<BaseAllocator>, data: AllocatedBytes) -> Self {
        Self {
            allocator,
            storage: Some(data),
        }
    }

    /// Replaces storage (`Allocate`); `Ok(false)` when nothing was allocated, `Err` when the
    /// preceding `Free` raises.
    ///
    /// Upstream skips allocation only for a zero size without allocator and dereferences the
    /// null allocator otherwise; Rust reports `false` for any size without allocator.
    /// Example: `buffer.allocate(32)?`.
    pub fn allocate(&mut self, size: usize) -> Result<bool, Failure> {
        self.free()?;
        self.storage = self
            .allocator
            .as_ref()
            .and_then(|allocator| allocator.allocate(size));
        Ok(self.storage.is_some())
    }

    /// Releases storage while retaining allocator selection (`Free`); `buffer.free()?`.
    ///
    /// As upstream, the allocator frees the data even when there is none (`Free(nullptr)`),
    /// which the accumulating allocator refuses; on failure the buffer is unchanged.
    pub fn free(&mut self) -> Result<(), Failure> {
        if let Some(allocator) = &self.allocator {
            allocator.check_free(self.data_ptr())?;
        }
        self.storage = None;
        Ok(())
    }

    /// Reports missing allocation, independently of length (`IsEmpty`); `buffer.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.storage.is_none()
    }

    /// Returns the allocated byte count (`Size`); `buffer.size() == 16`.
    pub fn size(&self) -> usize {
        self.as_slice().len()
    }

    /// Borrows buffer bytes (`Data`); empty when unallocated. Example: `buffer.as_slice().iter()`.
    pub fn as_slice(&self) -> &[u8] {
        self.storage.as_ref().map_or(&[], AllocatedBytes::as_slice)
    }

    /// Borrows mutable buffer bytes (`ChangeData`); `buffer.as_mut_slice().fill(42)`.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.storage
            .as_mut()
            .map_or(&mut [], AllocatedBytes::as_mut_slice)
    }

    /// The data address, null when unallocated (native `Data()` pointer); `buffer.data_ptr()`.
    pub fn data_ptr(&self) -> *const u8 {
        self.storage
            .as_ref()
            .map_or(std::ptr::null(), AllocatedBytes::as_ptr)
    }

    /// Returns the selected allocator identity (`Allocator`); `buffer.allocator().is_some()`.
    pub fn allocator(&self) -> Option<&BaseAllocator> {
        self.allocator.as_ref()
    }

    /// Frees storage before changing allocator (`SetAllocator`); `buffer.set_allocator(None)?`.
    /// When that `Free` raises, the allocator is kept.
    pub fn set_allocator(&mut self, allocator: Option<BaseAllocator>) -> Result<(), Failure> {
        self.free()?;
        self.allocator = allocator;
        Ok(())
    }

    /// Dumps data address, size and allocator address (`DumpJson`); `theDepth` is unused.
    ///
    /// Addresses use `Standard_Dump::GetPointerInfo`'s short form, which keeps the stream's
    /// `x` after prepending `0x` (`"0xx7f..."`). Example: `buffer.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput, _depth: i32) -> io::Result<()> {
        let allocator = self.allocator.as_ref().map_or(0, |a| a.identity() as usize);
        dump_pointer(output, "Data", self.data_ptr() as usize)?;
        add_values_separator(output)?;
        output.write_fragment(format!("\"Size\": {}", self.size()).as_bytes())?;
        dump_pointer(output, "Allocator", allocator)
    }
}

/// `OCCT_DUMP_FIELD_VALUE_POINTER`: `"Name": "<GetPointerInfo(address)>"`.
fn dump_pointer(output: &mut impl DumpOutput, name: &str, address: usize) -> io::Result<()> {
    add_values_separator(output)?;
    output.write_fragment(format!("\"{name}\": \"{}\"", pointer_info(address)).as_bytes())
}

/// `Standard_Dump::GetPointerInfo(ptr, isShortInfo = true)` with glibc's `0x…` pointer text:
/// from the first non-`'0'` symbol (the `x`), prefixed again by `0x`; empty for null.
///
/// Example: `pointer_info(0x10) == "0xx10"`.
fn pointer_info(address: usize) -> String {
    if address == 0 {
        return String::new();
    }
    let text = format!("{address:#x}");
    match text[..text.len() - 1].find(|symbol| symbol != '0') {
        Some(first) => format!("0x{}", &text[first..]),
        None => text,
    }
}
