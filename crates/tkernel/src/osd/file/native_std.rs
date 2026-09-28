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

//! The POSIX calls of the OSD file classes emulated over `std::fs` on hosts without libc
//! (Windows). The native Windows build of OSD_File/OSD_FileNode (Win32 handles, ACL
//! protections, `LockFileEx`) is not ported: this backend runs the Unix-build algorithm
//! with the closest std primitive for each call, and errno values are the raw OS codes.
use super::calls::{
    AccessMode, CallResult, Errno, NodeStatus, NodeType, OpenFlags, PosixFileCalls, RecordLock,
    SeekOrigin,
};
use crate::osd::LocalTime;
use crate::osd::OpenMode;
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Seek, SeekFrom, Write},
    sync::Mutex,
};

/// The std-emulated calls; `NativePosixCalls.stat(b"C:/")`.
#[derive(Debug, Default)]
pub struct NativePosixCalls;

/// Open descriptors: the emulated descriptor numbers start at 3, as on POSIX.
static DESCRIPTORS: Mutex<BTreeMap<i32, fs::File>> = Mutex::new(BTreeMap::new());

fn errno(error: std::io::Error) -> Errno {
    Errno(error.raw_os_error().unwrap_or(-1))
}

fn text(path: &[u8]) -> String {
    String::from_utf8_lossy(path).into_owned()
}

fn status_of(metadata: &fs::Metadata) -> NodeStatus {
    let node_type: NodeType = if metadata.is_dir() {
        NodeType::Directory
    } else if metadata.is_file() {
        NodeType::Regular
    } else {
        NodeType::Other
    };
    // Read-only files have no write bits; everything else reads as rw-r--r-- (0644).
    let permissions: u32 = if metadata.permissions().readonly() {
        0o444
    } else {
        0o644
    };
    let change_time: i64 = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |elapsed| elapsed.as_secs() as i64);
    NodeStatus {
        node_type,
        permissions,
        size: metadata.len(),
        change_time,
        block_size: 4096,
    }
}

fn with_descriptor<T>(
    fd: i32,
    action: impl FnOnce(&mut fs::File) -> std::io::Result<T>,
) -> CallResult<T> {
    let mut table = DESCRIPTORS.lock().expect("descriptor table");
    let file: &mut fs::File = table.get_mut(&fd).ok_or(Errno(9))?;
    action(file).map_err(errno)
}

fn register(file: fs::File) -> i32 {
    let mut table = DESCRIPTORS.lock().expect("descriptor table");
    let fd: i32 = (3..)
        .find(|number| !table.contains_key(number))
        .expect("free descriptor");
    table.insert(fd, file);
    fd
}

