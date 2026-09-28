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

//! The Unix `OSD_Error::Perror` switch, as a table of (errno, reporting classes) rules.
//!
//! Only the Linux/macOS branches are compiled natively: the `SUN`, `IRIX4` and AIX
//! blocks (EBADMSG, EDEADLK, EWOULDBLOCK, ENOTEMPTY-as-EEXIST) are absent, as in the
//! native build of this profile.
use super::error_list::*;
use crate::osd::WhoAmI;

/// One `case errno: switch (myCode)` arm; an empty `from` list is the `default:` arm.
struct PerrorRule {
    errno: i32,
    from: &'static [WhoAmI],
    text: &'static str,
    code: i32,
}

const fn rule(errno: i32, from: &'static [WhoAmI], text: &'static str, code: i32) -> PerrorRule {
    PerrorRule {
        errno,
        from,
        text,
        code,
    }
}

use WhoAmI::{Directory as D, File as F, FileNode as N, Package as P};

/// ENOENT for a file or node reads "File doesn't exist or" before this text unless the
/// context message is exactly "Open".
const NOENT_TEXT: &str = "Invalid path (empty string)";

// Native order; within one errno the specific arms precede the `default:` arm.
static RULES: &[PerrorRule] = &[
    rule(
        libc::EBADF,
        &[F],
        "Invalid file descriptor or bad mode",
        ERR_FBADF,
    ),
    rule(libc::EINVAL, &[N], "Can't unlink '.' or '..'", ERR_FNINVAL),
    rule(libc::EINVAL, &[F], "Invalid file descriptor", ERR_FINVAL),
    rule(libc::EINVAL, &[], "User error : Bad parameter", ERR_INVAL),
    rule(
        libc::EDQUOT,
        &[D, N, F],
        "Exceed quota of disk blocks",
        ERR_QUOT,
    ),
    rule(
        libc::ENOLCK,
        &[F],
        "No more file lock entries available",
        ERR_FNOLCK,
    ),
    rule(
        libc::EOPNOTSUPP,
        &[F],
        "File descriptor doesn't refer to a file",
        ERR_FWFD,
    ),
    rule(libc::EACCES, &[], "Permission denied", ERR_ACCESS),
    rule(
        libc::EBUSY,
        &[N],
        "Still used by system or a process",
        ERR_FNBUSY,
    ),
    rule(
        libc::ERANGE,
        &[F],
        "Not enough or too many bytes written",
        ERR_FRANGE,
    ),
    rule(libc::EPERM, &[P], "Permission denied", ERR_PPERM),
    rule(
        libc::EPERM,
        &[N],
        "Permission denied or can't unlink directory",
        ERR_FPERM,
    ),
    rule(
        libc::EPERM,
        &[],
        "abnormal error : you modified OSD library",
        ERR_PERM,
    ),
    rule(libc::EROFS, &[N, F], "Read only file system", ERR_ROFS),
    rule(libc::ENXIO, &[D, N], "I/O error", ERR_IO),
    rule(
        libc::ENXIO,
        &[F],
        "I/O error or Hang up from terminal",
        ERR_FIO,
    ),
    rule(libc::EIO, &[D, N], "I/O error", ERR_IO),
    rule(
        libc::EIO,
        &[F],
        "I/O error or Hang up from terminal",
        ERR_FIO,
    ),
    rule(libc::EISDIR, &[N, F], "The File is a Directory", ERR_ISDIR),
    rule(
        libc::EAGAIN,
        &[F],
        "No data ready to be read/written",
        ERR_FAGAIN,
    ),
    rule(
        libc::ENOTDIR,
        &[D, N, F],
        "A component of path is not a Directory",
        ERR_NOTDIR,
    ),
    rule(libc::EMLINK, &[D], "Too many links", ERR_DMLINK),
    // Native quirk: the ELOOP arm writes its text but never sets extCode.
    rule(
        libc::ELOOP,
        &[D, N, F],
        "Too many symbolic links",
        ERR_SURPRISE,
    ),
    rule(
        libc::EFAULT,
        &[],
        "User error : arguments point to an illegal address",
        ERR_FAULT,
    ),
    rule(
        libc::EFBIG,
        &[F],
        "Exceed process's file size limit or the maximum file size",
        ERR_FFBIG,
    ),
    rule(libc::EINTR, &[], "operation broken by a signal", ERR_INTR),
    rule(libc::ENOMEM, &[], "Not enough memory", ERR_NOMEM),
    rule(
        libc::EMFILE,
        &[F],
        "Too many file descriptors are currently in use by this process",
        ERR_FMFILE,
    ),
    rule(
        libc::ENAMETOOLONG,
        &[],
        "File name too long",
        ERR_NAMETOOLONG,
    ),
    rule(
        libc::ENFILE,
        &[F],
        "Too many files are currently open in the system",
        ERR_FNFILE,
    ),
    rule(
        libc::EXDEV,
        &[N],
        "The link named by path2 and the file named by path1 are\non different logical devices (file systems)",
        ERR_FNXDEV,
    ),
    rule(libc::ENOENT, &[N, F], NOENT_TEXT, ERR_NOENT),
    rule(
        libc::ENOENT,
        &[D],
        "A component of the path prefix of path does not exist",
        ERR_DNOENT,
    ),
    rule(
        libc::ENOSPC,
        &[D, F],
        "No more free space on file system",
        ERR_FNOSPC,
    ),
    rule(libc::ENOTEMPTY, &[N], "Directory not empty", ERR_FNNOTEMPTY),
    rule(libc::EEXIST, &[N], "Directory not empty", ERR_FNNOTEMPTY),
    rule(
        libc::EEXIST,
        &[F],
        "OSD_Create and OSD_Exclude are set and the named file exists",
        ERR_FEXIST,
    ),
    rule(
        libc::EEXIST,
        &[],
        "Identifier already exists for this key",
        ERR_EXIST,
    ),
    rule(
        libc::E2BIG,
        &[],
        "Too many Semaphore/Shared memory for a process.Reconfigure Kernel with greater values",
        ERR_TOOBIG,
    ),
];

/// Builds the `OSD_OSDError` text and the portable code for a nonzero errno recorded by
/// the raw class `from` with context `message`; `unix_perror_report(libc::EACCES, 3, b"")`.
///
/// A known errno whose arms do not name `from` leaves only the frame text and
/// `ERR_SURPRISE`; an unknown errno reads "Unknowm error #n" (native spelling).
pub fn unix_perror_report(errno: i32, from: i32, message: &[u8]) -> (String, i32) {
    let mut buffer: String = String::from(" :\n ");
    let mut code: i32 = ERR_SURPRISE;
    let mut arms = RULES.iter().filter(|arm| arm.errno == errno).peekable();
    if arms.peek().is_none() {
        buffer = format!("{buffer}Unknowm error #{errno}");
        code = ERR_UNKNOWN;
    }
    if let Some(arm) = arms.find(|arm| arm_matches(arm, from)) {
        push_arm_text(&mut buffer, arm, message);
        code = arm.code;
    }
    buffer.push_str(".\n\n");
    (buffer, code)
}

fn arm_matches(arm: &PerrorRule, from: i32) -> bool {
    arm.from.is_empty() || arm.from.iter().any(|&class| class as i32 == from)
}

fn push_arm_text(buffer: &mut String, arm: &PerrorRule, message: &[u8]) {
    if arm.errno == libc::ENOENT && arm.code == ERR_NOENT && message != b"Open" {
        buffer.push_str("File doesn't exist or");
    }
    buffer.push_str(arm.text);
}
