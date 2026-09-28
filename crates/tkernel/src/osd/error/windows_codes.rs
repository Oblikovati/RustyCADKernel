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

//! The Windows `OSD_Error` tables: Win32 error codes to portable `ERR_*` codes, and the
//! `Perror` header. Pure data, compiled on every host so that Linux tests cover it.
use super::error_list::*;
use crate::osd::WhoAmI;

// Win32 error numbers from winerror.h, spelled numerically so the tables need no
// Windows-only crate; a Windows-only unit test compares them with windows-sys.
pub(crate) mod win32 {
    pub const INVALID_FUNCTION: u32 = 1;
    pub const FILE_NOT_FOUND: u32 = 2;
    pub const PATH_NOT_FOUND: u32 = 3;
    pub const TOO_MANY_OPEN_FILES: u32 = 4;
    pub const ACCESS_DENIED: u32 = 5;
    pub const INVALID_HANDLE: u32 = 6;
    pub const ARENA_TRASHED: u32 = 7;
    pub const NOT_ENOUGH_MEMORY: u32 = 8;
    pub const INVALID_BLOCK: u32 = 9;
    pub const BAD_ENVIRONMENT: u32 = 10;
    pub const INVALID_ACCESS: u32 = 12;
    pub const INVALID_DATA: u32 = 13;
    pub const INVALID_DRIVE: u32 = 15;
    pub const CURRENT_DIRECTORY: u32 = 16;
    pub const NOT_SAME_DEVICE: u32 = 17;
    pub const NO_MORE_FILES: u32 = 18;
    pub const WRITE_FAULT: u32 = 29;
    pub const READ_FAULT: u32 = 30;
    pub const SHARING_VIOLATION: u32 = 32;
    pub const LOCK_VIOLATION: u32 = 33;
    pub const BAD_NETPATH: u32 = 53;
    pub const NETWORK_ACCESS_DENIED: u32 = 65;
    pub const BAD_NET_NAME: u32 = 67;
    pub const FILE_EXISTS: u32 = 80;
    pub const CANNOT_MAKE: u32 = 82;
    pub const FAIL_I24: u32 = 83;
    pub const INVALID_PARAMETER: u32 = 87;
    pub const NET_WRITE_FAULT: u32 = 88;
    pub const DRIVE_LOCKED: u32 = 108;
    pub const BUFFER_OVERFLOW: u32 = 111;
    pub const DISK_FULL: u32 = 112;
    pub const INVALID_TARGET_HANDLE: u32 = 114;
    pub const DIRECT_ACCESS_HANDLE: u32 = 130;
    pub const NEGATIVE_SEEK: u32 = 131;
    pub const SEEK_ON_DEVICE: u32 = 132;
    pub const DIR_NOT_EMPTY: u32 = 145;
    pub const NOT_LOCKED: u32 = 158;
    pub const BAD_THREADID_ADDR: u32 = 159;
    pub const BAD_PATHNAME: u32 = 161;
    pub const LOCK_FAILED: u32 = 167;
    pub const ALREADY_EXISTS: u32 = 183;
    pub const FILENAME_EXCED_RANGE: u32 = 206;
    pub const INVALID_ADDRESS: u32 = 487;
    pub const IO_PENDING: u32 = 997;
    pub const IO_DEVICE: u32 = 1117;
    pub const MAPPED_ALIGNMENT: u32 = 1132;
    pub const NOT_ENOUGH_QUOTA: u32 = 1816;
}
use win32::*;

type CodeTable = &'static [(u32, i32)];

