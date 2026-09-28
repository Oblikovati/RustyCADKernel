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
// Rust translation of ModelingData/TKGeomBase/GeomTools/GeomTools_SurfaceSet.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomTools_SurfaceSet`: the indexed `Surfaces` table of the BRep ASCII format.
use super::console::{ConsoleOutput, stdout_console};
use super::geometry_table::{GeometryTable, TableLabels};
use super::{surface_print, surface_read};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::geom::GeomSurface;
use std::sync::Arc;

static LABELS: TableLabels = TableLabels {
    dump_plural: b"surfaces",
    tag: b"Surfaces",
    scope: "Surfaces",
    not_a_table: b"Not a surface table",
};

/// Stores a set of `Geom_Surface`s, each at the index it was first added with.
/// Unlike [`super::CurveSet`], a null handle is stored like any surface.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkgeombase::geom_tools::SurfaceSet;
/// let mut set = SurfaceSet::new();
/// assert_eq!(set.add(&Handle::default()), 1);
/// assert_eq!(set.index(&Handle::default()), 1);
/// ```
pub struct SurfaceSet {
    table: GeometryTable<dyn GeomSurface>,
}

impl Default for SurfaceSet {
    fn default() -> Self {
        Self::new()
    }
}

impl SurfaceSet {
    /// `GeomTools_SurfaceSet()`: an empty set whose `Read` reports on standard
    /// output; `SurfaceSet::new()`.
    pub fn new() -> Self {
        Self::with_console(stdout_console())
    }

    /// An empty set whose `Read` writes its "Not a surface table" line to `console`;
    /// `SurfaceSet::with_console(Arc::new(StdoutConsole))`.
    pub fn with_console(console: Arc<dyn ConsoleOutput>) -> Self {
        Self {
            table: GeometryTable::new(&LABELS, console),
        }
    }

    /// `Clear()`: removes every surface; `set.clear()`.
    pub fn clear(&mut self) {
        self.table.clear();
    }

    /// `Add(S)`: the index of `surface`, added when new; a null handle is stored too;
    /// `set.add(&surface)`.
    pub fn add(&mut self, surface: &Handle<dyn GeomSurface>) -> i32 {
        self.table.add(surface)
    }

    /// `Surface(I)`: the surface of index `index`, null outside the table; `set.surface(1)`.
    pub fn surface(&self, index: i32) -> Handle<dyn GeomSurface> {
        self.table.get(index)
    }

    /// `Index(S)`: the index of `surface`, 0 when absent; a stored null handle has
    /// an index; `set.index(&surface)`.
    pub fn index(&self, surface: &Handle<dyn GeomSurface>) -> i32 {
        self.table.index(surface)
    }

    /// `Dump(OS)`: every surface in the dump layout; Standard_NullObject for a null
    /// element, which the native code dereferences; `set.dump(&mut stream)?`.
    pub fn dump(&self, stream: &mut IoStream) -> Result<(), Failure> {
        self.table.dump(stream, surface_print::print_surface)
    }

    /// `Write(OS, theProgress)`: the `Surfaces n` table in the compact layout at
    /// precision 17; `set.write(&mut stream, &ProgressRange::default())?`.
    pub fn write(
        &self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        self.table
            .write(stream, progress, surface_print::print_surface)
    }

    /// `Read(IS, theProgress)`: appends the surfaces of a `Surfaces n` table; another
    /// tag writes "Not a surface table" to the console;
    /// `set.read(&mut stream, &ProgressRange::default())`.
    pub fn read(&mut self, stream: &mut IoStream, progress: &ProgressRange<'_>) {
        self.table
            .read(stream, progress, surface_read::read_surface);
    }

    /// `PrintSurface(S, OS, compact)`;
    /// `SurfaceSet::print_surface(&surface, &mut stream, true)?`.
    pub fn print_surface(
        surface: &Handle<dyn GeomSurface>,
        stream: &mut IoStream,
        compact: bool,
    ) -> Result<(), Failure> {
        surface_print::print_surface(surface, stream, compact)
    }

    /// `ReadSurface(IS)`: one compact record; null when it cannot be read or built;
    /// `SurfaceSet::read_surface(&mut stream)`.
    pub fn read_surface(stream: &mut IoStream) -> Handle<dyn GeomSurface> {
        surface_read::read_surface(stream)
    }
}
