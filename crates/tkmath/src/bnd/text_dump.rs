// Created on: 1991-03-08
// Created by: Christophe MARION
// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_Box.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Shared writer for the `Dump()` console text of `Bnd_Box` and `Bnd_Box2d`.
use rustycad_tkernel::tcollection::AsciiString;
use std::io;

/// One stored side limit: its label and the raw value, or `None` when the side is open.
pub(super) type DumpSide = (&'static str, Option<f64>);

/// Writes the native `Dump()` text: title, `Void`/`Whole` or each raw side, then the gap.
///
/// `std::cout << double` uses the default six significant digits, which is the `%g`
/// rendering of `AsciiString::from_real`.
/// Example: `write_box_text(&mut out, "Box2d : ", false, false, &sides, 0.0)?`.
pub(super) fn write_box_text(
    output: &mut impl io::Write,
    title: &str,
    state: (bool, bool),
    sides: &[DumpSide],
    gap: f64,
) -> io::Result<()> {
    let (is_void, is_whole) = state;
    output.write_all(title.as_bytes())?;
    if is_void {
        output.write_all(b"Void")?;
    } else if is_whole {
        output.write_all(b"Whole")?;
    } else {
        write_sides(output, sides)?;
    }
    output.write_all(b"\n Gap : ")?;
    output.write_all(AsciiString::from_real(gap).as_bytes())?;
    output.write_all(b"\n")
}

fn write_sides(output: &mut impl io::Write, sides: &[DumpSide]) -> io::Result<()> {
    for (label, value) in sides {
        write!(output, "\n {label} : ")?;
        match value {
            Some(value) => output.write_all(AsciiString::from_real(*value).as_bytes())?,
            None => output.write_all(b"Infinite")?,
        }
    }
    Ok(())
}
