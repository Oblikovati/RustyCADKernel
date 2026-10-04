//! The POSIX calls the non-`_WIN32` OSD file classes make, as a project-owned interface.
//!
//! Every failure is the native `errno` value, which the OSD classes record in their
//! `OSD_Error` exactly where the C++ code does. `NativePosixCalls` performs the real calls;
//! tests inject a named fake.
use super::directory_stream::{DirectoryEntries, buffered_directory_entries};
use crate::osd::LocalTime;
use std::sync::Arc;

/// A failing call's `errno`; `Errno(libc::ENOENT)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Errno(pub i32);

/// The result of one POSIX call.
pub type CallResult<T> = Result<T, Errno>;

/// The `S_IFMT` classes the OSD code distinguishes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    /// `S_ISREG`.
    Regular,
    /// `S_ISDIR`.
    Directory,
    /// `S_ISLNK` (only `lstat` reports it; `stat` follows links).
    Link,
    /// `S_ISFIFO`.
    Fifo,
    /// `S_ISSOCK`.
    Socket,
    /// Character/block devices and anything else.
    Other,
}

/// The `struct stat` fields the OSD code reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeStatus {
    /// File type.
    pub node_type: NodeType,
    /// Permission bits including set-id bits (`st_mode & 07777`).
    pub permissions: u32,
    /// `st_size`.
    pub size: u64,
    /// `st_ctime` in seconds since the epoch.
    pub change_time: i64,
    /// `st_blksize`: the stdio buffer size derives from it.
    pub block_size: u64,
}

/// `access` mode bits (`F_OK` is the empty set).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AccessMode {
    /// `R_OK`.
    pub read: bool,
    /// `W_OK`.
    pub write: bool,
    /// `X_OK`.
    pub execute: bool,
}

/// The `open` flags the OSD code combines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenFlags {
    /// `O_RDONLY`, `O_WRONLY` or `O_RDWR`.
    pub access: super::super::OpenMode,
    /// `O_CREAT`.
    pub create: bool,
    /// `O_TRUNC`.
    pub truncate: bool,
    /// `O_APPEND`.
    pub append: bool,
}

/// `lseek` origins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeekOrigin {
    /// `SEEK_SET`.
    Start,
    /// `SEEK_CUR`.
    Current,
    /// `SEEK_END`.
    End,
}

/// The `fcntl` record locks of the SYSV branch (whole file, from offset 0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordLock {
    /// `F_SETLKW` with `F_RDLCK`.
    Read,
    /// `F_SETLKW` with `F_WRLCK`.
    Write,
    /// `F_SETLK` with `F_UNLCK`.
    Unlock,
}

/// The POSIX calls of the OSD file classes; paths are the bytes of a C string.
pub trait PosixFileCalls: Send + Sync {
    /// `access(path, mode)`.
    fn access(&self, path: &[u8], mode: AccessMode) -> CallResult<()>;
    /// `stat(path)` (follows links).
    fn stat(&self, path: &[u8]) -> CallResult<NodeStatus>;
    /// `fstat(fd)`.
    fn fstat(&self, fd: i32) -> CallResult<NodeStatus>;
    /// `open(path, flags, mode)`; returns the descriptor.
    fn open(&self, path: &[u8], flags: OpenFlags, mode: u32) -> CallResult<i32>;
    /// `close(fd)`.
    fn close(&self, fd: i32) -> CallResult<()>;
    /// `read(fd, buffer, len)`; returns the byte count.
    fn read(&self, fd: i32, buffer: &mut [u8]) -> CallResult<usize>;
    /// `write(fd, buffer, len)`; returns the byte count.
    fn write(&self, fd: i32, buffer: &[u8]) -> CallResult<usize>;
    /// `lseek(fd, offset, origin)`; returns the new offset.
    fn lseek(&self, fd: i32, offset: i64, origin: SeekOrigin) -> CallResult<i64>;
    /// `fcntl(fd, F_SETLKW/F_SETLK, lock)` over the whole file.
    fn record_lock(&self, fd: i32, lock: RecordLock) -> CallResult<()>;
    /// `rename(from, to)`.
    fn rename(&self, from: &[u8], to: &[u8]) -> CallResult<()>;
    /// `unlink(path)`.
    fn unlink(&self, path: &[u8]) -> CallResult<()>;
    /// `rmdir(path)`.
    fn rmdir(&self, path: &[u8]) -> CallResult<()>;
    /// `mkdir(path, mode)`.
    fn mkdir(&self, path: &[u8], mode: u32) -> CallResult<()>;
    /// `chmod(path, mode)`.
    fn chmod(&self, path: &[u8], mode: u32) -> CallResult<()>;
    /// `umask(mask)`; returns the previous mask.
    fn umask(&self, mask: u32) -> u32;
    /// `mkstemp(template)`; returns the descriptor and the generated name.
    fn mkstemp(&self, template: &[u8]) -> CallResult<(i32, Vec<u8>)>;
    /// `mkdtemp(template)`; returns the generated name.
    fn mkdtemp(&self, template: &[u8]) -> CallResult<Vec<u8>>;
    /// `opendir` + every `readdir` name, in the order `readdir` returns them.
    fn read_directory(&self, path: &[u8]) -> CallResult<Vec<Vec<u8>>>;
    /// Opens a lazy directory cursor; `calls.open_directory(b".")?` advances on demand.
    /// Existing injected backends retain their buffered behavior by default.
    fn open_directory(&self, path: &[u8]) -> CallResult<Box<dyn DirectoryEntries>> {
        self.read_directory(path).map(buffered_directory_entries)
    }
    /// `localtime(seconds)` (the shared `OSD_Process` broken-down time, millisecond 0), or
    /// `None` when the time cannot be broken down.
    fn local_time(&self, seconds: i64) -> Option<LocalTime>;
}

/// The process-wide native implementation; `native_calls()`.
pub fn native_calls() -> Arc<dyn PosixFileCalls> {
    Arc::new(super::NativePosixCalls)
}
