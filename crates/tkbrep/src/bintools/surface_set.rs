// Created on: 2004-05-20
// Created by: Sergey ZARITCHNY
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
// Rust translation of ModelingData/TKBRep/BinTools/BinTools_SurfaceSet.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BinTools_SurfaceSet`: the indexed `Surfaces` table of the binary shape set.
use super::handle_table::HandleTable;
use super::ostream::BinOStream;
use super::table_io::{TableLabels, read_table, write_table};
use super::{surface_read, surface_write};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::geom::GeomSurface;

static LABELS: TableLabels = TableLabels {
    tag: b"Surfaces",
    write_scope: "Writing surfaces",
    read_scope: "Reading surfaces",
    not_a_table: "BinTools_SurfaceSet::Read:  Not a surface table\n",
};

/// Stores a set of `Geom_Surface`s, each at the index it was first added with. `Add`
/// and `Index` do not test for a null handle and `Surface(0)` is out of range, as
/// natively.
///
/// ```
/// use rustycad_tkbrep::bintools::BinSurfaceSet;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
/// use rustycad_tkmath::gp::Axis3;
/// let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).into_cast();
/// let mut set = BinSurfaceSet::new();
/// assert_eq!(set.add(&plane), 1);
/// assert!(set.surface(1).unwrap() == plane);
/// ```
pub struct BinSurfaceSet {
    table: HandleTable<dyn GeomSurface>,
}

impl Default for BinSurfaceSet {
    fn default() -> Self {
        Self::new()
    }
}

impl BinSurfaceSet {
    /// `BinTools_SurfaceSet()`: an empty set; `BinSurfaceSet::new()`.
    pub fn new() -> Self {
        Self {
            table: HandleTable::new("BinTools_SurfaceSet"),
        }
    }

    /// `Clear()`; `set.clear()`.
    pub fn clear(&mut self) {
        self.table.clear();
    }

    /// `Add(S)`: the index of `surface`, added when new (a null handle too);
    /// `set.add(&surface)`.
    pub fn add(&mut self, surface: &Handle<dyn GeomSurface>) -> i32 {
        self.table.add(surface)
    }

    /// `Surface(I)`: the surface of index `index`; Standard_OutOfRange outside
    /// `1..=Extent`; `set.surface(1)?`.
    pub fn surface(&self, index: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
        self.table.get(index)
    }

    /// `Index(S)`: the index of `surface`, 0 when absent; `set.index(&surface)`.
    pub fn index(&self, surface: &Handle<dyn GeomSurface>) -> i32 {
        self.table.index(surface)
    }

    /// The number of stored surfaces (`myMap.Extent()`); `set.extent()`.
    pub fn extent(&self) -> i32 {
        self.table.extent()
    }

    /// `Write(OS, theRange)`: `Surfaces n\n`, then each surface record; a stored null
    /// surface raises Standard_NullObject (a crash natively);
    /// `set.write(&mut stream, &ProgressRange::default())?`.
    pub fn write(
        &self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        write_table(stream, progress, &LABELS, self.extent(), |index, out| {
            Self::write_surface(&self.table.get(index)?, out)
        })
    }

    /// `Read(IS, theRange)`: appends the surfaces of a `Surfaces n` table; another tag
    /// raises Standard_Failure ("Not a surface table");
    /// `set.read(&mut stream, &ProgressRange::default())?`.
    pub fn read(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let table = &mut self.table;
        read_table(stream, progress, &LABELS, |stream| {
            table.add(&Self::read_surface(stream)?);
            Ok(())
        })
    }

    /// `WriteSurface(S, OS)`; `BinSurfaceSet::write_surface(&surface, &mut out)?`.
    pub fn write_surface(
        surface: &Handle<dyn GeomSurface>,
        out: &mut BinOStream<'_>,
    ) -> Result<(), Failure> {
        surface_write::write_surface(surface, out)
    }

    /// `ReadSurface(IS, S)`; `BinSurfaceSet::read_surface(&mut stream)?`.
    pub fn read_surface(stream: &mut IoStream) -> Result<Handle<dyn GeomSurface>, Failure> {
        surface_read::read_surface(stream)
    }
}
