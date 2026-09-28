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

//! Linux (and other non-Apple Unix) metrics: `times`, the thread CPU clock and procfs.
#![allow(unsafe_code)]
use super::{Counters, is_active};
use crate::osd::{CpuClock, MemInfoCounter, NativeCpuClock};

/// `OSD_Chronometer::GetProcessCPU`: `times()` ticks over `sysconf(_SC_CLK_TCK)`.
pub(in crate::message::metric_sampler) fn process_cpu() -> (f64, f64) {
    let mut current = libc::tms {
        tms_utime: 0,
        tms_stime: 0,
        tms_cutime: 0,
        tms_cstime: 0,
    };
    // SAFETY: `current` is a valid, writable tms; sysconf has no preconditions.
    let ticks = unsafe {
        libc::times(&mut current);
        libc::sysconf(libc::_SC_CLK_TCK)
    };
    (
        current.tms_utime as f64 / ticks as f64,
        current.tms_stime as f64 / ticks as f64,
    )
}

/// `OSD_Chronometer::GetThreadCPU`: only user time is available for threads on Linux.
pub(in crate::message::metric_sampler) fn thread_cpu() -> (f64, f64) {
    (NativeCpuClock.now(), 0.0)
}

/// `OSD_MemInfo::Update` on Linux: heap from `mallinfo2`, the rest from `/proc/<pid>/status`.
pub(in crate::message::metric_sampler) fn memory_bytes(counters: &[MemInfoCounter]) -> Counters {
    let mut values: Counters = [None; 7];
    if is_active(counters, MemInfoCounter::HeapUsage) {
        values[MemInfoCounter::HeapUsage as usize] = Some(heap_usage());
    }
    let wanted = [
        MemInfoCounter::Virtual,
        MemInfoCounter::WorkingSet,
        MemInfoCounter::WorkingSetPeak,
        MemInfoCounter::Private,
    ];
    if !wanted.iter().any(|counter| is_active(counters, *counter)) {
        return values;
    }
    let path = format!("/proc/{}/status", std::process::id());
    let Ok(status) = std::fs::read(path) else {
        return values;
    };
    for line in status.split(|&byte| byte == b'\n') {
        status_line(line, counters, &mut values);
    }
    values
}

#[cfg(target_env = "gnu")]
fn heap_usage() -> u64 {
    // SAFETY: mallinfo2 only reads allocator statistics.
    unsafe { libc::mallinfo2() }.uordblks as u64
}
#[cfg(not(target_env = "gnu"))]
fn heap_usage() -> u64 {
    0
}

/// `atol` of the text after a `Vm*:` label, in KiB, scaled to bytes.
fn kibibytes(text: &[u8]) -> u64 {
    let text = &text[text
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(text.len())..];
    let digits = text.iter().take_while(|byte| byte.is_ascii_digit()).count();
    let value: u64 = std::str::from_utf8(&text[..digits])
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    value * 1024
}

/// One status line: the first matching active label of the native `else if` chain.
fn status_line(line: &[u8], counters: &[MemInfoCounter], values: &mut Counters) {
    let labels: [(&[u8], MemInfoCounter); 5] = [
        (b"VmSize:", MemInfoCounter::Virtual),
        (b"VmRSS:", MemInfoCounter::WorkingSet),
        (b"VmHWM:", MemInfoCounter::WorkingSetPeak),
        (b"VmData:", MemInfoCounter::Private),
        (b"VmStk:", MemInfoCounter::Private),
    ];
    let Some((label, counter)) = labels
        .into_iter()
        .find(|(label, counter)| is_active(counters, *counter) && line.starts_with(label))
    else {
        return;
    };
    let value = kibibytes(&line[label.len()..]);
    let slot = &mut values[counter as usize];
    *slot = Some(match (counter, *slot) {
        // MemPrivate sums VmData and VmStk, starting from 0 instead of size_t(-1).
        (MemInfoCounter::Private, Some(previous)) => previous + value,
        _ => value,
    });
}
