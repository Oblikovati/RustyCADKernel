// Rust translation of the static Put*/Get* members of
// ModelingData/TKBRep/BinTools/BinTools.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The scalar encodings of the binary format. Every value is written as its bytes in
//! the platform byte order: `DO_INVERSE` is not defined at the pinned revision (no
//! header defines it), so no FSD_BinaryFile swapping happens, and a little-endian
//! build writes little-endian values.
use super::errors::stream_type_mismatch;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;

/// `OS.put(byte)` (and the formatted `OS << (uint8_t)byte`, identical at width 0).
pub(crate) fn put_byte(stream: &mut IoStream, value: u8) -> &mut IoStream {
    stream.write(&[value])
}

/// `IS.get()`: the next byte, or -1 (`EOF`) with `eofbit|failbit`.
pub(crate) fn stream_get(stream: &mut IoStream) -> i32 {
    let mut byte = 0;
    stream.get(&mut byte);
    if stream.gcount() == 1 {
        i32::from(byte)
    } else {
        -1
    }
}

/// `IS.read(buffer)` of a fixed-size value; `Storage_StreamTypeMismatchError` when the
/// stream fails (a short read, or a stream that had already failed).
pub(crate) fn read_exact<const N: usize>(stream: &mut IoStream) -> Result<[u8; N], Failure> {
    let mut bytes = [0_u8; N];
    stream.read(&mut bytes);
    if stream.fail() {
        return Err(stream_type_mismatch());
    }
    Ok(bytes)
}

/// `BinTools::PutReal`: the 8 bytes of the double.
///
/// ```
/// use rustycad_tkernel::storage::IoStream;
/// let mut stream = IoStream::from_bytes(Vec::new());
/// rustycad_tkbrep::bintools::put_real(&mut stream, 1.5);
/// assert_eq!(stream.tellp(), 8);
/// ```
pub fn put_real(stream: &mut IoStream, value: f64) -> &mut IoStream {
    stream.write(&value.to_ne_bytes())
}

/// `BinTools::PutShortReal`: the 4 bytes of the float;
/// `bintools::put_short_real(&mut stream, 0.5)`.
pub fn put_short_real(stream: &mut IoStream, value: f32) -> &mut IoStream {
    stream.write(&value.to_ne_bytes())
}

/// `BinTools::PutInteger`: the 4 bytes of the int; `bintools::put_integer(&mut stream, 7)`.
pub fn put_integer(stream: &mut IoStream, value: i32) -> &mut IoStream {
    stream.write(&value.to_ne_bytes())
}

/// `BinTools::PutBool`: one byte, 1 or 0; `bintools::put_bool(&mut stream, true)`.
pub fn put_bool(stream: &mut IoStream, value: bool) -> &mut IoStream {
    put_byte(stream, u8::from(value))
}

/// `BinTools::PutExtChar`: the 2 bytes of the `char16_t`;
/// `bintools::put_ext_char(&mut stream, 3)`.
pub fn put_ext_char(stream: &mut IoStream, value: u16) -> &mut IoStream {
    stream.write(&value.to_ne_bytes())
}

/// `BinTools::GetReal`: 8 bytes; `Storage_StreamTypeMismatchError` when they cannot all
/// be read.
///
/// ```
/// use rustycad_tkernel::storage::IoStream;
/// let mut stream = IoStream::from_bytes(2.5_f64.to_ne_bytes().to_vec());
/// assert_eq!(rustycad_tkbrep::bintools::get_real(&mut stream).unwrap(), 2.5);
/// assert!(rustycad_tkbrep::bintools::get_real(&mut stream).is_err());
/// ```
pub fn get_real(stream: &mut IoStream) -> Result<f64, Failure> {
    read_exact::<8>(stream).map(f64::from_ne_bytes)
}

/// `BinTools::GetShortReal`: 4 bytes, else `Storage_StreamTypeMismatchError`;
/// `bintools::get_short_real(&mut stream)?`.
pub fn get_short_real(stream: &mut IoStream) -> Result<f32, Failure> {
    read_exact::<4>(stream).map(f32::from_ne_bytes)
}

/// `BinTools::GetInteger`: 4 bytes, else `Storage_StreamTypeMismatchError`;
/// `bintools::get_integer(&mut stream)?`.
pub fn get_integer(stream: &mut IoStream) -> Result<i32, Failure> {
    read_exact::<4>(stream).map(i32::from_ne_bytes)
}

/// `BinTools::GetExtChar`: 2 bytes, else `Storage_StreamTypeMismatchError`;
/// `bintools::get_ext_char(&mut stream)?`.
pub fn get_ext_char(stream: &mut IoStream) -> Result<u16, Failure> {
    read_exact::<2>(stream).map(u16::from_ne_bytes)
}

/// `BinTools::GetBool`: `IS.get() != 0`. It never raises: at the end of the stream
/// `get()` returns `EOF` (-1), which reads as true, as natively;
/// `bintools::get_bool(&mut stream)`.
pub fn get_bool(stream: &mut IoStream) -> bool {
    stream_get(stream) != 0
}
