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
// Rust translation of ModelingData/TKBRep/BinTools/BinTools_CurveSet.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BinTools_CurveSet`: the indexed `Curves` table of the binary shape set.
use super::handle_table::HandleTable;
use super::ostream::BinOStream;
use super::table_io::{TableLabels, read_table, write_table};
use super::{curve_read, curve_write};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::geom::GeomCurve;

static LABELS: TableLabels = TableLabels {
    tag: b"Curves",
    write_scope: "Writing curves",
    read_scope: "Reading curves",
    not_a_table: "BinTools_CurveSet::Read:  Not a Curve table\n",
};

/// Stores a set of `Geom_Curve`s, each at the index it was first added with.
///
/// ```
/// use rustycad_tkbrep::bintools::BinCurveSet;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
/// use rustycad_tkmath::gp::{Direction3, Point3};
/// let line: Handle<dyn GeomCurve> =
///     Handle::new(GeomLine::from_point(Point3::new(0., 0., 0.), Direction3::default())).into_cast();
/// let mut set = BinCurveSet::new();
/// assert_eq!(set.add(&line), 1);
/// assert_eq!(set.index(&line), 1);
/// assert_eq!(set.add(&Handle::default()), 0);
/// ```
pub struct BinCurveSet {
    table: HandleTable<dyn GeomCurve>,
}

impl Default for BinCurveSet {
    fn default() -> Self {
        Self::new()
    }
}

impl BinCurveSet {
    /// `BinTools_CurveSet()`: an empty set; `BinCurveSet::new()`.
    pub fn new() -> Self {
        Self {
            table: HandleTable::new("BinTools_CurveSet"),
        }
    }

    /// `Clear()`; `set.clear()`.
    pub fn clear(&mut self) {
        self.table.clear();
    }

    /// `Add(C)`: the index of `curve`, added when new; 0 for a null handle, which is not
    /// stored; `set.add(&curve)`.
    pub fn add(&mut self, curve: &Handle<dyn GeomCurve>) -> i32 {
        if curve.is_null() {
            return 0;
        }
        self.table.add(curve)
    }

    /// `Curve(I)`: a null handle for 0, else the curve of index `index`;
    /// Standard_OutOfRange outside `1..=Extent`; `set.curve(1)?`.
    pub fn curve(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        if index == 0 {
            return Ok(Handle::default());
        }
        self.table.get(index)
    }

    /// `Index(C)`: the index of `curve`, 0 when absent or null; `set.index(&curve)`.
    pub fn index(&self, curve: &Handle<dyn GeomCurve>) -> i32 {
        if curve.is_null() {
            return 0;
        }
        self.table.index(curve)
    }

    /// The number of stored curves (`myMap.Extent()`); `set.extent()`.
    pub fn extent(&self) -> i32 {
        self.table.extent()
    }

    /// `Write(OS, theRange)`: `Curves n\n`, then each curve record; stops early when the
    /// progress indicator breaks; `set.write(&mut stream, &ProgressRange::default())?`.
    pub fn write(
        &self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        write_table(stream, progress, &LABELS, self.extent(), |index, out| {
            Self::write_curve(&self.table.get(index)?, out)
        })
    }

    /// `Read(IS, theRange)`: appends the curves of a `Curves n` table (the set is not
    /// cleared); another tag raises Standard_Failure ("Not a Curve table"), and a record
    /// that cannot be read raises the wrapped failure of `ReadCurve`;
    /// `set.read(&mut stream, &ProgressRange::default())?`.
    pub fn read(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let table = &mut self.table;
        read_table(stream, progress, &LABELS, |stream| {
            table.add(&Self::read_curve(stream)?);
            Ok(())
        })
    }

    /// `WriteCurve(C, OS)`: the type code and the fields of the curve; another class
    /// raises "UNKNOWN CURVE TYPE" (wrapped), a null curve Standard_NullObject (a crash
    /// natively); `BinCurveSet::write_curve(&curve, &mut out)?`.
    pub fn write_curve(
        curve: &Handle<dyn GeomCurve>,
        out: &mut BinOStream<'_>,
    ) -> Result<(), Failure> {
        curve_write::write_curve(curve, out)
    }

    /// `ReadCurve(IS, C)`: one curve record; a failure is raised wrapped in "EXCEPTION
    /// in BinTools_CurveSet::ReadCurve(..)"; `BinCurveSet::read_curve(&mut stream)?`.
    pub fn read_curve(stream: &mut IoStream) -> Result<Handle<dyn GeomCurve>, Failure> {
        curve_read::read_curve(stream)
    }
}
