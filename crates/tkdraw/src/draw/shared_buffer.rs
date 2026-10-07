//! An in-memory byte device that both an [`IoStream`] and its owner can see, so the
//! DRAW text streams can hand their bytes to the `std::iostream` model of TKernel
//! (which the GeomTools and Poly readers and writers take) and read the result back.
use rustycad_tkernel::storage::IoStream;
use std::cell::RefCell;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::rc::Rc;

/// A cursor over a byte vector, shared between a stream and its creator.
#[derive(Debug, Clone, Default)]
pub(crate) struct SharedBuffer {
    cursor: Rc<RefCell<Cursor<Vec<u8>>>>,
}

impl SharedBuffer {
    /// A buffer holding `bytes`, positioned at the start.
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self {
            cursor: Rc::new(RefCell::new(Cursor::new(bytes))),
        }
    }

    /// A readable and writable stream over this buffer with the given precision.
    pub(crate) fn stream(&self, precision: usize) -> IoStream {
        let mut stream = IoStream::with_device(Box::new(self.clone()), true, true);
        stream.set_precision(precision);
        stream
    }

    /// The bytes and the position once `stream` is closed (its look-ahead returned).
    pub(crate) fn finish(&self, stream: IoStream) -> (Vec<u8>, u64) {
        drop(stream.into_device());
        let cursor = self.cursor.borrow();
        (cursor.get_ref().clone(), cursor.position())
    }
}

impl Read for SharedBuffer {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.borrow_mut().read(buffer)
    }
}

impl Write for SharedBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.cursor.borrow_mut().write(bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Seek for SharedBuffer {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.cursor.borrow_mut().seek(position)
    }
}

#[cfg(test)]
mod tests {
    use super::SharedBuffer;

    #[test]
    fn the_position_excludes_the_unread_look_ahead() {
        let buffer = SharedBuffer::new(b"12 34 rest".to_vec());
        let mut stream = buffer.stream(6);
        let mut value = 0;
        assert!(stream.extract_int(&mut value));
        assert_eq!(value, 12);
        let (bytes, position) = buffer.finish(stream);
        assert_eq!(position, 2);
        assert_eq!(bytes, b"12 34 rest");
    }

    #[test]
    fn written_bytes_are_kept() {
        let buffer = SharedBuffer::default();
        let mut stream = buffer.stream(3);
        stream.insert_real(0.123456);
        assert_eq!(buffer.finish(stream).0, b"0.123");
    }
}
