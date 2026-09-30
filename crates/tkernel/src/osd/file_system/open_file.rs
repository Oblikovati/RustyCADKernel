//! `OSD_OpenFile.hxx`: opening files by Unicode name as a C stream (`fopen`), a file
//! descriptor, a `std::filebuf` or a file stream, and reading the change time.
//!
//! `FILE*` and descriptors become an owned `std::fs::File` (`None` for NULL / -1); names
//! given as `TCollection_ExtendedString` are converted to UTF-8 as on Linux (on Windows
//! std converts them to the UTF-16 path `_wfopen` would receive).
use super::{
    FileBuffer, IStream, IosOpenMode, OStream, SharedStreamBuf, StreamAccess, StreamIo,
    file_buffer::glibc_append_start,
};
use crate::tcollection::ExtendedString;
use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// The path a UTF-16 name designates; `extended_path(&ExtendedString::new("a.brep"))`.
pub(crate) fn extended_path(name: &ExtendedString) -> PathBuf {
    PathBuf::from(name.to_utf8())
}

/// glibc `fopen` mode parsing: 'r', 'w' or 'a' first, then up to six modifiers of which
/// '+' (update) and 'x' (exclusive create) matter; anything else is ignored. The flag
/// tells whether the stream is write-only append (glibc starts it at the end).
fn fopen_options(mode: &str) -> Option<(OpenOptions, bool)> {
    let bytes: &[u8] = mode.as_bytes();
    let (mut read, mut write, append, create) = match bytes.first()? {
        b'r' => (true, false, false, false),
        b'w' => (false, true, false, true),
        b'a' => (false, false, true, true),
        _ => return None,
    };
    let mut exclusive: bool = false;
    for &flag in bytes.iter().skip(1).take(6).take_while(|&&byte| byte != 0) {
        match flag {
            b'+' => (read, write) = (true, !append),
            b'x' => exclusive = true,
            _ => {}
        }
    }
    let mut options: OpenOptions = OpenOptions::new();
    options.read(read).write(write).append(append);
    // O_EXCL without O_CREAT has no effect for regular files.
    options
        .create(create && !exclusive)
        .create_new(create && exclusive);
    options.truncate(bytes[0] == b'w');
    Some((options, append && !read))
}

/// `OSD_OpenFile(const char*, const char*)`: `fopen` of a UTF-8 name; `None` when the
/// mode is invalid or the file cannot be opened; `open_file_utf8("a.txt", "rb")`.
pub fn open_file_utf8(name: &str, mode: &str) -> Option<File> {
    let name: &str = name.split('\0').next().unwrap_or_default();
    let mode: &str = mode.split('\0').next().unwrap_or_default();
    let (options, append_only) = fopen_options(mode)?;
    let mut file: File = options.open(name).ok()?;
    glibc_append_start(&mut file, append_only).ok()?;
    Some(file)
}

/// `OSD_OpenFile(const TCollection_ExtendedString&, const char*)`: `fopen` of a UTF-16
/// name; `open_file(&ExtendedString::new("a.txt"), "w")`.
pub fn open_file(name: &ExtendedString, mode: &str) -> Option<File> {
    open_file_utf8(&name.to_utf8(), mode)
}

/// `OSD_FileStatCTime`: `stat().st_ctime` of a UTF-8 name in seconds, 0 when `stat`
/// fails. Unix reports the status-change time; Windows `_wstat64` the creation time;
/// `file_stat_ctime("a.txt")`.
pub fn file_stat_ctime(name: &str) -> i64 {
    let name: &str = name.split('\0').next().unwrap_or_default();
    std::fs::metadata(name).map_or(0, |metadata| ctime_of(&metadata))
}

#[cfg(unix)]
fn ctime_of(metadata: &std::fs::Metadata) -> i64 {
    use std::os::unix::fs::MetadataExt;
    metadata.ctime()
}

#[cfg(not(unix))]
fn ctime_of(metadata: &std::fs::Metadata) -> i64 {
    metadata
        .created()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |duration| duration.as_secs() as i64)
}

/// `OSD_OpenFileDescriptor`: `open()` with the native openmode-to-flags table. Native
/// quirk kept: any mode with `out` is write-only, created and truncated even with `in`
/// or without `trunc` (O_RDONLY is 0); a mode without `out` is read-only. Created files
/// get mode 0600. `None` for -1; `open_file_descriptor(&name, IosOpenMode::IN)`.
///
/// Windows: `_wsopen_s` with the same flags; its text-mode newline translation (no
/// `binary`) is not reproduced, the Rust descriptor is always binary.
pub fn open_file_descriptor(name: &ExtendedString, mode: IosOpenMode) -> Option<File> {
    let path: PathBuf = extended_path(name);
    if !mode.contains(IosOpenMode::OUT) {
        return File::open(path).ok();
    }
    let append: bool = mode.contains(IosOpenMode::APP);
    open_truncated_for_write(&path, append)
}

