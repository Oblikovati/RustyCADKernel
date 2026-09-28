// Created on: 1993-07-01
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
// Rust translation of ModelingData/TKBRep/TopTools/TopTools_ShapeSet.hxx and the
// non-virtual members of TopTools_ShapeSet.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopTools_ShapeSet`: the indexed set of TShapes that the BRep ASCII format writes
//! (docs/topology-design.md section 8).
use super::shape_set_io;
use super::shape_set_text::shape_counts;
use super::{IndexedMapOfShape, LocationSet, format_version};
use crate::topods::{Shape, ShapeIterator};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_tools::{ConsoleOutput, StdoutConsole};
use rustycad_tkmath::toploc::Location;
use std::sync::Arc;

/// `THE_ASCII_VERSIONS`: the copyright line of each format version (index 0 unused).
pub const THE_ASCII_VERSIONS: [&str; format_version::UPPER as usize + 1] = [
    "",
    "CASCADE Topology V1, (c) Matra-Datavision",
    "CASCADE Topology V2, (c) Matra-Datavision",
    "CASCADE Topology V3, (c) Open Cascade",
];

/// The private data of `TopTools_ShapeSet`: the TShapes (at the identity location), the
/// locations and the format number, plus the console that stands for `std::cout`.
pub struct ShapeSetData {
    pub(super) shapes: IndexedMapOfShape,
    pub(super) locations: LocationSet,
    pub(super) format_nb: i32,
    pub(super) console: Arc<dyn ConsoleOutput>,
}

impl Default for ShapeSetData {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapeSetData {
    /// The data of `TopTools_ShapeSet()`: empty, current format, standard output;
    /// `ShapeSetData::new()`.
    pub fn new() -> Self {
        Self::with_console(Arc::new(StdoutConsole))
    }

    /// Empty data whose `Read` messages go to `console`; `ShapeSetData::with_console(c)`.
    pub fn with_console(console: Arc<dyn ConsoleOutput>) -> Self {
        Self {
            shapes: IndexedMapOfShape::default(),
            locations: LocationSet::with_console(console.clone()),
            format_nb: format_version::CURRENT,
            console,
        }
    }
}

/// `TopTools_ShapeSet`: the non-virtual members are provided methods written once over
/// [`ShapeSetData`]; the virtual members are provided methods with the native bodies
/// (no-op geometry hooks), which a derived set (BRepTools, BinTools) overrides.
///
/// ```
/// use rustycad_tkbrep::topods::{Builder, Compound};
/// use rustycad_tkbrep::toptools::{ShapeSet, TopToolsShapeSet};
/// let mut compound = Compound::default();
/// Builder.make_compound(&mut compound);
/// let mut set = TopToolsShapeSet::new();
/// assert_eq!(set.add(&compound)?, 1);
/// assert_eq!(set.nb_shapes(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub trait ShapeSet {
    /// The embedded base data.
    fn shape_set_data(&self) -> &ShapeSetData;
    /// The embedded base data, for the mutating members.
    fn shape_set_data_mut(&mut self) -> &mut ShapeSetData;

