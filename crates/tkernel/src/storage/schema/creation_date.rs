// Copyright (c) 1996-1999 Matra Datavision
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

//! The clock behind `Storage_Schema::ICreationDate` (`strftime("%m/%d/%Y")` of local time).
//! Narrow platform FFI boundary for the local calendar date.
#![allow(unsafe_code)]
use crate::tcollection::AsciiString;

/// Supplies the local calendar date a schema stamps on written data.
pub trait CreationDateSource: Send + Sync {
    /// Local `(month, day, year)`; `clock.local_date()`.
    fn local_date(&self) -> (u32, u32, i32);

    /// The native `%m/%d/%Y` text; `clock.creation_date()`.
    fn creation_date(&self) -> AsciiString {
        let (month, day, year) = self.local_date();
        AsciiString::new(&format!("{month:02}/{day:02}/{year}"))
    }
}

/// The host's local time zone, as `localtime` reads it; `Arc::new(NativeCreationDate)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeCreationDate;

impl CreationDateSource for NativeCreationDate {
    fn local_date(&self) -> (u32, u32, i32) {
        native_local_date()
    }
}

#[cfg(unix)]
fn native_local_date() -> (u32, u32, i32) {
    // SAFETY: `time` accepts a null pointer; `parts` is a zeroed, writable `tm` that
    // `localtime_r` fills and that outlives the call.
    let parts = unsafe {
        let now = libc::time(std::ptr::null_mut());
        let mut parts: libc::tm = std::mem::zeroed();
        libc::localtime_r(&now, &mut parts);
        parts
    };
    (
        (parts.tm_mon + 1) as u32,
        parts.tm_mday as u32,
        parts.tm_year + 1900,
    )
}

#[cfg(windows)]
fn native_local_date() -> (u32, u32, i32) {
    use windows_sys::Win32::{Foundation::SYSTEMTIME, System::SystemInformation::GetLocalTime};
    // SAFETY: an all-zero SYSTEMTIME is valid; `GetLocalTime` only writes into it.
    let now = unsafe {
        let mut now: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut now);
        now
    };
    (
        u32::from(now.wMonth),
        u32::from(now.wDay),
        i32::from(now.wYear),
    )
}
