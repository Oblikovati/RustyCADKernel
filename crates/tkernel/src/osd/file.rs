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

//! File-system nodes: `OSD_FileNode`, `OSD_File`, `OSD_Directory`,
//! `OSD_DirectoryIterator` and `OSD_FileIterator`.
//!
//! The classes follow the non-`_WIN32` OSD sources (the verified Linux profile, also
//! macOS). Every system call goes through [`PosixFileCalls`]: `NativePosixCalls` makes
//! the libc calls on Unix hosts; on Windows it emulates them over `std::fs`, because the
//! Win32 build of these classes (handles, ACL protections, `LockFileEx`) is not ported.
mod calls;
mod directory;
mod file_io;
mod file_node;
mod iterators;
#[cfg(not(unix))]
mod native_std;
#[cfg(unix)]
mod native_unix;
mod osd_file;
mod stdio;

pub use calls::{
    AccessMode, CallResult, Errno, LocalTime, NodeStatus, NodeType, OpenFlags, PosixFileCalls,
    RecordLock, SeekOrigin, native_calls,
};
pub use directory::Directory;
pub use file_node::FileNode;
pub use iterators::{DirectoryIterator, FileIterator, mask_matches};
#[cfg(not(unix))]
pub use native_std::NativePosixCalls;
#[cfg(unix)]
pub use native_unix::NativePosixCalls;
pub use osd_file::File;
