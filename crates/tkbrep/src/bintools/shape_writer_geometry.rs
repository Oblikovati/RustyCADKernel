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
// Rust translation of BinTools_ShapeWriter::WriteCurve (both overloads),
// WriteSurface, WritePolygon (both overloads) and WriteTriangulation
// (BinTools_ShapeWriter.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry objects of the shape-reference format: the empty marker for a null
//! handle, a back reference for an object already written, else the object type and
//! the object, whose position is recorded before it is written.
use super::curve_write::write_curve;
use super::curve2d_write::write_curve2d;
use super::handle_table::HandleKey;
use super::object_type::ObjectType;
use super::ostream::BinOStream;
use super::shape_writer::{BinShapeWriter, PositionMap};
use super::surface_write::write_surface;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation};

/// The prologue of every geometry writer: true when the object itself follows.
fn open_record<T: ?Sized + RuntimeTyped>(
    positions: &mut PositionMap<T>,
    handle: &Handle<T>,
    out: &mut BinOStream<'_>,
    (empty, kind): (ObjectType, ObjectType),
) -> bool {
    if handle.is_null() {
        out.write_object_type(empty);
        return false;
    }
    let key = HandleKey(handle.clone());
    if let Some(position) = positions.seek(&key).copied() {
        out.write_reference(position);
        return false;
    }
    positions.bind(key, out.position());
    out.write_object_type(kind);
    true
}

impl BinShapeWriter {
    /// `WriteCurve(theStream, Geom_Curve)`: the curve record of `BinTools_CurveSet`.
    pub(super) fn write_curve(
        &mut self,
        out: &mut BinOStream<'_>,
        curve: &Handle<dyn GeomCurve>,
    ) -> Result<(), Failure> {
        let kinds = (ObjectType::EMPTY_CURVE, ObjectType::CURVE);
        if !open_record(&mut self.curve_positions, curve, out, kinds) {
            return Ok(());
        }
        write_curve(curve, out)
    }

    /// `WriteCurve(theStream, Geom2d_Curve)`: the record of `BinTools_Curve2dSet`.
    pub(super) fn write_curve2d(
        &mut self,
        out: &mut BinOStream<'_>,
        curve: &Handle<dyn Geom2dCurve>,
    ) -> Result<(), Failure> {
        let kinds = (ObjectType::EMPTY_CURVE2D, ObjectType::CURVE2D);
        if !open_record(&mut self.curve2d_positions, curve, out, kinds) {
            return Ok(());
        }
        write_curve2d(curve, out)
    }

    /// `WriteSurface(theStream, theSurface)`: the record of `BinTools_SurfaceSet`.
    pub(super) fn write_surface(
        &mut self,
        out: &mut BinOStream<'_>,
        surface: &Handle<dyn GeomSurface>,
    ) -> Result<(), Failure> {
        let kinds = (ObjectType::EMPTY_SURFACE, ObjectType::SURFACE);
        if !open_record(&mut self.surface_positions, surface, out, kinds) {
            return Ok(());
        }
        write_surface(surface, out)
    }

    /// `WritePolygon(theStream, Poly_Polygon3D)`: the node count, the parameter flag,
    /// the deflection, the nodes and the parameters.
    pub(super) fn write_polygon3d(
        &mut self,
        out: &mut BinOStream<'_>,
        polygon: &Handle<Polygon3D>,
    ) -> Result<(), Failure> {
        let kinds = (ObjectType::EMPTY_POLYGON3D, ObjectType::POLYGON3D);
        if !open_record(&mut self.polygon3d_positions, polygon, out, kinds) {
            return Ok(());
        }
        let count = polygon.nb_nodes();
        out.write_integer(count);
        out.write_bool(polygon.has_parameters());
        out.write_real(polygon.deflection());
        let nodes = polygon.nodes();
        for index in 1..=count {
            out.write_point3(
                *nodes
                    .value(index)
                    .map_err(|e| e.into_failure("NCollection_Array1::Value"))?,
            );
        }
        if polygon.has_parameters() {
            let parameters = polygon.parameters()?;
            for index in 1..=count {
                out.write_real(
                    *parameters
                        .value(index)
                        .map_err(|e| e.into_failure("NCollection_Array1::Value"))?,
                );
            }
        }
        Ok(())
    }

    /// `WritePolygon(theStream, Poly_PolygonOnTriangulation)`: the node count and
    /// indices, the deflection, then `true` and the parameters, or `false`.
    pub(super) fn write_polygon_on_triangulation(
        &mut self,
        out: &mut BinOStream<'_>,
        polygon: &Handle<PolygonOnTriangulation>,
    ) -> Result<(), Failure> {
        let kinds = (
            ObjectType::EMPTY_POLYGON_ON_TRIANGULATION,
            ObjectType::POLYGON_ON_TRIANGULATION,
        );
        if !open_record(&mut self.polygon_positions, polygon, out, kinds) {
            return Ok(());
        }
        let nodes = polygon.nodes();
        let length = nodes.len() as i32;
        out.write_integer(length);
        for index in 1..=length {
            out.write_integer(
                *nodes
                    .value(index)
                    .map_err(|e| e.into_failure("NCollection_Array1::Value"))?,
            );
        }
        out.write_real(polygon.deflection());
        let parameters = polygon.parameters();
        let Some(parameters) = parameters.get() else {
            out.write_bool(false);
            return Ok(());
        };
        out.write_bool(true);
        for index in 1..=parameters.len() as i32 {
            out.write_real(parameters.value(index)?);
        }
        Ok(())
    }

    /// `WriteTriangulation(theStream, theTriangulation, theNeedToWriteNormals)`: the
    /// sizes, the UV and normal flags, the deflection, the nodes, the UV nodes, the
    /// triangles and, when asked, the normals. A triangulation already written is a
    /// back reference whatever the normal flag.
    pub(super) fn write_triangulation(
        &mut self,
        out: &mut BinOStream<'_>,
        triangulation: &Handle<Triangulation>,
        normals: bool,
    ) -> Result<(), Failure> {
        let kinds = (ObjectType::EMPTY_TRIANGULATION, ObjectType::TRIANGULATION);
        if !open_record(&mut self.triangulation_positions, triangulation, out, kinds) {
            return Ok(());
        }
        let (nb_nodes, nb_triangles) = (triangulation.nb_nodes(), triangulation.nb_triangles());
        out.write_integer(nb_nodes);
        out.write_integer(nb_triangles);
        out.write_bool(triangulation.has_uv_nodes());
        out.write_bool(normals);
        out.write_real(triangulation.deflection());
        for index in 1..=nb_nodes {
            out.write_point3(triangulation.node(index)?);
        }
        if triangulation.has_uv_nodes() {
            for index in 1..=nb_nodes {
                out.write_point2(triangulation.uv_node(index)?);
            }
        }
        for index in 1..=nb_triangles {
            out.write_triangle(&triangulation.triangle(index)?);
        }
        if normals {
            for index in 1..=nb_nodes {
                out.write_vec3f(triangulation.normal_vec3(index)?);
            }
        }
        Ok(())
    }
}
