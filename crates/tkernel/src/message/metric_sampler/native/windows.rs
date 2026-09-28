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

//! Windows metrics: performance counter, process/thread times, PSAPI and the CRT heap.
#![allow(unsafe_code)]
use super::{Counters, is_active};
use crate::osd::MemInfoCounter;
use core::ffi::c_int;
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentThread, GetProcessTimes, GetThreadTimes,
};

#[repr(C)]
#[derive(Default)]
struct MemoryStatusEx {
    length: u32,
    memory_load: u32,
    total_phys: u64,
    avail_phys: u64,
    total_page_file: u64,
    avail_page_file: u64,
    total_virtual: u64,
    avail_virtual: u64,
    avail_extended_virtual: u64,
}
#[repr(C)]
#[derive(Default)]
struct ProcessMemoryCountersEx {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
    private_usage: usize,
}
#[repr(C)]
struct HeapInfo {
    entry: *mut c_int,
    size: usize,
    use_flag: c_int,
}
const HEAP_OK: c_int = -2;
const USED_ENTRY: c_int = 1;
#[link(name = "kernel32")]
unsafe extern "system" {
    fn QueryPerformanceCounter(count: *mut i64) -> i32;
    fn QueryPerformanceFrequency(frequency: *mut i64) -> i32;
    fn GetTickCount64() -> u64;
    fn GlobalMemoryStatusEx(status: *mut MemoryStatusEx) -> i32;
    fn K32GetProcessMemoryInfo(
        process: *mut core::ffi::c_void,
        counters: *mut ProcessMemoryCountersEx,
        size: u32,
    ) -> i32;
}
unsafe extern "C" {
    fn _heapwalk(info: *mut HeapInfo) -> c_int;
}

/// `OSD_Timer::GetWallClockTime` on Windows: the performance counter, else the tick count.
pub(in crate::message::metric_sampler) fn wall_clock_time() -> f64 {
    let (mut frequency, mut count) = (0_i64, 0_i64);
    // SAFETY: both outputs are valid writable integers.
    unsafe {
        if QueryPerformanceFrequency(&mut frequency) != 0
            && QueryPerformanceCounter(&mut count) != 0
        {
            return count as f64 / frequency as f64;
        }
        0.001 * GetTickCount64() as f64
    }
}

fn seconds(time: FILETIME) -> f64 {
    0.0000001 * ((u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime)) as f64
}

const ZERO_TIME: FILETIME = FILETIME {
    dwLowDateTime: 0,
    dwHighDateTime: 0,
};

/// `OSD_Chronometer::GetProcessCPU` on Windows: `GetProcessTimes` user and kernel time.
pub(in crate::message::metric_sampler) fn process_cpu() -> (f64, f64) {
    let (mut start, mut exit, mut kernel, mut user) = (ZERO_TIME, ZERO_TIME, ZERO_TIME, ZERO_TIME);
    // SAFETY: the pseudo-handle needs no closing; the four outputs are distinct FILETIMEs.
    unsafe {
        GetProcessTimes(
            GetCurrentProcess(),
            &mut start,
            &mut exit,
            &mut kernel,
            &mut user,
        )
    };
    (seconds(user), seconds(kernel))
}

/// `OSD_Chronometer::GetThreadCPU` on Windows: `GetThreadTimes` user and kernel time.
pub(in crate::message::metric_sampler) fn thread_cpu() -> (f64, f64) {
    let (mut start, mut exit, mut kernel, mut user) = (ZERO_TIME, ZERO_TIME, ZERO_TIME, ZERO_TIME);
    // SAFETY: the pseudo-handle needs no closing; the four outputs are distinct FILETIMEs.
    unsafe {
        GetThreadTimes(
            GetCurrentThread(),
            &mut start,
            &mut exit,
            &mut kernel,
            &mut user,
        )
    };
    (seconds(user), seconds(kernel))
}

/// `OSD_MemInfo::Update` on Windows.
pub(in crate::message::metric_sampler) fn memory_bytes(counters: &[MemInfoCounter]) -> Counters {
    let mut values: Counters = [None; 7];
    if is_active(counters, MemInfoCounter::Virtual) {
        let mut status = MemoryStatusEx {
            length: size_of::<MemoryStatusEx>() as u32,
            ..MemoryStatusEx::default()
        };
        // SAFETY: `status` is writable and its length field is set as the API requires.
        unsafe { GlobalMemoryStatusEx(&mut status) };
        values[MemInfoCounter::Virtual as usize] =
            Some(status.total_virtual.wrapping_sub(status.avail_virtual));
    }
    process_counters(counters, &mut values);
    if is_active(counters, MemInfoCounter::HeapUsage) {
        values[MemInfoCounter::HeapUsage as usize] = Some(heap_usage());
    }
    values
}

fn process_counters(counters: &[MemInfoCounter], values: &mut Counters) {
    use MemInfoCounter::{Private, SwapUsage, SwapUsagePeak, WorkingSet, WorkingSetPeak};
    if ![
        Private,
        WorkingSet,
        WorkingSetPeak,
        SwapUsage,
        SwapUsagePeak,
    ]
    .iter()
    .any(|counter| is_active(counters, *counter))
    {
        return;
    }
    let mut memory = ProcessMemoryCountersEx::default();
    let size = size_of::<ProcessMemoryCountersEx>() as u32;
    // SAFETY: the pseudo-handle needs no closing; `memory` spans `size` writable bytes.
    if unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut memory, size) } == 0 {
        return;
    }
    values[Private as usize] = Some(memory.private_usage as u64);
    values[WorkingSet as usize] = Some(memory.working_set_size as u64);
    values[WorkingSetPeak as usize] = Some(memory.peak_working_set_size as u64);
    values[SwapUsage as usize] = Some(memory.pagefile_usage as u64);
    values[SwapUsagePeak as usize] = Some(memory.peak_pagefile_usage as u64);
}

/// Sum of the used CRT heap blocks (`_heapwalk`).
fn heap_usage() -> u64 {
    let mut info = HeapInfo {
        entry: std::ptr::null_mut(),
        size: 0,
        use_flag: 0,
    };
    let mut total = 0_u64;
    // SAFETY: `_heapwalk` starts from a null entry and advances the caller-owned cursor.
    while unsafe { _heapwalk(&mut info) } == HEAP_OK {
        if info.use_flag == USED_ENTRY {
            total += info.size as u64;
        }
    }
    total
}
