// Rust translation of the exceptions the ModelingData/TKBRep/BinTools classes raise
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The failures of the binary format: the stream mismatch of a short read, and the
//! `catch (Standard_Failure) { ... throw Standard_Failure(aMsg) }` wrappers every
//! reader and writer puts around its body.
use rustycad_tkernel::standard::Failure;

/// `throw Storage_StreamTypeMismatchError()`: a short binary read, without a message.
pub(crate) fn stream_type_mismatch() -> Failure {
    Failure::with_message("Storage_StreamTypeMismatchError", "")
}

/// The native wrapper `aMsg << context << '\n' << anException << '\n'`, thrown as a
/// plain Standard_Failure: the cause is printed as its type, `": "` and its message.
pub(crate) fn wrapped(context: &str, cause: &Failure) -> Failure {
    Failure::new(&format!("{context}\n{cause}\n"))
}

/// Runs `body` inside the native `try { ... } catch (Standard_Failure)` of `context`.
pub(crate) fn wrap_failure<T>(
    context: &str,
    body: impl FnOnce() -> Result<T, Failure>,
) -> Result<T, Failure> {
    body().map_err(|cause| wrapped(context, &cause))
}

/// `aMsg << (uint8_t)value`: a byte inserted as a character. The message is Rust
/// text, so a byte from 0x80 on becomes the Latin-1 character of that code (two UTF-8
/// bytes), where the native message holds the raw byte.
pub(crate) fn byte_text(value: u8) -> char {
    char::from(value)
}

/// The Standard_NullObject the port raises where the native writer dereferences a
/// null geometry handle (a crash natively).
pub(crate) fn null_geometry(member: &str, class: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("{member}: null handle; expected a {class}"),
        "",
    )
}

/// The Standard_OutOfRange of `NCollection_IndexedMap::FindKey(index)` (`myMap(I)`).
pub(crate) fn index_out_of_range(member: &str, index: i32, extent: usize) -> Failure {
    Failure::out_of_range(&format!("{member}: index {index}; expected 1..={extent}"))
}
