//! `OSD_File` of the non-`_WIN32` build: construction, opening, closing, kind, size,
//! access checks, locks and temporary files. Reading and writing are in `file_io`.
use super::{
    FileNode,
    calls::{AccessMode, NodeType, OpenFlags, PosixFileCalls, RecordLock},
    native_calls,
    stdio::StdioStream,
};
use crate::{
    osd::{CadPath, KindFile, LockType, OpenMode, Protection, WhoAmI},
    standard::Failure,
};
use std::sync::Arc;

pub(super) const IAM: i32 = WhoAmI::File as i32;

/// `OSD_File`; `let mut file = File::new("part.step"); file.open(OpenMode::ReadOnly, &Protection::new())?`.
///
/// Dereferences to its [`FileNode`] for the inherited members (`exists`, `remove`, ...).
#[derive(Debug)]
pub struct File {
    pub(super) node: FileNode,
    pub(super) channel: i32,
    pub(super) stream: Option<StdioStream>,
    /// `myIO == EOF`: set by a short read or write, cleared only by `Close`.
    pub(super) at_end: bool,
    lock: LockType,
    pub(super) mode: OpenMode,
    imperative: bool,
}

impl std::ops::Deref for File {
    type Target = FileNode;
    fn deref(&self) -> &FileNode {
        &self.node
    }
}
impl std::ops::DerefMut for File {
    fn deref_mut(&mut self) -> &mut FileNode {
        &mut self.node
    }
}

pub(super) fn program_error(message: &str) -> Failure {
    Failure::with_type("Standard_ProgramError", message, "")
}

impl Default for File {
    fn default() -> Self {
        Self::empty()
    }
}