static COMMON: CodeTable = &[
    (INVALID_FUNCTION, ERR_INVAL),
    (FILE_NOT_FOUND, ERR_NOENT),
    (PATH_NOT_FOUND, ERR_NOENT),
    (ACCESS_DENIED, ERR_ACCESS),
    (ARENA_TRASHED, ERR_NOMEM),
    (NOT_ENOUGH_MEMORY, ERR_NOMEM),
    (INVALID_BLOCK, ERR_NOMEM),
    (BAD_ENVIRONMENT, ERR_TOOBIG),
    (INVALID_ACCESS, ERR_INVAL),
    (INVALID_DATA, ERR_INVAL),
    (INVALID_DRIVE, ERR_NOENT),
    (CURRENT_DIRECTORY, ERR_ACCESS),
    (NO_MORE_FILES, ERR_NOENT),
    (LOCK_VIOLATION, ERR_ACCESS),
    (SHARING_VIOLATION, ERR_ACCESS),
    (BAD_NETPATH, ERR_NOENT),
    (NETWORK_ACCESS_DENIED, ERR_ACCESS),
    (BAD_NET_NAME, ERR_NOENT),
    (FILE_EXISTS, ERR_EXIST),
    (CANNOT_MAKE, ERR_ACCESS),
    (FAIL_I24, ERR_ACCESS),
    (INVALID_PARAMETER, ERR_INVAL),
    (DRIVE_LOCKED, ERR_ACCESS),
    (INVALID_HANDLE, ERR_INVAL),
    (NEGATIVE_SEEK, ERR_INVAL),
    (SEEK_ON_DEVICE, ERR_ACCESS),
    (NOT_LOCKED, ERR_ACCESS),
    (BAD_PATHNAME, ERR_NOENT),
    (LOCK_FAILED, ERR_ACCESS),
    (ALREADY_EXISTS, ERR_EXIST),
    (FILENAME_EXCED_RANGE, ERR_NOENT),
    (NOT_ENOUGH_QUOTA, ERR_QUOT),
    (IO_DEVICE, ERR_IO),
    // Native quirk: INVALID_BLOCK appears twice; the first entry (ERR_NOMEM) wins.
    (INVALID_BLOCK, ERR_FAULT),
    (BAD_THREADID_ADDR, ERR_FAULT),
    (INVALID_ADDRESS, ERR_FAULT),
    (MAPPED_ALIGNMENT, ERR_FAULT),
    (BUFFER_OVERFLOW, ERR_NAMETOOLONG),
];

static DIRECTORY: CodeTable = &[
    (FILE_NOT_FOUND, ERR_NOENT),
    (PATH_NOT_FOUND, ERR_NOENT),
    (INVALID_DRIVE, ERR_NOENT),
    (NO_MORE_FILES, ERR_NOENT),
    (BAD_NETPATH, ERR_NOENT),
    (BAD_NET_NAME, ERR_NOENT),
    (BAD_PATHNAME, ERR_NOENT),
    (FILENAME_EXCED_RANGE, ERR_NOENT),
];

static FILE: CodeTable = &[
    (INVALID_HANDLE, ERR_FBADF),
    (INVALID_TARGET_HANDLE, ERR_FBADF),
    (DIRECT_ACCESS_HANDLE, ERR_FBADF),
    (FILE_EXISTS, ERR_EXIST),
    (ALREADY_EXISTS, ERR_EXIST),
    (TOO_MANY_OPEN_FILES, ERR_FMFILE),
    (INVALID_FUNCTION, ERR_FINVAL),
    (INVALID_ACCESS, ERR_FINVAL),
    (INVALID_DATA, ERR_FINVAL),
    (INVALID_PARAMETER, ERR_FINVAL),
    // Native quirk: INVALID_HANDLE is listed again; the first entry (ERR_FBADF) wins.
    (INVALID_HANDLE, ERR_FINVAL),
    (NEGATIVE_SEEK, ERR_FINVAL),
    (IO_PENDING, ERR_FAGAIN),
    (WRITE_FAULT, ERR_FIO),
    (READ_FAULT, ERR_FIO),
    (NET_WRITE_FAULT, ERR_FIO),
    (IO_DEVICE, ERR_FIO),
    (LOCK_VIOLATION, ERR_FLOCKED),
    (LOCK_FAILED, ERR_FLOCKED),
];

static FILE_NODE: CodeTable = &[
    (INVALID_FUNCTION, ERR_FNINVAL),
    (INVALID_ACCESS, ERR_FNINVAL),
    (INVALID_DATA, ERR_FNINVAL),
    (INVALID_PARAMETER, ERR_FNINVAL),
    (INVALID_HANDLE, ERR_FNINVAL),
    (NEGATIVE_SEEK, ERR_FNINVAL),
    (DISK_FULL, ERR_FNOSPC),
    (DIR_NOT_EMPTY, ERR_FNNOTEMPTY),
    (NOT_SAME_DEVICE, ERR_FNXDEV),
];

