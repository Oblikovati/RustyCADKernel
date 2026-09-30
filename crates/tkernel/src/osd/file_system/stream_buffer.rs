//! `OSD_StreamBuffer<T>`: a stream owning a shared `std::streambuf` and remembering the
//! URL it was opened from. The C++ template parameter (istream, ostream, iostream)
//! becomes a direction marker; like the distinct C++ instantiations, an
//! `IoStreamBuffer` is not an `IStreamBuffer` for the file systems' checks.
use super::{IStream, OStream, SharedStreamBuf, StreamAccess, StreamIo};
use std::marker::PhantomData;

/// Marker for `OSD_StreamBuffer<std::istream>`.
#[derive(Debug)]
pub struct InputDirection;
/// Marker for `OSD_StreamBuffer<std::ostream>`.
#[derive(Debug)]
pub struct OutputDirection;
/// Marker for `OSD_StreamBuffer<std::iostream>`.
#[derive(Debug)]
pub struct InputOutputDirection;

/// `OSD_StreamBuffer<T>`; `IStreamBuffer::new("file.brep", buffer).url()`.
pub struct StreamBuffer<D> {
    url: String,
    io: StreamIo,
    direction: PhantomData<fn() -> D>,
}
/// `OSD_IStreamBuffer`.
pub type IStreamBuffer = StreamBuffer<InputDirection>;
/// `OSD_OStreamBuffer`.
pub type OStreamBuffer = StreamBuffer<OutputDirection>;
/// `OSD_IOStreamBuffer`.
pub type IoStreamBuffer = StreamBuffer<InputOutputDirection>;

impl<D> StreamBuffer<D> {
    /// Wraps the shared buffer and keeps the URL; `IStreamBuffer::new("a.step", buffer)`.
    pub fn new(url: impl Into<String>, buffer: SharedStreamBuf) -> Self {
        Self {
            url: url.into(),
            io: StreamIo::new(buffer),
            direction: PhantomData,
        }
    }
    /// `Url()`: the URL given at construction; `stream.url()`.
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl<D: 'static> StreamAccess for StreamBuffer<D> {
    fn stream_io(&self) -> &StreamIo {
        &self.io
    }
    fn stream_io_mut(&mut self) -> &mut StreamIo {
        &mut self.io
    }
}
impl IStream for IStreamBuffer {}
impl IStream for IoStreamBuffer {}
impl OStream for OStreamBuffer {}
impl OStream for IoStreamBuffer {}
