// Copyright (c) 1998-1999 Matra Datavision
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

//! Timing, signals and package statics: `OSD_Timer`, `OSD_Chronometer` and `OSD`.
mod c_format;
mod chronometer;
mod clocks;
mod cpu_native;
mod osd_statics;
mod real_text;
mod signal_native;
mod signals;
mod sleep;
mod timer;
mod wall_native;

pub use chronometer::Chronometer;
pub use clocks::{CpuTimes, NativeCpuTimes, NativeWallClock, WallClock};
pub use osd_statics::Osd;
pub use real_text::{c_string_to_real, real_to_c_string};
pub use signals::{NativeSignalHost, SignalHost, SignalState};
pub use sleep::{NativeSleeper, Sleeper, milli_sec_sleep_duration, sec_sleep_duration};
pub use timer::Timer;
