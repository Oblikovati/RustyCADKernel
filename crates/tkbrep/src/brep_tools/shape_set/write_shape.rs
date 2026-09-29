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
// Rust translation of BRepTools_ShapeSet::WriteGeometry(S, OS)
// (ModelingData/TKBRep/BRepTools/BRepTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry record of one shape in the ASCII format.
use super::BRepToolsShapeSet;
use super::regularity::{gcurve_range, print_regularity, uv_points, uv_points2};
use crate::brep::access::{edge_tshape, face_tshape, vertex_tshape};
use crate::brep::{BRepCurveRepresentation, BRepPointRepresentation, BRepTool};
use crate::topods::{self, Shape};
use crate::toptools::{ShapeSet, format_version};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::poly::mesh_purpose;

/// `OS << value << text` for the integer fields of the records.
fn put_int(stream: &mut IoStream, value: i32) {
    stream.insert_integer(i64::from(value));
}

/// `OS << (flag ? 1 : 0)`.
fn put_flag(stream: &mut IoStream, flag: bool) {
    put_int(stream, i32::from(flag));
}

impl BRepToolsShapeSet {
    /// `WriteGeometry(S, OS)`: the record of a vertex (tolerance, point, point
    /// representations), an edge (tolerance, flags, curve representations) or a face
    /// (surface and triangulation references) that `ReadGeometry(T, IS, S)` reads back.
    pub(super) fn write_geometry_of(
        &self,
        shape: &Shape,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        match shape.shape_type()? {
            ShapeEnum::Vertex => self.write_vertex(shape, stream),
            ShapeEnum::Edge => self.write_edge(shape, stream),
            ShapeEnum::Face => self.write_face(shape, stream),
            _ => Ok(()),
        }
    }

    fn write_vertex(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        let vertex = topods::vertex(shape)?;
        stream.insert_real(BRepTool::vertex_tolerance(&vertex)?);
        stream.insert_bytes(b"\n");
        let point = BRepTool::pnt(&vertex)?;
        stream.insert_real(point.x()).insert_bytes(b" ");
        stream.insert_real(point.y()).insert_bytes(b" ");
        stream.insert_real(point.z()).insert_bytes(b"\n");
        let tvertex = vertex_tshape(shape, "BRepTools_ShapeSet::WriteGeometry")?;
        for point in tvertex.points() {
            let Some(point) = point.get() else { continue };
            self.write_point_representation(point, stream)?;
        }
        stream.insert_bytes(b"0 0\n");
        Ok(())
    }

    /// `parameter kind references location` of one point representation.
    fn write_point_representation(
        &self,
        point: &dyn BRepPointRepresentation,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        stream.insert_real(point.parameter());
        if point.is_point_on_curve() {
            stream.insert_bytes(b" 1 ");
            put_int(stream, self.curves.index(&point.curve()?));
        } else if point.is_point_on_curve_on_surface() {
            stream.insert_bytes(b" 2 ");
            put_int(stream, self.curves2d.index(&point.pcurve()?));
            stream.insert_bytes(b" ");
            put_int(stream, self.surfaces.index(&point.surface()?));
        } else if point.is_point_on_surface() {
            stream.insert_bytes(b" 3 ");
            stream.insert_real(point.parameter2()?);
            stream.insert_bytes(b" ");
            put_int(stream, self.surfaces.index(&point.surface()?));
        }
        stream.insert_bytes(b" ");
        put_int(stream, self.locations().index(&point.location()));
        stream.insert_bytes(b"\n");
        Ok(())
    }

    fn write_edge(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        let tedge = edge_tshape(shape, "BRepTools_ShapeSet::WriteGeometry")?;
        stream.insert_bytes(b" ");
        stream.insert_real(tedge.tolerance());
        stream.insert_bytes(b" ");
        put_flag(stream, tedge.same_parameter());
        stream.insert_bytes(b" ");
        put_flag(stream, tedge.same_range());
        stream.insert_bytes(b" ");
        put_flag(stream, tedge.degenerated());
        stream.insert_bytes(b"\n");
        for representation in tedge.curves() {
            self.write_curve_representation(&representation, stream)?;
        }
        stream.insert_bytes(b"0\n");
        Ok(())
    }

    /// One curve representation record: 1 (3D curve), 2 or 3 (curve on a surface, on a
    /// closed surface), 4 (regularity) and, with triangles, 5 (3D polygon), 6 or 7
    /// (polygon on a triangulation, on a closed one); other kinds write nothing.
    fn write_curve_representation(
        &self,
        representation: &Handle<dyn BRepCurveRepresentation>,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        let Some(curve) = representation.get() else {
            return Ok(());
        };
        if curve.is_curve3d() {
            return self.write_curve3d(representation, curve, stream);
        }
        if curve.is_curve_on_surface() {
            return self.write_curve_on_surface(representation, curve, stream);
        }
        if curve.is_regularity() {
            return self.write_regularity(curve, stream);
        }
        if self.with_triangles {
            return self.write_mesh_representation(curve, stream);
        }
        Ok(())
    }

    fn write_curve3d(
        &self,
        representation: &Handle<dyn BRepCurveRepresentation>,
        curve: &dyn BRepCurveRepresentation,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        let curve3d = curve.curve3d()?;
        if curve3d.is_null() {
            return Ok(());
        }
        let (first, last) = gcurve_range(representation)?;
        stream.insert_bytes(b"1 ");
        stream.insert_bytes(b" ");
        put_int(stream, self.curves.index(&curve3d));
        stream.insert_bytes(b" ");
        put_int(stream, self.locations().index(&curve.location()));
        self.write_range(first, last, stream);
        Ok(())
    }

