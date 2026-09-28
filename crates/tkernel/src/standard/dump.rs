// Copyright (c) 2019 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of FoundationClasses/TKernel/Standard/Standard_Dump.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT's stream-oriented diagnostic JSON fragments and one-based parser positions.
use crate::tcollection::AsciiString;
use std::io;
mod parsing;
mod stream;
pub use parsing::{
    init_integer_field, init_real_field, init_real_values, init_value, init_vector,
    process_stream_name,
};
pub use stream::{DumpOutput, DumpStream};
/// Appends native separators using write position and available content; `add_values_separator(&mut stream)?`.
pub fn add_values_separator(output: &mut impl DumpOutput) -> io::Result<()> {
    if output.position() <= 0 {
        return Ok(());
    }
    if let Some(bytes) = output.content() {
        let bytes = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
        if bytes.is_empty() || bytes.ends_with(b"{") || bytes.ends_with(b", ") {
            return Ok(());
        }
    }
    output.write_fragment(b", ")
}
/// Emits a named vector with the native six-digit precision; `dump_vector(&mut stream,"gp_Pnt",&[1.0,2.0,3.0])?`.
pub fn dump_vector(output: &mut impl DumpOutput, name: &str, values: &[f64]) -> io::Result<()> {
    add_values_separator(output)?;
    output.write_fragment(format!("\"{name}\": [").as_bytes())?;
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.write_fragment(b", ")?;
        }
        output.write_fragment(AsciiString::from_real(*value).as_bytes())?;
    }
    output.write_fragment(b"]")
}
/// Emits a named scalar; `dump_real(&mut stream,"Gap",1.5)?`.
pub fn dump_real(output: &mut impl DumpOutput, name: &str, value: f64) -> io::Result<()> {
    add_values_separator(output)?;
    output.write_fragment(format!("\"{name}\": ").as_bytes())?;
    output.write_fragment(AsciiString::from_real(value).as_bytes())
}
/// Emits the `OCCT_DUMP_CLASS_BEGIN` class-name field; `dump_class_begin(&mut stream,"gp_Ax2d")?`.
pub fn dump_class_begin(output: &mut impl DumpOutput, class_name: &str) -> io::Result<()> {
    add_values_separator(output)?;
    output.write_fragment(format!("\"className\": \"{class_name}\"").as_bytes())
}
/// Wraps an already dumped field as `"key": {field}` like `DumpKeyToClass`; `dump_key_to_class(&mut stream,"loc",text)?`.
pub fn dump_key_to_class(output: &mut impl DumpOutput, key: &str, field: &str) -> io::Result<()> {
    add_values_separator(output)?;
    output.write_fragment(format!("\"{key}\": {{{field}}}").as_bytes())
}
/// Emits an integer without floating-point rounding; `dump_integer(&mut stream,"Flags",1)?`.
pub fn dump_integer(output: &mut impl DumpOutput, name: &str, value: i32) -> io::Result<()> {
    add_values_separator(output)?;
    output.write_fragment(format!("\"{name}\": {value}").as_bytes())
}