impl File {
    /// `OSD_File()`: an empty path over the native calls; `File::empty()`.
    pub fn empty() -> Self {
        Self::with_calls(CadPath::default(), native_calls())
    }
    /// `OSD_File(OSD_Path(name))` over the native calls; `File::new("part.step")`.
    pub fn new(name: impl AsRef<[u8]>) -> Self {
        Self::with_path(CadPath::new(name))
    }
    /// `OSD_File(path)` over the native calls; `File::with_path(CadPath::new("a.txt"))`.
    pub fn with_path(path: CadPath) -> Self {
        Self::with_calls(path, native_calls())
    }
    /// `OSD_File(path)` over injected calls; `File::with_calls(path, Arc::new(fake))`.
    pub fn with_calls(path: CadPath, calls: Arc<dyn PosixFileCalls>) -> Self {
        Self {
            node: FileNode::with_calls(path, calls),
            channel: -1,
            stream: None,
            at_end: false,
            lock: LockType::NoLock,
            mode: OpenMode::ReadWrite,
            imperative: false,
        }
    }
    /// `Build`: creates or truncates the file (`O_CREAT | O_TRUNC`) with the protection's
    /// mode; `file.build(OpenMode::ReadWrite, &Protection::new())?`.
    ///
    /// Raises `Standard_ProgramError` for a directory, an open file or an empty name.
    pub fn build(&mut self, mode: OpenMode, protection: &Protection) -> Result<(), Failure> {
        self.check_openable("Build")?;
        let flags: OpenFlags = OpenFlags {
            access: mode,
            create: true,
            truncate: true,
            append: false,
        };
        self.open_channel(mode, flags, protection, stream_reads(mode));
        Ok(())
    }
    /// `Open`: opens an existing file; `file.open(OpenMode::ReadOnly, &Protection::new())?`.
    ///
    /// Raises `Standard_ProgramError` for a directory, an open file or an empty name.
    pub fn open(&mut self, mode: OpenMode, protection: &Protection) -> Result<(), Failure> {
        self.check_openable("Open")?;
        let flags: OpenFlags = OpenFlags {
            access: mode,
            create: false,
            truncate: false,
            append: false,
        };
        self.open_channel(mode, flags, protection, stream_reads(mode));
        Ok(())
    }
    /// `Append`: opens with `O_APPEND`, creating the file when it does not exist;
    /// `file.append(OpenMode::WriteOnly, &Protection::new())?`.
    pub fn append(&mut self, mode: OpenMode, protection: &Protection) -> Result<(), Failure> {
        self.check_openable("Append")?;
        let create: bool = !self.exists();
        let flags: OpenFlags = OpenFlags {
            access: mode,
            create,
            truncate: false,
            append: true,
        };
        self.open_channel(mode, flags, protection, stream_reads(mode));
        Ok(())
    }
    fn check_openable(&self, member: &str) -> Result<(), Failure> {
        if self.kind_of_file() == KindFile::Directory {
            return Err(program_error(&format!(
                "OSD_File::{member}(): it is a directory"
            )));
        }
        if self.is_open() {
            let message: String =
                format!("OSD_File::{member}(): incorrect call - file already opened");
            return Err(program_error(&message));
        }
        if self.node.path.name().is_empty() {
            return Err(program_error(&format!(
                "OSD_File::{member}(): no name was given"
            )));
        }
        Ok(())
    }
    fn open_channel(
        &mut self,
        mode: OpenMode,
        flags: OpenFlags,
        protection: &Protection,
        reads: bool,
    ) {
        self.mode = mode;
        let name: Vec<u8> = self.node.system_name();
        match self.node.calls.open(&name, flags, protection.internal()) {
            Ok(fd) => {
                self.channel = fd;
                self.stream = Some(StdioStream::attach(fd, reads));
            }
            Err(errno) => {
                self.channel = -1;
                self.node.record(errno, IAM, "Open");
            }
        }
    }
    /// `Close`: closes the descriptor and its stream and clears the end-of-file state;
    /// `file.close()?`. Raises when not open, or re-raises a pending error (`Perror`).
    pub fn close(&mut self) -> Result<(), Failure> {
        self.check_open("Close")?;
        self.raise_pending()?;
        if let Err(errno) = self.node.calls.close(self.channel) {
            self.node.record(errno, IAM, "Close");
        }
        // The native fclose closes the same descriptor again and ignores its EBADF.
        self.channel = -1;
        self.stream = None;
        self.at_end = false;
        Ok(())
    }
    pub(super) fn check_open(&self, member: &str) -> Result<(), Failure> {
        if self.is_open() {
            return Ok(());
        }
        Err(program_error(&format!(
            "OSD_File::{member}(): file is not open"
        )))
    }
    /// `if (Failed()) Perror();`: a pending Unix error raises `OSD_OSDError`.
    pub(super) fn raise_pending(&mut self) -> Result<(), Failure> {
        if self.node.error.failed() {
            self.node.error.perror()?;
        }
        Ok(())
    }
    /// `IsAtEnd`: a short read or write happened since opening; `file.is_at_end()?`.
    pub fn is_at_end(&self) -> Result<bool, Failure> {
        self.check_open("IsAtEnd")?;
        Ok(self.at_end)
    }
    /// `KindOfFile`: `stat` follows links, so a link reports its target; `file.kind_of_file()`.
    pub fn kind_of_file(&self) -> KindFile {
        match self
            .node
            .calls
            .stat(&self.node.system_name())
            .map(|status| status.node_type)
        {
            Ok(NodeType::Directory) => KindFile::Directory,
            Ok(NodeType::Regular) => KindFile::File,
            Ok(NodeType::Link) => KindFile::Link,
            Ok(NodeType::Socket) => KindFile::Socket,
            _ => KindFile::Unknown,
        }
    }
    /// `BuildTemporary`: closes an open file, then `mkstemp("/tmp/CSFXXXXXX")` opened for
    /// reading and writing; `file.build_temporary()`.
    pub fn build_temporary(&mut self) -> Result<(), Failure> {
        if self.is_open() {
            self.close()?;
        }
        let template: &[u8] = b"/tmp/CSFXXXXXX";
        let (fd, name) = self
            .node
            .calls
            .mkstemp(template)
            .unwrap_or((-1, template.to_vec()));
        self.channel = fd;
        self.node.set_path(CadPath::new(name));
        self.stream = (fd >= 0).then(|| StdioStream::attach(fd, true));
        self.mode = OpenMode::ReadWrite;
        Ok(())
    }
    /// `SetLock`: an `fcntl` record lock (blocking); an exclusive lock also sets the
    /// set-group-id bit, even when the lock failed; `file.set_lock(LockType::WriteLock)?`.
    pub fn set_lock(&mut self, lock: LockType) -> Result<(), Failure> {
        self.check_open("SetLock")?;
        let request: RecordLock = match lock {
            LockType::NoLock => return Ok(()),
            LockType::ReadLock => RecordLock::Read,
            LockType::WriteLock | LockType::ExclusiveLock => RecordLock::Write,
        };
        match self.node.calls.record_lock(self.channel, request) {
            Ok(()) => self.lock = lock,
            Err(errno) => self.node.record(errno, IAM, "SetLock"),
        }
        if lock == LockType::ExclusiveLock {
            self.toggle_set_group_id(true);
            self.imperative = true;
        }
        Ok(())
    }
    /// `UnLock`: clears the set-group-id bit of an exclusive lock, then unlocks the whole
    /// file; `file.unlock()?`.
    ///
    /// Deliberate deviation: the native `flock` passed to `F_SETLK` has uninitialized
    /// `l_whence`/`l_start`/`l_len`, so the native result depends on stack contents (on the
    /// verified Linux profile it fails with EINVAL and keeps the lock); here the unlock is
    /// well defined.
    pub fn unlock(&mut self) -> Result<(), Failure> {
        self.check_open("UnLock")?;
        if self.imperative {
            self.toggle_set_group_id(false);
            self.imperative = false;
        }
        match self
            .node
            .calls
            .record_lock(self.channel, RecordLock::Unlock)
        {
            Ok(()) => self.lock = LockType::NoLock,
            Err(errno) => self.node.record(errno, IAM, "UnSetLock"),
        }
        Ok(())
    }
    /// `chmod(name, st_mode | S_ISGID)` (or without it), ignoring both calls' failures.
    fn toggle_set_group_id(&self, set: bool) {
        const SET_GROUP_ID: u32 = 0o2000;
        let permissions: u32 = self
            .node
            .calls
            .fstat(self.channel)
            .map_or(0, |status| status.permissions);
        let mode: u32 = if set {
            permissions | SET_GROUP_ID
        } else {
            permissions & !SET_GROUP_ID
        };
        let _ = self.node.calls.chmod(&self.node.system_name(), mode);
    }
    /// `GetLock`: the last lock set successfully; `file.lock()`.
    pub const fn lock(&self) -> LockType {
        self.lock
    }
    /// `IsLocked`: a lock other than `NoLock` is held (the Unix header branch; the Win32
    /// build returns its `LockFileEx` flag instead); `file.is_locked()`.
    pub fn is_locked(&self) -> bool {
        self.lock != LockType::NoLock
    }
    /// `Size`: `st_size` of the named file (0 and an error when `stat` fails);
    /// `file.size()?`. Raises `Standard_ProgramError` for an empty name.
    pub fn size(&mut self) -> Result<usize, Failure> {
        if self.node.path.name().is_empty() {
            return Err(program_error("OSD_File::Size(): empty file name"));
        }
        match self.node.calls.stat(&self.node.system_name()) {
            Ok(status) => Ok(status.size as usize),
            Err(errno) => {
                self.node.record(errno, IAM, "Size");
                Ok(0)
            }
        }
    }
    /// `IsOpen`: the descriptor is valid; `file.is_open()`.
    pub const fn is_open(&self) -> bool {
        self.channel != -1
    }
    /// `IsReadable`: `access(name, F_OK | R_OK)`; `file.is_readable()`.
    pub fn is_readable(&self) -> bool {
        self.accessible(AccessMode {
            read: true,
            ..AccessMode::default()
        })
    }
    /// `IsWriteable`: `access(name, F_OK | R_OK | W_OK)`; `file.is_writeable()`.
    pub fn is_writeable(&self) -> bool {
        self.accessible(AccessMode {
            read: true,
            write: true,
            execute: false,
        })
    }
    /// `IsExecutable`: `access(name, F_OK | X_OK)`; `file.is_executable()`.
    pub fn is_executable(&self) -> bool {
        self.accessible(AccessMode {
            execute: true,
            ..AccessMode::default()
        })
    }
    fn accessible(&self, mode: AccessMode) -> bool {
        self.node
            .calls
            .access(&self.node.system_name(), mode)
            .is_ok()
    }
    /// `Edit`: natively prints "Function OSD_File::Edit() not yet implemented." and
    /// returns false; `file.edit()`.
    pub fn edit(&self) -> bool {
        println!("Function OSD_File::Edit() not yet implemented.");
        false
    }
}

/// Whether `fdopen` reads: "r", "w+" and "a+" do, "w" and "a" do not.
const fn stream_reads(mode: OpenMode) -> bool {
    !matches!(mode, OpenMode::WriteOnly)
}

impl Drop for File {
    /// `~OSD_File`: unlocks and closes an open file. Natively `Close` may raise from the
    /// destructor (terminating the process) when an error is pending; here the pending
    /// error is not re-raised.
    fn drop(&mut self) {
        if !self.is_open() {
            return;
        }
        if self.is_locked() {
            let _ = self.unlock();
        }
        let _ = self.node.calls.close(self.channel);
        self.channel = -1;
    }
}
