// Created on: 2004-06-15
// Created by: Sergey ZARITCHNY <szy@opencascade.com>
// Copyright (c) 2004-2014 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/BinTools/BinTools_LocationSet.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BinTools_LocationSet`: the indexed `Locations` table of the binary shape set.
use super::errors::{byte_text, index_out_of_range, wrap_failure};
use super::ostream::transform_values;
use super::scalars::{get_integer, get_real, put_byte, put_integer, put_real, stream_get};
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkmath::gp::Transform3;
use rustycad_tkmath::toploc::Location;

/// `char buffer[255]` of `Read`, which extracts the table tag with `IS >> buffer`.
const TAG_BUFFER: usize = 255;

/// `operator<<(Standard_OStream&, const gp_Trsf&)`: the twelve reals of the
/// transformation, row by row with the translation last, written with `PutReal`.
///
/// ```
/// use rustycad_tkernel::storage::IoStream;
/// use rustycad_tkmath::gp::Transform3;
/// let mut stream = IoStream::from_bytes(Vec::new());
/// rustycad_tkbrep::bintools::put_transform(&mut stream, &Transform3::default());
/// assert_eq!(stream.tellp(), 96);
/// ```
pub fn put_transform<'s>(stream: &'s mut IoStream, transform: &Transform3) -> &'s mut IoStream {
    for value in transform_values(transform) {
        put_real(stream, value);
    }
    stream
}

/// The static `operator>>(Standard_IStream&, gp_Trsf&)`: twelve `GetReal`, then
/// `SetValues`.
fn get_transform(stream: &mut IoStream) -> Result<Transform3, Failure> {
    let mut rows = [[0.0; 4]; 3];
    for row in &mut rows {
        for value in row.iter_mut() {
            *value = get_real(stream)?;
        }
    }
    let mut transform = Transform3::default();
    transform.set_values(rows)?;
    Ok(transform)
}

/// A set of locations, each complex location stored after the elementary locations
/// (datums) it is made of, so that `Write` can refer to them by index.
///
/// ```
/// use rustycad_tkbrep::bintools::BinLocationSet;
/// use rustycad_tkmath::gp::Transform3;
/// use rustycad_tkmath::toploc::Location;
/// let mut set = BinLocationSet::new();
/// let location = Location::new(Transform3::translation([1.0, 2.0, 3.0]));
/// assert_eq!(set.add(&location), 1);
/// assert_eq!(set.index(&Location::default()), 0);
/// ```
#[derive(Default)]
pub struct BinLocationSet {
    map: IndexedMap<Location>,
}

impl BinLocationSet {
    /// `BinTools_LocationSet()`: an empty set; `BinLocationSet::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Clear()`; `set.clear()`.
    pub fn clear(&mut self) {
        self.map.clear(true);
    }

    /// `Add(L)`: the index of `location`, 0 for the identity. A new location is stored
    /// after each of its datums (as elementary locations); `set.add(&location)`.
    pub fn add(&mut self, location: &Location) -> i32 {
        if location.is_identity() {
            return 0;
        }
        let index = self.map.find_index(location);
        if index > 0 {
            return index as i32;
        }
        let mut rest = location.clone();
        while let Ok(datum) = rest.first_datum() {
            self.map.add(Location::from_datum(datum.clone()));
            rest = rest.next_location().clone();
        }
        self.map.add(location.clone()) as i32
    }

    /// `Location(I)`: the identity for 0, else the location of index `index`;
    /// Standard_OutOfRange outside `1..=NbLocations` (`myMap(I)`); `set.location(1)?`.
    pub fn location(&self, index: i32) -> Result<Location, Failure> {
        if index == 0 {
            return Ok(Location::default());
        }
        usize::try_from(index)
            .ok()
            .and_then(|position| self.map.find_key(position).ok())
            .cloned()
            .ok_or_else(|| {
                index_out_of_range("BinTools_LocationSet::Location", index, self.map.extent())
            })
    }

