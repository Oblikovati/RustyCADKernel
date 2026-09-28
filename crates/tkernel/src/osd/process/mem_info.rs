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

//! `OSD_MemInfo`: memory counters of the running process.
//!
//! Linux reads `mallinfo2` and `/proc/<pid>/status`; macOS reads `task_info` and the
//! default malloc zone. The Windows counters (Psapi, `_heapwalk`) are not ported: there
//! `update` leaves every counter unknown. A counter nobody filled stays `usize::MAX`
//! (native `size_t(-1)`), e.g. both swap counters on Linux.
use crate::tcollection::AsciiString;

mod native;
mod status;
pub use native::NativeMemInfo;

/// `OSD_MemInfo::Counter`, in native order; `MemCounter::ALL.len()` is `MemCounter_NB`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum MemCounter {
    /// `MemPrivate`: data and stack memory excluding libraries.
    Private,
    /// `MemVirtual`: reserved and committed virtual address space.
    Virtual,
    /// `MemWorkingSet`: pages resident in physical memory.
    WorkingSet,
    /// `MemWorkingSetPeak`: peak working set.
    WorkingSetPeak,
    /// `MemSwapUsage`: space allocated in the pagefile.
    SwapUsage,
    /// `MemSwapUsagePeak`: peak pagefile space.
    SwapUsagePeak,
    /// `MemHeapUsage`: bytes allocated from the heap.
    HeapUsage,
}
impl MemCounter {
    /// Every counter in native order (`MemCounter_NB` entries).
    pub const ALL: [Self; 7] = [
        Self::Private,
        Self::Virtual,
        Self::WorkingSet,
        Self::WorkingSetPeak,
        Self::SwapUsage,
        Self::SwapUsagePeak,
        Self::HeapUsage,
    ];
}

/// The raw process figures `OSD_MemInfo::Update` reads; implement for deterministic tests.
pub trait MemInfoSource {
    /// Linux: `mallinfo2().uordblks` (0 without glibc); `source.heap_usage()`.
    fn heap_usage(&self) -> usize;
    /// Linux: the text of `/proc/<pid>/status`, `None` when it cannot be opened;
    /// `source.proc_status()`.
    fn proc_status(&self) -> Option<String>;
    /// macOS: (virtual size, resident size, malloc bytes in use), `None` when `task_info`
    /// fails; `source.task_counters()`.
    fn task_counters(&self) -> Option<(usize, usize, usize)>;
}

const UNKNOWN: usize = usize::MAX;

