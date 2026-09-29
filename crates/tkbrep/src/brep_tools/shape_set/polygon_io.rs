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
// Rust translation of BRepTools_ShapeSet::WritePolygon3D, DumpPolygon3D, ReadPolygon3D,
// WritePolygonOnTriangulation, DumpPolygonOnTriangulation and ReadPolygonOnTriangulation
// (ModelingData/TKBRep/BRepTools/BRepTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Polygon3D` and `PolygonOnTriangulations` sections of the ASCII format.
use super::BRepToolsShapeSet;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::HArray1OfReal;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkgeombase::geom_tools::get_real;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation};

/// `char buffer[255]` of the section readers.
pub(super) const SECTION_BUFFER: usize = 255;

/// `IS >> buffer; if (strstr(buffer, tag) == nullptr) return;` then `IS >> count`: the
/// count of the section, or none when the word does not contain the tag.
pub(super) fn read_section_count(stream: &mut IoStream, tag: &[u8]) -> Option<i32> {
    let word = stream.extract_word(SECTION_BUFFER);
    if !word.windows(tag.len()).any(|window| window == tag) {
        return None;
    }
    let mut count = 0;
    stream.extract_int(&mut count);
    Some(count)
}

/// The dump header of a section: `" -------\nDump of n title\n -------\n"`.
pub(super) fn dump_header(stream: &mut IoStream, count: i32, title: &[u8]) {
    stream.insert_bytes(b" -------\nDump of ");
    stream.insert_integer(i64::from(count));
    stream.insert_bytes(b" ");
    stream.insert_bytes(title);
    stream.insert_bytes(b"\n -------\n");
}

/// The compact header of a section: `"tag n\n"`.
pub(super) fn compact_header(stream: &mut IoStream, count: i32, tag: &[u8]) {
    stream.insert_bytes(tag);
    stream.insert_bytes(b" ");
    stream.insert_integer(i64::from(count));
    stream.insert_bytes(b"\n");
}

/// The three coordinates of a node: each after `std::setw(17)` in the dump layout.
pub(super) fn write_coordinates(stream: &mut IoStream, coordinates: &[f64], compact: bool) {
    let last = coordinates.len() - 1;
    for (position, value) in coordinates.iter().enumerate() {
        if !compact {
            stream.set_width(17);
        }
        stream.insert_real(*value);
        if position < last {
            stream.insert_bytes(b" ");
        }
    }
    stream.insert_bytes(if compact { b" " } else { b"\n" });
}

/// `std::setw(10) << j << " : "` of the dump layout.
pub(super) fn dump_rank(stream: &mut IoStream, rank: i32, compact: bool) {
    if !compact {
        stream.set_width(10);
        stream.insert_integer(i64::from(rank));
        stream.insert_bytes(b" : ");
    }
}

/// `Nodes(1, n)` of a record; Standard_RangeError for a negative count.
fn check_count(count: i32, member: &str) -> Result<(), Failure> {
    if count < 0 {
        return Err(Failure::with_type(
            "Standard_RangeError",
            &format!("{member}: node count {count}; expected at least 0"),
            "",
        ));
    }
    Ok(())
}

impl BRepToolsShapeSet {
    /// `WritePolygon3D(OS, Compact, theProgress)`: the 3D polygons, compact (the layout
    /// `ReadPolygon3D` accepts) or readable; `set.write_polygon3d(&mut s, true, &range)?`.
    pub fn write_polygon3d(
        &self,
        stream: &mut IoStream,
        compact: bool,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let count = self.polygons3d.extent();
        let scope = ProgressScope::with_literal(progress, "3D Polygons", f64::from(count), false);
        if compact {
            compact_header(stream, count, b"Polygon3D");
        } else {
            dump_header(stream, count, b"Polygon3Ds");
        }
        let mut index = 1;
        while index <= count && scope.more() {
            write_polygon3d_record(stream, index, &self.polygons3d.get(index)?, compact)?;
            index += 1;
            drop(scope.next(1.0));
        }
        Ok(())
    }

    /// `DumpPolygon3D(OS)`: `WritePolygon3D(OS, false)`; `set.dump_polygon3d(&mut s)?`.
    pub fn dump_polygon3d(&self, stream: &mut IoStream) -> Result<(), Failure> {
        self.write_polygon3d(stream, false, &ProgressRange::default())
    }

