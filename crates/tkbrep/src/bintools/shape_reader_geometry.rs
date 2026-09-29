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
// Rust translation of BinTools_ShapeReader::ReadCurve, ReadCurve2d, ReadSurface,
// ReadPolygon3d, ReadPolygon and ReadTriangulation (BinTools_ShapeReader.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry objects of the shape-reference format: a back reference to an object
//! read before (or read now at its position), the object itself after its type, or a
//! null handle for any other type (the empty markers included).
use super::curve_read::read_curve;
use super::curve2d_read::read_curve2d;
use super::istream::BinIStream;
use super::object_type::ObjectType;
use super::shape_reader::{BinShapeReader, ObjectMap};
use super::surface_read::read_surface;
use rustycad_tkernel::ncollection::{HArray1OfReal, Vec3};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangle, Triangulation};

/// How one kind of object is found and read.
struct ObjectKind<T: ?Sized + RuntimeTyped> {
    kind: ObjectType,
    map: fn(&mut BinShapeReader) -> &mut ObjectMap<T>,
    body: fn(&mut BinIStream<'_>) -> Result<Handle<T>, Failure>,
}

impl BinShapeReader {
    /// The reading scheme every geometry reader shares; the object is recorded at the
    /// position of its type byte.
    fn read_object<T: ?Sized + RuntimeTyped>(
        &mut self,
        input: &mut BinIStream<'_>,
        object: &ObjectKind<T>,
    ) -> Result<Handle<T>, Failure> {
        let position = input.position();
        let kind = input.read_type();
        if input.is_reference() {
            let reference = input.read_reference()?;
            if let Some(found) = (object.map)(self).seek(&reference) {
                return Ok(found.clone());
            }
            let current = input.position();
            input.go_to(reference);
            let result = self.read_object(input, object)?;
            input.go_to(current);
            return Ok(result);
        }
        if kind != object.kind {
            return Ok(Handle::default());
        }
        let result = (object.body)(input)?;
        (object.map)(self).bind(position, result.clone());
        Ok(result)
    }

    /// `ReadCurve(theStream)`: the record of `BinTools_CurveSet::ReadCurve`.
    pub(super) fn read_curve(
        &mut self,
        input: &mut BinIStream<'_>,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        let object = ObjectKind {
            kind: ObjectType::CURVE,
            map: |reader| &mut reader.curve_positions,
            body: |input| {
                let curve = read_curve(input.stream())?;
                input.update_position();
                Ok(curve)
            },
        };
        self.read_object(input, &object)
    }

    /// `ReadCurve2d(theStream)`: the record of `BinTools_Curve2dSet::ReadCurve2d`.
    pub(super) fn read_curve2d(
        &mut self,
        input: &mut BinIStream<'_>,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let object = ObjectKind {
            kind: ObjectType::CURVE2D,
            map: |reader| &mut reader.curve2d_positions,
            body: |input| {
                let curve = read_curve2d(input.stream())?;
                input.update_position();
                Ok(curve)
            },
        };
        self.read_object(input, &object)
    }

    /// `ReadSurface(theStream)`: the record of `BinTools_SurfaceSet::ReadSurface`.
    pub(super) fn read_surface(
        &mut self,
        input: &mut BinIStream<'_>,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        let object = ObjectKind {
            kind: ObjectType::SURFACE,
            map: |reader| &mut reader.surface_positions,
            body: |input| {
                let surface = read_surface(input.stream())?;
                input.update_position();
                Ok(surface)
            },
        };
        self.read_object(input, &object)
    }

    /// `ReadPolygon3d(theStream)`.
    pub(super) fn read_polygon3d(
        &mut self,
        input: &mut BinIStream<'_>,
    ) -> Result<Handle<Polygon3D>, Failure> {
        let object = ObjectKind {
            kind: ObjectType::POLYGON3D,
            map: |reader| &mut reader.polygon3d_positions,
            body: |input| polygon3d_body(input).map(Handle::new),
        };
        self.read_object(input, &object)
    }

    /// `ReadPolygon(theStream)`: a polygon on a triangulation.
    pub(super) fn read_polygon_on_triangulation(
        &mut self,
        input: &mut BinIStream<'_>,
    ) -> Result<Handle<PolygonOnTriangulation>, Failure> {
        let object = ObjectKind {
            kind: ObjectType::POLYGON_ON_TRIANGULATION,
            map: |reader| &mut reader.polygon_positions,
            body: |input| polygon_on_triangulation_body(input).map(Handle::new),
        };
        self.read_object(input, &object)
    }

    /// `ReadTriangulation(theStream)`.
    pub(super) fn read_triangulation(
        &mut self,
        input: &mut BinIStream<'_>,
    ) -> Result<Handle<Triangulation>, Failure> {
        let object = ObjectKind {
            kind: ObjectType::TRIANGULATION,
            map: |reader| &mut reader.triangulation_positions,
            body: |input| triangulation_body(input).map(Handle::new),
        };
        self.read_object(input, &object)
    }
}

