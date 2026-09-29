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
// Rust translation of BRepTools_Modifier::Rebuild
// (ModelingData/TKBRep/BRepTools/BRepTools_Modifier.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_Modifier::Rebuild`: one shape and, recursively, its sub-shapes.
//!
//! Natively `result` is a reference to `myMap(S)`, so every change of the result is
//! visible in the map at once. Here the result is a local value written back to the
//! map at the points where the native code, or a recursive call, reads the map.
use super::{Modifier, set_shape_flags};
use crate::brep::{BRepBuilder, BRepTool};
use crate::brep_tools::modification::BRepToolsModification;
use crate::topexp::Explorer;
use crate::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkmath::poly::{Polygon3D, Triangulation};

/// What the geometry step decided for the shape being rebuilt.
#[derive(Default)]
pub(super) struct RebuildState {
    /// The shape is rebuilt.
    pub(super) rebuild: bool,
    /// The wires of the face are reversed (`RevWires`).
    pub(super) rev_wires: bool,
    /// The final orientation of the result (`ResOr`).
    pub(super) res_or: Option<Orientation>,
    /// The edge got no 3D curve (`No3DCurve`).
    pub(super) no_3d_curve: bool,
}

impl Modifier {
    /// `Rebuild(S, M, theNewGeom, theProgress)`: the result of `shape` (computed once);
    /// true when it differs from `shape`. `new_geom` tells whether the shape or a
    /// sub-shape got new geometry.
    pub(super) fn rebuild(
        &mut self,
        shape: &Shape,
        m: &dyn BRepToolsModification,
        new_geom: &mut bool,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let mut result = self.result_of(shape)?;
        if !result.is_null() {
            *new_geom = self.has_new_geom.contains(shape);
            return Ok(!shape.is_same(&result));
        }
        let kind = shape.shape_type()?;
        let mut state = RebuildState::default();
        match kind {
            ShapeEnum::Face => {
                self.new_face_geometry(&topods::face(shape)?, m, &mut result, &mut state)?
            }
            ShapeEnum::Edge => {
                self.new_edge_geometry(&topods::edge(shape)?, m, &mut result, &mut state)?
            }
            _ => {}
        }
        self.store(shape, &result)?;
        let new_geometry = state.rebuild;
        *new_geom = state.rebuild;
        if !self.rebuild_children(shape, m, (&mut state.rebuild, new_geom), progress)? {
            return Ok(false);
        }
        if *new_geom {
            self.has_new_geom.add(shape.clone());
        }
        if state.rebuild && !new_geometry {
            result = shape.empty_copied()?;
            result.set_orientation(Orientation::Forward);
        }
        if state.rebuild {
            self.copy_sub_shapes(shape, &mut result, &state)?;
            self.store(shape, &result)?;
            self.rebuild_geometry(shape, kind, m, (&result, &state, *new_geom, new_geometry))?;
            result.set_orientable(shape.orientable()?)?;
            result.set_closed(shape.closed()?)?;
            result.set_infinite(shape.infinite()?)?;
        } else {
            result = shape.clone();
        }
        result.set_orientation(state.res_or.unwrap_or(Orientation::Forward));
        set_shape_flags(shape, &result)?;
        self.store(shape, &result)?;
        Ok(state.rebuild)
    }

    /// Rebuilds every sub-shape; false when the progress indicator breaks.
    fn rebuild_children(
        &mut self,
        shape: &Shape,
        m: &dyn BRepToolsModification,
        (rebuild, new_geom): (&mut bool, &mut bool),
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let children: Vec<Shape> = ShapeIterator::new(shape, false, true).collect();
        let scope = ProgressScope::with_literal(
            progress,
            "Converting SubShapes",
            children.len() as f64,
            false,
        );
        for child in &children {
            if !scope.more() {
                break;
            }
            let mut sub_new_geom = false;
            let sub_rebuilt = self.rebuild(child, m, &mut sub_new_geom, &scope.next(1.0))?;
            *rebuild = sub_rebuilt || *rebuild;
            *new_geom = *new_geom || sub_new_geom;
        }
        Ok(scope.more())
    }

    /// The results of the sub-shapes added to the result, reversed when the wires are
    /// reversed or the sub-shape result is REVERSED.
    fn copy_sub_shapes(
        &self,
        shape: &Shape,
        result: &mut Shape,
        state: &RebuildState,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        for child in ShapeIterator::new(shape, false, true) {
            let mut orientation = child.orientation();
            let child_result = self.result_of(&child)?;
            if state.rev_wires || child_result.orientation() == Orientation::Reversed {
                orientation = top_abs::reverse(orientation);
            }
            builder.add(result, &child_result.oriented(orientation))?;
        }
        Ok(())
    }

    /// The p-curves of a rebuilt face, or the vertex parameters of a rebuilt edge with
    /// a 3D curve.
    fn rebuild_geometry(
        &mut self,
        shape: &Shape,
        kind: ShapeEnum,
        m: &dyn BRepToolsModification,
        (result, state, new_geom, new_geometry): (&Shape, &RebuildState, bool, bool),
    ) -> Result<(), Failure> {
        if kind == ShapeEnum::Face {
            let face = topods::face(shape)?;
            return self.rebuild_pcurves(
                &face,
                m,
                (result, state.rev_wires, new_geom, new_geometry),
            );
        }
        if kind == ShapeEnum::Edge && !state.no_3d_curve {
            return self.update_edge_vertices(&topods::edge(shape)?, m, result);
        }
        Ok(())
    }