    /// `ReadPolygon3D(IS, theProgress)`: appends the polygons of a `Polygon3D n` section;
    /// another word reads nothing; `set.read_polygon3d(&mut s, &range)?`.
    pub fn read_polygon3d(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let Some(count) = read_section_count(stream, b"Polygon3D") else {
            return Ok(());
        };
        let scope = ProgressScope::with_literal(progress, "3D Polygons", f64::from(count), false);
        let mut index = 1;
        while index <= count && scope.more() {
            let polygon = read_polygon3d_record(stream)?;
            self.polygons3d.add(&Handle::new(polygon));
            index += 1;
            drop(scope.next(1.0));
        }
        Ok(())
    }

    /// `WritePolygonOnTriangulation(OS, Compact, theProgress)`: the node arrays on the
    /// triangulations, compact or readable; `set.write_polygon_on_triangulation(&mut s, true, &range)?`.
    pub fn write_polygon_on_triangulation(
        &self,
        stream: &mut IoStream,
        compact: bool,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let count = self.nodes.extent();
        let scope = ProgressScope::with_literal(
            progress,
            "Polygons On Triangulation",
            f64::from(count),
            false,
        );
        if compact {
            compact_header(stream, count, b"PolygonOnTriangulations");
        } else {
            dump_header(stream, count, b"PolygonOnTriangulations");
        }
        let mut index = 1;
        while index <= count && scope.more() {
            write_nodes_record(stream, index, &self.nodes.get(index)?, compact)?;
            index += 1;
            drop(scope.next(1.0));
        }
        Ok(())
    }

    /// `DumpPolygonOnTriangulation(OS)`: `WritePolygonOnTriangulation(OS, false)`;
    /// `set.dump_polygon_on_triangulation(&mut s)?`.
    pub fn dump_polygon_on_triangulation(&self, stream: &mut IoStream) -> Result<(), Failure> {
        self.write_polygon_on_triangulation(stream, false, &ProgressRange::default())
    }

    /// `ReadPolygonOnTriangulation(IS, theProgress)`: appends the node arrays of a
    /// `PolygonOnTriangulations n` section; another word reads nothing;
    /// `set.read_polygon_on_triangulation(&mut s, &range)?`.
    pub fn read_polygon_on_triangulation(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let Some(count) = read_section_count(stream, b"PolygonOnTriangulations") else {
            return Ok(());
        };
        let scope = ProgressScope::with_literal(
            progress,
            "Polygons On Triangulation",
            f64::from(count),
            false,
        );
        let mut index = 1;
        while index <= count && scope.more() {
            let polygon = read_nodes_record(stream)?;
            self.nodes.add(&Handle::new(polygon));
            index += 1;
            drop(scope.next(1.0));
        }
        Ok(())
    }
}

fn polygon_of(polygon: &Handle<Polygon3D>) -> Result<&Polygon3D, Failure> {
    polygon.get().ok_or_else(|| {
        crate::brep::errors::null_object("BRepTools_ShapeSet::WritePolygon3D: null polygon")
    })
}

/// One polygon: its sizes, deflection, nodes and parameters.
fn write_polygon3d_record(
    stream: &mut IoStream,
    index: i32,
    polygon: &Handle<Polygon3D>,
    compact: bool,
) -> Result<(), Failure> {
    let polygon = polygon_of(polygon)?;
    let count = polygon.nb_nodes();
    if compact {
        stream.insert_integer(i64::from(count));
        stream.insert_bytes(b" ");
        stream.insert_bytes(if polygon.has_parameters() {
            b"1\n"
        } else {
            b"0\n"
        });
    } else {
        stream.insert_bytes(b"  ");
        stream.insert_integer(i64::from(index));
        stream.insert_bytes(b" : Polygon3D with ");
        stream.insert_integer(i64::from(count));
        stream.insert_bytes(b" Nodes\n");
        stream.insert_bytes(if polygon.has_parameters() {
            b"with"
        } else {
            b"without"
        });
        stream.insert_bytes(b" parameters\n");
        stream.insert_bytes(b"Deflection : ");
    }
    stream.insert_real(polygon.deflection());
    stream.insert_bytes(b"\n");
    if !compact {
        stream.insert_bytes(b"\nNodes :\n");
    }
    let nodes = polygon.nodes();
    for rank in 1..=count {
        dump_rank(stream, rank, compact);
        let node: Point3 = *nodes
            .value(rank)
            .map_err(|e| e.into_failure("Poly_Polygon3D::Nodes"))?;
        write_coordinates(stream, &[node.x(), node.y(), node.z()], compact);
    }
    stream.insert_bytes(b"\n");
    if polygon.has_parameters() {
        if !compact {
            stream.insert_bytes(b"\nParameters :\n");
        }
        for value in polygon.parameters()?.iter() {
            stream.insert_real(*value);
            stream.insert_bytes(b" ");
        }
        stream.insert_bytes(b"\n");
    }
    Ok(())
}

