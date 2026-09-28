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
// Rust translation of TopTools_ShapeSet::Clear, Dump and Write and of the shape
// records Dump(S, OS), Write(S, OS) and Read(S, IS, nbshapes)
// (ModelingData/TKBRep/TopTools/TopTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The base bodies of the virtual `Clear`, `Dump` and `Write` of `TopTools_ShapeSet`,
//! which a derived set calls to run the base class member; `Read` is in
//! `shape_set_read.rs`.
use super::IndexedMapOfShape;
use super::shape_set::{ShapeSet, ShapeSetData, THE_ASCII_VERSIONS};
use super::shape_set_text::{c_atoi, print_orientation, print_shape_enum};
use crate::topods::{Shape, ShapeIterator};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

pub use super::shape_set_read::read;

/// `char buffer[255]` of the shape readers.
pub(super) const WORD_BUFFER: usize = 255;

/// `TopTools_ShapeSet::Clear()`: empties the shapes and the locations; `clear(data)`.
pub fn clear(data: &mut ShapeSetData) {
    data.shapes.clear(true);
    data.locations.clear();
}

/// `myShapes(I)`: Standard_OutOfRange outside `1..=Extent`.
pub(super) fn shape_at(shapes: &IndexedMapOfShape, index: i32) -> Result<Shape, Failure> {
    let position = usize::try_from(index).unwrap_or(0);
    shapes.find_key(position).cloned().map_err(|_| {
        Failure::out_of_range(&format!(
            "TopTools_ShapeSet::Shape: index {index}; expected 1..={}",
            shapes.extent()
        ))
    })
}

/// The index of `shape` at the identity location (`FindIndex(S.Located(Location()))`).
fn unlocated_index(data: &ShapeSetData, shape: &Shape) -> Result<i32, Failure> {
    let unlocated = shape.located(&Location::default(), false)?;
    Ok(data.shapes.find_index(&unlocated) as i32)
}

/// The eight flags of `Dump` (seven, without Locked, for `Write`), as digits.
fn write_flags(stream: &mut IoStream, shape: &Shape, with_locked: bool) -> Result<(), Failure> {
    let mut flags = vec![
        shape.free()?,
        shape.modified()?,
        shape.checked()?,
        shape.orientable()?,
        shape.closed()?,
        shape.infinite()?,
        shape.convex()?,
    ];
    if with_locked {
        flags.push(shape.locked()?);
    }
    for flag in flags {
        stream.insert_bytes(if flag { b"1" } else { b"0" });
    }
    Ok(())
}

/// `TopTools_ShapeSet::Dump(OS)`: the TShapes from last to first with their type,
/// flags, address and sub-shapes, then the geometry and the locations;
/// `shape_set_io::dump(&set, &mut stream)?`.
pub fn dump<S: ShapeSet + ?Sized>(set: &S, stream: &mut IoStream) -> Result<(), Failure> {
    let data = set.shape_set_data();
    let count = data.shapes.extent() as i32;
    stream.insert_bytes(b"\nDump of ");
    stream.insert_integer(i64::from(count));
    stream.insert_bytes(b" TShapes\n\n-----------------\n\n");
    stream.insert_bytes(
        b"Flags : Free, Modified, Checked, Orientable, Closed, Infinite, Convex, Locked\n\n",
    );
    for index in (1..=count).rev() {
        let shape = shape_at(&data.shapes, index)?;
        dump_tshape(data, &shape, count - index + 1, stream)?;
        set.dump_shape_geometry(&shape, stream)?;
    }
    set.dump_geometry(stream)?;
    data.locations.dump(stream);
    stream.insert_bytes(b"\n");
    Ok(())
}

/// One `TShape # n` entry of `Dump(OS)`.
fn dump_tshape(
    data: &ShapeSetData,
    shape: &Shape,
    number: i32,
    stream: &mut IoStream,
) -> Result<(), Failure> {
    stream.insert_bytes(b"TShape # ");
    stream.insert_integer(i64::from(number));
    stream.insert_bytes(b" : ");
    print_shape_enum(shape.shape_type()?, stream, false);
    stream.insert_bytes(b" ");
    write_flags(stream, shape, true)?;
    // `OS << (void*)&(*S.TShape())`: libstdc++ prints a pointer as 0x and lowercase hex.
    stream.insert_bytes(format!(" {:#x}\n    ", shape.tshape_address()).as_bytes());
    dump_sub_shapes(data, shape, stream)
}

