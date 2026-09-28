// Created on: 1993-07-16
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
// Rust translation of the static WriteTrsf and ReadTrsf and of the location records
// of ModelingData/TKBRep/TopTools/TopTools_LocationSet.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The text of one location in the `Locations` table.
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkgeombase::geom_tools::get_real;
use rustycad_tkmath::gp::Transform3;
use rustycad_tkmath::toploc::Location;

/// How a complex location lists its datums.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ChainStyle {
    /// `Dump`: `Complex : L1^2 * L3`, powers only when not 1.
    Dump,
    /// `Write`: ` 1 2 3 1`, every index followed by its power.
    Write,
}

/// The datum indices and powers of `location`, head first, in `style`.
pub(super) fn write_chain(
    stream: &mut IoStream,
    location: &Location,
    map: &IndexedMap<Location>,
    style: ChainStyle,
) {
    let mut rest = location;
    let mut first = true;
    while let (Ok(datum), Ok(power)) = (rest.first_datum(), rest.first_power()) {
        let index = map.find_index(&Location::from_datum(datum.clone()));
        write_item(stream, index, power, style, first);
        first = false;
        rest = rest.next_location();
    }
}

fn write_item(stream: &mut IoStream, index: usize, power: i32, style: ChainStyle, first: bool) {
    let prefix: &[u8] = match (style, first) {
        (ChainStyle::Dump, true) => b" L",
        (ChainStyle::Dump, false) => b" * L",
        (ChainStyle::Write, _) => b" ",
    };
    stream.insert_bytes(prefix);
    stream.insert_integer(index as i64);
    if style == ChainStyle::Write {
        stream.insert_bytes(b" ");
        stream.insert_integer(i64::from(power));
    } else if power != 1 {
        stream.insert_bytes(b"^");
        stream.insert_integer(i64::from(power));
    }
}

/// `WriteTrsf(T, OS, compact)`: three rows of the vectorial part and the translation,
/// each number right-aligned in 15 columns; the dump (`compact` false) brackets rows.
pub(super) fn write_trsf(stream: &mut IoStream, transform: &Transform3, compact: bool) {
    let matrix = transform.vectorial_part();
    let translation = transform.translation_part();
    let offsets = [translation.x(), translation.y(), translation.z()];
    for (row, offset) in matrix.coefficients().iter().zip(offsets) {
        if !compact {
            stream.insert_bytes(b"( ");
        }
        for value in row.iter().copied().chain([offset]) {
            stream.set_width(15);
            stream.insert_real(value);
            stream.insert_bytes(b" ");
        }
        if !compact {
            stream.insert_bytes(b" )");
        }
        stream.insert_bytes(b"\n");
    }
}

/// The variables of `Read` that live across the records: `l1` and `p` are declared
/// once, and a failed extraction leaves them unchanged.
#[derive(Default)]
pub(super) struct ReadState {
    datum: i32,
    power: i32,
}

/// One record of `Read`: `1` and a matrix (a new datum), `2` and `index power` pairs
/// ending with `0`, or any other type, which keeps `previous`.
pub(super) fn read_location(
    stream: &mut IoStream,
    map: &IndexedMap<Location>,
    state: &mut ReadState,
    previous: Location,
) -> Result<Location, Failure> {
    let mut kind = 0;
    stream.extract_int(&mut kind);
    match kind {
        1 => Ok(Location::new(read_trsf(stream)?)),
        2 => read_complex(stream, map, state),
        _ => Ok(previous),
    }
}

/// `L = L1.Powered(p) * L` for each pair. Natively a stream that fails inside the
/// list leaves `l1` unchanged and loops forever; the port stops at the failure.
fn read_complex(
    stream: &mut IoStream,
    map: &IndexedMap<Location>,
    state: &mut ReadState,
) -> Result<Location, Failure> {
    let mut location = Location::default();
    stream.extract_int(&mut state.datum);
    while state.datum != 0 && !stream.fail() {
        stream.extract_int(&mut state.power);
        let index = usize::try_from(state.datum).unwrap_or(0);
        let datum = map
            .find_key(index)
            .map_err(|_| out_of_range(state.datum, map))?;
        location = datum.powered(state.power)?.multiplied(&location)?;
        stream.extract_int(&mut state.datum);
    }
    Ok(location)
}

/// The Standard_OutOfRange of `myMap(l1)`.
fn out_of_range(index: i32, map: &IndexedMap<Location>) -> Failure {
    Failure::out_of_range(&format!(
        "NCollection_IndexedMap::FindKey: index {index}; expected 1..={}",
        map.extent()
    ))
}

/// `ReadTrsf(T, IS)`: twelve `GetReal`, row by row, then `SetValues`.
fn read_trsf(stream: &mut IoStream) -> Result<Transform3, Failure> {
    let mut rows = [[0.0; 4]; 3];
    for row in &mut rows {
        for value in row.iter_mut() {
            *value = get_real(stream);
        }
    }
    let mut transform = Transform3::default();
    transform.set_values(rows)?;
    Ok(transform)
}
