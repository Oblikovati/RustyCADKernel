// Copyright (c) 2021 OPEN CASCADE SAS
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

//! Stream file systems: `OSD_FileSystem`, `OSD_LocalFileSystem`, `OSD_CachedFileSystem`,
//! `OSD_FileSystemSelector`, `OSD_StreamBuffer` and the `OSD_OpenFile` helpers.
//!
//! `occ::handle<OSD_FileSystem>` becomes [`FileSystemHandle`], an `Arc` of the trait
//! object: the crate's `Handle<T>` projects to one concrete type, while file systems are
//! an open interface that applications implement. Handle equality stays pointer equality.
//! C++ file-system members that mutate state (`OpenIStream`, `AddProtocol`, ...) take
//! `&self` and lock internally, because the default file system is shared process-wide.
use crate::{standard::Failure, tcollection::AsciiString};
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

mod cached;
mod file_buffer;
mod ios;
mod local;
mod open_file;
mod selector;
mod stream_buffer;
mod stream_io;
pub use cached::CachedFileSystem;
pub use file_buffer::FileBuffer;
pub(crate) use ios::lock_shared;
pub use ios::{IosOpenMode, SeekDir, SharedStreamBuf, StreamBuf, share_stream_buf};
pub use local::LocalFileSystem;
pub use open_file::{
    FileStream, file_stat_ctime, open_file, open_file_descriptor, open_file_stream,
    open_file_stream_utf8, open_file_utf8, open_stream,
};
pub use selector::FileSystemSelector;
pub use stream_buffer::{
    IStreamBuffer, InputDirection, InputOutputDirection, IoStreamBuffer, OStreamBuffer,
    OutputDirection, StreamBuffer,
};
pub(crate) use stream_io::same_object;
pub use stream_io::{
    IStream, OStream, SharedIStream, SharedOStream, StreamAccess, StreamIo, StreamState,
};

/// `occ::handle<OSD_FileSystem>`.
pub type FileSystemHandle = Arc<dyn FileSystem>;

/// `OSD_FileSystem`: a provider of streams for the URLs of one protocol; implement it
/// for a custom protocol and register it with [`add_default_protocol`].
pub trait FileSystem: Send + Sync + Any {
    /// `IsSupportedPath`: tells whether the URL belongs to this protocol;
    /// `fs.is_supported_path(&"file.brep".into())`.
    fn is_supported_path(&self, url: &AsciiString) -> bool;
    /// `IsOpenIStream`: tells whether the stream is open for reading; `fs.is_open_istream(&s)`.
    fn is_open_istream(&self, stream: &SharedIStream) -> bool;
    /// `IsOpenOStream`: tells whether the stream is open for writing; `fs.is_open_ostream(&s)`.
    fn is_open_ostream(&self, stream: &SharedOStream) -> bool;
    /// `OpenIStream`: a new input stream at `offset` (`in` is added to `mode`), or `old`
    /// reused when it is an open `IStreamBuffer` of the same URL (offset -1 keeps its
    /// position). `Ok(None)` when opening failed; an offset below -1 raises
    /// `Standard_ProgramError`. `fs.open_istream(&url, IosOpenMode::BINARY, 0, None)?`.
    fn open_istream(
        &self,
        url: &AsciiString,
        mode: IosOpenMode,
        offset: i64,
        old: Option<&SharedIStream>,
    ) -> Result<Option<SharedIStream>, Failure> {
        default_open_istream(self, url, mode, offset, old)
    }
    /// `OpenOStream`: a new output stream (`out` is added to `mode`) or `None`;
    /// `fs.open_ostream(&url, IosOpenMode::BINARY)?`.
    fn open_ostream(
        &self,
        url: &AsciiString,
        mode: IosOpenMode,
    ) -> Result<Option<SharedOStream>, Failure> {
        default_open_ostream(self, url, mode)
    }
    /// `OpenStreamBuffer`: a buffer positioned at `offset`, or `None`; when opened for
    /// reading, `size` receives the buffer length; `fs.open_stream_buffer(&url, IosOpenMode::IN, 0, None)?`.
    fn open_stream_buffer(
        &self,
        url: &AsciiString,
        mode: IosOpenMode,
        offset: i64,
        size: Option<&mut i64>,
    ) -> Result<Option<SharedStreamBuf>, Failure>;
}

