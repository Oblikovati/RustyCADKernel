//! Named fakes for the I/O BinTools touches: the console, an in-memory OSD file system
//! with a controllable flush, a refusing file system and a breaking progress indicator;
//! plus the in-memory stream helpers.
use rustycad_tkernel::message::{ProgressIndicatorHooks, ProgressScope};
use rustycad_tkernel::osd::{
    FileSystem, IosOpenMode, SeekDir, SharedIStream, SharedOStream, SharedStreamBuf, StreamBuf,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkgeombase::geom_tools::ConsoleOutput;
use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};

/// The bytes written to an in-memory stream.
pub fn written(write: impl FnOnce(&mut IoStream)) -> Vec<u8> {
    let mut stream = IoStream::from_bytes(Vec::new());
    write(&mut stream);
    stream_bytes(stream)
}

/// Every byte of an in-memory stream.
pub fn stream_bytes(stream: IoStream) -> Vec<u8> {
    let mut device = stream.into_device().unwrap();
    device.seek(SeekFrom::Start(0)).unwrap();
    let mut bytes = Vec::new();
    device.read_to_end(&mut bytes).unwrap();
    bytes
}

/// A console that records what the readers print instead of standard output.
#[derive(Default)]
pub struct RecordingConsole {
    text: Mutex<Vec<u8>>,
}
impl ConsoleOutput for RecordingConsole {
    fn write(&self, bytes: &[u8]) {
        self.text.lock().unwrap().extend_from_slice(bytes);
    }
}
impl RecordingConsole {
    pub fn text(&self) -> String {
        String::from_utf8(self.text.lock().unwrap().clone()).unwrap()
    }
}

/// A progress indicator that asks to stop after `limit` break checks.
pub struct BreakingIndicator {
    pub calls: Arc<AtomicI32>,
    pub limit: i32,
}
impl ProgressIndicatorHooks for BreakingIndicator {
    fn show(&self, _: &ProgressScope<'_>, _: f64, _: bool) {}
    fn user_break(&self) -> bool {
        self.calls.fetch_add(1, Ordering::SeqCst) + 1 > self.limit
    }
}

type Files = Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>;

/// One open file of [`MemoryFileSystem`]: reads from and writes to the shared map.
struct MemoryFile {
    files: Files,
    name: Vec<u8>,
    position: usize,
    sync_result: i32,
}
impl MemoryFile {
    fn with_bytes<R>(&self, action: impl FnOnce(&mut Vec<u8>) -> R) -> R {
        action(
            self.files
                .lock()
                .unwrap()
                .entry(self.name.clone())
                .or_default(),
        )
    }
}
impl StreamBuf for MemoryFile {
    fn sgetn(&mut self, out: &mut [u8]) -> usize {
        let position = self.position;
        let count = self.with_bytes(|bytes| {
            let count = out.len().min(bytes.len().saturating_sub(position));
            out[..count].copy_from_slice(&bytes[position..position + count]);
            count
        });
        self.position += count;
        count
    }
    fn sputn(&mut self, data: &[u8]) -> usize {
        let position = self.position;
        self.with_bytes(|bytes| {
            bytes.truncate(position);
            bytes.extend_from_slice(data);
        });
        self.position += data.len();
        data.len()
    }
    fn pubseekoff(&mut self, offset: i64, direction: SeekDir, _which: IosOpenMode) -> i64 {
        let length = self.with_bytes(|bytes| bytes.len()) as i64;
        let base = match direction {
            SeekDir::Beg => 0,
            SeekDir::Cur => self.position as i64,
            SeekDir::End => length,
        };
        let target = base + offset;
        if target < 0 || target > length {
            return -1;
        }
        self.position = target as usize;
        target
    }
    fn pubsync(&mut self) -> i32 {
        self.sync_result
    }
}

/// An OSD file system held in memory: opening for output truncates, opening a missing
/// file for input fails, and `pubsync` returns `sync_result` (-1 makes a flush fail).
#[derive(Clone, Default)]
pub struct MemoryFileSystem {
    files: Files,
    sync_result: i32,
}
impl MemoryFileSystem {
    /// A file system whose `pubsync` fails, so that every flush fails.
    pub fn failing_flush() -> Self {
        Self {
            sync_result: -1,
            ..Self::default()
        }
    }
    pub fn contents(&self, name: &str) -> Option<Vec<u8>> {
        self.files.lock().unwrap().get(name.as_bytes()).cloned()
    }
    pub fn insert(&self, name: &str, bytes: Vec<u8>) {
        self.files
            .lock()
            .unwrap()
            .insert(name.as_bytes().to_vec(), bytes);
    }
}
impl FileSystem for MemoryFileSystem {
    fn is_supported_path(&self, _url: &AsciiString) -> bool {
        true
    }
    fn is_open_istream(&self, _stream: &SharedIStream) -> bool {
        true
    }
    fn is_open_ostream(&self, _stream: &SharedOStream) -> bool {
        true
    }
    fn open_stream_buffer(
        &self,
        url: &AsciiString,
        mode: IosOpenMode,
        _offset: i64,
        _size: Option<&mut i64>,
    ) -> Result<Option<SharedStreamBuf>, Failure> {
        let name = url.as_bytes().to_vec();
        let mut files = self.files.lock().unwrap();
        if mode.contains(IosOpenMode::OUT) {
            files.insert(name.clone(), Vec::new());
        } else if !files.contains_key(&name) {
            return Ok(None);
        }
        let file = MemoryFile {
            files: Arc::clone(&self.files),
            name,
            position: 0,
            sync_result: self.sync_result,
        };
        Ok(Some(Arc::new(Mutex::new(file))))
    }
}

/// A file system that opens nothing (the native null stream).
pub struct RefusingFileSystem;
impl FileSystem for RefusingFileSystem {
    fn is_supported_path(&self, _url: &AsciiString) -> bool {
        true
    }
    fn is_open_istream(&self, _stream: &SharedIStream) -> bool {
        false
    }
    fn is_open_ostream(&self, _stream: &SharedOStream) -> bool {
        false
    }
    fn open_stream_buffer(
        &self,
        _url: &AsciiString,
        _mode: IosOpenMode,
        _offset: i64,
        _size: Option<&mut i64>,
    ) -> Result<Option<SharedStreamBuf>, Failure> {
        Ok(None)
    }
}
