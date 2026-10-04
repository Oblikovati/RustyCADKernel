// Rust translation of FoundationClasses/TKernel/Standard/Standard_CString.hxx/.cxx and
// Standard_CStringHasher.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Plain C string helpers: `IsEqual`, `Atof` and the `Standard_CStringHasher` policy.
//!
//! A C string is a byte slice read up to its first NUL (or its end), so embedded NULs
//! end the string exactly as `strlen`/`strcmp` do. `Strtod` is [`crate::standard::strtod`].
//!
//! The formatting wrappers support `%c`, `%d/%i/%o/%u/%x/%X/%f/%F/%e/%E/%g/%G/%s/%%`,
//! integer `hh/h/l/ll/z/t` lengths, literal or `*` width/precision, and the private formatter's flags. They require typed
//! [`FormatArg`] values and injected sinks; they do not expose C varargs or `FILE*`. Vec
//! buffer variants replace storage; caller-slice variants preserve bytes after the final NUL.

use super::hashing::{HashWord, fnv_hash, hash_bytes};
pub use crate::c_format::FormatArg;
use crate::c_format::checked_sprintf;
use crate::ncollection::HashPolicy;
use std::io::{self, Write};

/// The bytes `strlen` would count: everything before the first NUL.
fn c_str_prefix(text: &[u8]) -> &[u8] {
    let length: usize = text
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(text.len());
    &text[..length]
}

/// `IsEqual(const char*, const char*)`: `strcmp(a, b) == 0`, comparing up to the first NUL.
///
/// ```
/// assert!(rustycad_tkernel::standard::c_string::is_equal(b"abc\0x", b"abc"));
/// ```
pub fn is_equal(first: &[u8], second: &[u8]) -> bool {
    c_str_prefix(first) == c_str_prefix(second)
}

/// `Atof(const char*)`: `Strtod(text, nullptr)`, the C-locale numeric prefix (0 if none).
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::c_string::atof(" 2.5e1xyz"), 25.0);
/// ```
pub fn atof(text: &str) -> f64 {
    super::strtod(text).0
}

/// `Atof(const char*)` for arbitrary C-string bytes; `atof_bytes(b"1.25\xff") == 1.25`.
pub fn atof_bytes(text: &[u8]) -> f64 {
    super::strtod_bytes(text).0
}