fn lookup(table: CodeTable, errcode: u32) -> Option<i32> {
    table
        .iter()
        .find(|&&(win32, _)| win32 == errcode)
        .map(|&(_, code)| code)
}

/// The portable code the Windows `OSD_Error::SetValue` stores for a Win32 error recorded
/// by the raw class `from`; `windows_error_code(2, WhoAmI::Directory as i32) == ERR_NOENT`.
///
/// Directory, file and file-node errors try their own table first; every miss falls back
/// to the common table, and a code in no table is `ERR_SURPRISE`.
pub fn windows_error_code(errcode: i32, from: i32) -> i32 {
    // The native comparison casts the int to DWORD, so negative codes wrap.
    let errcode: u32 = errcode as u32;
    let own: Option<CodeTable> = match WhoAmI::from_raw(from) {
        Some(WhoAmI::Directory) => Some(DIRECTORY),
        Some(WhoAmI::File) => Some(FILE),
        Some(WhoAmI::FileNode) => Some(FILE_NODE),
        _ => None,
    };
    own.and_then(|table| lookup(table, errcode))
        .or_else(|| lookup(COMMON, errcode))
        .unwrap_or(ERR_SURPRISE)
}

/// The header the Windows `OSD_Error::Perror` writes before the message;
/// `windows_perror_header(WhoAmI::File as i32) == "Error ( OSD_File )"`.
///
/// Native quirks: `OSD_WProcess` prints "OSD_Environment", and the 32-wchar buffer cuts
/// "Error ( OSD_EnvironmentIterator )" to its first 31 characters.
pub fn windows_perror_header(from: i32) -> String {
    let class: &str = match WhoAmI::from_raw(from) {
        Some(WhoAmI::DirectoryIterator) => "OSD_DirectoryIterator",
        Some(WhoAmI::Directory) => "OSD_Directory",
        Some(WhoAmI::FileIterator) => "OSD_FileIterator",
        Some(WhoAmI::File) => "OSD_File",
        Some(WhoAmI::FileNode) => "OSD_FileNode",
        Some(WhoAmI::Host) => "OSD_Host",
        Some(WhoAmI::Process) | Some(WhoAmI::Environment) => "OSD_Environment",
        Some(WhoAmI::EnvironmentIterator) => "OSD_EnvironmentIterator",
        Some(WhoAmI::Disk) => "OSD_Disk",
        _ => "Unknown",
    };
    bounded_cat(bounded_cat(String::from("Error ( "), class), " )")
}

/// StringCchCatW on a 32-wchar buffer: a suffix that does not fit is cut to the 31
/// characters the buffer holds, and a later concatenation to a full buffer adds nothing.
fn bounded_cat(mut buffer: String, suffix: &str) -> String {
    const CAPACITY: usize = 31;
    let room: usize = CAPACITY.saturating_sub(buffer.len());
    buffer.push_str(&suffix[..suffix.len().min(room)]);
    buffer
}

#[cfg(all(test, windows))]
mod tests {
    use super::win32;
    use windows_sys::Win32::Foundation as native;
    #[test]
    fn win32_numbers_match_the_platform_headers() {
        assert_eq!(win32::FILE_NOT_FOUND, native::ERROR_FILE_NOT_FOUND);
        assert_eq!(win32::DIR_NOT_EMPTY, native::ERROR_DIR_NOT_EMPTY);
        assert_eq!(win32::NOT_ENOUGH_QUOTA, native::ERROR_NOT_ENOUGH_QUOTA);
        assert_eq!(win32::MAPPED_ALIGNMENT, native::ERROR_MAPPED_ALIGNMENT);
        assert_eq!(win32::IO_PENDING, native::ERROR_IO_PENDING);
        assert_eq!(
            win32::DIRECT_ACCESS_HANDLE,
            native::ERROR_DIRECT_ACCESS_HANDLE
        );
        assert_eq!(win32::BUFFER_OVERFLOW, native::ERROR_BUFFER_OVERFLOW);
        assert_eq!(win32::INVALID_ADDRESS, native::ERROR_INVALID_ADDRESS);
    }
}
