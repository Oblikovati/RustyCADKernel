// Created on: 1994-08-25
// Created by: Jacques GOUSSARD
// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of the face part of BRepTools_Modifier::Rebuild (the p-curves, the
// vertex parameters of edges without 3D curve and the polygons on triangulation)
// (ModelingData/TKBRep/BRepTools/BRepTools_Modifier.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The p-curves and the polygons on triangulation of a rebuilt face.
use super::Modifier;
use crate::brep::{BRepBuilder, BRepTool};
use crate::brep_tools::is_really_closed;
use crate::brep_tools::modification::BRepToolsModification;
use crate::topexp::Explorer;
use crate::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkmath::gp::Axis2d;
use rustycad_tkmath::poly::{PolygonOnTriangulation, mesh_purpose};

/// The face being rebuilt: the source, its result and the orientation used to explore
/// its edges (`fcor`).
struct FaceRebuild<'a> {
    face: &'a Face,
    result: Face,
    fcor: Orientation,
    rev_wires: bool,
    new_geom: bool,
    new_geometry: bool,
}

impl Modifier {
    /// The p-curves of each edge of a rebuilt face with new geometry, the vertex
    /// parameters of the edges left without 3D curve and the polygons on
    /// triangulation.
    pub(super) fn rebuild_pcurves(
        &mut self,
        face: &Face,
        m: &dyn BRepToolsModification,
        (result, rev_wires, new_geom, new_geometry): (&Shape, bool, bool, bool),
    ) -> Result<(), Failure> {
        let fcor = if face.orientation() == Orientation::Reversed {
            Orientation::Reversed
        } else {
            Orientation::Forward
        };
        let context = FaceRebuild {
            face,
            result: topods::face(result)?,
            fcor,
            rev_wires,
            new_geom,
            new_geometry,
        };
        let mut curve2d = Handle::<dyn Geom2dCurve>::default();
        for current in Explorer::new(&face.oriented(fcor), ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge = topods::edge(&current)?;
            self.rebuild_edge_pcurve(&context, &edge, m, &mut curve2d)?;
            self.copy_polygons_on_triangulation(&context, &edge, m)?;
        }
        Ok(())
    }

    /// The new p-curve of one edge, stored as a seam when the edge is closed on the
    /// face (or on another face of the same surface, OCC21772).
    fn rebuild_edge_pcurve(
        &mut self,
        context: &FaceRebuild<'_>,
        edge: &Edge,
        m: &dyn BRepToolsModification,
        curve2d: &mut Handle<dyn Geom2dCurve>,
    ) -> Result<(), Failure> {
        if !context.new_geom {
            return Ok(());
        }
        let new_edge = topods::edge(&self.result_of(edge)?)?;
        let mut tolerance = 0.0;
        if !m.new_curve2d(
            edge,
            context.face,
            &new_edge,
            &context.result,
            curve2d,
            &mut tolerance,
        )? {
            return Ok(());
        }
        if self.is_closed_for_pcurves(context, edge)? {
            self.update_seam(context, edge, curve2d)?;
        } else {
            BRepBuilder::default().update_edge_pcurve_on_face(
                &new_edge,
                curve2d,
                &context.result,
                0.0,
            )?;
        }
        let (c3d, _, _) = BRepTool::curve(&new_edge)?;
        if c3d.is_null() {
            self.update_vertices_without_curve(edge, m)?;
        }
        Ok(())
    }

