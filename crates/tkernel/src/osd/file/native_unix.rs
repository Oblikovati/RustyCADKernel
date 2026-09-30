//! The narrow libc boundary of the OSD file classes on Unix hosts: the exact system calls
//! the non-`_WIN32` OSD sources make, so errno values and side effects are native.
#![allow(unsafe_code)]
use super::calls::{
    AccessMode, CallResult, Errno, NodeStatus, NodeType, OpenFlags, PosixFileCalls, RecordLock,
    SeekOrigin,
};
use crate::osd::LocalTime;
use crate::osd::OpenMode;
use std::ffi::{CStr, CString};

/// The real POSIX calls; `NativePosixCalls.stat(b"/tmp")`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativePosixCalls;

fn last_errno() -> Errno {
    Errno(std::io::Error::last_os_error().raw_os_error().unwrap_or(0))
}

fn c_path(path: &[u8]) -> CString {
    // The OSD paths are C strings: everything from the first NUL on is not part of them.
    let end: usize = path
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(path.len());
    CString::new(&path[..end]).expect("no interior NUL after truncation")
}

fn status_zero(status: libc::c_int) -> CallResult<()> {
    if status == -1 {
        Err(last_errno())
    } else {
        Ok(())
    }
}

// mode_t is u16 on macOS and the stat field widths vary by target: the casts are needed there.
#[allow(clippy::unnecessary_cast)]
fn node_status(raw: &libc::stat) -> NodeStatus {
    let mode: u32 = raw.st_mode as u32;
    let node_type: NodeType = match mode & (libc::S_IFMT as u32) {
        kind if kind == libc::S_IFREG as u32 => NodeType::Regular,
        kind if kind == libc::S_IFDIR as u32 => NodeType::Directory,
        kind if kind == libc::S_IFLNK as u32 => NodeType::Link,
        kind if kind == libc::S_IFIFO as u32 => NodeType::Fifo,
        kind if kind == libc::S_IFSOCK as u32 => NodeType::Socket,
        _ => NodeType::Other,
    };
    NodeStatus {
        node_type,
        permissions: mode & 0o7777,
        size: raw.st_size as u64,
        change_time: raw.st_ctime as i64,
        block_size: raw.st_blksize as u64,
    }
}

