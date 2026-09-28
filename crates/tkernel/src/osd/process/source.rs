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

//! The system queries `OSD_Process` makes, as an injectable interface.

/// A broken-down local time, as `localtime` (Unix) or `GetLocalTime` (Windows) gives it,
/// with the year already offset from 1900; `LocalTime::default()` is the native failure
/// value (all zero, year 1900).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalTime {
    /// Month, 1..=12.
    pub month: i32,
    /// Day of the month, 1..=31.
    pub day: i32,
    /// Full year.
    pub year: i32,
    /// Hour, 0..=23.
    pub hour: i32,
    /// Minute, 0..=59.
    pub minute: i32,
    /// Second, 0..=59.
    pub second: i32,
    /// Millisecond (Windows only; the Unix build passes 0).
    pub millisecond: i32,
}
impl Default for LocalTime {
    fn default() -> Self {
        // Native Unix SystemDate starts every field at 0 and adds 1900 to the year.
        Self {
            month: 0,
            day: 0,
            year: 1900,
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 0,
        }
    }
}

/// Process queries behind `OSD_Process`; implement for deterministic tests.
///
/// Failures carry the native error number (`errno` on Unix, `GetLastError` on Windows).
pub trait ProcessSource {
    /// The process id; `source.process_id()`.
    fn process_id(&self) -> i32;
    /// Tells whether the process runs with administrator rights; `source.is_super_user()`.
    fn is_super_user(&self) -> bool;
    /// The login name bytes, empty when no account entry exists; `source.user_name()`.
    fn user_name(&self) -> Result<Vec<u8>, i32>;
    /// The current local time; `source.local_time()`.
    fn local_time(&self) -> Result<LocalTime, i32>;
    /// The working directory bytes, without a trailing separator; `source.current_directory()`.
    fn current_directory(&self) -> Result<Vec<u8>, i32>;
    /// Changes the working directory; `source.set_current_directory(b"/tmp")`.
    fn set_current_directory(&self, path: &[u8]) -> Result<(), i32>;
    /// The running executable's absolute path, empty when unknown; `source.executable_path()`.
    fn executable_path(&self) -> Vec<u8>;
}