    /// `Index(L)`: the index of `location`, 0 for the identity or an unknown location;
    /// `set.index(&location)`.
    pub fn index(&self, location: &Location) -> i32 {
        if location.is_identity() {
            return 0;
        }
        self.map.find_index(location) as i32
    }

    /// `NbLocations()`; `set.nb_locations()`.
    pub fn nb_locations(&self) -> i32 {
        self.map.extent() as i32
    }

    /// `Write(OS)`: `Locations n\n`, then each location as the byte 1 and its matrix,
    /// or the byte 2, its `datum index, power` integer pairs and a closing 0;
    /// `set.write(&mut stream)?`.
    pub fn write(&self, stream: &mut IoStream) -> Result<(), Failure> {
        let count = self.nb_locations();
        stream.insert_bytes(b"Locations ");
        stream.insert_integer(i64::from(count)).insert_bytes(b"\n");
        wrap_failure("EXCEPTION in BinTools_LocatioSet::Write(..)", || {
            for index in 1..=count {
                self.write_location(stream, &self.location(index)?)?;
            }
            Ok(())
        })
    }

    fn write_location(&self, stream: &mut IoStream, location: &Location) -> Result<(), Failure> {
        let next = location.next_location();
        let power = location.first_power()?;
        if next.is_identity() && power == 1 {
            put_byte(stream, 1);
            put_transform(stream, &location.transformation());
            return Ok(());
        }
        put_byte(stream, 2);
        self.write_datum(stream, location)?;
        let mut rest = next.clone();
        while !rest.is_identity() {
            self.write_datum(stream, &rest)?;
            rest = rest.next_location().clone();
        }
        put_integer(stream, 0);
        Ok(())
    }

    /// The index of the first datum of `chain` and its power.
    fn write_datum(&self, stream: &mut IoStream, chain: &Location) -> Result<(), Failure> {
        let datum = Location::from_datum(chain.first_datum()?.clone());
        put_integer(stream, self.map.find_index(&datum) as i32);
        put_integer(stream, chain.first_power()?);
        Ok(())
    }

    /// `Read(IS)`: clears the set and reads a `Locations n` table. Another tag raises
    /// Standard_Failure ("Not a location table"); a record type other than 1 or 2, a
    /// short read, a datum index outside the table or a singular matrix raise a
    /// Standard_Failure that wraps the cause; `set.read(&mut stream)?`.
    pub fn read(&mut self, stream: &mut IoStream) -> Result<(), Failure> {
        self.map.clear(true);
        if stream.extract_word(TAG_BUFFER) != b"Locations" || stream.fail() {
            return Err(Failure::new(
                "BinTools_LocationSet::Read: Not a location table\n",
            ));
        }
        // Natively `nbLoc` is uninitialised when the extraction cannot start (the tag
        // ended the stream); the port reads 0.
        let mut count = 0;
        stream.extract_int(&mut count);
        stream_get(stream);
        wrap_failure("EXCEPTION in BinTools_LocationSet::Read(..)", || {
            for _ in 1..=count {
                let location = self.read_location(stream)?;
                if !location.is_identity() {
                    self.map.add(location);
                }
            }
            Ok(())
        })
    }

    /// One record: the type byte, then a matrix or a datum list.
    fn read_location(&self, stream: &mut IoStream) -> Result<Location, Failure> {
        let kind = stream_get(stream) as u8;
        match kind {
            1 => Ok(Location::new(get_transform(stream)?)),
            2 => self.read_chain(stream),
            _ => Err(Failure::new(&format!(
                "Unexpected location's type = {}\n",
                byte_text(kind)
            ))),
        }
    }

    /// `L = L1.Powered(p) * L` for each `index power` pair until a zero index.
    fn read_chain(&self, stream: &mut IoStream) -> Result<Location, Failure> {
        let mut location = Location::default();
        let mut datum = get_integer(stream)?;
        while datum != 0 {
            let power = get_integer(stream)?;
            // `myMap(l1)`: the loop stops at 0, so every index here goes to the map.
            let base = self.location(datum)?;
            location = base.powered(power)?.multiplied(&location)?;
            datum = get_integer(stream)?;
        }
        Ok(location)
    }
}
