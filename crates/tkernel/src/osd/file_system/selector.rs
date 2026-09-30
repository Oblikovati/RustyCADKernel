//! `OSD_FileSystemSelector`: tries each registered file system in order.
use super::{
    FileSystem, FileSystemHandle, IStreamBuffer, IosOpenMode, OStreamBuffer, SharedIStream,
    SharedOStream, SharedStreamBuf, lock_shared, same_object,
};
use crate::{standard::Failure, tcollection::AsciiString};
use std::{any::Any, sync::RwLock};

/// `OSD_FileSystemSelector`; `selector.add_protocol(Arc::new(LocalFileSystem::new()), false)`.
#[derive(Default)]
pub struct FileSystemSelector {
    protocols: RwLock<Vec<FileSystemHandle>>,
}
impl FileSystemSelector {
    /// `OSD_FileSystemSelector()`: no protocol; `FileSystemSelector::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `AddProtocol`: removes an earlier registration of the same object, then puts it
    /// first when `preferred`, else last; `selector.add_protocol(fs, true)`.
    pub fn add_protocol(&self, file_system: FileSystemHandle, preferred: bool) {
        let mut protocols = self.protocols.write().unwrap_or_else(|e| e.into_inner());
        remove_first(&mut protocols, &file_system);
        if preferred {
            protocols.insert(0, file_system);
        } else {
            protocols.push(file_system);
        }
    }
    /// `RemoveProtocol`: drops the first registration of the object; `selector.remove_protocol(&fs)`.
    pub fn remove_protocol(&self, file_system: &FileSystemHandle) {
        let mut protocols = self.protocols.write().unwrap_or_else(|e| e.into_inner());
        remove_first(&mut protocols, file_system);
    }
    /// The registered protocols in trial order; `selector.protocols().len()`.
    pub fn protocols(&self) -> Vec<FileSystemHandle> {
        self.protocols
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

/// The protocols supporting `url`, tested lazily in trial order over a snapshot (so a
/// protocol may call back into the selector without deadlock).
fn supporting(
    protocols: Vec<FileSystemHandle>,
    url: &AsciiString,
) -> impl Iterator<Item = FileSystemHandle> + '_ {
    protocols
        .into_iter()
        .filter(move |file_system| file_system.is_supported_path(url))
}

fn remove_first(protocols: &mut Vec<FileSystemHandle>, file_system: &FileSystemHandle) {
    if let Some(index) = protocols
        .iter()
        .position(|item| same_object(item, file_system))
    {
        protocols.remove(index);
    }
}

/// The URL of an `OSD_IStreamBuffer`/`OSD_OStreamBuffer` stream, or `None` for any other
/// stream (the native `dynamic_pointer_cast`).
fn stream_url<T: Any>(stream: &dyn Any, url_of: fn(&T) -> &str) -> Option<AsciiString> {
    stream
        .downcast_ref::<T>()
        .map(|buffer| AsciiString::from(url_of(buffer)))
}

impl FileSystem for FileSystemSelector {
    /// True when any protocol supports the URL.
    fn is_supported_path(&self, url: &AsciiString) -> bool {
        self.protocols()
            .iter()
            .any(|file_system| file_system.is_supported_path(url))
    }
    /// An `IStreamBuffer` open for a protocol supporting its URL.
    fn is_open_istream(&self, stream: &SharedIStream) -> bool {
        let url: Option<AsciiString> = {
            let guard = lock_shared(stream);
            stream_url::<IStreamBuffer>(&*guard, IStreamBuffer::url)
        };
        url.is_some_and(|url| {
            self.protocols().iter().any(|file_system| {
                file_system.is_supported_path(&url) && file_system.is_open_istream(stream)
            })
        })
    }
    /// An `OStreamBuffer` open for a protocol supporting its URL.
    fn is_open_ostream(&self, stream: &SharedOStream) -> bool {
        let url: Option<AsciiString> = {
            let guard = lock_shared(stream);
            stream_url::<OStreamBuffer>(&*guard, OStreamBuffer::url)
        };
        url.is_some_and(|url| {
            self.protocols().iter().any(|file_system| {
                file_system.is_supported_path(&url) && file_system.is_open_ostream(stream)
            })
        })
    }
    /// The first stream a supporting protocol opens.
    fn open_istream(
        &self,
        url: &AsciiString,
        mode: IosOpenMode,
        offset: i64,
        old: Option<&SharedIStream>,
    ) -> Result<Option<SharedIStream>, Failure> {
        for file_system in supporting(self.protocols(), url) {
            if let Some(stream) = file_system.open_istream(url, mode, offset, old)? {
                return Ok(Some(stream));
            }
        }
        Ok(None)
    }
    /// The first stream a supporting protocol opens.
    fn open_ostream(
        &self,
        url: &AsciiString,
        mode: IosOpenMode,
    ) -> Result<Option<SharedOStream>, Failure> {
        for file_system in supporting(self.protocols(), url) {
            if let Some(stream) = file_system.open_ostream(url, mode)? {
                return Ok(Some(stream));
            }
        }
        Ok(None)
    }
    /// The first buffer a supporting protocol opens; `size` is passed to each attempt.
    fn open_stream_buffer(
        &self,
        url: &AsciiString,
        mode: IosOpenMode,
        offset: i64,
        mut size: Option<&mut i64>,
    ) -> Result<Option<SharedStreamBuf>, Failure> {
        for file_system in supporting(self.protocols(), url) {
            let attempt: Option<&mut i64> = size.as_deref_mut();
            if let Some(buffer) = file_system.open_stream_buffer(url, mode, offset, attempt)? {
                return Ok(Some(buffer));
            }
        }
        Ok(None)
    }
}
