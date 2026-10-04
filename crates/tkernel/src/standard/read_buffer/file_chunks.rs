//! FILE-overload adapters for the `Standard_ReadBuffer` chunk templates.
use super::{ReadBuffer, exceptions};
use crate::standard::FreadStream;
use bytemuck::AnyBitPattern;
use std::io::Read;

impl ReadBuffer {
    /// Returns the next chunk through an `fread`-style adapter; `buffer.read_data_chunk_file(&mut file)`.
    /// Unlike the line-buffer hook, native ReadBuffer accepts a full read even when `ferror` was already set.
    pub fn read_data_chunk_file<R: Read>(&mut self, stream: &mut FreadStream<R>) -> Option<&[u8]> {
        let start = self.next_chunk_with(|buffer| {
            let count = stream.read_block_count(buffer);
            (count, count == buffer.len())
        })?;
        Some(&self.cache[start..start + self.chunk_len])
    }

    /// Copies a typed chunk from an `fread`-style source; `buffer.read_chunk_file::<u32>(&mut file)?`.
    pub fn read_chunk_file<T: AnyBitPattern, R: Read>(
        &mut self,
        stream: &mut FreadStream<R>,
    ) -> Result<Option<T>, crate::standard::Failure> {
        let size = size_of::<T>();
        if size > self.chunk_len {
            return Err(exceptions::program_error(&format!(
                "Standard_ReadBuffer::ReadChunk: a {size}-byte value does not fit a {}-byte chunk",
                self.chunk_len
            )));
        }
        Ok(self
            .read_data_chunk_file(stream)
            .map(|bytes| bytemuck::pod_read_unaligned(&bytes[..size])))
    }
}

#[cfg(test)]
mod tests {
    use super::ReadBuffer;
    use crate::standard::{FreadStream, ReadLineBuffer};
    use std::io::{self, Cursor, Read};

    struct ErrorThenBytes {
        source: Cursor<Vec<u8>>,
        fail_once: bool,
    }

    impl Read for ErrorThenBytes {
        fn read(&mut self, target: &mut [u8]) -> io::Result<usize> {
            if self.fail_once {
                self.fail_once = false;
                return Err(io::Error::other("injected ferror"));
            }
            self.source.read(target)
        }
    }

    #[test]
    fn a_full_fread_count_succeeds_with_ferror_still_set() {
        let source = ErrorThenBytes {
            source: Cursor::new(b"data".to_vec()),
            fail_once: true,
        };
        let mut file = FreadStream::new(source);
        let mut line = ReadLineBuffer::new(1);
        let mut count = usize::MAX;
        assert!(!line.read_stream_file(&mut file, 1, &mut count));
        assert_eq!(count, 0);
        let mut chunks = ReadBuffer::new(4, 4, false).expect("valid chunk buffer");
        assert_eq!(
            chunks.read_data_chunk_file(&mut file),
            Some(b"data".as_slice())
        );
        assert!(file.is_error());
    }

    #[test]
    fn typed_file_chunk_reads_only_the_requested_value_bytes() {
        let mut file = FreadStream::new(Cursor::new(0x1234_u32.to_ne_bytes().to_vec()));
        let mut chunks = ReadBuffer::new(4, 4, false).expect("valid chunk buffer");
        assert_eq!(
            chunks.read_chunk_file::<u16, _>(&mut file).unwrap(),
            Some(0x1234)
        );
    }
}