impl PosixFileCalls for NativePosixCalls {
    fn access(&self, path: &[u8], mode: AccessMode) -> CallResult<()> {
        let metadata: fs::Metadata = fs::metadata(text(path)).map_err(errno)?;
        if mode.write && metadata.permissions().readonly() {
            return Err(Errno(13));
        }
        Ok(())
    }
    fn stat(&self, path: &[u8]) -> CallResult<NodeStatus> {
        fs::metadata(text(path))
            .map(|metadata| status_of(&metadata))
            .map_err(errno)
    }
    fn fstat(&self, fd: i32) -> CallResult<NodeStatus> {
        with_descriptor(fd, |file| file.metadata()).map(|metadata| status_of(&metadata))
    }
    fn open(&self, path: &[u8], flags: OpenFlags, _mode: u32) -> CallResult<i32> {
        let mut options: fs::OpenOptions = fs::OpenOptions::new();
        options
            .read(flags.access != OpenMode::WriteOnly)
            .write(flags.access != OpenMode::ReadOnly || flags.truncate)
            .append(flags.append)
            .create(flags.create)
            .truncate(flags.truncate);
        options.open(text(path)).map(register).map_err(errno)
    }
    fn close(&self, fd: i32) -> CallResult<()> {
        let mut table = DESCRIPTORS.lock().expect("descriptor table");
        table.remove(&fd).map(drop).ok_or(Errno(9))
    }
    fn read(&self, fd: i32, buffer: &mut [u8]) -> CallResult<usize> {
        with_descriptor(fd, |file| file.read(buffer))
    }
    fn write(&self, fd: i32, buffer: &[u8]) -> CallResult<usize> {
        with_descriptor(fd, |file| file.write(buffer))
    }
    fn lseek(&self, fd: i32, offset: i64, origin: SeekOrigin) -> CallResult<i64> {
        let target: SeekFrom = match origin {
            SeekOrigin::Start if offset < 0 => return Err(Errno(22)),
            SeekOrigin::Start => SeekFrom::Start(offset as u64),
            SeekOrigin::Current => SeekFrom::Current(offset),
            SeekOrigin::End => SeekFrom::End(offset),
        };
        with_descriptor(fd, |file| file.seek(target)).map(|position| position as i64)
    }
    fn record_lock(&self, fd: i32, lock: RecordLock) -> CallResult<()> {
        with_descriptor(fd, |file| match lock {
            RecordLock::Read => file.lock_shared(),
            RecordLock::Write => file.lock(),
            RecordLock::Unlock => file.unlock(),
        })
    }
    fn rename(&self, from: &[u8], to: &[u8]) -> CallResult<()> {
        fs::rename(text(from), text(to)).map_err(errno)
    }
    fn unlink(&self, path: &[u8]) -> CallResult<()> {
        fs::remove_file(text(path)).map_err(errno)
    }
    fn rmdir(&self, path: &[u8]) -> CallResult<()> {
        fs::remove_dir(text(path)).map_err(errno)
    }
    fn mkdir(&self, path: &[u8], _mode: u32) -> CallResult<()> {
        fs::create_dir(text(path)).map_err(errno)
    }
    fn chmod(&self, path: &[u8], mode: u32) -> CallResult<()> {
        let name: String = text(path);
        let mut permissions: fs::Permissions = fs::metadata(&name).map_err(errno)?.permissions();
        permissions.set_readonly(mode & 0o200 == 0);
        fs::set_permissions(name, permissions).map_err(errno)
    }
    fn umask(&self, _mask: u32) -> u32 {
        // Windows has no creation mask for these calls.
        0
    }
    fn mkstemp(&self, template: &[u8]) -> CallResult<(i32, Vec<u8>)> {
        let name: Vec<u8> = unique_name(template);
        let file: fs::File = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(text(&name))
            .map_err(errno)?;
        Ok((register(file), name))
    }
    fn mkdtemp(&self, template: &[u8]) -> CallResult<Vec<u8>> {
        let name: Vec<u8> = unique_name(template);
        fs::create_dir(text(&name)).map_err(errno)?;
        Ok(name)
    }
    fn read_directory(&self, path: &[u8]) -> CallResult<Vec<Vec<u8>>> {
        // std skips "." and ".."; readdir reports them first.
        let mut names: Vec<Vec<u8>> = vec![b".".to_vec(), b"..".to_vec()];
        for entry in fs::read_dir(text(path)).map_err(errno)? {
            let entry: fs::DirEntry = entry.map_err(errno)?;
            names.push(entry.file_name().to_string_lossy().as_bytes().to_vec());
        }
        Ok(names)
    }
    fn local_time(&self, seconds: i64) -> Option<LocalTime> {
        // std has no time-zone database: the broken-down time is UTC.
        let days: i64 = seconds.div_euclid(86_400);
        let of_day: i64 = seconds.rem_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        Some(LocalTime {
            month,
            day,
            year,
            hour: (of_day / 3600) as i32,
            minute: (of_day / 60 % 60) as i32,
            second: (of_day % 60) as i32,
            millisecond: 0,
        })
    }
}

/// The template with its trailing `XXXXXX` replaced by the process id and a counter.
fn unique_name(template: &[u8]) -> Vec<u8> {
    static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let stem: &[u8] = template.strip_suffix(b"XXXXXX").unwrap_or(template);
    let serial: u32 = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temp: std::path::PathBuf = std::env::temp_dir();
    let base: &[u8] = stem.rsplit(|&byte| byte == b'/').next().unwrap_or(stem);
    let mut name: Vec<u8> = temp.to_string_lossy().as_bytes().to_vec();
    name.push(b'/');
    name.extend_from_slice(base);
    name.extend_from_slice(format!("{:x}{serial:02x}", std::process::id()).as_bytes());
    name
}

/// Howard Hinnant's days-to-civil conversion.
fn civil_from_days(days: i64) -> (i32, i32, i32) {
    let shifted: i64 = days + 719_468;
    let era: i64 = shifted.div_euclid(146_097);
    let day_of_era: i64 = shifted - era * 146_097;
    let year_of_era: i64 =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year: i64 = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month: i64 = (5 * day_of_year + 2) / 153;
    let day: i64 = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month: i64 = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year: i64 = year_of_era + era * 400 + i64::from(month <= 2);
    (year as i32, month as i32, day as i32)
}
