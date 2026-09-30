//! `OSD_CachedFileSystem`: keeps the last input stream (and read buffer) of a linked
//! file system, so reopening the same URL at another offset reuses the open file.
use super::{
    FileSystem, FileSystemHandle, IosOpenMode, SharedIStream, SharedOStream, SharedStreamBuf,
    default_file_system, lock_shared,
};
use crate::{standard::Failure, tcollection::AsciiString};
use std::sync::{Mutex, RwLock};

/// `OSD_CachedFileSystem::OSD_CachedStream`: the cached URL, stream and buffer.
#[derive(Default)]
struct CachedStream {
    url: AsciiString,
    stream: Option<SharedIStream>,
    buffer: Option<SharedStreamBuf>,
}
impl CachedStream {
    /// Switches the cache to `url`, dropping the stream and buffer of another URL.
    fn select(&mut self, url: &AsciiString) {
        if self.url.as_bytes() != url.as_bytes() {
            self.url = url.clone();
            self.stream = None;
            self.buffer = None;
        }
    }
}

/// `OSD_CachedFileSystem`; one per working thread, as natively (the cache is shared state).
pub struct CachedFileSystem {
    linked: RwLock<FileSystemHandle>,
    cache: Mutex<CachedStream>,
}
impl Default for CachedFileSystem {
    fn default() -> Self {
        Self::new(None)
    }
}
impl CachedFileSystem {
    /// `OSD_CachedFileSystem(linked)`: a null handle links the default file system;
    /// `CachedFileSystem::new(None)`.
    pub fn new(linked: Option<FileSystemHandle>) -> Self {
        Self {
            linked: RwLock::new(linked.unwrap_or_else(default_file_system)),
            cache: Mutex::new(CachedStream::default()),
        }
    }
    /// `LinkedFileSystem`; `cached.linked_file_system()`.
    pub fn linked_file_system(&self) -> FileSystemHandle {
        self.linked
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    /// `SetLinkedFileSystem`: the cache is kept, as natively; `cached.set_linked_file_system(fs)`.
    pub fn set_linked_file_system(&self, linked: FileSystemHandle) {
        *self.linked.write().unwrap_or_else(|e| e.into_inner()) = linked;
    }
}

impl FileSystem for CachedFileSystem {
    /// Delegates to the linked file system.
    fn is_supported_path(&self, url: &AsciiString) -> bool {
        self.linked_file_system().is_supported_path(url)
    }
    /// Delegates to the linked file system.
    fn is_open_istream(&self, stream: &SharedIStream) -> bool {
        self.linked_file_system().is_open_istream(stream)
    }
    /// Delegates to the linked file system.
    fn is_open_ostream(&self, stream: &SharedOStream) -> bool {
        self.linked_file_system().is_open_ostream(stream)
    }
    /// Ignores `old`: offers the cached stream of the same URL to the linked file system
    /// and caches whatever it returns (a failed open clears the cache).
    fn open_istream(
        &self,
        url: &AsciiString,
        mode: IosOpenMode,
        offset: i64,
        _old: Option<&SharedIStream>,
    ) -> Result<Option<SharedIStream>, Failure> {
        let mut cache = lock_shared(&self.cache);
        cache.select(url);
        let cached: Option<SharedIStream> = cache.stream.clone();
        cache.stream =
            self.linked_file_system()
                .open_istream(url, mode, offset, cached.as_ref())?;
        Ok(cache.stream.clone())
    }
    /// Delegates to the linked file system without caching.
    fn open_ostream(
        &self,
        url: &AsciiString,
        mode: IosOpenMode,
    ) -> Result<Option<SharedOStream>, Failure> {
        self.linked_file_system().open_ostream(url, mode)
    }
    /// Output buffers pass through; input buffers are cached per URL.
    fn open_stream_buffer(
        &self,
        url: &AsciiString,
        mode: IosOpenMode,
        offset: i64,
        size: Option<&mut i64>,
    ) -> Result<Option<SharedStreamBuf>, Failure> {
        if mode.contains(IosOpenMode::OUT) {
            return self
                .linked_file_system()
                .open_stream_buffer(url, mode, offset, size);
        }
        let mut cache = lock_shared(&self.cache);
        cache.select(url);
        cache.buffer = self
            .linked_file_system()
            .open_stream_buffer(url, mode, offset, size)?;
        Ok(cache.buffer.clone())
    }
}
