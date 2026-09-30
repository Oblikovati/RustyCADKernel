// Rust translation of FoundationClasses/TKernel/Standard/Standard_Dump.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT's stream-oriented diagnostic JSON (`Standard_Dump`): writing fragments, formatting,
//! splitting and reading values back at one-based positions.
//!
//! The functions below `dump_vector` stand for the header's `OCCT_DUMP_*` macros, which
//! are not class members: each writes the same text the macro expansion writes.
use crate::tcollection::AsciiString;
use std::io;
mod format;
mod keys;
mod parsing;
mod split;
mod stream;
mod text;
mod values;
pub use format::{format_json, text};
pub use keys::{JsonKey, get_pointer_prefix, json_key_length, json_key_to_string};
pub use parsing::{
    init_integer_field, init_real_field, init_real_values, init_value, init_vector,
    process_field_name, process_stream_name,
};
pub use split::{DumpValue, DumpValues, has_child_key, hierarchical_value_indices, split_json};
pub use stream::{DumpOutput, DumpStream};
pub use values::{
    add_values_separator, dump_character_values, dump_field_to_name, dump_key_to_class,
    dump_real_values, get_handle_pointer_info, get_pointer_info,
};

/// `OCCT_DUMP_VECTOR_CLASS`: a named real vector with the native six-digit precision;
/// `dump_vector(&mut stream, "gp_Pnt", &[1.0, 2.0, 3.0])?`.
pub fn dump_vector(output: &mut impl DumpOutput, name: &str, values: &[f64]) -> io::Result<()> {
    add_values_separator(output)?;
    output.write_fragment(format!("\"{name}\": [").as_bytes())?;
    dump_real_values(output, values)?;
    output.write_fragment(b"]")
}

/// `OCCT_DUMP_FIELD_VALUE_NUMERICAL` for a real field under an already converted key;
/// `dump_real(&mut stream, "Gap", 1.5)?`.
pub fn dump_real(output: &mut impl DumpOutput, name: &str, value: f64) -> io::Result<()> {
    add_values_separator(output)?;
    output.write_fragment(format!("\"{name}\": ").as_bytes())?;
    output.write_fragment(AsciiString::from_real(value).as_bytes())
}

/// `OCCT_DUMP_CLASS_BEGIN`: the class-name field; `dump_class_begin(&mut stream, "gp_Ax2d")?`.
pub fn dump_class_begin(output: &mut impl DumpOutput, class_name: &str) -> io::Result<()> {
    add_values_separator(output)?;
    output.write_fragment(format!("\"className\": \"{class_name}\"").as_bytes())
}

/// `OCCT_DUMP_FIELD_VALUE_NUMERICAL` for an integer field; `dump_integer(&mut stream, "Flags", 1)?`.
pub fn dump_integer(output: &mut impl DumpOutput, name: &str, value: i32) -> io::Result<()> {
    add_values_separator(output)?;
    output.write_fragment(format!("\"{name}\": {value}").as_bytes())
}