    /// Whether the edge keeps two p-curves on the surface: it is a seam of the face
    /// (really closed when the face got a new surface), or it also bounds another face
    /// of the shape built on the same located surface.
    fn is_closed_for_pcurves(
        &self,
        context: &FaceRebuild<'_>,
        edge: &Edge,
    ) -> Result<bool, Failure> {
        if !BRepTool::is_closed_on_face(edge, context.face)? {
            return Ok(false);
        }
        if !context.new_geometry || is_really_closed(edge, context.face)? {
            return Ok(true);
        }
        let (surface, location) =
            BRepTool::surface_with_location(&self.result_or_self(context.face)?)?;
        for other in Explorer::new(&self.shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            if other.is_same(context.face) {
                continue;
            }
            let other = topods::face(&other)?;
            let (other_surface, other_location) =
                BRepTool::surface_with_location(&self.result_or_self(&other)?)?;
            if surface != other_surface || !location.is_equal(&other_location) {
                continue;
            }
            for other_edge in Explorer::new(&other, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if edge.is_same(&other_edge) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// The result of a face when it has a non-null one, the face otherwise.
    fn result_or_self(&self, face: &Face) -> Result<Face, Failure> {
        let result = if self.map.is_bound(face) {
            self.result_of(face)?
        } else {
            face.clone().into_shape()
        };
        if result.is_null() {
            return Ok(face.clone());
        }
        topods::face(&result)
    }

    /// The new p-curve with the other p-curve of the new edge (a line when it has
    /// none), on the FORWARD result face, in the order the edge and face orientations
    /// give; the range of the new edge becomes the range of its p-curve.
    fn update_seam(
        &self,
        context: &FaceRebuild<'_>,
        edge: &Edge,
        curve2d: &Handle<dyn Geom2dCurve>,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut current = topods::edge(&self.result_of(edge)?)?;
        let face = topods::face(&context.result.oriented(Orientation::Forward))?;
        let same = context.fcor == edge.orientation();
        let forward_first = (!context.rev_wires && !same) || (context.rev_wires && same);
        current.set_orientation(if forward_first {
            Orientation::Forward
        } else {
            Orientation::Reversed
        });
        let mut other = BRepTool::curve_on_face(&current, &face)?.pcurve;
        if other.is_null() {
            other = Handle::new(Geom2dLine::new(Axis2d::default())).cast();
        }
        let pair = if forward_first {
            (&other, curve2d)
        } else {
            (curve2d, &other)
        };
        builder.update_edge_pcurves_on_face(&current, pair, &face, 0.0)?;
        let found = BRepTool::curve_on_face(&current, &face)?;
        builder.range(&current, found.first, found.last, false)
    }

    /// The vertex parameters on a new edge without 3D curve, relative to the FORWARD
    /// edge.
    fn update_vertices_without_curve(
        &self,
        edge: &Edge,
        m: &dyn BRepToolsModification,
    ) -> Result<(), Failure> {
        let new_edge = topods::edge(&self.result_of(edge)?)?;
        for current in Explorer::new(edge, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            let vertex = topods::vertex(&current)?;
            let (parameter, tolerance) = self.vertex_parameter(&vertex, edge, m)?;
            let mut relative = vertex.orientation();
            if edge.orientation() == Orientation::Reversed {
                relative = top_abs::reverse(relative);
            }
            let local = topods::vertex(&self.result_of(&vertex)?.oriented(relative))?;
            BRepBuilder::default()
                .update_vertex_on_edge(&local, parameter, &new_edge, tolerance)?;
        }
        Ok(())
    }

    /// The polygons on triangulation of the edge (both of a seam) the modification
    /// gives, stored on the triangulation of the result face.
    fn copy_polygons_on_triangulation(
        &self,
        context: &FaceRebuild<'_>,
        edge: &Edge,
        m: &dyn BRepToolsModification,
    ) -> Result<(), Failure> {
        let mut first = Handle::<PolygonOnTriangulation>::default();
        let mut second = Handle::<PolygonOnTriangulation>::default();
        let mut is_new = m.new_polygon_on_triangulation(edge, context.face, &mut first)?;
        if is_really_closed(edge, context.face)? {
            let reversed = topods::edge(&edge.reversed())?;
            is_new =
                m.new_polygon_on_triangulation(&reversed, context.face, &mut second)? || is_new;
            if first.is_null() && !second.is_null() {
                first = std::mem::take(&mut second);
            }
        }
        if !is_new {
            return Ok(());
        }
        let face_result = topods::face(&self.result_of(context.face)?)?;
        let (mesh, location) = BRepTool::triangulation(&face_result, mesh_purpose::NONE)?;
        let new_edge = topods::edge(&self.result_of(edge)?)?;
        let builder = BRepBuilder::default();
        if second.is_null() {
            return builder
                .update_edge_polygon_on_triangulation(&new_edge, &first, &mesh, &location);
        }
        let pair = if edge.orientation() == Orientation::Forward {
            (&first, &second)
        } else {
            (&second, &first)
        };
        builder.update_edge_polygons_on_triangulation(&new_edge, pair, &mesh, &location)
    }
}
