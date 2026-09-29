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
// Rust translation of BRepTools_ShapeSet::DumpGeometry(OS), WriteGeometry(OS, theProgress)
// and ReadGeometry(IS, theProgress) (ModelingData/TKBRep/BRepTools/BRepTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry tables of the ASCII format, in their section order.
use super::BRepToolsShapeSet;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;

impl BRepToolsShapeSet {
    /// `DumpGeometry(OS)`: the 2D curves, the curves, the 3D polygons, the polygons on
    /// triangulations, the surfaces and the triangulations, readable.
    pub(super) fn dump_tables(&self, stream: &mut IoStream) -> Result<(), Failure> {
        self.curves2d.dump(stream)?;
        self.curves.dump(stream)?;
        self.dump_polygon3d(stream)?;
        self.dump_polygon_on_triangulation(stream)?;
        self.surfaces.dump(stream)?;
        self.dump_triangulation(stream)
    }

    /// `WriteGeometry(OS, theProgress)`: the same sections, compact, in a "Geometry"
    /// progress scope of weights 20, 20, 10, 10, 20 and 20; a user break stops the
    /// write after the current section.
    pub(super) fn write_tables(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let scope = ProgressScope::with_literal(progress, "Geometry", 100.0, false);
        self.curves2d.write(stream, &scope.next(20.0))?;
        if scope.user_break() {
            return Ok(());
        }
        self.curves.write(stream, &scope.next(20.0))?;
        if scope.user_break() {
            return Ok(());
        }
        self.write_polygon3d(stream, true, &scope.next(10.0))?;
        if scope.user_break() {
            return Ok(());
        }
        self.write_polygon_on_triangulation(stream, true, &scope.next(10.0))?;
        if scope.user_break() {
            return Ok(());
        }
        self.surfaces.write(stream, &scope.next(20.0))?;
        if scope.user_break() {
            return Ok(());
        }
        self.write_triangulation(stream, true, &scope.next(20.0))
    }

    /// `ReadGeometry(IS, theProgress)`: the sections back, in a "Geometry" progress scope
    /// of weights 20, 20, 15, 15, 15 and 15; a user break stops after the current
    /// section.
    pub(super) fn read_tables(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let scope = ProgressScope::with_literal(progress, "Geometry", 100.0, false);
        self.curves2d.read(stream, &scope.next(20.0));
        if scope.user_break() {
            return Ok(());
        }
        self.curves.read(stream, &scope.next(20.0));
        if scope.user_break() {
            return Ok(());
        }
        self.read_polygon3d(stream, &scope.next(15.0))?;
        if scope.user_break() {
            return Ok(());
        }
        self.read_polygon_on_triangulation(stream, &scope.next(15.0))?;
        if scope.user_break() {
            return Ok(());
        }
        self.surfaces.read(stream, &scope.next(15.0));
        if scope.user_break() {
            return Ok(());
        }
        self.read_triangulation(stream, &scope.next(15.0))
    }
}
