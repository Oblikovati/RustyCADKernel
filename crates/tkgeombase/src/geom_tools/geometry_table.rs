// Created on: 1993-07-19
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
// Rust translation of the members GeomTools_CurveSet.cxx, GeomTools_Curve2dSet.cxx and
// GeomTools_SurfaceSet.cxx repeat (Clear, Add, the indexed getter, Index, Dump, Write,
// Read) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The indexed table of geometry handles behind the three GeomTools sets. The native
//! classes differ only in their labels, their element type and whether `Add` and
//! `Index` accept a null handle, so the port writes the members once.
use super::console::ConsoleOutput;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::{HashPolicy, IndexedMap};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkernel::storage::IoStream;
use std::fmt;
use std::sync::Arc;

/// `char buffer[255]` of the set readers, which extract the table tag with `IS >> buffer`.
const TAG_BUFFER: usize = 255;

/// The labels one set class writes and reads.
pub(crate) struct TableLabels {
    /// `Dump of N <plural> `.
    pub(crate) dump_plural: &'static [u8],
    /// The tag `Write` puts before the count and `Read` expects (`Curves`).
    pub(crate) tag: &'static [u8],
    /// The Message_ProgressScope name of Write and Read (`3D Curves`).
    pub(crate) scope: &'static str,
    /// The console line of a Read that finds another tag (`Not a Curve table`).
    pub(crate) not_a_table: &'static [u8],
}

/// Prints one element (`PrintCurve`, `PrintCurve2d`, `PrintSurface`).
pub(crate) type ElementPrinter<T> = fn(&Handle<T>, &mut IoStream, bool) -> Result<(), Failure>;
/// Reads one element (`ReadCurve`, `ReadCurve2d`, `ReadSurface`).
pub(crate) type ElementReader<T> = fn(&mut IoStream) -> Handle<T>;

/// A key of `NCollection_IndexedMap<occ::handle<Standard_Transient>>`: object identity.
struct TableKey<T: ?Sized + RuntimeTyped>(Handle<T>);

impl<T: ?Sized + RuntimeTyped> fmt::Debug for TableKey<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "TableKey({:#x})", self.0.hash_code())
    }
}

/// `NCollection_DefaultHasher` of a handle: the object address; null is a valid key.
#[derive(Clone, Copy, Debug, Default)]
struct IdentityHasher;

impl<T: ?Sized + RuntimeTyped> HashPolicy<TableKey<T>> for IdentityHasher {
    fn hash(&self, key: &TableKey<T>) -> usize {
        key.0.hash_code()
    }
    fn equal(&self, first: &TableKey<T>, second: &TableKey<T>) -> bool {
        first.0 == second.0
    }
}

/// The handles of one set, indexed from 1 in insertion order.
pub(crate) struct GeometryTable<T: ?Sized + RuntimeTyped> {
    map: IndexedMap<TableKey<T>, IdentityHasher>,
    console: Arc<dyn ConsoleOutput>,
    labels: &'static TableLabels,
}

impl<T: ?Sized + RuntimeTyped> GeometryTable<T> {
    /// An empty table that reports on `console`.
    pub(crate) fn new(labels: &'static TableLabels, console: Arc<dyn ConsoleOutput>) -> Self {
        Self {
            map: IndexedMap::default(),
            console,
            labels,
        }
    }

    /// `myMap.Clear()`: keeps the bucket storage, as the native default does.
    pub(crate) fn clear(&mut self) {
        self.map.clear(false);
    }

    /// `myMap.Add(handle)`: the index of the object, a null handle included.
    pub(crate) fn add(&mut self, handle: &Handle<T>) -> i32 {
        self.map.add(TableKey(handle.clone())) as i32
    }

    /// The handle of index `index`, null outside `1..=Extent()`.
    pub(crate) fn get(&self, index: i32) -> Handle<T> {
        if index <= 0 || index as usize > self.map.extent() {
            return Handle::default();
        }
        self.map[index as usize].0.clone()
    }

    /// `myMap.FindIndex(handle)`: 0 when absent.
    pub(crate) fn index(&self, handle: &Handle<T>) -> i32 {
        self.map.find_index(&TableKey(handle.clone())) as i32
    }

    /// `Dump(OS)`: a banner, then each element with its index, in the dump layout.
    pub(crate) fn dump(
        &self,
        stream: &mut IoStream,
        print: ElementPrinter<T>,
    ) -> Result<(), Failure> {
        let count = self.map.extent() as i32;
        stream
            .insert_bytes(b"\n -------\n")
            .insert_bytes(b"Dump of ");
        stream.insert_integer(i64::from(count)).insert_bytes(b" ");
        stream
            .insert_bytes(self.labels.dump_plural)
            .insert_bytes(b" ");
        stream.insert_bytes(b"\n -------\n\n");
        for index in 1..=count {
            stream.set_width(4);
            stream.insert_integer(i64::from(index)).insert_bytes(b" : ");
            print(&self.get(index), stream, false)?;
        }
        Ok(())
    }

    /// `Write(OS, theProgress)`: the tag and count, then each element in the compact
    /// layout at precision 17; the previous precision is restored afterwards.
    pub(crate) fn write(
        &self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
        print: ElementPrinter<T>,
    ) -> Result<(), Failure> {
        let precision = stream.precision();
        stream.set_precision(17);
        let written = self.write_elements(stream, progress, print);
        stream.set_precision(precision);
        written
    }

    fn write_elements(
        &self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
        print: ElementPrinter<T>,
    ) -> Result<(), Failure> {
        let count = self.map.extent() as i32;
        stream.insert_bytes(self.labels.tag).insert_bytes(b" ");
        stream.insert_integer(i64::from(count)).insert_bytes(b"\n");
        let scope =
            ProgressScope::with_literal(progress, self.labels.scope, f64::from(count), false);
        let mut index = 1;
        while index <= count && scope.more() {
            print(&self.get(index), stream, true)?;
            index += 1;
            drop(scope.next(1.0));
        }
        Ok(())
    }

    /// `Read(IS, theProgress)`: checks the tag (a console line and no change when it
    /// differs), then reads and adds the announced number of elements. The table is
    /// not cleared first, although the native documentation says so.
    pub(crate) fn read(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
        read: ElementReader<T>,
    ) {
        let tag = stream.extract_word(TAG_BUFFER);
        if tag != self.labels.tag {
            self.console
                .write(&[self.labels.not_a_table, b"\n"].concat());
            return;
        }
        let mut count = 0;
        stream.extract_int(&mut count);
        let scope =
            ProgressScope::with_literal(progress, self.labels.scope, f64::from(count), false);
        let mut index = 1;
        while index <= count && scope.more() {
            let element = read(stream);
            self.map.add(TableKey(element));
            index += 1;
            drop(scope.next(1.0));
        }
    }
}
