// Rust translation of FoundationClasses/TKernel/Standard/Standard_Dump.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Writing dump values: separators, keyed classes, value lists, pointers and field names.
use super::keys::get_pointer_prefix;
use super::stream::DumpOutput;
use super::text::sub_string;
use crate::standard::{Failure, Handle, RuntimeTyped};
use crate::tcollection::AsciiString;
use std::io;

/// Appends native separators using write position and available content; `add_values_separator(&mut stream)?`.
/// Nothing is written at position 0 or after `{` / `, `; an output whose content cannot be
/// read (not a string stream) always gets the separator once its position is positive.
pub fn add_values_separator(output: &mut impl DumpOutput) -> io::Result<()> {
    if output.position() <= 0 {
        return Ok(());
    }
    if let Some(bytes) = output.content() {
        // Text(stream) builds the string from str().c_str(), so it ends at the first NUL.
        let bytes = c_string(bytes);
        if bytes.is_empty() || bytes.ends_with(b"{") || bytes.ends_with(b", ") {
            return Ok(());
        }
    }
    output.write_fragment(b", ")
}

/// `DumpKeyToClass`: writes `"key": {field}` after a separator; `dump_key_to_class(&mut s, "loc", text)?`.
/// Both strings are streamed as C strings (`operator<<` writes `ToCString()`), so each ends
/// at its first NUL.
pub fn dump_key_to_class(
    output: &mut impl DumpOutput,
    key: impl AsRef<[u8]>,
    field: impl AsRef<[u8]>,
) -> io::Result<()> {
    add_values_separator(output)?;
    output.write_fragment(b"\"")?;
    output.write_fragment(c_string(key.as_ref()))?;
    output.write_fragment(b"\": {")?;
    output.write_fragment(c_string(field.as_ref()))?;
    output.write_fragment(b"}")
}

/// The bytes a C string view of `bytes` sees: up to the first NUL; `c_string(b"a\0b") == b"a"`.
fn c_string(bytes: &[u8]) -> &[u8] {
    bytes.split(|byte| *byte == 0).next().unwrap_or_default()
}

/// `DumpCharacterValues`: writes `"v1", "v2", ...`; each value is a C string, so it ends at
/// its first NUL; `dump_character_values(&mut s, &["a", "b"])?`.
pub fn dump_character_values(
    output: &mut impl DumpOutput,
    values: &[impl AsRef<[u8]>],
) -> io::Result<()> {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.write_fragment(b", ")?;
        }
        let text = c_string(value.as_ref());
        output.write_fragment(b"\"")?;
        output.write_fragment(text)?;
        output.write_fragment(b"\"")?;
    }
    Ok(())
}

/// `DumpRealValues`: writes `v1, v2, ...` with the stream's default six-digit `%g` text;
/// `dump_real_values(&mut s, &[1.0, 0.5])?` writes `1, 0.5`.
pub fn dump_real_values(output: &mut impl DumpOutput, values: &[f64]) -> io::Result<()> {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.write_fragment(b", ")?;
        }
        output.write_fragment(AsciiString::from_real(*value).as_bytes())?;
    }
    Ok(())
}

/// `GetPointerInfo(const void*, bool)`: the address as `operator<<(const void*)` prints it,
/// empty for null; `get_pointer_info(Some(0x1234), false) == "0x1234"`.
///
/// The short form keeps the text after its leading zeros, prefixed with "0x". The scan
/// stops before the last character and the libstdc++ text already starts with "0x", so
/// there the short form is "0xx1234" (a native quirk; MSVC prints zero-padded hex).
pub fn get_pointer_info(
    pointer: Option<usize>,
    is_short_info: bool,
) -> Result<AsciiString, Failure> {
    let Some(address) = pointer.filter(|address| *address != 0) else {
        return Ok(AsciiString::default());
    };
    let text = pointer_text(address);
    let bytes = text.as_bytes();
    if !is_short_info {
        return Ok(AsciiString::from_bytes(bytes));
    }
    let length = bytes.len() as i32;
    let Some(first) = (1..length).find(|id| bytes[*id as usize - 1] != b'0') else {
        return Ok(AsciiString::from_bytes(bytes));
    };
    let mut short = get_pointer_prefix();
    short.append(&AsciiString::from_bytes(sub_string(bytes, first, length)?));
    Ok(short)
}

/// `GetPointerInfo(const handle<Standard_Transient>&, bool)`: the object's address, empty
/// for a null handle; `get_handle_pointer_info(&handle, true)?`.
pub fn get_handle_pointer_info<T: RuntimeTyped>(
    handle: &Handle<T>,
    is_short_info: bool,
) -> Result<AsciiString, Failure> {
    if handle.is_null() {
        return Ok(AsciiString::default());
    }
    get_pointer_info(Some(handle.as_ptr() as usize), is_short_info)
}

/// `operator<<(const void*)`: libstdc++ writes `0x` and lowercase hex, MSVC `%p` digits.
fn pointer_text(address: usize) -> String {
    if cfg!(target_env = "msvc") {
        format!("{address:0width$X}", width = 2 * size_of::<usize>())
    } else {
        format!("{address:#x}")
    }
}

/// `DumpFieldToName`: strips `&`, an `a`/`an`/`my` prefix and a trailing `()` or `.get()`;
/// `dump_field_to_name("myValue")? == "Value"`.
///
/// Native quirks kept: any name longer than one byte that starts with `a` loses it (and
/// `an` loses both bytes, so "angle" becomes "gle"); a name that is exactly "()" or
/// ".get()" raises Standard_OutOfRange from SubString.
pub fn dump_field_to_name(field: impl AsRef<[u8]>) -> Result<AsciiString, Failure> {
    let mut name = field.as_ref().to_vec();
    if name.starts_with(b"&") {
        name.remove(0);
    }
    let prefix = match name.as_slice() {
        [b'a', b'n', _, ..] => 2,
        [b'a', _, ..] => 1,
        [first, b'y', _, ..] if first.eq_ignore_ascii_case(&b'm') => 2,
        _ => 0,
    };
    name.drain(..prefix);
    let length = name.len() as i32;
    let kept = if name.ends_with(b".get()") {
        sub_string(&name, 1, length - 6)?
    } else if name.ends_with(b"()") {
        sub_string(&name, 1, length - 2)?
    } else {
        &name
    };
    Ok(AsciiString::from_bytes(kept))
}