/// One compact polygon record: `n hasParameters deflection nodes [parameters]`.
fn read_polygon3d_record(stream: &mut IoStream) -> Result<Polygon3D, Failure> {
    let (mut count, mut has_parameters) = (0, 0);
    stream.extract_int(&mut count);
    stream.extract_int(&mut has_parameters);
    check_count(count, "BRepTools_ShapeSet::ReadPolygon3D")?;
    let deflection = get_real(stream);
    let polygon = Polygon3D::new(count, has_parameters != 0)?;
    for rank in 1..=count {
        let (x, y, z) = (get_real(stream), get_real(stream), get_real(stream));
        polygon
            .change_nodes(|nodes| nodes.set_value(rank, Point3::new(x, y, z)))
            .map_err(|e| e.into_failure("NCollection_Array1::SetValue"))?;
    }
    if has_parameters != 0 {
        for rank in 1..=count {
            let value = get_real(stream);
            polygon
                .change_parameters(|parameters| parameters.set_value(rank, value))?
                .map_err(|e| e.into_failure("NCollection_Array1::SetValue"))?;
        }
    }
    polygon.set_deflection(deflection);
    Ok(polygon)
}

/// One node array: its nodes, `p`, the deflection and the parameters (or `0`).
fn write_nodes_record(
    stream: &mut IoStream,
    index: i32,
    polygon: &Handle<PolygonOnTriangulation>,
    compact: bool,
) -> Result<(), Failure> {
    let polygon = polygon.get().ok_or_else(|| {
        crate::brep::errors::null_object(
            "BRepTools_ShapeSet::WritePolygonOnTriangulation: null polygon",
        )
    })?;
    let nodes = polygon.nodes();
    let count = nodes.length();
    if compact {
        stream.insert_integer(i64::from(count));
        stream.insert_bytes(b" ");
    } else {
        stream.insert_bytes(b"  ");
        stream.insert_integer(i64::from(index));
        stream.insert_bytes(b" : PolygonOnTriangulation with ");
        stream.insert_integer(i64::from(count));
        stream.insert_bytes(b" Nodes\n  ");
    }
    for node in nodes.iter() {
        stream.insert_integer(i64::from(*node));
        stream.insert_bytes(b" ");
    }
    stream.insert_bytes(if compact {
        b"\np "
    } else {
        b"\n  Deflection : "
    });
    stream.insert_real(polygon.deflection());
    stream.insert_bytes(b" ");
    if !compact {
        stream.insert_bytes(b"\n");
    }
    write_parameters(stream, &polygon.parameters(), compact)
}

/// The parameters of a node array: `1 values` (compact) or `  Parameters :  values`,
/// or `0 ` without parameters.
fn write_parameters(
    stream: &mut IoStream,
    parameters: &Handle<HArray1OfReal>,
    compact: bool,
) -> Result<(), Failure> {
    let Some(parameters) = parameters.get() else {
        stream.insert_bytes(b"0 \n");
        return Ok(());
    };
    stream.insert_bytes(if compact { b"1 " } else { b"  Parameters :  " });
    for rank in parameters.lower()..=parameters.upper() {
        stream.insert_real(parameters.value(rank)?);
        stream.insert_bytes(b" ");
    }
    stream.insert_bytes(b"\n");
    Ok(())
}

/// One compact node-array record.
fn read_nodes_record(stream: &mut IoStream) -> Result<PolygonOnTriangulation, Failure> {
    let mut count = 0;
    stream.extract_int(&mut count);
    check_count(count, "BRepTools_ShapeSet::ReadPolygonOnTriangulation")?;
    let polygon = PolygonOnTriangulation::new(count, false)?;
    for rank in 1..=count {
        let mut node = 0;
        stream.extract_int(&mut node);
        polygon.set_node(rank, node)?;
    }
    // The `p` word.
    stream.extract_word(SECTION_BUFFER);
    let deflection = get_real(stream);
    let mut has_parameters = 0;
    stream.extract_int(&mut has_parameters);
    if has_parameters != 0 {
        let parameters = HArray1OfReal::with_bounds(1, count)?;
        for rank in 1..=count {
            parameters.set_value(rank, get_real(stream))?;
        }
        polygon.set_parameters(Handle::new(parameters))?;
    }
    polygon.set_deflection(deflection);
    Ok(polygon)
}
