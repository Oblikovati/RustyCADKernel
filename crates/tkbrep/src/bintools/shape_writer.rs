// Copyright (c) 2021 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/BinTools/BinTools_ShapeWriter.hxx and the
// members Clear, Write, WriteShape and WriteLocation of BinTools_ShapeWriter.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BinTools_ShapeWriter`: the shape-reference format. Each shape, location and
//! geometry is written where it is first used; a later use writes a back reference to
//! the byte position of the first one.
use super::errors::wrap_failure;
use super::handle_table::{HandleKey, IdentityHasher};
use super::object_type::ObjectType;
use super::ostream::BinOStream;
use super::shape_set_base::{BinShapeSetBase, ShapeSetBaseData};
use crate::topods::{Shape, ShapeIterator};
use crate::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation};
use rustycad_tkmath::toploc::Location;

/// The byte positions of the objects written so far, by object identity.
pub(super) type PositionMap<T> = DataMap<HandleKey<T>, u64, IdentityHasher>;

/// Writes shapes in the shape-reference format. The positions are kept between calls,
/// so a second shape written through the same writer refers back to the objects of
/// the first one (`Clear` forgets them).
///
/// ```
/// use rustycad_tkbrep::bintools::{BinShapeSetBase, BinShapeWriter};
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkernel::storage::IoStream;
/// let mut stream = IoStream::from_bytes(Vec::new());
/// let mut writer = BinShapeWriter::new();
/// writer.write_shape(&Shape::default(), &mut stream).unwrap();
/// assert_eq!(stream.tellp(), 1, "a null shape is the EmptyShape byte");
/// ```
#[derive(Default)]
pub struct BinShapeWriter {
    base: ShapeSetBaseData,
    pub(super) shape_positions: DataMap<Shape, u64, ShapeMapHasher>,
    pub(super) location_positions: DataMap<Location, u64>,
    pub(super) curve_positions: PositionMap<dyn GeomCurve>,
    pub(super) curve2d_positions: PositionMap<dyn Geom2dCurve>,
    pub(super) surface_positions: PositionMap<dyn GeomSurface>,
    pub(super) polygon3d_positions: PositionMap<Polygon3D>,
    pub(super) polygon_positions: PositionMap<PolygonOnTriangulation>,
    pub(super) triangulation_positions: PositionMap<Triangulation>,
}

impl BinShapeWriter {
    /// `BinTools_ShapeWriter()`: a writer of the current format without triangles;
    /// `BinShapeWriter::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `WriteShape(theStream, theShape)` (private natively): `EmptyShape` for a null
    /// shape; a back reference, the location and the orientation byte for a shape
    /// already written (compared with `IsSame`); else the shape type and orientation
    /// byte, the location, the geometry, the seven flags as one byte, the sub-shapes
    /// and `EndShape`.
    pub(super) fn write_shape_record(
        &mut self,
        out: &mut BinOStream<'_>,
        shape: &Shape,
    ) -> Result<(), Failure> {
        if shape.is_null() {
            out.write_object_type(ObjectType::EMPTY_SHAPE);
            return Ok(());
        }
        let unlocated = shape.located(&Location::default(), false)?;
        if let Some(position) = self.shape_positions.seek(&unlocated).copied() {
            out.write_reference(position);
            self.write_location(out, shape.location())?;
            out.write_byte(shape.orientation() as u8);
            return Ok(());
        }
        self.shape_positions.bind(unlocated.clone(), out.position());
        out.write_shape(unlocated.shape_type()?, unlocated.orientation());
        self.write_location(out, shape.location())?;
        wrap_failure("EXCEPTION in BinTools_ShapeWriter::WriteShape", || {
            self.write_shape_geometry(out, &unlocated)
        })?;
        out.put_bools7(flags_of(&unlocated)?);
        let mut children = ShapeIterator::new(&unlocated, false, false);
        while children.more() {
            self.write_shape_record(out, children.value()?)?;
            children.next();
        }
        out.write_object_type(ObjectType::END_SHAPE);
        Ok(())
    }

    /// `WriteLocation(theStream, theLocation)`: `EmptyLocation` for the identity, a back
    /// reference for a location already written, else `SimpleLocation` and the matrix
    /// of an elementary location, or `Location`, each datum (as a location) with its
    /// power, and `LocationEnd`. The position is recorded once the location is written.
    /// Failures are wrapped in "EXCEPTION in BinTools_ShapeWriter::WriteLocation";
    /// `writer.write_location(&mut out, &location)?`.
    pub fn write_location(
        &mut self,
        out: &mut BinOStream<'_>,
        location: &Location,
    ) -> Result<(), Failure> {
        if location.is_identity() {
            out.write_object_type(ObjectType::EMPTY_LOCATION);
            return Ok(());
        }
        if let Some(position) = self.location_positions.seek(location).copied() {
            out.write_reference(position);
            return Ok(());
        }
        let position = out.position();
        wrap_failure("EXCEPTION in BinTools_ShapeWriter::WriteLocation", || {
            self.write_location_body(out, location)?;
            self.location_positions.bind(location.clone(), position);
            Ok(())
        })
    }

    fn write_location_body(
        &mut self,
        out: &mut BinOStream<'_>,
        location: &Location,
    ) -> Result<(), Failure> {
        let mut rest = location.next_location().clone();
        let power = location.first_power()?;
        let datum = Location::from_datum(location.first_datum()?.clone());
        if rest.is_identity() && power == 1 {
            out.write_object_type(ObjectType::SIMPLE_LOCATION);
            out.write_transform(&location.transformation());
            return Ok(());
        }
        out.write_object_type(ObjectType::LOCATION);
        self.write_location(out, &datum)?;
        out.write_integer(power);
        while !rest.is_identity() {
            let datum = Location::from_datum(rest.first_datum()?.clone());
            let power = rest.first_power()?;
            rest = rest.next_location().clone();
            self.write_location(out, &datum)?;
            out.write_integer(power);
        }
        out.write_object_type(ObjectType::LOCATION_END);
        Ok(())
    }
}

/// Free, Modified, Checked, Orientable, Closed, Infinite and Convex.
fn flags_of(shape: &Shape) -> Result<[bool; 7], Failure> {
    Ok([
        shape.free()?,
        shape.modified()?,
        shape.checked()?,
        shape.orientable()?,
        shape.closed()?,
        shape.infinite()?,
        shape.convex()?,
    ])
}

impl BinShapeSetBase for BinShapeWriter {
    fn base_data(&self) -> &ShapeSetBaseData {
        &self.base
    }
    fn base_data_mut(&mut self) -> &mut ShapeSetBaseData {
        &mut self.base
    }

    /// `Clear()`: forgets every recorded position; the options are kept.
    fn clear(&mut self) {
        self.shape_positions.clear(true);
        self.location_positions.clear(true);
        self.curve_positions.clear(true);
        self.curve2d_positions.clear(true);
        self.surface_positions.clear(true);
        self.polygon3d_positions.clear(true);
        self.polygon_positions.clear(true);
        self.triangulation_positions.clear(true);
    }

    /// `Write(theShape, theStream)`: the shape record through a `BinTools_OStream` that
    /// starts at the current stream position.
    fn write_shape(&mut self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        let mut out = BinOStream::new(stream);
        self.write_shape_record(&mut out, shape)
    }
}
