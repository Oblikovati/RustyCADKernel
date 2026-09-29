// Created on: 1993-07-02
// Created by: Remi LEQUETTE
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of the static PrintShapeEnum, PrintOrientation and ReadShapeEnum
// and of the DumpExtent counts of ModelingData/TKBRep/TopTools/TopTools_ShapeSet.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The words of the TShape table: type and orientation codes and the extent counts.
use super::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `PrintShapeEnum(T, S, C)`: the two-letter code (`compact`) or the padded name.
pub(super) fn print_shape_enum(kind: ShapeEnum, stream: &mut IoStream, compact: bool) {
    let (code, name): (&[u8], &[u8]) = match kind {
        ShapeEnum::Vertex => (b"Ve", b"VERTEX   "),
        ShapeEnum::Edge => (b"Ed", b"EDGE     "),
        ShapeEnum::Wire => (b"Wi", b"WIRE     "),
        ShapeEnum::Face => (b"Fa", b"FACE     "),
        ShapeEnum::Shell => (b"Sh", b"SHELL    "),
        ShapeEnum::Solid => (b"So", b"SOLID    "),
        ShapeEnum::CompSolid => (b"CS", b"COMPSOLID"),
        ShapeEnum::Compound => (b"Co", b"COMPOUND "),
        ShapeEnum::Shape => (b"Sp", b"SHAPE"),
    };
    stream.insert_bytes(if compact { code } else { name });
}

/// `PrintOrientation(O, S, C)`: `+ - i e` (`compact`) or the full name.
pub(super) fn print_orientation(orientation: Orientation, stream: &mut IoStream, compact: bool) {
    let (code, name): (&[u8], &[u8]) = match orientation {
        Orientation::Forward => (b"+", b"FORWARD"),
        Orientation::Reversed => (b"-", b"REVERSED"),
        Orientation::Internal => (b"i", b"INTERNAL"),
        Orientation::External => (b"e", b"EXTERNAL"),
    };
    stream.insert_bytes(if compact { code } else { name });
}

/// `ReadShapeEnum(IS)` on the extracted word: its first letter (and the second for
/// `Sh`/`So` and `CS`/`Co`), COMPOUND for anything else (an unread word included,
/// where the native buffer is uninitialised).
pub(super) fn shape_enum_of(word: &[u8]) -> ShapeEnum {
    match (word.first(), word.get(1)) {
        (Some(b'V'), _) => ShapeEnum::Vertex,
        (Some(b'E'), _) => ShapeEnum::Edge,
        (Some(b'W'), _) => ShapeEnum::Wire,
        (Some(b'F'), _) => ShapeEnum::Face,
        (Some(b'S'), Some(b'h')) => ShapeEnum::Shell,
        (Some(b'S'), _) => ShapeEnum::Solid,
        (Some(b'C'), Some(b'S')) => ShapeEnum::CompSolid,
        _ => ShapeEnum::Compound,
    }
}

/// `atoi(text)`: optional spaces and sign, then decimal digits; 0 without digits.
/// Overflow is undefined natively; glibc's `strtol` saturates, then the value is
/// truncated to `int`, which the port reproduces.
pub(crate) fn c_atoi(text: &[u8]) -> i32 {
    let start = text
        .iter()
        .position(|byte| !is_c_space(*byte))
        .unwrap_or(text.len());
    let mut rest = &text[start..];
    let negative = rest.first() == Some(&b'-');
    if matches!(rest.first(), Some(b'+' | b'-')) {
        rest = &rest[1..];
    }
    // Accumulate beyond the `long` range, then clamp as `strtol` does (LONG_MIN included).
    let mut magnitude: i128 = 0;
    for digit in rest.iter().take_while(|byte| byte.is_ascii_digit()) {
        magnitude = (magnitude * 10 + i128::from(digit - b'0')).min(i128::from(u64::MAX));
    }
    let signed = if negative { -magnitude } else { magnitude };
    let long = signed.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
    long as i32
}

fn is_c_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r')
}

/// The text of `DumpExtent`: the number of shapes of each type, then the total.
pub(super) fn shape_counts(shapes: &IndexedMapOfShape) -> Result<String, Failure> {
    const LABELS: [&str; 8] = [
        " VERTEX    : ",
        " EDGE      : ",
        " WIRE      : ",
        " FACE      : ",
        " SHELL     : ",
        " SOLID     : ",
        " COMPSOLID : ",
        " COMPOUND  : ",
    ];
    let mut text = String::new();
    for (label, count) in LABELS.iter().zip(count_types(shapes)?) {
        text.push_str(&format!("{label}{count}\n"));
    }
    text.push_str(&format!(" SHAPE     : {}\n", shapes.extent()));
    Ok(text)
}

/// The number of shapes of each type, VERTEX (7) first and COMPOUND (0) last; SHAPE
/// is not counted.
fn count_types(shapes: &IndexedMapOfShape) -> Result<[i32; 8], Failure> {
    let mut counts = [0_i32; 8];
    for index in 1..=shapes.extent() {
        let kind = shapes.find_key(index)?.shape_type()?;
        if let Some(slot) = 7_usize.checked_sub(kind as usize) {
            counts[slot] += 1;
        }
    }
    Ok(counts)
}