/// `Printf` with typed arguments and an injected writer; `printf_to(&mut out, b"%d", &[FormatArg::Int(4)])`.
/// Returns bytes written and propagates sink errors; C stdout behavior remains partial.
pub fn printf_to(
    output: &mut impl Write,
    format: &[u8],
    args: &[FormatArg<'_>],
) -> io::Result<usize> {
    write_format(output, format, args)
}

/// `Fprintf` with an injected writer instead of a process-owned `FILE*`;
/// `fprintf_to(&mut file, b"%s", &[FormatArg::Text(b"shape")])`.
/// Returns bytes written; C `FILE*` locking and error-state behavior remain partial.
pub fn fprintf_to(
    output: &mut impl Write,
    format: &[u8],
    args: &[FormatArg<'_>],
) -> io::Result<usize> {
    write_format(output, format, args)
}

/// `Sprintf` into caller-owned growable storage, including the terminating NUL;
/// `sprintf_into(&mut buffer, b"%04d", &[FormatArg::Int(7)])` returns 4.
/// The `Vec` is replaced, so fixed-buffer tail bytes and C `int` count overflow are partial.
pub fn sprintf_into(
    output: &mut Vec<u8>,
    format: &[u8],
    args: &[FormatArg<'_>],
) -> io::Result<usize> {
    let rendered = render_format(format, args)?;
    let count = rendered.len();
    output.clear();
    output.extend_from_slice(&rendered);
    output.push(0);
    Ok(count)
}

/// `Sprintf` into a fixed caller slice; `sprintf_into_slice(&mut b, b"%s", &[FormatArg::Text(b"edge")])`.
/// The slice must fit the bytes plus final NUL; remaining bytes are left untouched.
pub fn sprintf_into_slice(
    output: &mut [u8],
    format: &[u8],
    args: &[FormatArg<'_>],
) -> io::Result<usize> {
    let rendered = render_format(format, args)?;
    write_formatted_slice(output, &rendered)
}

/// Typed `Vsprintf` counterpart; `vsprintf_into(&mut b, b"%s", &[FormatArg::Text(b"edge")])` writes `edge\0`.
/// `va_list`, fixed-buffer tail bytes, and C `int` count overflow remain partial.
pub fn vsprintf_into(
    output: &mut Vec<u8>,
    format: &[u8],
    args: &[FormatArg<'_>],
) -> io::Result<usize> {
    sprintf_into(output, format, args)
}

/// Typed `Vsprintf` into a fixed caller slice; `vsprintf_into_slice(&mut b, b"%s", &[FormatArg::Text(b"edge")])`.
/// `va_list` is replaced by `FormatArg` values; short slices return an error without mutation.
pub fn vsprintf_into_slice(
    output: &mut [u8],
    format: &[u8],
    args: &[FormatArg<'_>],
) -> io::Result<usize> {
    sprintf_into_slice(output, format, args)
}

fn write_format(
    output: &mut impl Write,
    format: &[u8],
    args: &[FormatArg<'_>],
) -> io::Result<usize> {
    let rendered = render_format(format, args)?;
    output.write_all(&rendered)?;
    Ok(rendered.len())
}

fn write_formatted_slice(output: &mut [u8], rendered: &[u8]) -> io::Result<usize> {
    let required = required_slice_capacity(rendered.len())?;
    ensure_slice_capacity(output.len(), required)?;
    write_slice_contents(output, rendered);
    Ok(rendered.len())
}

fn required_slice_capacity(rendered_length: usize) -> io::Result<usize> {
    rendered_length
        .checked_add(1)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "formatted length overflow"))
}

fn ensure_slice_capacity(actual: usize, required: usize) -> io::Result<()> {
    if actual >= required {
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("formatted result needs {required} bytes; caller buffer has {actual}"),
    ))
}

fn write_slice_contents(output: &mut [u8], rendered: &[u8]) {
    output[..rendered.len()].copy_from_slice(rendered);
    output[rendered.len()] = 0;
}

fn render_format(format: &[u8], args: &[FormatArg<'_>]) -> io::Result<Vec<u8>> {
    checked_sprintf(c_str_prefix(format), args).map_err(format_error)
}

fn format_error(error: crate::c_format::FormatError) -> io::Error {
    let conversion = if error.conversion == 0 {
        "end-of-format".to_owned()
    } else {
        format!("'{}'", char::from(error.conversion))
    };
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "format conversion {conversion} at byte {}: expected {}",
            error.offset, error.expected
        ),
    )
}

/// `Standard_CStringHasher`: the hash/equality policy for C-string keys.
///
/// Strings shorter than four bytes use FNV-1a over their bytes (so the empty string
/// hashes to the FNV offset basis); longer ones use `opencascade::hashBytes` (Murmur).
#[derive(Clone, Copy, Debug, Default)]
pub struct CStringHasher;

impl CStringHasher {
    /// `size_t operator()(const char*)`: the hash of the C string in `text`.
    ///
    /// ```
    /// use rustycad_tkernel::standard::c_string::CStringHasher;
    /// assert_eq!(CStringHasher.hash_code(b"abc\0tail"), CStringHasher.hash_code(b"abc"));
    /// ```
    pub fn hash_code(&self, text: &[u8]) -> usize {
        let text: &[u8] = c_str_prefix(text);
        if text.len() < 4 {
            return fnv_hash::hash_combine(text, text.len(), usize::FNV_SEED);
        }
        hash_bytes(text)
    }

    /// `bool operator()(const char*, const char*)`: `strcmp(a, b) == 0`.
    ///
    /// ```
    /// use rustycad_tkernel::standard::c_string::CStringHasher;
    /// assert!(CStringHasher.is_equal(b"abc", b"abc\0"));
    /// ```
    pub fn is_equal(&self, first: &[u8], second: &[u8]) -> bool {
        is_equal(first, second)
    }
}

impl<K: AsRef<[u8]>> HashPolicy<K> for CStringHasher {
    fn hash(&self, key: &K) -> usize {
        self.hash_code(key.as_ref())
    }
    fn equal(&self, first: &K, second: &K) -> bool {
        self.is_equal(first.as_ref(), second.as_ref())
    }
}
