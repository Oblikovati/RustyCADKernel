// Created on: 1999-07-29
// Created by: Roman LYGIN
// Copyright (c) 1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! The Message enumerations: gravity, console colors and alert metrics.

/// Ordered OCCT severity levels (`Message_Gravity`); `Gravity::Warning > Gravity::Info`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum Gravity {
    /// Low-level algorithm detail.
    Trace = 0,
    /// Informational output.
    Info = 1,
    /// Warning condition.
    Warning = 2,
    /// Nonfatal error.
    Alarm = 3,
    /// Fatal error.
    Fail = 4,
}
impl Gravity {
    /// Every gravity in native enumerator order, the `Message_Trace..=Message_Fail` loops.
    pub const ALL: [Gravity; 5] = [
        Gravity::Trace,
        Gravity::Info,
        Gravity::Warning,
        Gravity::Alarm,
        Gravity::Fail,
    ];
    /// The enumerator's native integer; `Gravity::Fail.index() == 4`.
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Console text colors of `Message_ConsoleColor`; `ConsoleColor::Red`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ConsoleColor {
    /// Default (white) color.
    Default = 0,
    /// Black.
    Black,
    /// White.
    White,
    /// Red.
    Red,
    /// Blue.
    Blue,
    /// Green.
    Green,
    /// Yellow.
    Yellow,
    /// Cyan.
    Cyan,
    /// Magenta.
    Magenta,
}

/// Alert metrics of `Message_MetricType`; `MetricType::WallClock as i32 == 5`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum MetricType {
    /// No computation.
    None = 0,
    /// `OSD_Chronometer::GetThreadCPU` user time.
    ThreadCpuUserTime,
    /// `OSD_Chronometer::GetThreadCPU` system time.
    ThreadCpuSystemTime,
    /// `OSD_Chronometer::GetProcessCPU` user time.
    ProcessCpuUserTime,
    /// `OSD_Chronometer::GetProcessCPU` system time.
    ProcessCpuSystemTime,
    /// `OSD_Timer` elapsed time.
    WallClock,
    /// `OSD_MemInfo::MemPrivate`.
    MemPrivate,
    /// `OSD_MemInfo::MemVirtual`.
    MemVirtual,
    /// `OSD_MemInfo::MemWorkingSet`.
    MemWorkingSet,
    /// `OSD_MemInfo::MemWorkingSetPeak`.
    MemWorkingSetPeak,
    /// `OSD_MemInfo::MemSwapUsage`.
    MemSwapUsage,
    /// `OSD_MemInfo::MemSwapUsagePeak`.
    MemSwapUsagePeak,
    /// `OSD_MemInfo::MemHeapUsage`.
    MemHeapUsage,
}
impl MetricType {
    /// Every metric in enumerator order, the native `Message_Table_PrintMetricTypeEnum` order.
    pub const ALL: [MetricType; 13] = [
        MetricType::None,
        MetricType::ThreadCpuUserTime,
        MetricType::ThreadCpuSystemTime,
        MetricType::ProcessCpuUserTime,
        MetricType::ProcessCpuSystemTime,
        MetricType::WallClock,
        MetricType::MemPrivate,
        MetricType::MemVirtual,
        MetricType::MemWorkingSet,
        MetricType::MemWorkingSetPeak,
        MetricType::MemSwapUsage,
        MetricType::MemSwapUsagePeak,
        MetricType::MemHeapUsage,
    ];
}
impl crate::ncollection::HashKey for MetricType {
    fn occt_hash(&self) -> usize {
        *self as usize
    }
}