    /// `" " << first << " " << last << "\n"`.
    fn write_range(&self, first: f64, last: f64, stream: &mut IoStream) {
        stream.insert_bytes(b" ");
        stream.insert_real(first);
        stream.insert_bytes(b" ");
        stream.insert_real(last);
        stream.insert_bytes(b"\n");
    }

    /// Record 2 or 3; the regularity of a seam follows its second p-curve without a
    /// space. The UV points follow only in version 2 (the UV points of the second
    /// p-curve for a seam).
    fn write_curve_on_surface(
        &self,
        representation: &Handle<dyn BRepCurveRepresentation>,
        curve: &dyn BRepCurveRepresentation,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        let (first, last) = gcurve_range(representation)?;
        let closed = curve.is_curve_on_closed_surface();
        stream.insert_bytes(if closed { b"3 " } else { b"2 " });
        stream.insert_bytes(b" ");
        put_int(stream, self.curves2d.index(&curve.pcurve()?));
        if closed {
            stream.insert_bytes(b" ");
            put_int(stream, self.curves2d.index(&curve.pcurve2()?));
            print_regularity(curve.continuity()?, stream);
        }
        stream.insert_bytes(b" ");
        put_int(stream, self.surfaces.index(&curve.surface()?));
        stream.insert_bytes(b" ");
        put_int(stream, self.locations().index(&curve.location()));
        self.write_range(first, last, stream);
        if self.format_nb() == format_version::VERSION_2 {
            let (uv_first, uv_last) = if closed {
                uv_points2(representation)
            } else {
                uv_points(representation)
            };
            for (value, separator) in [
                (uv_first.x(), b" "),
                (uv_first.y(), b" "),
                (uv_last.x(), b" "),
                (uv_last.y(), b"\n"),
            ] {
                stream.insert_real(value);
                stream.insert_bytes(separator);
            }
        }
        Ok(())
    }

    fn write_regularity(
        &self,
        curve: &dyn BRepCurveRepresentation,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        stream.insert_bytes(b"4 ");
        print_regularity(curve.continuity()?, stream);
        stream.insert_bytes(b" ");
        put_int(stream, self.surfaces.index(&curve.surface()?));
        stream.insert_bytes(b" ");
        put_int(stream, self.locations().index(&curve.location()));
        stream.insert_bytes(b" ");
        put_int(stream, self.surfaces.index(&curve.surface2()?));
        stream.insert_bytes(b" ");
        put_int(stream, self.locations().index(&curve.location2()?));
        stream.insert_bytes(b"\n");
        Ok(())
    }

    /// Records 5, 6 and 7; a polygon on a surface is not written.
    fn write_mesh_representation(
        &self,
        curve: &dyn BRepCurveRepresentation,
        stream: &mut IoStream,
    ) -> Result<(), Failure> {
        if curve.is_polygon3d() {
            let polygon = curve.polygon3d()?;
            if !polygon.is_null() {
                stream.insert_bytes(b"5 ");
                stream.insert_bytes(b" ");
                put_int(stream, self.polygons3d.index(&polygon));
                stream.insert_bytes(b" ");
                put_int(stream, self.locations().index(&curve.location()));
                stream.insert_bytes(b"\n");
            }
            return Ok(());
        }
        if !curve.is_polygon_on_triangulation() {
            return Ok(());
        }
        let closed = curve.is_polygon_on_closed_triangulation();
        stream.insert_bytes(if closed { b"7 " } else { b"6 " });
        stream.insert_bytes(b" ");
        put_int(stream, self.nodes.index(&curve.polygon_on_triangulation()?));
        if closed {
            stream.insert_bytes(b" ");
            put_int(
                stream,
                self.nodes.index(&curve.polygon_on_triangulation2()?),
            );
        }
        stream.insert_bytes(b" ");
        put_int(stream, self.triangulations.index(&curve.triangulation()?));
        stream.insert_bytes(b" ");
        put_int(stream, self.locations().index(&curve.location()));
        stream.insert_bytes(b"\n");
        Ok(())
    }

    /// `natural-restriction  tolerance surface location` (zeros without a surface),
    /// then, with triangles or without a surface, `2  triangulation` without a newline.
    fn write_face(&self, shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
        let tface = face_tshape(shape, "BRepTools_ShapeSet::WriteGeometry")?;
        let surface = tface.surface();
        if surface.is_null() {
            put_int(stream, 0);
            stream.insert_bytes(b" ");
            stream.insert_bytes(b" ");
            stream.insert_real(tface.tolerance());
            stream.insert_bytes(b" 0 0\n");
        } else {
            put_flag(
                stream,
                BRepTool::natural_restriction(&topods::face(shape)?)?,
            );
            stream.insert_bytes(b" ");
            stream.insert_bytes(b" ");
            stream.insert_real(tface.tolerance());
            stream.insert_bytes(b" ");
            put_int(stream, self.surfaces.index(&surface));
            stream.insert_bytes(b" ");
            put_int(stream, self.locations().index(&tface.location()));
            stream.insert_bytes(b"\n");
        }
        if self.with_triangles || surface.is_null() {
            let triangulation = tface.triangulation(mesh_purpose::NONE);
            if !triangulation.is_null() {
                put_int(stream, 2);
                stream.insert_bytes(b" ");
                stream.insert_bytes(b" ");
                put_int(stream, self.triangulations.index(&triangulation));
            }
        }
        Ok(())
    }
}