impl PosixFileCalls for NativePosixCalls {
    fn access(&self, path: &[u8], mode: AccessMode) -> CallResult<()> {
        let mut bits: libc::c_int = libc::F_OK;
        bits |= if mode.read { libc::R_OK } else { 0 };
        bits |= if mode.write { libc::W_OK } else { 0 };
        bits |= if mode.execute { libc::X_OK } else { 0 };
        let name: CString = c_path(path);
        // SAFETY: name is a valid NUL-terminated string alive for the call.
        status_zero(unsafe { libc::access(name.as_ptr(), bits) })
    }
    fn stat(&self, path: &[u8]) -> CallResult<NodeStatus> {
        let name: CString = c_path(path);
        // SAFETY: an all-zero stat is a valid value of this plain C struct.
        let mut raw: libc::stat = unsafe { std::mem::zeroed() };
        // SAFETY: name is NUL-terminated; raw is a writable stat alive for the call.
        status_zero(unsafe { libc::stat(name.as_ptr(), &mut raw) })?;
        Ok(node_status(&raw))
    }
    fn fstat(&self, fd: i32) -> CallResult<NodeStatus> {
        // SAFETY: an all-zero stat is a valid value of this plain C struct.
        let mut raw: libc::stat = unsafe { std::mem::zeroed() };
        // SAFETY: raw is a writable stat alive for the call; any fd value is accepted.
        status_zero(unsafe { libc::fstat(fd, &mut raw) })?;
        Ok(node_status(&raw))
    }
    fn open(&self, path: &[u8], flags: OpenFlags, mode: u32) -> CallResult<i32> {
        let mut bits: libc::c_int = match flags.access {
            OpenMode::ReadOnly => libc::O_RDONLY,
            OpenMode::WriteOnly => libc::O_WRONLY,
            OpenMode::ReadWrite => libc::O_RDWR,
        };
        bits |= if flags.create { libc::O_CREAT } else { 0 };
        bits |= if flags.truncate { libc::O_TRUNC } else { 0 };
        bits |= if flags.append { libc::O_APPEND } else { 0 };
        let name: CString = c_path(path);
        // SAFETY: name is NUL-terminated; the variadic mode is passed as c_uint.
        let fd: libc::c_int = unsafe { libc::open(name.as_ptr(), bits, mode as libc::c_uint) };
        if fd < 0 { Err(last_errno()) } else { Ok(fd) }
    }
    fn close(&self, fd: i32) -> CallResult<()> {
        // SAFETY: closing a descriptor number has no memory-safety requirement.
        status_zero(unsafe { libc::close(fd) })
    }
    fn read(&self, fd: i32, buffer: &mut [u8]) -> CallResult<usize> {
        // SAFETY: buffer is writable for its full length during the call.
        let count: isize = unsafe { libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len()) };
        usize::try_from(count).map_err(|_| last_errno())
    }
    fn write(&self, fd: i32, buffer: &[u8]) -> CallResult<usize> {
        // SAFETY: buffer is readable for its full length during the call.
        let count: isize = unsafe { libc::write(fd, buffer.as_ptr().cast(), buffer.len()) };
        usize::try_from(count).map_err(|_| last_errno())
    }
    fn lseek(&self, fd: i32, offset: i64, origin: SeekOrigin) -> CallResult<i64> {
        let whence: libc::c_int = match origin {
            SeekOrigin::Start => libc::SEEK_SET,
            SeekOrigin::Current => libc::SEEK_CUR,
            SeekOrigin::End => libc::SEEK_END,
        };
        // SAFETY: lseek only takes plain integers.
        let position: libc::off_t = unsafe { libc::lseek(fd, offset as libc::off_t, whence) };
        if position == -1 {
            Err(last_errno())
        } else {
            Ok(position as i64)
        }
    }
    fn record_lock(&self, fd: i32, lock: RecordLock) -> CallResult<()> {
        // SAFETY: an all-zero flock is a valid value of this plain C struct.
        let mut key: libc::flock = unsafe { std::mem::zeroed() };
        key.l_whence = libc::SEEK_SET as _;
        let (command, kind) = match lock {
            RecordLock::Read => (libc::F_SETLKW, libc::F_RDLCK),
            RecordLock::Write => (libc::F_SETLKW, libc::F_WRLCK),
            RecordLock::Unlock => (libc::F_SETLK, libc::F_UNLCK),
        };
        key.l_type = kind as _;
        // SAFETY: key is a valid flock alive for the call.
        status_zero(unsafe { libc::fcntl(fd, command, &key) })
    }
    fn rename(&self, from: &[u8], to: &[u8]) -> CallResult<()> {
        let (from, to) = (c_path(from), c_path(to));
        // SAFETY: both names are NUL-terminated and alive for the call.
        status_zero(unsafe { libc::rename(from.as_ptr(), to.as_ptr()) })
    }
    fn unlink(&self, path: &[u8]) -> CallResult<()> {
        let name: CString = c_path(path);
        // SAFETY: name is NUL-terminated and alive for the call.
        status_zero(unsafe { libc::unlink(name.as_ptr()) })
    }
    fn rmdir(&self, path: &[u8]) -> CallResult<()> {
        let name: CString = c_path(path);
        // SAFETY: name is NUL-terminated and alive for the call.
        status_zero(unsafe { libc::rmdir(name.as_ptr()) })
    }
    fn mkdir(&self, path: &[u8], mode: u32) -> CallResult<()> {
        let name: CString = c_path(path);
        // SAFETY: name is NUL-terminated and alive for the call.
        status_zero(unsafe { libc::mkdir(name.as_ptr(), mode as libc::mode_t) })
    }
    fn chmod(&self, path: &[u8], mode: u32) -> CallResult<()> {
        let name: CString = c_path(path);
        // SAFETY: name is NUL-terminated and alive for the call.
        status_zero(unsafe { libc::chmod(name.as_ptr(), mode as libc::mode_t) })
    }
    fn umask(&self, mask: u32) -> u32 {
        // SAFETY: umask only takes and returns a plain integer.
        unsafe { libc::umask(mask as libc::mode_t) as u32 }
    }
    fn mkstemp(&self, template: &[u8]) -> CallResult<(i32, Vec<u8>)> {
        let mut name: Vec<u8> = c_path(template).into_bytes_with_nul();
        // SAFETY: name is a writable NUL-terminated buffer that mkstemp edits in place.
        let fd: libc::c_int = unsafe { libc::mkstemp(name.as_mut_ptr().cast()) };
        name.pop();
        if fd < 0 {
            Err(last_errno())
        } else {
            Ok((fd, name))
        }
    }
    fn mkdtemp(&self, template: &[u8]) -> CallResult<Vec<u8>> {
        let mut name: Vec<u8> = c_path(template).into_bytes_with_nul();
        // SAFETY: name is a writable NUL-terminated buffer that mkdtemp edits in place.
        let made: *mut libc::c_char = unsafe { libc::mkdtemp(name.as_mut_ptr().cast()) };
        name.pop();
        if made.is_null() {
            Err(last_errno())
        } else {
            Ok(name)
        }
    }
    fn read_directory(&self, path: &[u8]) -> CallResult<Vec<Vec<u8>>> {
        let name: CString = c_path(path);
        // SAFETY: name is NUL-terminated and alive for the call.
        let stream: *mut libc::DIR = unsafe { libc::opendir(name.as_ptr()) };
        if stream.is_null() {
            return Err(last_errno());
        }
        let mut names: Vec<Vec<u8>> = Vec::new();
        loop {
            // SAFETY: stream is an open DIR owned by this loop.
            let entry: *mut libc::dirent = unsafe { libc::readdir(stream) };
            if entry.is_null() {
                break;
            }
            // SAFETY: readdir returned a valid entry whose d_name is NUL-terminated.
            let entry_name: &CStr = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };
            names.push(entry_name.to_bytes().to_vec());
        }
        // SAFETY: stream is open and closed exactly once.
        unsafe { libc::closedir(stream) };
        Ok(names)
    }
    fn local_time(&self, seconds: i64) -> Option<LocalTime> {
        let time: libc::time_t = seconds as libc::time_t;
        // SAFETY: an all-zero tm is a valid value of this plain C struct.
        let mut fields: libc::tm = unsafe { std::mem::zeroed() };
        // SAFETY: time and fields are valid for the call; localtime_r is reentrant.
        let result: *mut libc::tm = unsafe { libc::localtime_r(&time, &mut fields) };
        (!result.is_null()).then_some(LocalTime {
            month: fields.tm_mon + 1,
            day: fields.tm_mday,
            year: fields.tm_year + 1900,
            hour: fields.tm_hour,
            minute: fields.tm_min,
            second: fields.tm_sec,
            millisecond: 0,
        })
    }
}
