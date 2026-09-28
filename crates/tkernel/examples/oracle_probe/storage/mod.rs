//! Storage/FSD differential operations (twin of `tools/oracle/tkernel_storage_probe.cxx`).
//!
//! Byte strings travel as lowercase hex (`-` when empty), UTF-16 strings as four hex digits
//! per unit, and exceptions by class name only. Drivers open files in an in-memory file
//! system that holds the bytes the native probe writes to a scratch file.
use std::str::SplitWhitespace;

mod bstream;
mod common;
mod drivers;
mod schema;
mod streams;

/// The output of a storage operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    streams::evaluate(op, input)
        .or_else(|| drivers::evaluate(op, input))
        .or_else(|| schema::evaluate(op, input))
}