/// The sub-shapes line of a `Dump(OS)` entry: orientation code, reversed index and
/// `(Ln)` for a located sub-shape.
fn dump_sub_shapes(
    data: &ShapeSetData,
    shape: &Shape,
    stream: &mut IoStream,
) -> Result<(), Failure> {
    let count = data.shapes.extent() as i32;
    let mut iterator = ShapeIterator::new(shape, false, false);
    while let Ok(child) = iterator.value() {
        print_orientation(child.orientation(), stream, true);
        stream.insert_integer(i64::from(count - unlocated_index(data, child)? + 1));
        if !child.location().is_identity() {
            stream.insert_bytes(b"(L");
            stream.insert_integer(i64::from(data.locations.index(child.location())));
            stream.insert_bytes(b")");
        }
        stream.insert_bytes(b" ");
        iterator.next();
    }
    stream.insert_bytes(b"\n");
    Ok(())
}

/// `TopTools_ShapeSet::Write(OS, theProgress)`: the version line, the locations, the
/// geometry and the TShapes (sub-shapes first) at precision 15. When the progress
/// indicator breaks, "Interrupted by the user" is written and, on the two early
/// returns, the precision is not restored (native behaviour);
/// `shape_set_io::write(&mut set, &mut stream, &ProgressRange::default())?`.
pub fn write<S: ShapeSet + ?Sized>(
    set: &mut S,
    stream: &mut IoStream,
    progress: &ProgressRange<'_>,
) -> Result<(), Failure> {
    let previous = stream.precision();
    stream.set_precision(15);
    let version = THE_ASCII_VERSIONS[set.format_nb() as usize];
    stream.insert_bytes(format!("\n{version}\n").as_bytes());
    let scope = ProgressScope::with_literal(progress, "Writing", 100.0, false);
    if !write_sections(set, stream, &scope)? {
        return Ok(());
    }
    stream.insert_bytes(b"\n");
    stream.set_precision(previous);
    interrupted(&scope, stream);
    Ok(())
}

/// The locations, the geometry and the TShapes; false when a user break stopped the
/// write early (the caller then leaves the precision at 15, as natively).
fn write_sections<S: ShapeSet + ?Sized>(
    set: &mut S,
    stream: &mut IoStream,
    scope: &ProgressScope<'_>,
) -> Result<bool, Failure> {
    set.shape_set_data()
        .locations
        .write(stream, &scope.next(10.0));
    if interrupted(scope, stream) {
        return Ok(false);
    }
    set.write_geometry(stream, &scope.next(75.0))?;
    if interrupted(scope, stream) {
        return Ok(false);
    }
    write_tshapes(set, stream, &scope.next(15.0))?;
    Ok(true)
}

/// `if (aPS.UserBreak()) { OS << "Interrupted by the user\n"; return; }`.
fn interrupted(scope: &ProgressScope<'_>, stream: &mut IoStream) -> bool {
    if !scope.user_break() {
        return false;
    }
    stream.insert_bytes(b"Interrupted by the user\n");
    true
}

/// The `TShapes n` table of `Write`: each TShape with its type code, its geometry,
/// seven flags and its sub-shapes (ten per line), ended by `*`.
fn write_tshapes<S: ShapeSet + ?Sized>(
    set: &S,
    stream: &mut IoStream,
    progress: &ProgressRange<'_>,
) -> Result<(), Failure> {
    let data = set.shape_set_data();
    let count = data.shapes.extent() as i32;
    stream.insert_bytes(format!("\nTShapes {count}\n").as_bytes());
    let scope = ProgressScope::with_literal(progress, "Shapes", f64::from(count), false);
    let mut index = 1;
    while index <= count && scope.more() {
        write_tshape(set, &shape_at(&data.shapes, index)?, stream)?;
        index += 1;
        drop(scope.next(1.0));
    }
    Ok(())
}

