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
// Rust translation of BinTools_ShapeSet::ReadPolygon3D, ReadPolygonOnTriangulation
// and ReadTriangulation (BinTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Reading the mesh sections of the grouped binary format back.
use super::errors::wrap_failure;
use super::format_version;
use super::geometry_in::read_point3;
use super::scalars::{get_bool, get_integer, get_real, get_short_real, stream_get};
use super::shape_set::BinShapeSet;
use super::shape_set_base::BinShapeSetBase;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::{HArray1OfReal, Vec3};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangle, Triangulation};

/// `char aHeader[255]` of the section readers.
const HEADER_BUFFER: usize = 255;

/// One mesh section header: `IS >> aHeader` must contain `tag` (`strstr`, so a longer
/// word that contains it is accepted), then the count and the line feed after it.
fn read_header(stream: &mut IoStream, tag: &[u8], missing: &str) -> Result<i32, Failure> {
    let header = stream.extract_word(HEADER_BUFFER);
    if stream.fail() || !header.windows(tag.len()).any(|window| window == tag) {
        return Err(Failure::new(missing));
    }
    let mut count = 0;
    stream.extract_int(&mut count);
    stream_get(stream);
    Ok(count)
}

/// `count` records through `read_record`, stopping when the progress indicator
/// breaks; failures are wrapped in `context`.
fn read_records(
    progress: &ProgressRange<'_>,
    scope_name: &'static str,
    count: i32,
    context: &str,
    mut read_record: impl FnMut() -> Result<(), Failure>,
) -> Result<(), Failure> {
    wrap_failure(context, || {
        let scope = ProgressScope::with_literal(progress, scope_name, f64::from(count), false);
        let mut index = 1;
        while index <= count && scope.more() {
            read_record()?;
            index += 1;
            drop(scope.next(1.0));
        }
        Ok(())
    })
}

impl BinShapeSet {
    /// `ReadPolygon3D(IS, theRange)`: a `Polygon3D n` section; a header without the tag
    /// raises Standard_Failure ("Not a Polygon3D section");
    /// `set.read_polygon3d(&mut stream, &ProgressRange::default())?`.
    pub fn read_polygon3d(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let missing = "BinTools_ShapeSet::ReadPolygon3D: Not a Polygon3D section";
        let count = read_header(stream, b"Polygon3D", missing)?;
        let context = "EXCEPTION in BinTools_ShapeSet::ReadPolygon3D(..)";
        let table = &mut self.polygons3d;
        read_records(progress, "Reading polygones 3D", count, context, || {
            table.add(&Handle::new(read_polygon3d_record(stream)?));
            Ok(())
        })
    }

    /// `ReadPolygonOnTriangulation(IS, theRange)`: a `PolygonOnTriangulations n`
    /// section; `set.read_polygon_on_triangulation(&mut stream, &ProgressRange::default())?`.
    pub fn read_polygon_on_triangulation(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let missing =
            "BinTools_ShapeSet::ReadPolygonOnTriangulation: Not a PolygonOnTriangulation section";
        let count = read_header(stream, b"PolygonOnTriangulations", missing)?;
        let context = "EXCEPTION in BinTools_ShapeSet::ReadPolygonOnTriangulation(..)";
        let table = &mut self.nodes;
        read_records(
            progress,
            "Reading Polygones on triangulation",
            count,
            context,
            || {
                table.add(&Handle::new(read_nodes_record(stream)?));
                Ok(())
            },
        )
    }

    /// `ReadTriangulation(IS, theRange)`: a `Triangulations n` section; each
    /// triangulation is stored with its normal flag (read from version 4 on);
    /// `set.read_triangulation(&mut stream, &ProgressRange::default())?`.
    pub fn read_triangulation(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let missing = "BinTools_ShapeSet::Triangulation: Not a Triangulation section";
        let count = read_header(stream, b"Triangulations", missing)?;
        let with_normal_flag = self.format_nb() >= format_version::VERSION_4;
        let context = "EXCEPTION in BinTools_ShapeSet::ReadTriangulation(..)";
        let table = &mut self.triangulations;
        read_records(progress, "Reading triangulation", count, context, || {
            let (triangulation, normals) = read_triangulation_record(stream, with_normal_flag)?;
            table.add(&Handle::new(triangulation), normals);
            Ok(())
        })
    }
}

fn read_polygon3d_record(stream: &mut IoStream) -> Result<Polygon3D, Failure> {
    let count = get_integer(stream)?;
    let has_parameters = get_bool(stream);
    let deflection = get_real(stream)?;
    let polygon = Polygon3D::new(count, has_parameters)?;
    polygon.set_deflection(deflection);
    for index in 1..=count {
        let point = read_point3(stream)?;
        polygon
            .change_nodes(|nodes| nodes.set_value(index, point))
            .map_err(|e| e.into_failure("NCollection_Array1::ChangeValue"))?;
    }
    if has_parameters {
        for index in 1..=count {
            let value = get_real(stream)?;
            polygon
                .change_parameters(|parameters| parameters.set_value(index, value))?
                .map_err(|e| e.into_failure("NCollection_Array1::ChangeValue"))?;
        }
    }
    Ok(polygon)
}

fn read_nodes_record(stream: &mut IoStream) -> Result<PolygonOnTriangulation, Failure> {
    let count = get_integer(stream)?;
    let polygon = PolygonOnTriangulation::new(count, false)?;
    for index in 1..=count {
        polygon.set_node(index, get_integer(stream)?)?;
    }
    polygon.set_deflection(get_real(stream)?);
    if get_bool(stream) {
        let parameters = HArray1OfReal::with_bounds(1, count)?;
        for index in 1..=count {
            parameters.set_value(index, get_real(stream)?)?;
        }
        polygon.set_parameters(Handle::new(parameters))?;
    }
    Ok(polygon)
}

/// One triangulation and whether it has normals (always false before version 4).
fn read_triangulation_record(
    stream: &mut IoStream,
    with_normal_flag: bool,
) -> Result<(Triangulation, bool), Failure> {
    let (nb_nodes, nb_triangles) = (get_integer(stream)?, get_integer(stream)?);
    let has_uv = get_bool(stream);
    let has_normals = with_normal_flag && get_bool(stream);
    let deflection = get_real(stream)?;
    let triangulation = Triangulation::with_sizes(nb_nodes, nb_triangles, has_uv, has_normals)?;
    triangulation.set_deflection(deflection);
    for index in 1..=nb_nodes {
        triangulation.set_node(index, read_point3(stream)?)?;
    }
    if has_uv {
        for index in 1..=nb_nodes {
            let uv = Point2::new(get_real(stream)?, get_real(stream)?);
            triangulation.set_uv_node(index, uv)?;
        }
    }
    for index in 1..=nb_triangles {
        let (a, b, c) = (
            get_integer(stream)?,
            get_integer(stream)?,
            get_integer(stream)?,
        );
        triangulation.set_triangle(index, Triangle::new(a, b, c))?;
    }
    if has_normals {
        for index in 1..=nb_nodes {
            let (x, y, z) = (
                get_short_real(stream)?,
                get_short_real(stream)?,
                get_short_real(stream)?,
            );
            triangulation.set_normal_vec3(index, Vec3::new(x, y, z))?;
        }
    }
    Ok((triangulation, has_normals))
}
