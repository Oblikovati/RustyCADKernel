//! The block read both buffered readers issue: `std::istream::read` / `fread`.
use std::io::{ErrorKind, Read};

/// Bytes stored by one block read and whether the stream reported an error.
pub(crate) struct Filled {
    /// `gcount()` / the `fread` result: bytes stored before end of data or an error.
    pub(crate) count: usize,
    /// True when the stream failed (`bad()` / `ferror`), not merely ran out of data.
    pub(crate) failed: bool,
}

/// Reads until `buffer` is full, the stream ends or it fails, like `istream::read`;
/// `fill_from(&mut stream, &mut chunk).count`. Interrupted reads are retried.
pub(crate) fn fill_from(stream: &mut impl Read, buffer: &mut [u8]) -> Filled {
    let mut count = 0;
    while count < buffer.len() {
        match stream.read(&mut buffer[count..]) {
            Ok(0) => break,
            Ok(read) => count += read,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(_) => {
                return Filled {
                    count,
                    failed: true,
                };
            }
        }
    }
    Filled {
        count,
        failed: false,
    }
}
