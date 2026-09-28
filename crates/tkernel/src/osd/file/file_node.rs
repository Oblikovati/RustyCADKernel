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

//! `OSD_FileNode`: the path and last error shared by files and directories, with the
//! node-level operations of the non-`_WIN32` build.
use super::calls::{
    AccessMode, CallResult, Errno, NodeStatus, NodeType, OpenFlags, PosixFileCalls,
};
use crate::{
    Error,
    osd::{CadPath, OpenMode, OsdError, OsdErrorAccess, Protection, SingleProtection, WhoAmI},
    quantity::Date,
};
use std::sync::Arc;

/// `OSD_FileNode`; reached through `File` and `Directory` (`file.exists()`).
#[derive(Clone)]
pub struct FileNode {
    pub(super) path: CadPath,
    pub(super) error: OsdError,
    pub(super) calls: Arc<dyn PosixFileCalls>,
}

impl std::fmt::Debug for FileNode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FileNode")
            .field("path", &self.path)
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl OsdErrorAccess for FileNode {
    fn osd_error(&self) -> &OsdError {
        &self.error
    }
    fn osd_error_mut(&mut self) -> &mut OsdError {
        &mut self.error
    }
}

const IAM: i32 = WhoAmI::FileNode as i32;

impl FileNode {
    /// `OSD_FileNode(name)` over injected calls (the constructors are protected natively;
    /// `File`/`Directory` use them); `FileNode::with_calls(path, calls)`.
    pub fn with_calls(path: CadPath, calls: Arc<dyn PosixFileCalls>) -> Self {
        Self {
            path,
            error: OsdError::new(),
            calls,
        }
    }
    /// The injected system calls; `node.calls()`.
    pub fn calls(&self) -> &Arc<dyn PosixFileCalls> {
        &self.calls
    }
    /// `Path(name)`: the output reference becomes the returned path; `node.path()`.
    pub fn path(&self) -> CadPath {
        self.path.clone()
    }
    /// `SetPath`: replaces the path and forgets the last error; `node.set_path(path)`.
    pub fn set_path(&mut self, path: CadPath) {
        self.error.reset();
        self.path = path;
    }
    /// The C string every call receives: `SystemName()` of the path.
    pub(super) fn system_name(&self) -> Vec<u8> {
        self.path.system_name().as_bytes().to_vec()
    }
    /// `Exists`: `access(name, F_OK) == 0`; an empty path is absent; `node.exists()`.
    pub fn exists(&self) -> bool {
        self.calls
            .access(&self.system_name(), AccessMode::default())
            .is_ok()
    }
    /// Records `errno` with this class and a context message (`myError.SetValue`).
    pub(super) fn record(&mut self, errno: Errno, who: i32, message: &str) {
        self.error.set_value(errno.0, who, message);
    }
    /// `Remove`: `rmdir` a directory, `unlink` a regular file or FIFO; the node must be
    /// writable itself (native quirk: `access(name, W_OK)` on the node, not its parent),
    /// and any other type records EINVAL; `node.remove()`.
    pub fn remove(&mut self) {
        if let Err(errno) = self.remove_node() {
            self.record(errno, IAM, "Remove");
        }
    }
    fn remove_node(&self) -> CallResult<()> {
        let name: Vec<u8> = self.system_name();
        let write: AccessMode = AccessMode {
            write: true,
            ..AccessMode::default()
        };
        self.calls.access(&name, write)?;
        match self.calls.stat(&name)?.node_type {
            NodeType::Directory => self.calls.rmdir(&name),
            NodeType::Regular | NodeType::Link | NodeType::Fifo => self.calls.unlink(&name),
            _ => Err(Errno(einval())),
        }
    }
    /// `Move`: `rename` to the other path's system name; `node.move_to(&target)`.
    pub fn move_to(&mut self, new_path: &CadPath) {
        let target: Vec<u8> = new_path.system_name().as_bytes().to_vec();
        if let Err(errno) = self.calls.rename(&self.system_name(), &target) {
            self.record(errno, IAM, "Move");
        }
    }
    /// `Copy`: copies the bytes into a truncated 0644 target in 4096-byte chunks; any
    /// failure records errno -1 with "Copy failed" (native quirk: the real errno is
    /// dropped, so `Perror` reports "Unknowm error #-1"); `node.copy_to(&target)`.
    pub fn copy_to(&mut self, to_path: &CadPath) {
        let target: Vec<u8> = to_path.system_name().as_bytes().to_vec();
        if copy_file(self.calls.as_ref(), &self.system_name(), &target) != 0 {
            self.record(Errno(-1), IAM, "Copy failed");
        }
    }
    /// `Protection`: user, group and world rights from the mode bits, with system equal
    /// to group; a failing `stat` records errno and (natively reading an uninitialized
    /// buffer) yields no rights here; `node.protection()`.
    pub fn protection(&mut self) -> Protection {
        let permissions: u32 = match self.calls.stat(&self.system_name()) {
            Ok(status) => status.permissions,
            Err(errno) => {
                self.record(errno, IAM, "Protection");
                0
            }
        };
        let group: SingleProtection = rights(permissions >> 3);
        Protection::with_values(group, rights(permissions >> 6), group, rights(permissions))
    }
    /// `SetProtection`: `chmod` with `Protection::Internal`; `node.set_protection(&rights)`.
    pub fn set_protection(&mut self, protection: &Protection) {
        if let Err(errno) = self.calls.chmod(&self.system_name(), protection.internal()) {
            self.record(errno, IAM, "SetProtection");
        }
    }
    /// `CreationMoment`: the local time of `st_ctime`, or 1/1/1979 when `stat` fails;
    /// `node.creation_moment()?`. A time before 1979 raises `Quantity_DateDefinitionError`.
    pub fn creation_moment(&self) -> Result<Date, Error> {
        self.change_moment()
    }
    /// `AccessMoment`: natively also `st_ctime` (not `st_atime`); `node.access_moment()?`.
    pub fn access_moment(&self) -> Result<Date, Error> {
        self.change_moment()
    }
    fn change_moment(&self) -> Result<Date, Error> {
        let local = self
            .calls
            .stat(&self.system_name())
            .ok()
            .and_then(|status: NodeStatus| self.calls.local_time(status.change_time));
        match local {
            Some(time) => Date::new(
                time.month,
                time.day,
                time.year,
                time.hour,
                time.minute,
                time.second,
                0,
                0,
            ),
            None => Date::new(1, 1, 1979, 0, 0, 0, 0, 0),
        }
    }
}

