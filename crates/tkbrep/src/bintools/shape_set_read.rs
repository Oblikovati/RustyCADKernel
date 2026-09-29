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
// Rust translation of BinTools_ShapeSet::Read(IS, theRange), ReadGeometry,
// ReadFlagsAndSubs and ReadSubs (BinTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Reading the sections of the grouped binary format back.
use super::format_version;
use super::raw_enums::orientation_from_raw;
use super::scalars::{get_bool, get_integer, stream_get};
use super::shape_set::BinShapeSet;
use super::shape_set_base::{BinShapeSetBase, THE_ASCII_VERSIONS};
use crate::brep_tools::update_face;
use crate::topods::{Face, Shape};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `char vers[101]` read with `getline(vers, 100, '\n')`.
const VERSION_LINE: i64 = 100;
/// `char buffer[255]` of the `TShapes` tag.
const TAG_BUFFER: usize = 255;

impl BinShapeSet {
    /// The body of `Read(IS, theRange)`: clears the set, looks for a known version line
    /// (without one, "File was not written with this version of the topology" goes to
    /// the console and nothing is read), then reads the locations, the geometry and
    /// the `TShapes` table; stops early when the progress indicator breaks.
    pub(super) fn read_set(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        self.clear();
        if !self.find_version(stream) {
            self.console_write(
                b"BinTools_ShapeSet::Read: File was not written with this version of the topology\n",
            );
            return Ok(());
        }
        self.locations.read(stream)?;
        let scope = ProgressScope::with_literal(progress, "Reading", 2.0, false);
        self.read_geometry(stream, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        if stream.extract_word(TAG_BUFFER) != b"TShapes" || stream.fail() {
            return Err(Failure::new(
                "BinTools_ShapeSet::Read: Not a TShape table\n",
            ));
        }
        let mut count = 0;
        stream.extract_int(&mut count);
        stream_get(stream);
        self.read_tshapes(stream, count, &scope.next(1.0))
    }

    /// The version loop: reads lines until one, without its trailing `\r` and `\n`
    /// (never the first byte), is a known version line, and sets the format number.
    fn find_version(&mut self, stream: &mut IoStream) -> bool {
        loop {
            let mut line = Vec::new();
            stream.getline(&mut line, VERSION_LINE, b'\n');
            // `strlen(vers)`: the line is a C string.
            if let Some(end) = line.iter().position(|byte| *byte == 0) {
                line.truncate(end);
            }
            // BUC60769 PTV 18.10.2000: remove a possible '\r' at the end of the line.
            while line.len() > 1 && matches!(line.last(), Some(b'\r' | b'\n')) {
                line.pop();
            }
            let known = (format_version::LOWER..=format_version::UPPER)
                .find(|version| line == THE_ASCII_VERSIONS[*version as usize].as_bytes());
            if let Some(version) = known {
                self.set_format_nb(version);
                return !stream.fail();
            }
            if stream.fail() {
                return false;
            }
        }
    }

    /// The `count` shape records: type byte, geometry, flags and sub-shapes.
    fn read_tshapes(
        &mut self,
        stream: &mut IoStream,
        count: i32,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let scope =
            ProgressScope::with_literal(progress, "Reading Shapes", f64::from(count), false);
        let mut index = 1;
        while index <= count && scope.more() {
            let mut shape = Shape::default();
            let kind = stream_get(stream);
            self.read_shape_geometry(kind, stream, &mut shape)?;
            self.read_flags_and_subs(&mut shape, kind, stream, count)?;
            self.shapes.add(shape);
            index += 1;
            drop(scope.next(1.0));
        }
        Ok(())
    }

    /// `ReadGeometry(IS, theRange)`: the 2D curves, the curves, the 3D polygons, the
    /// polygons on triangulations, the surfaces and the triangulations, stopping when
    /// the progress indicator breaks; `set.read_geometry(&mut stream, &ProgressRange::default())?`.
    pub fn read_geometry(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let scope = ProgressScope::with_literal(progress, "Reading geometry", 6.0, false);
        self.curves2d.read(stream, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        self.curves.read(stream, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        self.read_polygon3d(stream, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        self.read_polygon_on_triangulation(stream, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        self.surfaces.read(stream, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        self.read_triangulation(stream, &scope.next(1.0))
    }

    /// `ReadFlagsAndSubs(S, T, IS, NbShapes)`: the seven flag bytes, the sub-shape
    /// references up to `*` (each added with `AddShapes`), then the flags. A version 1
    /// file clears Checked, and each of its faces gets the UV points of its edges
    /// updated (`BRepTools::Update`), which marks it Checked again;
    /// `set.read_flags_and_subs(&mut shape, kind, &mut stream, count)?`.
    pub fn read_flags_and_subs(
        &mut self,
        shape: &mut Shape,
        kind: i32,
        stream: &mut IoStream,
        nb_shapes: i32,
    ) -> Result<(), Failure> {
        let mut flags = [false; 7];
        for flag in &mut flags {
            *flag = get_bool(stream);
        }
        loop {
            let mut child = Shape::default();
            self.read_subs(&mut child, stream, nb_shapes)?;
            if child.is_null() {
                break;
            }
            self.add_shapes(shape, &child)?;
        }
        let version1 = self.format_nb() == format_version::VERSION_1;
        if version1 {
            // Force the check at reading.
            flags[2] = false;
        }
        apply_flags(shape, flags)?;
        if version1 && kind == ShapeEnum::Face as i32 {
            update_face(&Face::unchecked(shape.clone()))?;
        }
        Ok(())
    }

    /// `ReadSubs(S, IS, NbShapes)`: `*` (read with `IS >> char`, which skips spaces)
    /// gives a null shape; otherwise the byte is the orientation, followed by the index
    /// of the TShape counted from the end of the `NbShapes` table and the location
    /// index; `set.read_subs(&mut shape, &mut stream, count)?`.
    pub fn read_subs(
        &self,
        shape: &mut Shape,
        stream: &mut IoStream,
        nb_shapes: i32,
    ) -> Result<(), Failure> {
        let mut byte = 0;
        stream.extract_char(&mut byte);
        if byte == b'*' {
            *shape = Shape::default();
            return Ok(());
        }
        let index = get_integer(stream)?;
        *shape = self.shape(nb_shapes.wrapping_sub(index).wrapping_add(1))?;
        // `(TopAbs_Orientation)aChar`: `char` is signed on the pinned platform.
        shape.set_orientation(orientation_from_raw(i32::from(byte as i8))?);
        let location = get_integer(stream)?;
        shape.set_location(&self.locations.location(location)?, false)
    }
}

/// Free, Modified, Checked, Orientable, Closed, Infinite and Convex, in that order (the
/// shape reader sets them the same way).
pub(super) fn apply_flags(shape: &Shape, flags: [bool; 7]) -> Result<(), Failure> {
    shape.set_free(flags[0])?;
    shape.set_modified(flags[1])?;
    shape.set_checked(flags[2])?;
    shape.set_orientable(flags[3])?;
    shape.set_closed(flags[4])?;
    shape.set_infinite(flags[5])?;
    shape.set_convex(flags[6])
}
