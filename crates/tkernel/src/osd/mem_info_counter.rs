// Created on: 2011-10-05
// Created by: Kirill GAVRILOV
// Copyright (c) 2013-2014 OPEN CASCADE SAS
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

//! `OSD_MemInfo::Counter`, the memory counters named by `Message_MetricType`.

/// One process memory counter of `OSD_MemInfo`; `MemInfoCounter::WorkingSet as i32 == 2`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum MemInfoCounter {
    /// Virtual memory allocated for data and stack excluding libraries.
    Private = 0,
    /// Reserved and committed memory of the virtual address space.
    Virtual,
    /// Memory pages currently resident in physical memory.
    WorkingSet,
    /// Peak working set size.
    WorkingSetPeak,
    /// Space allocated for the pagefile.
    SwapUsage,
    /// Peak space allocated for the pagefile.
    SwapUsagePeak,
    /// Total space allocated from the heap.
    HeapUsage,
}
impl crate::ncollection::HashKey for MemInfoCounter {
    fn occt_hash(&self) -> usize {
        *self as usize
    }
}