/// `OSD_MemInfo`; `MemInfo::new(true).to_text()`.
pub struct MemInfo<S: MemInfoSource = NativeMemInfo> {
    source: S,
    counters: [usize; 7],
    active: [bool; 7],
}
impl MemInfo<NativeMemInfo> {
    /// `OSD_MemInfo(theImmediateUpdate)`: every counter active, then updated or cleared;
    /// `MemInfo::new(true)`.
    pub fn new(immediate_update: bool) -> Self {
        Self::with_source(NativeMemInfo, immediate_update)
    }
    /// `OSD_MemInfo::PrintInfo`: the text of a freshly updated object; `MemInfo::print_info()`.
    pub fn print_info() -> AsciiString {
        Self::new(true).to_text()
    }
}
impl<S: MemInfoSource> MemInfo<S> {
    /// The constructor over an injected source; `MemInfo::with_source(fake, true)`.
    pub fn with_source(source: S, immediate_update: bool) -> Self {
        let mut info: Self = Self {
            source,
            counters: [UNKNOWN; 7],
            active: [true; 7],
        };
        if immediate_update {
            info.update();
        }
        info
    }
    /// `OSD_MemInfo::IsActive`; `info.is_active(MemCounter::HeapUsage)`.
    pub fn is_active(&self, counter: MemCounter) -> bool {
        self.active[counter as usize]
    }
    /// `OSD_MemInfo::SetActive(bool)`: activates or deactivates every counter;
    /// `info.set_active(false)`.
    pub fn set_active(&mut self, active: bool) {
        self.active = [active; 7];
    }
    /// `OSD_MemInfo::SetActive(Counter, bool)`; `info.set_counter_active(MemCounter::Virtual, false)`.
    pub fn set_counter_active(&mut self, counter: MemCounter, active: bool) {
        self.active[counter as usize] = active;
    }
    /// `OSD_MemInfo::Clear`: every counter becomes unknown; `info.clear()`.
    pub fn clear(&mut self) {
        self.counters = [UNKNOWN; 7];
    }
    /// `OSD_MemInfo::Update`: clears, then samples the active counters; `info.update()`.
    pub fn update(&mut self) {
        self.clear();
        if cfg!(target_os = "linux") {
            self.update_linux();
        } else if cfg!(target_vendor = "apple") {
            self.update_apple();
        }
    }
    /// `OSD_MemInfo::ToString`: one line per known active counter, in MiB; `info.to_text()`.
    pub fn to_text(&self) -> AsciiString {
        use MemCounter::*;
        let mut text: String = String::new();
        self.push_line(&mut text, "  Private memory:     ", Private, None);
        self.push_line(
            &mut text,
            "  Working Set:        ",
            WorkingSet,
            Some(WorkingSetPeak),
        );
        self.push_line(
            &mut text,
            "  Pagefile usage:     ",
            SwapUsage,
            Some(SwapUsagePeak),
        );
        self.push_line(&mut text, "  Virtual memory:     ", Virtual, None);
        self.push_line(&mut text, "  Heap memory:     ", HeapUsage, None);
        AsciiString::from(text.as_str())
    }
    /// `OSD_MemInfo::Value`: bytes, `usize::MAX` when inactive or unknown; `info.value(c)`.
    pub fn value(&self, counter: MemCounter) -> usize {
        if !self.is_active(counter) {
            return UNKNOWN;
        }
        self.counters[counter as usize]
    }
    /// `OSD_MemInfo::ValueMiB`: whole mebibytes (integer division), `usize::MAX` when
    /// inactive or unknown; `info.value_mib(c)`.
    pub fn value_mib(&self, counter: MemCounter) -> usize {
        match self.value(counter) {
            UNKNOWN => UNKNOWN,
            bytes => bytes / (1024 * 1024),
        }
    }
    /// `OSD_MemInfo::ValuePreciseMiB`: fractional mebibytes, -1.0 when inactive or
    /// unknown; `info.value_precise_mib(c)`.
    pub fn value_precise_mib(&self, counter: MemCounter) -> f64 {
        match self.value(counter) {
            UNKNOWN => -1.0,
            bytes => bytes as f64 / (1024.0 * 1024.0),
        }
    }
    /// The protected `hasValue`: active and known.
    fn has_value(&self, counter: MemCounter) -> bool {
        self.is_active(counter) && self.counters[counter as usize] != UNKNOWN
    }
    fn push_line(
        &self,
        text: &mut String,
        label: &str,
        main: MemCounter,
        peak: Option<MemCounter>,
    ) {
        if !self.has_value(main) {
            return;
        }
        text.push_str(&format!("{label}{} MiB", native_int(self.value_mib(main))));
        if let Some(peak) = peak.filter(|&peak| self.has_value(peak)) {
            text.push_str(&format!(
                " (peak: {} MiB)",
                native_int(self.value_mib(peak))
            ));
        }
        text.push('\n');
    }
    fn update_linux(&mut self) {
        use MemCounter::*;
        if self.is_active(HeapUsage) {
            self.counters[HeapUsage as usize] = self.source.heap_usage();
        }
        let wanted = [Virtual, WorkingSet, WorkingSetPeak, Private];
        if !wanted.iter().any(|&counter| self.is_active(counter)) {
            return;
        }
        if let Some(text) = self.source.proc_status() {
            status::apply(&text, &self.active, &mut self.counters);
        }
    }
    fn update_apple(&mut self) {
        use MemCounter::*;
        if !(self.is_active(Virtual) || self.is_active(WorkingSet) || self.is_active(HeapUsage)) {
            return;
        }
        // Native quirk: all three are stored whenever any of them is active.
        if let Some((virtual_size, resident, heap)) = self.source.task_counters() {
            self.counters[Virtual as usize] = virtual_size;
            self.counters[WorkingSet as usize] = resident;
            self.counters[HeapUsage as usize] = heap;
        }
    }
}

/// The native `int(size_t)` cast of a MiB count (two's-complement truncation).
fn native_int(value: usize) -> i32 {
    value as i32
}
