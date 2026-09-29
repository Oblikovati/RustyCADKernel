// Created on: 2004-05-11
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
// Rust translation of BinTools_ShapeSet::Write(OS, theRange), WriteGeometry and
// Write(S, OS) (BinTools_ShapeSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Writing the sections of the grouped binary format.
use super::scalars::{put_bool, put_byte, put_integer};
use super::shape_set::BinShapeSet;
use super::shape_set_base::{BinShapeSetBase, THE_ASCII_VERSIONS};
use crate::topods::{Shape, ShapeIterator};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkmath::toploc::Location;

impl BinShapeSet {
    /// The body of `Write(OS, theRange)`: `\n<version line>\n`, the locations, the
    /// geometry, then `\nTShapes n\n` and each shape: its type byte, its geometry
    /// (`WriteShape`), its seven flags and its sub-shape references ended by `*`.
    /// Stops early when the progress indicator breaks.
    pub(super) fn write_set(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        stream.insert_bytes(b"\n");
        stream.insert_bytes(THE_ASCII_VERSIONS[self.format_nb() as usize].as_bytes());
        stream.insert_bytes(b"\n");
        self.locations.write(stream)?;
        let scope = ProgressScope::with_literal(progress, "Writing geometry", 2.0, false);
        self.write_geometry(stream, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        self.write_tshapes(stream, &scope.next(1.0))
    }

    fn write_tshapes(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let count = self.nb_shapes();
        let scope =
            ProgressScope::with_literal(progress, "Writing shapes", f64::from(count), false);
        stream.insert_bytes(b"\nTShapes ");
        stream.insert_integer(i64::from(count)).insert_bytes(b"\n");
        let mut index = 1;
        while index <= count && scope.more() {
            let shape = self.shape(index)?;
            put_byte(stream, shape.shape_type()? as u8);
            self.write_shape_geometry(&shape, stream)?;
            self.write_flags(&shape, stream)?;
            let mut children = ShapeIterator::new(&shape, false, false);
            while children.more() {
                self.write_shape_reference(children.value()?, stream)?;
                children.next();
            }
            self.write_shape_reference(&Shape::default(), stream)?;
            index += 1;
            drop(scope.next(1.0));
        }
        Ok(())
    }

    /// Free, Modified, Checked, Orientable, Closed, Infinite and Convex, one byte each.
    fn write_flags(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        let flags = [
            shape.free()?,
            shape.modified()?,
            shape.checked()?,
            shape.orientable()?,
            shape.closed()?,
            shape.infinite()?,
            shape.convex()?,
        ];
        for flag in flags {
            put_bool(stream, flag);
        }
        Ok(())
    }

    /// `Write(S, OS)`: `*` for a null shape, else the orientation byte, the index of
    /// the TShape counted from the end of the table (`Extent - Index + 1`, so a shape
    /// missing from the table gives `Extent + 1`) and the index of its location.
    pub(super) fn write_shape_reference(
        &self,
        shape: &Shape,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        if shape.is_null() {
            stream.insert_bytes(b"*");
            return Ok(());
        }
        put_byte(stream, shape.orientation() as u8);
        let unlocated = shape.located(&Location::default(), false)?;
        put_integer(stream, self.nb_shapes() - self.index(&unlocated) + 1);
        put_integer(stream, self.locations.index(shape.location()));
        Ok(())
    }

    /// `WriteGeometry(OS, theRange)`: the 2D curves, the curves, the 3D polygons, the
    /// polygons on triangulations, the surfaces and the triangulations, stopping when
    /// the progress indicator breaks; `set.write_geometry(&mut stream, &ProgressRange::default())?`.
    pub fn write_geometry(
        &self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let scope = ProgressScope::with_literal(progress, "Writing geometry", 6.0, false);
        self.curves2d.write(stream, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        self.curves.write(stream, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        self.write_polygon3d(stream, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        self.write_polygon_on_triangulation(stream, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        self.surfaces.write(stream, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        self.write_triangulation(stream, &scope.next(1.0))
    }
}