#[cfg(unix)]
fn open_truncated_for_write(path: &std::path::Path, append: bool) -> Option<File> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut options: OpenOptions = OpenOptions::new();
    options.write(true).create(true).truncate(true).mode(0o600);
    if append {
        options.custom_flags(libc::O_APPEND);
    }
    options.open(path).ok()
}

#[cfg(not(unix))]
fn open_truncated_for_write(path: &std::path::Path, append: bool) -> Option<File> {
    let mut options: OpenOptions = OpenOptions::new();
    options.create(true);
    if append {
        // std refuses append with truncate; truncating after opening is equivalent.
        let file: File = options.append(true).open(path).ok()?;
        return file.set_len(0).ok().map(|()| file);
    }
    options.write(true).truncate(true).open(path).ok()
}

/// `OSD_OpenStream(std::filebuf&, const TCollection_ExtendedString&, openmode)`:
/// `filebuf::open` of the UTF-16 name; `open_stream(&mut buffer, &name, IosOpenMode::IN)`.
pub fn open_stream(buffer: &mut FileBuffer, name: &ExtendedString, mode: IosOpenMode) -> bool {
    buffer.open(&extended_path(name), mode)
}

/// `std::ifstream`, `std::ofstream` or `std::fstream`: a stream owning a `FileBuffer`;
/// `FileStream::input()`.
pub struct FileStream {
    file: Arc<Mutex<FileBuffer>>,
    implied: IosOpenMode,
    io: StreamIo,
}
impl FileStream {
    fn with_implied(implied: IosOpenMode) -> Self {
        let file: Arc<Mutex<FileBuffer>> = Arc::new(Mutex::new(FileBuffer::new()));
        let shared: SharedStreamBuf = Arc::clone(&file) as SharedStreamBuf;
        Self {
            file,
            implied,
            io: StreamIo::new(shared),
        }
    }
    /// `std::ifstream`: `open` adds `in`; `FileStream::input()`.
    pub fn input() -> Self {
        Self::with_implied(IosOpenMode::IN)
    }
    /// `std::ofstream`: `open` adds `out`; `FileStream::output()`.
    pub fn output() -> Self {
        Self::with_implied(IosOpenMode::OUT)
    }
    /// `std::fstream`: `open` adds nothing; `FileStream::input_output()`.
    pub fn input_output() -> Self {
        Self::with_implied(IosOpenMode::NONE)
    }
    /// `open`: failbit on failure, `clear()` on success; `stream.open(path, mode)`.
    pub fn open(&mut self, path: &std::path::Path, mode: IosOpenMode) {
        let opened: bool = super::lock_shared(&self.file).open(path, mode | self.implied);
        if opened {
            self.clear();
        } else {
            self.io.set_fail();
        }
    }
    /// `is_open`; `stream.is_open()`.
    pub fn is_open(&self) -> bool {
        super::lock_shared(&self.file).is_open()
    }
    /// `close`: failbit when nothing was open; `stream.close()`.
    pub fn close(&mut self) {
        if !super::lock_shared(&self.file).close() {
            self.io.set_fail();
        }
    }
}
impl StreamAccess for FileStream {
    fn stream_io(&self) -> &StreamIo {
        &self.io
    }
    fn stream_io_mut(&mut self) -> &mut StreamIo {
        &mut self.io
    }
}
impl IStream for FileStream {}
impl OStream for FileStream {}

/// `OSD_OpenStream(T&, const TCollection_ExtendedString&, openmode)`: `stream.open` of
/// the UTF-16 name; `open_file_stream(&mut stream, &name, IosOpenMode::IN)`.
pub fn open_file_stream(stream: &mut FileStream, name: &ExtendedString, mode: IosOpenMode) {
    stream.open(&extended_path(name), mode);
}

/// `OSD_OpenStream(T&, const char*, openmode)`: `stream.open` of the UTF-8 name (Windows
/// converts it to UTF-16 first); `open_file_stream_utf8(&mut stream, "a.txt", mode)`.
pub fn open_file_stream_utf8(stream: &mut FileStream, name: &str, mode: IosOpenMode) {
    let name: &str = name.split('\0').next().unwrap_or_default();
    stream.open(std::path::Path::new(name), mode);
}
