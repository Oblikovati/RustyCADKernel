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

//! `OSD_LocalFileSystem`: files on local or mounted file systems, through `std::filebuf`.
use super::{
    FileBuffer, FileSystem, IStreamBuffer, IosOpenMode, OStreamBuffer, SeekDir, SharedIStream,
    SharedOStream, SharedStreamBuf, StreamAccess, StreamBuf, assert_failure, lock_shared,
    open_file::extended_path,
};
use crate::{
    osd::CadPath,
    standard::Failure,
    tcollection::{AsciiString, ExtendedString},
};
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

/// `OSD_LocalFileSystem`; `LocalFileSystem::new().is_supported_path(&"a.brep".into())`.
#[derive(Debug, Default)]
pub struct LocalFileSystem;
impl LocalFileSystem {
    /// `OSD_LocalFileSystem()`; `LocalFileSystem::new()`.
    pub const fn new() -> Self {
        Self
    }
}

/// Tells whether the stream's buffer is an open `FileBuffer` (the native
/// `dynamic_cast<const std::filebuf*>(rdbuf())->is_open()`).
fn file_buffer_open(stream: &dyn StreamAccess) -> bool {
    let buffer: SharedStreamBuf = stream.rdbuf();
    let buffer = lock_shared(&buffer);
    let any: &dyn Any = &*buffer;
    any.downcast_ref::<FileBuffer>()
        .is_some_and(FileBuffer::is_open)
}

impl FileSystem for LocalFileSystem {
    /// Everything but a remote-protocol URL (`scheme://`) is local.
    fn is_supported_path(&self, url: &AsciiString) -> bool {
        !CadPath::is_remote_protocol_path(url.as_bytes())
    }
    /// Only an `IStreamBuffer` over an open `FileBuffer` counts.
    fn is_open_istream(&self, stream: &SharedIStream) -> bool {
        let stream = lock_shared(stream);
        let any: &dyn Any = &*stream;
        any.downcast_ref::<IStreamBuffer>()
            .is_some_and(|buffer| file_buffer_open(buffer))
    }
    /// Only an `OStreamBuffer` over an open `FileBuffer` counts.
    fn is_open_ostream(&self, stream: &SharedOStream) -> bool {
        let stream = lock_shared(stream);
        let any: &dyn Any = &*stream;
        any.downcast_ref::<OStreamBuffer>()
            .is_some_and(|buffer| file_buffer_open(buffer))
    }
    /// Opens a `FileBuffer`; with `size` and `in`, reports the file size and refuses an
    /// unreachable offset (the size is then 0). A negative offset raises
    /// `Standard_ProgramError`.
    fn open_stream_buffer(
        &self,
        url: &AsciiString,
        mode: IosOpenMode,
        offset: i64,
        size: Option<&mut i64>,
    ) -> Result<Option<SharedStreamBuf>, Failure> {
        if offset < 0 {
            return Err(assert_failure(
                "OSD_LocalFileSystem.cxx",
                "Incorrect negative stream position during stream buffer opening (theOffset >= 0)",
            ));
        }
        let mut buffer: FileBuffer = FileBuffer::new();
        // Native: OSD_OpenStream(filebuf, ExtendedString(url)), i.e. the UTF-8 URL.
        if !buffer.open(&extended_path(&ExtendedString::from_ascii(url)), mode) {
            return Ok(None);
        }
        Ok(position_buffer(&mut buffer, mode, offset, size)
            .then(|| Arc::new(Mutex::new(buffer)) as SharedStreamBuf))
    }
}

/// The native offset/size step after opening; false drops the buffer.
fn position_buffer(
    buffer: &mut FileBuffer,
    mode: IosOpenMode,
    offset: i64,
    size: Option<&mut i64>,
) -> bool {
    let reading: bool = mode.contains(IosOpenMode::IN);
    if let Some(size) = size.filter(|_| reading) {
        *size = buffer.pubseekoff(0, SeekDir::End, IosOpenMode::IN);
        if buffer.pubseekoff(offset, SeekDir::Beg, IosOpenMode::IN) < 0 {
            *size = 0;
            return false;
        }
        return true;
    }
    let which: IosOpenMode = if reading {
        IosOpenMode::IN
    } else {
        IosOpenMode::OUT
    };
    offset <= 0 || buffer.pubseekoff(offset, SeekDir::Beg, which) >= 0
}
