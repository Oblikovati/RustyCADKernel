//! A C `FILE*` model (`fread`, `fwrite`, `fseek`, `ftell`, `feof`) for `FSD_BinaryFile`.
use crate::storage::StreamDevice;
use std::io::SeekFrom;

/// The glibc stream semantics the binary driver relies on: a short `fread` keeps the bytes
/// it got and raises the end-of-file flag, which only `fseek` clears; reading a write-only
/// or writing a read-only stream fails without touching the flag.
pub(crate) struct CFile {
    device: Box<dyn StreamDevice>,
    readable: bool,
    writable: bool,
    end_of_file: bool,
}

impl CFile {
    pub(crate) fn new(device: Box<dyn StreamDevice>, readable: bool, writable: bool) -> Self {
        Self {
            device,
            readable,
            writable,
            end_of_file: false,
        }
    }

    /// `fread(buffer, size, count, file)`: the number of complete items read.
    pub(crate) fn read(&mut self, buffer: &mut [u8], size: usize) -> usize {
        if !self.readable || size == 0 {
            return 0;
        }
        let mut filled = 0;
        while filled < buffer.len() {
            match self.device.read(&mut buffer[filled..]) {
                Ok(0) | Err(_) => break,
                Ok(count) => filled += count,
            }
        }
        if filled < buffer.len() {
            self.end_of_file = true;
        }
        filled / size
    }

    /// `fwrite(bytes, size, count, file)`: the number of complete items written.
    pub(crate) fn write(&mut self, bytes: &[u8], size: usize) -> usize {
        if !self.writable || size == 0 {
            return 0;
        }
        match self.device.write_all(bytes) {
            Ok(()) => bytes.len() / size,
            Err(_) => 0,
        }
    }

    /// `ftell`: the position, -1 when the device cannot report it.
    pub(crate) fn tell(&mut self) -> i64 {
        self.device
            .stream_position()
            .map_or(-1, |position| position as i64)
    }

    /// `fseek(file, offset, SEEK_SET)`: 0 on success, -1 for a negative offset (EINVAL).
    pub(crate) fn seek(&mut self, offset: i64) -> i32 {
        let Ok(offset) = u64::try_from(offset) else {
            return -1;
        };
        if self.device.seek(SeekFrom::Start(offset)).is_err() {
            return -1;
        }
        self.end_of_file = false;
        0
    }

    /// `feof`.
    pub(crate) fn is_end_of_file(&self) -> bool {
        self.end_of_file
    }

    /// `fclose`: flushes the device.
    pub(crate) fn close(mut self) {
        let _ = self.device.flush();
    }
}
