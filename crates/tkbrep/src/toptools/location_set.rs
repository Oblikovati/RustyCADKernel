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
// Rust translation of ModelingData/TKBRep/TopTools/TopTools_LocationSet.hxx and
// TopTools_LocationSet.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopTools_LocationSet`: the indexed `Locations` table of the BRep ASCII format.
use super::location_text::{ChainStyle, ReadState, read_location, write_chain, write_trsf};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkgeombase::geom_tools::{ConsoleOutput, StdoutConsole};
use rustycad_tkmath::toploc::Location;
use std::sync::Arc;

/// `char buffer[255]` of `Read`, which extracts the table tag with `IS >> buffer`.
const TAG_BUFFER: usize = 255;

/// A set of locations, each complex location stored after the elementary locations
/// (datums) it is made of, so that `Write` can refer to them by index.
///
/// ```
/// use rustycad_tkbrep::toptools::LocationSet;
/// use rustycad_tkmath::gp::Transform3;
/// use rustycad_tkmath::toploc::Location;
/// let mut set = LocationSet::new();
/// let location = Location::new(Transform3::translation([1.0, 2.0, 3.0]));
/// assert_eq!(set.add(&location), 1);
/// assert_eq!(set.add(&Location::default()), 0);
/// assert!(set.location(1).is_equal(&location));
/// ```
pub struct LocationSet {
    map: IndexedMap<Location>,
    console: Arc<dyn ConsoleOutput>,
}

impl Default for LocationSet {
    fn default() -> Self {
        Self::new()
    }
}

impl LocationSet {
    /// `TopTools_LocationSet()`: an empty set whose `Read` reports on standard output;
    /// `LocationSet::new()`.
    pub fn new() -> Self {
        Self::with_console(Arc::new(StdoutConsole))
    }

    /// An empty set whose `Read` writes its "Not a location table" line to `console`
    /// (the native `std::cout`); `LocationSet::with_console(Arc::new(StdoutConsole))`.
    pub fn with_console(console: Arc<dyn ConsoleOutput>) -> Self {
        Self {
            map: IndexedMap::default(),
            console,
        }
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

    /// `Location(I)`: the location of index `index`, the identity outside
    /// `1..=Extent`; `set.location(1)`.
    pub fn location(&self, index: i32) -> Location {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.map.find_key(index).ok())
            .cloned()
            .unwrap_or_default()
    }

    /// `Index(L)`: the index of `location`, 0 for the identity or an unknown location;
    /// `set.index(&location)`.
    pub fn index(&self, location: &Location) -> i32 {
        if location.is_identity() {
            return 0;
        }
        self.map.find_index(location) as i32
    }

    /// The number of stored locations (`myMap.Extent()`); `set.extent()`.
    pub fn extent(&self) -> i32 {
        self.map.extent() as i32
    }

    /// `Dump(OS)`: a header, then each location with its composition and its matrix at
    /// the stream precision; `set.dump(&mut stream)`.
    pub fn dump(&self, stream: &mut IoStream) {
        let count = self.extent();
        stream.insert_bytes(b"\n\n\n -------\n Dump of ");
        stream.insert_integer(i64::from(count));
        stream.insert_bytes(b" Locations\n -------\n\n");
        for index in 1..=count {
            let location = self.location(index);
            stream.set_width(5);
            stream.insert_integer(i64::from(index));
            stream.insert_bytes(b" : \n");
            if is_elementary(&location) {
                stream.insert_bytes(b"Elementary location\n");
            } else {
                stream.insert_bytes(b"Complex :");
                write_chain(stream, &location, &self.map, ChainStyle::Dump);
                stream.insert_bytes(b"\n");
            }
            write_trsf(stream, &location.transformation(), false);
        }
    }

    /// `Write(OS, theProgress)`: `Locations n`, then each location as `1` and its matrix
    /// or `2` and its datum indices and powers, at precision 15 (the stream precision is
    /// restored); stops early when the progress indicator breaks;
    /// `set.write(&mut stream, &ProgressRange::default())`.
    pub fn write(&self, stream: &mut IoStream, progress: &ProgressRange<'_>) {
        let previous = stream.precision();
        stream.set_precision(15);
        let count = self.extent();
        stream.insert_bytes(b"Locations ");
        stream.insert_integer(i64::from(count));
        stream.insert_bytes(b"\n");
        let scope = ProgressScope::with_literal(progress, "Locations", f64::from(count), false);
        let mut index = 1;
        while index <= count && scope.more() {
            self.write_location(stream, &self.location(index));
            index += 1;
            drop(scope.next(1.0));
        }
        stream.set_precision(previous);
    }

    fn write_location(&self, stream: &mut IoStream, location: &Location) {
        if is_elementary(location) {
            stream.insert_bytes(b"1\n");
            write_trsf(stream, &location.transformation(), true);
            return;
        }
        stream.insert_bytes(b"2 ");
        write_chain(stream, location, &self.map, ChainStyle::Write);
        stream.insert_bytes(b" 0\n");
    }

    /// `Read(IS, theProgress)`: clears the set, then reads a `Locations n` table; another
    /// tag writes "Not a location table" to the console and reads nothing. A datum
    /// index outside the table raises Standard_OutOfRange and a singular matrix
    /// Standard_ConstructionError, as natively; `set.read(&mut stream, &ProgressRange::default())?`.
    pub fn read(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        self.map.clear(true);
        if stream.extract_word(TAG_BUFFER) != b"Locations" {
            self.console.write(b"Not a location table \n");
            return Ok(());
        }
        let mut count = 0;
        stream.extract_int(&mut count);
        let scope = ProgressScope::with_literal(progress, "Locations", f64::from(count), false);
        let (mut location, mut state) = (Location::default(), ReadState::default());
        let mut index = 1;
        while index <= count && scope.more() {
            location = read_location(stream, &self.map, &mut state, location)?;
            if !location.is_identity() {
                self.map.add(location.clone());
            }
            index += 1;
            drop(scope.next(1.0));
        }
        Ok(())
    }
}

/// `simple && p == 1`: one datum at power 1.
fn is_elementary(location: &Location) -> bool {
    location.next_location().is_identity() && matches!(location.first_power(), Ok(1))
}