/// The `OSD_R | OSD_W | OSD_X` right of three mode bits (`rwx` in bits 2..0).
fn rights(bits: u32) -> SingleProtection {
    let mut value: i32 = 0;
    value |= if bits & 0o4 != 0 {
        SingleProtection::R as i32
    } else {
        0
    };
    value |= if bits & 0o2 != 0 {
        SingleProtection::W as i32
    } else {
        0
    };
    value |= if bits & 0o1 != 0 {
        SingleProtection::X as i32
    } else {
        0
    };
    SingleProtection::from_raw(value).expect("three bits are a protection")
}

#[cfg(unix)]
pub(super) const fn einval() -> i32 {
    libc::EINVAL
}
#[cfg(not(unix))]
pub(super) const fn einval() -> i32 {
    22
}

/// The native `copy_file`: returns the errno it ends with, 0 on success.
fn copy_file(calls: &dyn PosixFileCalls, source: &[u8], target: &[u8]) -> i32 {
    let read_only: OpenFlags = OpenFlags {
        access: OpenMode::ReadOnly,
        create: false,
        truncate: false,
        append: false,
    };
    let source_fd: i32 = match calls.open(source, read_only, 0) {
        Ok(fd) => fd,
        Err(errno) => return errno.0,
    };
    let write: OpenFlags = OpenFlags {
        access: OpenMode::WriteOnly,
        create: true,
        truncate: true,
        append: false,
    };
    let target_fd: i32 = match calls.open(target, write, 0o644) {
        Ok(fd) => fd,
        Err(errno) => {
            let _ = calls.close(source_fd);
            return errno.0;
        }
    };
    let mut errno: i32 = copy_chunks(calls, source_fd, target_fd);
    // Native: `err = errno` survives unless it was 0, then each close may set it.
    if let Err(Errno(code)) = calls.close(target_fd) {
        errno = if errno == 0 { code } else { errno };
    }
    if let Err(Errno(code)) = calls.close(source_fd) {
        errno = if errno == 0 { code } else { errno };
    }
    errno
}

/// The read/write loop of `copy_file`; a short write with errno 0 reports ENOSPC.
fn copy_chunks(calls: &dyn PosixFileCalls, source_fd: i32, target_fd: i32) -> i32 {
    let mut buffer: [u8; 4096] = [0; 4096];
    loop {
        let count: usize = match calls.read(source_fd, &mut buffer) {
            Ok(0) => return 0,
            Ok(count) => count,
            Err(errno) => return errno.0,
        };
        match calls.write(target_fd, &buffer[..count]) {
            Ok(written) if written == count => continue,
            Ok(_) => return enospc(),
            Err(errno) => return errno.0,
        }
    }
}

#[cfg(unix)]
const fn enospc() -> i32 {
    libc::ENOSPC
}
#[cfg(not(unix))]
const fn enospc() -> i32 {
    28
}