    /// `Clear()` (virtual): empties the shapes and the locations; `set.clear()`.
    fn clear(&mut self) {
        shape_set_io::clear(self.shape_set_data_mut());
    }
    /// `Dump(OS)` (virtual): the shapes, the geometry and the locations;
    /// `set.dump(&mut stream)?`.
    fn dump(&self, stream: &mut IoStream) -> Result<(), Failure> {
        shape_set_io::dump(self, stream)
    }
    /// `Write(OS, theProgress)` (virtual): the format that `read` accepts;
    /// `set.write(&mut stream, &ProgressRange::default())?`.
    fn write(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        shape_set_io::write(self, stream, progress)
    }
    /// `Read(IS, theProgress)` (virtual): clears the set and reads it back;
    /// `set.read(&mut stream, &ProgressRange::default())?`.
    fn read(&mut self, stream: &mut IoStream, progress: &ProgressRange<'_>) -> Result<(), Failure> {
        shape_set_io::read(self, stream, progress)
    }
    /// `AddGeometry(S)` (virtual, no-op): stores the geometry of `shape`.
    fn add_geometry(&mut self, _shape: &Shape) -> Result<(), Failure> {
        Ok(())
    }
    /// `DumpGeometry(OS)` (virtual, no-op): dumps the stored geometry.
    fn dump_geometry(&self, _stream: &mut IoStream) -> Result<(), Failure> {
        Ok(())
    }
    /// `WriteGeometry(OS, theProgress)` (virtual, no-op): writes the stored geometry.
    fn write_geometry(
        &mut self,
        _stream: &mut IoStream,
        _progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    /// `ReadGeometry(IS, theProgress)` (virtual, no-op): reads the stored geometry.
    fn read_geometry(
        &mut self,
        _stream: &mut IoStream,
        _progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    /// `DumpGeometry(S, OS)` (virtual, no-op): dumps the geometry of one shape.
    fn dump_shape_geometry(&self, _shape: &Shape, _stream: &mut IoStream) -> Result<(), Failure> {
        Ok(())
    }
    /// `WriteGeometry(S, OS)` (virtual, no-op): writes the geometry of one shape.
    fn write_shape_geometry(&self, _shape: &Shape, _stream: &mut IoStream) -> Result<(), Failure> {
        Ok(())
    }
    /// `ReadGeometry(T, IS, S)` (virtual, no-op): reads the geometry of a shape of type
    /// `kind` and makes `shape`. The base leaves `shape` null, so reading a non-empty
    /// table with the base class fails on the flags (a crash natively).
    fn read_shape_geometry(
        &mut self,
        _kind: ShapeEnum,
        _stream: &mut IoStream,
        _shape: &mut Shape,
    ) -> Result<(), Failure> {
        Ok(())
    }
    /// `AddShapes(S1, S2)` (virtual, no-op): inserts `child` in `shape` with the builder
    /// of the derived set.
    fn add_shapes(&mut self, _shape: &mut Shape, _child: &Shape) -> Result<(), Failure> {
        Ok(())
    }
    /// `Check(T, S)` (virtual, no-op): called on each shape read from a version 1 file.
    fn check(&mut self, _kind: ShapeEnum, _shape: &mut Shape) -> Result<(), Failure> {
        Ok(())
    }

    /// `SetFormatNb(theFormatNb)`: a number outside `LOWER..=UPPER` is ignored
    /// (`Standard_ASSERT_RETURN` in a release build); `set.set_format_nb(2)`.
    fn set_format_nb(&mut self, format_nb: i32) {
        if (format_version::LOWER..=format_version::UPPER).contains(&format_nb) {
            self.shape_set_data_mut().format_nb = format_nb;
        }
    }
    /// `FormatNb()`; `set.format_nb()`.
    fn format_nb(&self) -> i32 {
        self.shape_set_data().format_nb
    }
    /// `Add(S)`: stores `shape` at the identity location after its sub-shapes (and its
    /// location in the location set), calling `add_geometry` on each new TShape; returns
    /// its index, 0 for a null shape; `set.add(&shape)?`.
    fn add(&mut self, shape: &Shape) -> Result<i32, Failure> {
        if shape.is_null() {
            return Ok(0);
        }
        self.shape_set_data_mut().locations.add(shape.location());
        let unlocated = shape.located(&Location::default(), false)?;
        let index = self.shape_set_data().shapes.find_index(&unlocated);
        if index != 0 {
            return Ok(index as i32);
        }
        self.add_geometry(&unlocated)?;
        let mut iterator = ShapeIterator::new(&unlocated, false, false);
        while let Ok(child) = iterator.value() {
            self.add(child)?;
            iterator.next();
        }
        Ok(self.shape_set_data_mut().shapes.add(unlocated) as i32)
    }
    /// `Shape(I)`: the shape of index `index`; Standard_OutOfRange outside
    /// `1..=NbShapes`; `set.shape(1)?`.
    fn shape(&self, index: i32) -> Result<Shape, Failure> {
        shape_set_io::shape_at(&self.shape_set_data().shapes, index)
    }
    /// `Index(S)`: the index of `shape` (compared with `IsSame`), 0 when absent;
    /// `set.index(&shape)`.
    fn index(&self, shape: &Shape) -> i32 {
        self.shape_set_data().shapes.find_index(shape) as i32
    }
    /// `Locations()`; `set.locations()`.
    fn locations(&self) -> &LocationSet {
        &self.shape_set_data().locations
    }
    /// `ChangeLocations()`; `set.change_locations()`.
    fn change_locations(&mut self) -> &mut LocationSet {
        &mut self.shape_set_data_mut().locations
    }
    /// `DumpExtent(OS)`: the number of shapes of each type; `set.dump_extent(&mut stream)?`.
    fn dump_extent(&self, stream: &mut IoStream) -> Result<(), Failure> {
        let text = shape_counts(&self.shape_set_data().shapes)?;
        stream.insert_bytes(text.as_bytes());
        Ok(())
    }
    /// `DumpExtent(S)`: appends the same text to `text`; `set.dump_extent_string(&mut s)?`.
    fn dump_extent_string(&self, text: &mut AsciiString) -> Result<(), Failure> {
        let counts = shape_counts(&self.shape_set_data().shapes)?;
        text.append(&AsciiString::from_bytes(counts.as_bytes()));
        Ok(())
    }
    /// `Dump(S, OS)`: the index, orientation and location index of `shape`;
    /// `set.dump_shape(&shape, &mut stream)?`.
    fn dump_shape(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        shape_set_io::dump_shape(self.shape_set_data(), shape, stream)
    }
    /// `Write(S, OS)`: `*` for a null shape, else the orientation, the reversed index and
    /// the location index; `set.write_shape(&shape, &mut stream)?`.
    fn write_shape(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        shape_set_io::write_shape(self.shape_set_data(), shape, stream)
    }
    /// `Read(S, IS)`: one shape reference written by `write_shape`;
    /// `set.read_shape(&mut stream)?`.
    fn read_shape(&self, stream: &mut IoStream) -> Result<Shape, Failure> {
        let data = self.shape_set_data();
        shape_set_io::read_shape_among(data, stream, data.shapes.extent() as i32)
    }
    /// `NbShapes()`: the number of stored TShapes; `set.nb_shapes()`.
    fn nb_shapes(&self) -> i32 {
        self.shape_set_data().shapes.extent() as i32
    }
}

/// The base class `TopTools_ShapeSet` itself, with the native no-op hooks.
#[derive(Default)]
pub struct TopToolsShapeSet {
    data: ShapeSetData,
}

impl TopToolsShapeSet {
    /// `TopTools_ShapeSet()`: an empty set at the current format; `TopToolsShapeSet::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// An empty set whose `Read` messages go to `console` (the native `std::cout`);
    /// `TopToolsShapeSet::with_console(console)`.
    pub fn with_console(console: Arc<dyn ConsoleOutput>) -> Self {
        Self {
            data: ShapeSetData::with_console(console),
        }
    }
}

impl ShapeSet for TopToolsShapeSet {
    fn shape_set_data(&self) -> &ShapeSetData {
        &self.data
    }
    fn shape_set_data_mut(&mut self) -> &mut ShapeSetData {
        &mut self.data
    }
}
