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
// Rust translation of ModelingData/TKGeomBase/GeomTools/GeomTools_CurveSet.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomTools_CurveSet`: the indexed `Curves` table of the BRep ASCII format.
use super::console::{ConsoleOutput, stdout_console};
use super::geometry_table::{GeometryTable, TableLabels};
use super::{curve_print, curve_read};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::geom::GeomCurve;
use std::sync::Arc;

static LABELS: TableLabels = TableLabels {
    dump_plural: b"Curves",
    tag: b"Curves",
    scope: "3D Curves",
    not_a_table: b"Not a Curve table",
};

/// Stores a set of `Geom_Curve`s, each at the index it was first added with.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkgeombase::geom_tools::CurveSet;
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
/// use rustycad_tkmath::gp::{Direction3, Point3};
/// let line: Handle<dyn GeomCurve> =
///     Handle::new(GeomLine::from_point(Point3::new(0., 0., 0.), Direction3::default())).into_cast();
/// let mut set = CurveSet::new();
/// assert_eq!(set.add(&line), 1);
/// assert_eq!(set.index(&line), 1);
/// ```
pub struct CurveSet {
    table: GeometryTable<dyn GeomCurve>,
}

impl Default for CurveSet {
    fn default() -> Self {
        Self::new()
    }
}

impl CurveSet {
    /// `GeomTools_CurveSet()`: an empty set whose `Read` reports on standard output;
    /// `CurveSet::new()`.
    pub fn new() -> Self {
        Self::with_console(stdout_console())
    }

    /// An empty set whose `Read` writes its "Not a Curve table" line to `console`
    /// (the native `std::cout`); `CurveSet::with_console(Arc::new(StdoutConsole))`.
    pub fn with_console(console: Arc<dyn ConsoleOutput>) -> Self {
        Self {
            table: GeometryTable::new(&LABELS, console),
        }
    }

    /// `Clear()`: removes every curve; `set.clear()`.
    pub fn clear(&mut self) {
        self.table.clear();
    }

    /// `Add(C)`: the index of `curve`, added when new; 0 for a null handle, which is
    /// not stored; `set.add(&curve)`.
    pub fn add(&mut self, curve: &Handle<dyn GeomCurve>) -> i32 {
        if curve.is_null() {
            return 0;
        }
        self.table.add(curve)
    }

    /// `Curve(I)`: the curve of index `index`, null outside `1..=NbCurves`; `set.curve(1)`.
    pub fn curve(&self, index: i32) -> Handle<dyn GeomCurve> {
        self.table.get(index)
    }

    /// `Index(C)`: the index of `curve`, 0 when absent or null; `set.index(&curve)`.
    pub fn index(&self, curve: &Handle<dyn GeomCurve>) -> i32 {
        if curve.is_null() {
            return 0;
        }
        self.table.index(curve)
    }

    /// `Dump(OS)`: every curve in the dump layout at the stream precision;
    /// Standard_NullObject for a null element (read back from a failed record), which
    /// the native code dereferences; `set.dump(&mut stream)?`.
    pub fn dump(&self, stream: &mut IoStream) -> Result<(), Failure> {
        self.table.dump(stream, curve_print::print_curve)
    }

    /// `Write(OS, theProgress)`: the `Curves n` table in the compact layout at
    /// precision 17, stopping early when the progress indicator breaks; the stream
    /// precision is restored; `set.write(&mut stream, &ProgressRange::default())?`.
    pub fn write(
        &self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        self.table.write(stream, progress, curve_print::print_curve)
    }

    /// `Read(IS, theProgress)`: appends the curves of a `Curves n` table; another tag
    /// writes "Not a Curve table" to the console and reads nothing. A record that
    /// fails to read is stored as a null curve (once); the set is not cleared first;
    /// `set.read(&mut stream, &ProgressRange::default())`.
    pub fn read(&mut self, stream: &mut IoStream, progress: &ProgressRange<'_>) {
        self.table.read(stream, progress, curve_read::read_curve);
    }

    /// `PrintCurve(C, OS, compact)`: the dump layout, or the compact layout `Read`
    /// accepts; Standard_NullObject for a null curve;
    /// `CurveSet::print_curve(&curve, &mut stream, true)?`.
    pub fn print_curve(
        curve: &Handle<dyn GeomCurve>,
        stream: &mut IoStream,
        compact: bool,
    ) -> Result<(), Failure> {
        curve_print::print_curve(curve, stream, compact)
    }

    /// `ReadCurve(IS)`: one compact record; null when it cannot be read or built;
    /// `CurveSet::read_curve(&mut stream)`.
    pub fn read_curve(stream: &mut IoStream) -> Handle<dyn GeomCurve> {
        curve_read::read_curve(stream)
    }
}
