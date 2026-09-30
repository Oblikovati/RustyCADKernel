//! `std::filebuf`, the buffer `OSD_LocalFileSystem` opens: the libstdc++ open-mode table
//! over `std::fs::File`.
//!
//! Reads are unbuffered (a logical position equals the file offset, as libstdc++ reports
//! it). Writes keep the libstdc++ put area: small writes wait in a pending buffer until a
//! seek, read, sync or close, and `pubseekoff(0, cur)` reports the file offset plus the
//! pending bytes. This is observable with `app`, where the pending bytes later land at the
//! end of the file although the reported position follows the last seek.
use super::{IosOpenMode, SeekDir, StreamBuf};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

/// The access a libstdc++ `fopen_mode` string grants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FopenAccess {
    read: bool,
    write: bool,
    append: bool,
    create: bool,
    truncate: bool,
}

/// libstdc++ `fopen_mode`: the only openmode combinations `std::filebuf::open` accepts
/// (binary ignored), as the fopen string each one means; `None` rejects the mode.
pub(crate) fn fopen_mode(mode: IosOpenMode) -> Option<&'static str> {
    let key: u32 = (mode
        & (IosOpenMode::IN | IosOpenMode::OUT | IosOpenMode::TRUNC | IosOpenMode::APP))
        .bits();
    const IN: u32 = 8;
    const OUT: u32 = 16;
    const TRUNC: u32 = 32;
    const APP: u32 = 1;
    match key {
        OUT => Some("w"),
        k if k == OUT | APP || k == APP => Some("a"),
        k if k == OUT | TRUNC => Some("w"),
        IN => Some("r"),
        k if k == IN | OUT => Some("r+"),
        k if k == IN | OUT | TRUNC => Some("w+"),
        k if k == IN | OUT | APP || k == IN | APP => Some("a+"),
        _ => None,
    }
}

/// glibc `fopen` puts a write-only append stream ("a", not "a+") at the end of the file
/// at once; `std::fs::File` starts at 0.
pub(crate) fn glibc_append_start(file: &mut File, append_only: bool) -> std::io::Result<()> {
    if append_only {
        file.seek(SeekFrom::End(0))?;
    }
    Ok(())
}

fn access_of(fopen: &str) -> FopenAccess {
    let update: bool = fopen.ends_with('+');
    let kind: u8 = fopen.as_bytes()[0];
    FopenAccess {
        read: kind == b'r' || update,
        write: kind != b'r' || update,
        append: kind == b'a',
        create: kind != b'r',
        truncate: kind == b'w',
    }
}

/// libstdc++ `BUFSIZ` put area less the overflow slot.
const PUT_AREA: usize = 8191;
/// libstdc++ `xsputn` writes a request of this size (or one filling the put area) directly.
const DIRECT_CHUNK: usize = 1024;

/// `std::filebuf`; `let mut buffer = FileBuffer::new(); buffer.open(path, IosOpenMode::IN)`.
#[derive(Debug, Default)]
pub struct FileBuffer {
    file: Option<File>,
    mode: IosOpenMode,
    pending: Vec<u8>,
}
impl FileBuffer {
    /// A closed buffer; `FileBuffer::new().is_open() == false`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `is_open`; `buffer.is_open()`.
    pub fn is_open(&self) -> bool {
        self.file.is_some()
    }
    /// `open`: fails when already open, for a mode outside the libstdc++ table, when the
    /// file cannot be opened, or when `ate` cannot seek to the end; `buffer.open(path, mode)`.
    pub fn open(&mut self, path: &Path, mode: IosOpenMode) -> bool {
        if self.is_open() {
            return false;
        }
        let Some(fopen) = fopen_mode(mode) else {
            return false;
        };
        let access: FopenAccess = access_of(fopen);
        let Ok(mut file) = open_options(access).open(path) else {
            return false;
        };
        if glibc_append_start(&mut file, access.append && !access.read).is_err() {
            return false;
        }
        self.file = Some(file);
        self.mode = mode;
        if mode.contains(IosOpenMode::ATE) && self.pubseekoff(0, SeekDir::End, mode) < 0 {
            self.file = None;
            return false;
        }
        true
    }
    /// `close`: writes the pending bytes; false when nothing was open or they could not
    /// be written; `buffer.close()`.
    pub fn close(&mut self) -> bool {
        let flushed: bool = self.flush_pending();
        self.file.take().is_some() && flushed
    }
    fn readable(&self) -> bool {
        self.mode.contains(IosOpenMode::IN)
    }
    fn writable(&self) -> bool {
        self.mode.intersects(IosOpenMode::OUT | IosOpenMode::APP)
    }
    /// Writes the put area to the file; false when some bytes could not be written.
    fn flush_pending(&mut self) -> bool {
        let pending: Vec<u8> = std::mem::take(&mut self.pending);
        match self.file.as_mut() {
            Some(file) => write_all_counted(file, &pending) == pending.len(),
            None => pending.is_empty(),
        }
    }
}
impl Drop for FileBuffer {
    fn drop(&mut self) {
        // The filebuf destructor closes, writing the put area.
        self.flush_pending();
    }
}