/// The base-class `OSD_FileSystem::OpenIStream`, callable from overriding implementations
/// the way C++ calls the parent method; `default_open_istream(fs, &url, mode, 0, None)`.
pub fn default_open_istream<F: FileSystem + ?Sized>(
    file_system: &F,
    url: &AsciiString,
    mode: IosOpenMode,
    offset: i64,
    old: Option<&SharedIStream>,
) -> Result<Option<SharedIStream>, Failure> {
    if offset < -1 {
        return Err(assert_failure(
            "OSD_FileSystem.cxx",
            "Incorrect negative stream position during stream opening (theOffset >= -1)",
        ));
    }
    if let Some(reused) = old.filter(|old| reusable_stream(file_system, url, old)) {
        let mut stream = lock_shared(reused);
        if !stream.good() {
            // Reset flags without re-opening.
            stream.clear();
        }
        if offset >= 0 {
            stream.seekg(offset, SeekDir::Beg);
        }
        return Ok(Some(Arc::clone(reused)));
    }
    let Some(buffer) = file_system.open_stream_buffer(url, mode | IosOpenMode::IN, 0, None)? else {
        return Ok(None);
    };
    let mut stream: IStreamBuffer = IStreamBuffer::new(url_text(url), buffer);
    if offset > 0 {
        stream.seekg(offset, SeekDir::Beg);
    }
    Ok(Some(Arc::new(Mutex::new(stream))))
}

/// An old stream is reused only when it is an `OSD_IStreamBuffer` of the same URL that
/// the file system still reports open.
fn reusable_stream<F: FileSystem + ?Sized>(
    file_system: &F,
    url: &AsciiString,
    old: &SharedIStream,
) -> bool {
    let same_url: bool = {
        let stream = lock_shared(old);
        let any: &dyn Any = &*stream;
        any.downcast_ref::<IStreamBuffer>()
            .is_some_and(|buffer| buffer.url().as_bytes() == url.as_bytes())
    };
    same_url && file_system.is_open_istream(old)
}

/// The base-class `OSD_FileSystem::OpenOStream`; `default_open_ostream(fs, &url, mode)`.
pub fn default_open_ostream<F: FileSystem + ?Sized>(
    file_system: &F,
    url: &AsciiString,
    mode: IosOpenMode,
) -> Result<Option<SharedOStream>, Failure> {
    let Some(buffer) = file_system.open_stream_buffer(url, mode | IosOpenMode::OUT, 0, None)?
    else {
        return Ok(None);
    };
    Ok(Some(Arc::new(Mutex::new(OStreamBuffer::new(
        url_text(url),
        buffer,
    )))))
}

/// The URL as the `std::string` an `OSD_StreamBuffer` stores (bytes up to the first NUL).
fn url_text(url: &AsciiString) -> String {
    String::from_utf8_lossy(url.as_bytes()).into_owned()
}

/// `Standard_ASSERT_RAISE`: `Standard_ProgramError` with the native message layout; the
/// file is the source basename (natively the absolute build path).
pub(crate) fn assert_failure(file: &str, description: &str) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        &format!("*** ERROR: ASSERT in file '{file}': \n{description}"),
        "",
    )
}

fn default_selector() -> &'static Arc<FileSystemSelector> {
    static DEFAULT: std::sync::OnceLock<Arc<FileSystemSelector>> = std::sync::OnceLock::new();
    DEFAULT.get_or_init(|| {
        let selector: FileSystemSelector = FileSystemSelector::new();
        selector.add_protocol(Arc::new(LocalFileSystem::new()), false);
        Arc::new(selector)
    })
}

/// `OSD_FileSystem::DefaultFileSystem`: the process-wide selector, created on first use
/// with one `LocalFileSystem`; `default_file_system().is_supported_path(&url)`.
pub fn default_file_system() -> FileSystemHandle {
    Arc::clone(default_selector()) as FileSystemHandle
}

/// `OSD_FileSystem::AddDefaultProtocol`: registers in the default selector, first when
/// `preferred`; `add_default_protocol(Arc::new(MyFs), true)`.
pub fn add_default_protocol(file_system: FileSystemHandle, preferred: bool) {
    default_selector().add_protocol(file_system, preferred);
}

/// `OSD_FileSystem::RemoveDefaultProtocol`; `remove_default_protocol(&my_fs)`.
pub fn remove_default_protocol(file_system: &FileSystemHandle) {
    default_selector().remove_protocol(file_system);
}
