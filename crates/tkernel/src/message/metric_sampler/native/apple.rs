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

//! Apple metrics: `times`, Mach task information and malloc zone statistics.
#![allow(unsafe_code)]
use super::{Counters, is_active};
use crate::osd::MemInfoCounter;
use core::ffi::{c_int, c_long, c_ulong, c_void};
use mach2::task::task_info;
use mach2::task_info::{
    TASK_BASIC_INFO, TASK_BASIC_INFO_COUNT, TASK_THREAD_TIMES_INFO, TASK_THREAD_TIMES_INFO_COUNT,
    task_basic_info, task_thread_times_info,
};
use mach2::traps::mach_task_self;

#[repr(C)]
struct Tms {
    user: c_ulong,
    system: c_ulong,
    children_user: c_ulong,
    children_system: c_ulong,
}
#[repr(C)]
#[derive(Default)]
struct MallocStatistics {
    blocks_in_use: u32,
    size_in_use: usize,
    max_size_in_use: usize,
    size_allocated: usize,
}
const SC_CLK_TCK: c_int = 3;
unsafe extern "C" {
    fn times(buffer: *mut Tms) -> c_ulong;
    fn sysconf(name: c_int) -> c_long;
    fn malloc_zone_statistics(zone: *mut c_void, statistics: *mut MallocStatistics);
}

/// `OSD_Chronometer::GetProcessCPU`: `times()` ticks over the clock tick rate.
pub(in crate::message::metric_sampler) fn process_cpu() -> (f64, f64) {
    let mut current = Tms {
        user: 0,
        system: 0,
        children_user: 0,
        children_system: 0,
    };
    // SAFETY: `current` matches `struct tms` and is writable; sysconf has no preconditions.
    let ticks = unsafe {
        times(&mut current);
        sysconf(SC_CLK_TCK)
    };
    (
        current.user as f64 / ticks as f64,
        current.system as f64 / ticks as f64,
    )
}

/// `OSD_Chronometer::GetThreadCPU`: task thread times, user and system.
pub(in crate::message::metric_sampler) fn thread_cpu() -> (f64, f64) {
    let mut info = task_thread_times_info::default();
    let mut count = TASK_THREAD_TIMES_INFO_COUNT;
    // SAFETY: the flavor and count match the initialized output structure.
    let status = unsafe {
        task_info(
            mach_task_self(),
            TASK_THREAD_TIMES_INFO,
            (&mut info as *mut task_thread_times_info).cast(),
            &mut count,
        )
    };
    if status != 0 {
        return (0.0, 0.0);
    }
    let (user, system) = (info.user_time, info.system_time);
    (
        f64::from(user.seconds) + 0.000001 * f64::from(user.microseconds),
        f64::from(system.seconds) + 0.000001 * f64::from(system.microseconds),
    )
}

/// `OSD_MemInfo::Update` on macOS: task virtual/resident sizes and the malloc heap.
pub(in crate::message::metric_sampler) fn memory_bytes(counters: &[MemInfoCounter]) -> Counters {
    let mut values: Counters = [None; 7];
    let wanted = [
        MemInfoCounter::Virtual,
        MemInfoCounter::WorkingSet,
        MemInfoCounter::HeapUsage,
    ];
    if !wanted.iter().any(|counter| is_active(counters, *counter)) {
        return values;
    }
    let mut info = task_basic_info::default();
    let mut count = TASK_BASIC_INFO_COUNT;
    // SAFETY: the flavor and count match the initialized output structure.
    let status = unsafe {
        task_info(
            mach_task_self(),
            TASK_BASIC_INFO,
            (&mut info as *mut task_basic_info).cast(),
            &mut count,
        )
    };
    if status != 0 {
        return values;
    }
    let mut statistics = MallocStatistics::default();
    // SAFETY: a null zone asks for all zones; `statistics` is a writable malloc_statistics_t.
    unsafe { malloc_zone_statistics(std::ptr::null_mut(), &mut statistics) };
    values[MemInfoCounter::Virtual as usize] = Some(info.virtual_size as u64);
    values[MemInfoCounter::WorkingSet as usize] = Some(info.resident_size as u64);
    values[MemInfoCounter::HeapUsage as usize] = Some(statistics.size_in_use as u64);
    values
}