fn open_options(access: FopenAccess) -> OpenOptions {
    let mut options: OpenOptions = OpenOptions::new();
    options
        .read(access.read)
        .write(access.write && !access.append)
        .append(access.append)
        .create(access.create)
        .truncate(access.truncate);
    options
}

fn write_all_counted(file: &mut File, bytes: &[u8]) -> usize {
    let mut count: usize = 0;
    while count < bytes.len() {
        match file.write(&bytes[count..]) {
            Ok(0) | Err(_) => break,
            Ok(written) => count += written,
        }
    }
    count
}

fn read_counted(file: &mut File, out: &mut [u8]) -> usize {
    let mut count: usize = 0;
    while count < out.len() {
        match file.read(&mut out[count..]) {
            Ok(0) | Err(_) => break,
            Ok(read) => count += read,
        }
    }
    count
}

fn file_position(file: &mut File, target: SeekFrom) -> i64 {
    file.seek(target)
        .ok()
        .and_then(|position| i64::try_from(position).ok())
        .unwrap_or(-1)
}

impl StreamBuf for FileBuffer {
    fn sgetn(&mut self, out: &mut [u8]) -> usize {
        if !self.readable() || !self.flush_pending() {
            return 0;
        }
        self.file.as_mut().map_or(0, |file| read_counted(file, out))
    }
    fn sputn(&mut self, bytes: &[u8]) -> usize {
        if !self.writable() || self.file.is_none() {
            return 0;
        }
        let available: usize = PUT_AREA - self.pending.len();
        if bytes.len() < DIRECT_CHUNK.min(available) {
            self.pending.extend_from_slice(bytes);
            return bytes.len();
        }
        // Large requests go out together with the put area; the count covers only the
        // caller's bytes that reached the file.
        let pending: usize = self.pending.len();
        self.pending.extend_from_slice(bytes);
        let all: Vec<u8> = std::mem::take(&mut self.pending);
        let file: &mut File = self.file.as_mut().expect("checked open");
        write_all_counted(file, &all).saturating_sub(pending)
    }
    fn pubseekoff(&mut self, offset: i64, direction: SeekDir, _which: IosOpenMode) -> i64 {
        if self.file.is_none() {
            return -1;
        }
        if direction == SeekDir::Cur && offset == 0 {
            // No movement: libstdc++ adds the put area to the file offset without writing.
            let pending: i64 = self.pending.len() as i64;
            let file: &mut File = self.file.as_mut().expect("checked open");
            let position: i64 = file_position(file, SeekFrom::Current(0));
            return if position < 0 { -1 } else { position + pending };
        }
        if !self.flush_pending() {
            return -1;
        }
        let file: &mut File = self.file.as_mut().expect("checked open");
        match direction {
            // lseek refuses a negative absolute position (EINVAL).
            SeekDir::Beg if offset < 0 => -1,
            SeekDir::Beg => file_position(file, SeekFrom::Start(offset as u64)),
            SeekDir::Cur => file_position(file, SeekFrom::Current(offset)),
            SeekDir::End => file_position(file, SeekFrom::End(offset)),
        }
    }
    fn pubsync(&mut self) -> i32 {
        if self.flush_pending() { 0 } else { -1 }
    }
}
