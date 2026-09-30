// Rust translation of the stream plumbing of BinTools::Write(S, theFile, ...) and
// BinTools::Read(S, theFile, ...) (BinTools.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The file variants of `BinTools::Write` and `BinTools::Read` open their stream with
//! `OSD_FileSystem::DefaultFileSystem()`, whose streams are the TKernel OSD stream
//! model, while the binary format reads and writes the storage iostream model. The
//! native `std::iostream` sits on the `std::streambuf` of the opened stream; here the
//! storage stream sits on the same buffer through this device.
use rustycad_tkernel::osd::{IosOpenMode, SeekDir, SharedStreamBuf, StreamBuf};
use std::io::{self, Read, Seek, SeekFrom, Write};

/// The `std::streambuf` of an opened OSD stream, as the byte device of a storage
/// stream: reads are `sgetn`, writes `sputn`, positions `pubseekoff` on the side
/// (`in` or `out`) the stream was opened for, and `flush` is `pubsync`.
pub(crate) struct StreamBufDevice {
    buffer: SharedStreamBuf,
    side: IosOpenMode,
}

impl StreamBufDevice {
    /// A device over `buffer`, positioned on `side`.
    pub(crate) fn new(buffer: SharedStreamBuf, side: IosOpenMode) -> Self {
        Self { buffer, side }
    }

    fn with_buffer<R>(&self, action: impl FnOnce(&mut dyn StreamBuf) -> R) -> R {
        let mut buffer = self
            .buffer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        action(&mut *buffer)
    }
}

impl Read for StreamBufDevice {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        Ok(self.with_buffer(|buffer| buffer.sgetn(out)))
    }
}

impl Write for StreamBufDevice {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let written = self.with_buffer(|buffer| buffer.sputn(bytes));
        if written == 0 && !bytes.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "streambuf refused the bytes",
            ));
        }
        Ok(written)
    }
    fn flush(&mut self) -> io::Result<()> {
        match self.with_buffer(|buffer| buffer.pubsync()) {
            -1 => Err(io::Error::other("streambuf pubsync failed")),
            _ => Ok(()),
        }
    }
}

impl Seek for StreamBufDevice {
    fn seek(&mut self, target: SeekFrom) -> io::Result<u64> {
        let (offset, direction) = match target {
            SeekFrom::Start(offset) => (offset as i64, SeekDir::Beg),
            SeekFrom::Current(offset) => (offset, SeekDir::Cur),
            SeekFrom::End(offset) => (offset, SeekDir::End),
        };
        let side = self.side;
        let position = self.with_buffer(|buffer| buffer.pubseekoff(offset, direction, side));
        u64::try_from(position)
            .map_err(|_| io::Error::other(format!("streambuf refused the position {target:?}")))
    }
}
