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

//! The narrow FFI boundary of `OSD_MemInfo::Update`: glibc `mallinfo2`, the procfs
//! status file, and the macOS `task_info`/`malloc_zone_statistics` pair.
#![allow(unsafe_code)]
use super::MemInfoSource;

/// The running process's memory figures; `NativeMemInfo.heap_usage()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeMemInfo;

impl MemInfoSource for NativeMemInfo {
    fn heap_usage(&self) -> usize {
        heap_usage()
    }
    fn proc_status(&self) -> Option<String> {
        // Native: std::ifstream on "/proc/<getpid()>/status"; unopenable reads as None.
        let path: String = format!("/proc/{}/status", std::process::id());
        std::fs::read(path)
            .ok()
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }
    fn task_counters(&self) -> Option<(usize, usize, usize)> {
        task_counters()
    }
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn heap_usage() -> usize {
    // SAFETY: mallinfo2 only reads allocator statistics and returns them by value.
    unsafe { libc::mallinfo2() }.uordblks
}
#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
fn heap_usage() -> usize {
    0 // Native: without HAS_MALLINFO the heap counter is 0.
}

#[cfg(target_vendor = "apple")]
fn task_counters() -> Option<(usize, usize, usize)> {
    use mach2::{
        kern_return::KERN_SUCCESS,
        task::task_info,
        task_info::{TASK_BASIC_INFO, TASK_BASIC_INFO_COUNT, task_basic_info},
        traps::mach_task_self,
    };
    let mut info: task_basic_info = task_basic_info::default();
    let mut count: u32 = TASK_BASIC_INFO_COUNT;
    // SAFETY: the flavor/count match the initialized output layout; the task port belongs
    // to this process.
    let status: i32 = unsafe {
        task_info(
            mach_task_self(),
            TASK_BASIC_INFO,
            (&mut info as *mut task_basic_info).cast(),
            &mut count,
        )
    };
    if status != KERN_SUCCESS {
        return None;
    }
    // SAFETY: a zeroed statistics record is valid; a null zone means every zone.
    let mut stats: libc::malloc_statistics_t = unsafe { std::mem::zeroed() };
    // SAFETY: `stats` is live and writable for the call.
    unsafe { libc::malloc_zone_statistics(std::ptr::null_mut(), &mut stats) };
    let (virtual_size, resident) = (info.virtual_size, info.resident_size);
    Some((virtual_size, resident, stats.size_in_use))
}
#[cfg(not(target_vendor = "apple"))]
fn task_counters() -> Option<(usize, usize, usize)> {
    None
}