    /// The new face: its new surface (at the face-relative location, with the natural
    /// restriction flag) and, when the modification gives one, its new triangulation.
    fn new_face_geometry(
        &self,
        face: &Face,
        m: &dyn BRepToolsModification,
        result: &mut Shape,
        state: &mut RebuildState,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        state.rebuild = self.ns_info.is_bound(face);
        let mut new_face = Face::default();
        if let Ok(info) = self.ns_info.find(face) {
            state.rev_wires = info.rev_wires;
            let location = info.location.predivided(face.location())?;
            builder.make_face_on_located_surface(
                &mut new_face,
                &info.surface,
                &location,
                info.tolerance,
            )?;
            new_face.set_location(face.location(), false)?;
            if info.rev_face {
                state.res_or = Some(Orientation::Reversed);
            }
            builder.natural_restriction(&new_face, BRepTool::natural_restriction(face)?)?;
            *result = new_face.clone().into_shape();
        }
        let mut triangulation = Handle::<Triangulation>::default();
        if m.new_triangulation(face, &mut triangulation)? {
            if state.rebuild {
                builder.update_face_triangulation(&new_face, &triangulation, true)?;
            } else {
                builder.make_face_with_triangulation(&mut new_face, &triangulation)?;
                new_face.set_location(face.location(), false)?;
                *result = new_face.into_shape();
            }
            state.rebuild = true;
        }
        Ok(())
    }

    /// The new edge: its new curve (or none, keeping the degenerated flag and the
    /// tolerance) at the edge-relative location with the SameParameter and SameRange
    /// flags, and, when the modification gives one, its new 3D polygon.
    fn new_edge_geometry(
        &self,
        edge: &Edge,
        m: &dyn BRepToolsModification,
        result: &mut Shape,
        state: &mut RebuildState,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        state.rebuild = self.nc_info.is_bound(edge);
        let mut new_edge = Edge::default();
        if let Ok(info) = self.nc_info.find(edge) {
            if info.curve.is_null() {
                builder.make_edge(&mut new_edge)?;
                builder.degenerated(&new_edge, BRepTool::degenerated(edge)?)?;
                builder.update_edge_tolerance(&new_edge, info.tolerance)?;
                state.no_3d_curve = true;
            } else {
                let location = info.location.predivided(edge.location())?;
                builder.make_edge_with_located_curve(
                    &mut new_edge,
                    &info.curve,
                    &location,
                    info.tolerance,
                )?;
                state.no_3d_curve = false;
            }
            new_edge.set_location(edge.location(), false)?;
            builder.same_parameter(&new_edge, BRepTool::same_parameter(edge)?)?;
            builder.same_range(&new_edge, BRepTool::same_range(edge)?)?;
            *result = new_edge.clone().into_shape();
        }
        let mut polygon = Handle::<Polygon3D>::default();
        if m.new_polygon(edge, &mut polygon)? {
            if state.rebuild {
                builder.update_edge_polygon3d(&new_edge, &polygon, edge.location())?;
            } else {
                builder.make_edge_with_polygon3d(&mut new_edge, &polygon)?;
                new_edge.set_location(edge.location(), false)?;
                *result = new_edge.into_shape();
            }
            state.rebuild = true;
        }
        Ok(())
    }

    /// The parameters of the vertices of a rebuilt edge with a 3D curve (the new ones,
    /// else the old ones), relative to the FORWARD edge; a kept vertex is updated only
    /// with a mutable input.
    fn update_edge_vertices(
        &self,
        edge: &Edge,
        m: &dyn BRepToolsModification,
        result: &Shape,
    ) -> Result<(), Failure> {
        let mut orientation = edge.orientation();
        if orientation != Orientation::Reversed {
            orientation = Orientation::Forward;
        }
        let new_edge = topods::edge(result)?;
        for current in Explorer::new(
            &edge.oriented(orientation),
            ShapeEnum::Vertex,
            ShapeEnum::Shape,
        )? {
            let vertex = topods::vertex(&current)?;
            let (parameter, tolerance) = self.vertex_parameter(&vertex, edge, m)?;
            let mut relative = vertex.orientation();
            if orientation == Orientation::Reversed {
                relative = top_abs::reverse(relative);
            }
            let local = topods::vertex(&self.result_of(&vertex)?.oriented(relative))?;
            if self.mutable_input || !local.is_same(&vertex) {
                BRepBuilder::default()
                    .update_vertex_on_edge(&local, parameter, &new_edge, tolerance)?;
            }
        }
        Ok(())
    }

    /// `M->NewParameter(vertex, edge, param, tol)`, else the vertex tolerance and its
    /// parameter on the edge.
    pub(super) fn vertex_parameter(
        &self,
        vertex: &Vertex,
        edge: &Edge,
        m: &dyn BRepToolsModification,
    ) -> Result<(f64, f64), Failure> {
        let (mut parameter, mut tolerance) = (0.0, 0.0);
        if !m.new_parameter(vertex, edge, &mut parameter, &mut tolerance)? {
            tolerance = BRepTool::vertex_tolerance(vertex)?;
            parameter = BRepTool::parameter(vertex, edge)?;
        }
        Ok((parameter, tolerance))
    }
}
