//! `MemoryStreamBuf`: a `std::stringbuf`-like in-memory buffer, the fake behind the
//! `std::istringstream` and `std::ostringstream` of the native tests.
use rustycad_tkernel::osd::{
    IStreamBuffer, IosOpenMode, OStreamBuffer, SeekDir, SharedStreamBuf, StreamBuf,
    share_stream_buf,
};

/// An in-memory buffer with one read/write position.
pub struct MemoryStreamBuf {
    bytes: Vec<u8>,
    position: usize,
}
impl MemoryStreamBuf {
    /// A shared buffer over `bytes`, positioned at the start.
    pub fn shared(bytes: &[u8]) -> SharedStreamBuf {
        share_stream_buf(Self {
            bytes: bytes.to_vec(),
            position: 0,
        })
    }
}
impl StreamBuf for MemoryStreamBuf {
    fn sgetn(&mut self, out: &mut [u8]) -> usize {
        let count: usize = out.len().min(self.bytes.len() - self.position);
        out[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
        self.position += count;
        count
    }
    fn sputn(&mut self, bytes: &[u8]) -> usize {
        self.bytes.truncate(self.position);
        self.bytes.extend_from_slice(bytes);
        self.position = self.bytes.len();
        bytes.len()
    }
    fn pubseekoff(&mut self, offset: i64, direction: SeekDir, _which: IosOpenMode) -> i64 {
        let base: i64 = match direction {
            SeekDir::Beg => 0,
            SeekDir::Cur => self.position as i64,
            SeekDir::End => self.bytes.len() as i64,
        };
        let target: i64 = base + offset;
        if target < 0 || target > self.bytes.len() as i64 {
            return -1;
        }
        self.position = target as usize;
        target
    }
    fn pubsync(&mut self) -> i32 {
        0
    }
}

/// `std::istringstream(text)`.
pub fn memory_istream(bytes: &[u8]) -> IStreamBuffer {
    IStreamBuffer::new("memory", MemoryStreamBuf::shared(bytes))
}

/// `std::ostringstream()`.
pub fn memory_ostream() -> OStreamBuffer {
    OStreamBuffer::new("memory", MemoryStreamBuf::shared(b""))
}