/// The node count, the parameter flag, the deflection, the nodes and the parameters.
fn polygon3d_body(input: &mut BinIStream<'_>) -> Result<Polygon3D, Failure> {
    let count = input.read_integer()?;
    let has_parameters = input.read_bool();
    let polygon = Polygon3D::new(count, has_parameters)?;
    polygon.set_deflection(input.read_real()?);
    for index in 1..=count {
        let point = input.read_pnt()?;
        polygon
            .change_nodes(|nodes| nodes.set_value(index, point))
            .map_err(|e| e.into_failure("NCollection_Array1::ChangeValue"))?;
    }
    if has_parameters {
        for index in 1..=count {
            let value = input.read_real()?;
            polygon
                .change_parameters(|parameters| parameters.set_value(index, value))?
                .map_err(|e| e.into_failure("NCollection_Array1::ChangeValue"))?;
        }
    }
    Ok(polygon)
}

/// The node count and indices, the deflection, then the parameters after a true flag.
fn polygon_on_triangulation_body(
    input: &mut BinIStream<'_>,
) -> Result<PolygonOnTriangulation, Failure> {
    let count = input.read_integer()?;
    let polygon = PolygonOnTriangulation::new(count, false)?;
    for index in 1..=count {
        polygon.set_node(index, input.read_integer()?)?;
    }
    polygon.set_deflection(input.read_real()?);
    if input.read_bool() {
        let parameters = HArray1OfReal::with_bounds(1, count)?;
        for index in 1..=count {
            parameters.set_value(index, input.read_real()?)?;
        }
        polygon.set_parameters(Handle::new(parameters))?;
    }
    Ok(polygon)
}

/// The sizes, the UV and normal flags, the deflection, the nodes, the UV nodes, the
/// triangles and the normals (read as floats without a check, as natively).
fn triangulation_body(input: &mut BinIStream<'_>) -> Result<Triangulation, Failure> {
    let (nb_nodes, nb_triangles) = (input.read_integer()?, input.read_integer()?);
    let (has_uv, has_normals) = (input.read_bool(), input.read_bool());
    let triangulation = Triangulation::with_sizes(nb_nodes, nb_triangles, has_uv, has_normals)?;
    triangulation.set_deflection(input.read_real()?);
    for index in 1..=nb_nodes {
        triangulation.set_node(index, input.read_pnt()?)?;
    }
    if has_uv {
        for index in 1..=nb_nodes {
            let uv = Point2::new(input.read_real()?, input.read_real()?);
            triangulation.set_uv_node(index, uv)?;
        }
    }
    for index in 1..=nb_triangles {
        let nodes = (
            input.read_integer()?,
            input.read_integer()?,
            input.read_integer()?,
        );
        triangulation.set_triangle(index, Triangle::new(nodes.0, nodes.1, nodes.2))?;
    }
    if has_normals {
        for index in 1..=nb_nodes {
            let normal = Vec3::new(
                input.read_short_real(),
                input.read_short_real(),
                input.read_short_real(),
            );
            triangulation.set_normal_vec3(index, normal)?;
        }
    }
    Ok(triangulation)
}