/// One record of the `TShapes` table.
fn write_tshape<S: ShapeSet + ?Sized>(
    set: &S,
    shape: &Shape,
    stream: &mut IoStream,
) -> Result<(), Failure> {
    print_shape_enum(shape.shape_type()?, stream, true);
    stream.insert_bytes(b"\n");
    set.write_shape_geometry(shape, stream)?;
    stream.insert_bytes(b"\n");
    write_flags(stream, shape, false)?;
    stream.insert_bytes(b"\n");
    write_sub_shapes(set.shape_set_data(), shape, stream)
}

fn write_sub_shapes(
    data: &ShapeSetData,
    shape: &Shape,
    stream: &mut IoStream,
) -> Result<(), Failure> {
    let mut on_line = 0;
    let mut iterator = ShapeIterator::new(shape, false, false);
    while let Ok(child) = iterator.value() {
        write_shape(data, child, stream)?;
        on_line += 1;
        if on_line == 10 {
            stream.insert_bytes(b"\n");
            on_line = 0;
        }
        iterator.next();
    }
    write_shape(data, &Shape::default(), stream)?;
    stream.insert_bytes(b"\n");
    Ok(())
}

/// `Dump(S, OS)`: "Null shape" for a null shape (and the entry still follows), then
/// the index at the identity location, the orientation and the location index.
pub(super) fn dump_shape(
    data: &ShapeSetData,
    shape: &Shape,
    stream: &mut IoStream,
) -> Result<(), Failure> {
    if shape.is_null() {
        stream.insert_bytes(b"Null shape\n");
    }
    stream.insert_bytes(b"Shape : ");
    stream.insert_integer(i64::from(unlocated_index(data, shape)?));
    stream.insert_bytes(b", ");
    print_orientation(shape.orientation(), stream, false);
    if !shape.location().is_identity() {
        stream.insert_bytes(b", location : ");
        stream.insert_integer(i64::from(data.locations.index(shape.location())));
    }
    stream.insert_bytes(b"\n");
    Ok(())
}

/// `Write(S, OS)`: `*`, or the orientation code, `Extent - index + 1` and the
/// location index.
pub(super) fn write_shape(
    data: &ShapeSetData,
    shape: &Shape,
    stream: &mut IoStream,
) -> Result<(), Failure> {
    if shape.is_null() {
        stream.insert_bytes(b"*");
        return Ok(());
    }
    print_orientation(shape.orientation(), stream, true);
    let count = data.shapes.extent() as i32;
    stream.insert_integer(i64::from(count - unlocated_index(data, shape)? + 1));
    stream.insert_bytes(b" ");
    stream.insert_integer(i64::from(data.locations.index(shape.location())));
    stream.insert_bytes(b" ");
    Ok(())
}

/// The private `Read(S, IS, nbshapes)`: null when the set is empty or the word is `*`;
/// else the shape `nbshapes - k + 1` for the word `ok`, oriented by the code `o`
/// (another code keeps the stored orientation), at the location read next. A word
/// that cannot be read gives a null shape, where the native buffer is uninitialised.
pub(super) fn read_shape_among(
    data: &ShapeSetData,
    stream: &mut IoStream,
    count: i32,
) -> Result<Shape, Failure> {
    if count < 1 {
        return Ok(Shape::default());
    }
    let word = stream.extract_word(WORD_BUFFER);
    let Some(&code) = word.first() else {
        return Ok(Shape::default());
    };
    if code == b'*' {
        return Ok(Shape::default());
    }
    located_reference(data, stream, &word, count)
}

/// The shape `count - k + 1` of the word `ok`, oriented by `o` and at the location
/// whose index follows.
fn located_reference(
    data: &ShapeSetData,
    stream: &mut IoStream,
    word: &[u8],
    count: i32,
) -> Result<Shape, Failure> {
    let mut shape = shape_at(&data.shapes, count - c_atoi(&word[1..]) + 1)?;
    if let Some(orientation) = orientation_code(word[0]) {
        shape.set_orientation(orientation);
    }
    let mut location_index = 0;
    stream.extract_int(&mut location_index);
    shape.set_location(&data.locations.location(location_index), false)?;
    Ok(shape)
}

fn orientation_code(code: u8) -> Option<Orientation> {
    match code {
        b'+' => Some(Orientation::Forward),
        b'-' => Some(Orientation::Reversed),
        b'i' => Some(Orientation::Internal),
        b'e' => Some(Orientation::External),
        _ => None,
    }
}
