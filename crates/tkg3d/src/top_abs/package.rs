// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/TopAbs/TopAbs.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The static members of the `TopAbs` class: orientation algebra, names and printing.
use super::enums::{Orientation, ShapeEnum, State};
use rustycad_tkernel::tcollection::AsciiString;
use std::io::{self, Write};

use Orientation::{External, Forward, Internal, Reversed};

// TopAbs_Table_PrintShapeEnum and TopAbs_Table_PrintOrientation of TopAbs.cxx.
const SHAPE_TYPE_NAMES: [&str; 9] = [
    "COMPOUND",
    "COMPSOLID",
    "SOLID",
    "SHELL",
    "FACE",
    "WIRE",
    "EDGE",
    "VERTEX",
    "SHAPE",
];
const ORIENTATION_NAMES: [&str; 4] = ["FORWARD", "REVERSED", "INTERNAL", "EXTERNAL"];
// TopAbs_Table_PrintState: in "ON", "IN", "OUT" order although the enumeration is
// IN, OUT, ON, so `Print(TopAbs_IN)` writes "ON" natively; kept as is.
const STATE_NAMES: [&str; 4] = ["ON", "IN", "OUT", "UNKNOWN"];

// The composition table, read as `COMPOSE[Or2][Or1]`.
const COMPOSE: [[Orientation; 4]; 4] = [
    [Forward, Reversed, Internal, External],
    [Reversed, Forward, Internal, External],
    [Internal, Internal, Internal, Internal],
    [External, External, External, External],
];

/// The orientation of a shape of orientation `or2` inside a shape of orientation
/// `or1` (`TopAbs::Compose`).
///
/// The native table is indexed `[Or2][Or1]`, so an INTERNAL or EXTERNAL `or2` wins
/// over any `or1` (`compose(External, Internal) == Internal`), unlike the table in
/// the header comment; `compose(Reversed, Reversed) == Forward`.
pub fn compose(or1: Orientation, or2: Orientation) -> Orientation {
    COMPOSE[or2 as usize][or1 as usize]
}

/// Exchange FORWARD and REVERSED, keep INTERNAL and EXTERNAL (`TopAbs::Reverse`);
/// `reverse(Forward) == Reversed`.
pub fn reverse(orientation: Orientation) -> Orientation {
    [Reversed, Forward, Internal, External][orientation as usize]
}

/// Exchange FORWARD and REVERSED, and INTERNAL and EXTERNAL: inside becomes outside
/// (`TopAbs::Complement`); `complement(Internal) == External`.
pub fn complement(orientation: Orientation) -> Orientation {
    [Reversed, Forward, External, Internal][orientation as usize]
}

/// The upper-case name of a shape type (`TopAbs::ShapeTypeToString`);
/// `shape_type_to_string(ShapeEnum::Face) == "FACE"`.
pub fn shape_type_to_string(shape_type: ShapeEnum) -> &'static str {
    SHAPE_TYPE_NAMES[shape_type as usize]
}

/// The upper-case name of an orientation (`TopAbs::ShapeOrientationToString`);
/// `shape_orientation_to_string(Orientation::External) == "EXTERNAL"`.
pub fn shape_orientation_to_string(orientation: Orientation) -> &'static str {
    ORIENTATION_NAMES[orientation as usize]
}

// `TCollection_AsciiString aName(text); aName.UpperCase();`: the C string ends at the
// first NUL and only ASCII letters change case.
fn upper_case_name(text: &str) -> AsciiString {
    let mut name: AsciiString = AsciiString::new(text);
    name.upper_case();
    name
}

/// The shape type named `text`, compared case-insensitively, or `None` when the name
/// is unknown (`bool TopAbs::ShapeTypeFromString(const char*, TopAbs_ShapeEnum&)`,
/// which then leaves its output unchanged); `try_shape_type_from_string("edge")`.
pub fn try_shape_type_from_string(text: &str) -> Option<ShapeEnum> {
    let name: AsciiString = upper_case_name(text);
    ShapeEnum::ALL
        .into_iter()
        .find(|&shape_type| name == shape_type_to_string(shape_type))
}

/// The shape type named `text`, or SHAPE when the name is unknown
/// (`TopAbs::ShapeTypeFromString(const char*)`);
/// `shape_type_from_string("Solid") == ShapeEnum::Solid`.
pub fn shape_type_from_string(text: &str) -> ShapeEnum {
    try_shape_type_from_string(text).unwrap_or(ShapeEnum::Shape)
}

/// The orientation named `text`, compared case-insensitively, or `None` when the name
/// is unknown (`bool TopAbs::ShapeOrientationFromString(const char*, TopAbs_Orientation&)`);
/// `try_shape_orientation_from_string("internal")`.
pub fn try_shape_orientation_from_string(text: &str) -> Option<Orientation> {
    let name: AsciiString = upper_case_name(text);
    Orientation::ALL
        .into_iter()
        .find(|&orientation| name == shape_orientation_to_string(orientation))
}

/// The orientation named `text`, or FORWARD when the name is unknown
/// (`TopAbs::ShapeOrientationFromString(const char*)`);
/// `shape_orientation_from_string("reversed") == Orientation::Reversed`.
pub fn shape_orientation_from_string(text: &str) -> Orientation {
    try_shape_orientation_from_string(text).unwrap_or(Forward)
}

/// Write the name of a shape type (`TopAbs::Print(TopAbs_ShapeEnum, Standard_OStream&)`);
/// `print_shape_type(ShapeEnum::Wire, &mut out)?`.
pub fn print_shape_type(shape_type: ShapeEnum, stream: &mut impl Write) -> io::Result<()> {
    stream.write_all(shape_type_to_string(shape_type).as_bytes())
}

/// Write the name of an orientation (`TopAbs::Print(TopAbs_Orientation, Standard_OStream&)`);
/// `print_orientation(Orientation::Forward, &mut out)?`.
pub fn print_orientation(orientation: Orientation, stream: &mut impl Write) -> io::Result<()> {
    stream.write_all(shape_orientation_to_string(orientation).as_bytes())
}

/// Write the name of a state (`TopAbs::Print(TopAbs_State, Standard_OStream&)`).
///
/// The native name table is shifted: IN prints "ON", OUT prints "IN", ON prints
/// "OUT" and UNKNOWN prints "UNKNOWN"; `print_state(State::In, &mut out)?` writes "ON".
pub fn print_state(state: State, stream: &mut impl Write) -> io::Result<()> {
    stream.write_all(STATE_NAMES[state as usize].as_bytes())
}
