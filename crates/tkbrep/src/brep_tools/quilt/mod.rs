// Created on: 1994-12-23
// Created by: Remi LEQUETTE
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
// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_Quilt.hxx and the members
// Add, Bind, IsCopied, Copy with the static NeedCopied and CopyShape of
// BRepTools_Quilt.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_Quilt`: glues faces along bound edges and vertices into shells.
mod shells;

use crate::brep::{BRepBuilder, BRepTool};
use crate::topexp::Explorer;
use crate::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use crate::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::IndexedDataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};

/// `NCollection_IndexedDataMap<TopoDS_Shape, TopoDS_Shape, TopTools_ShapeMapHasher>`.
type Bounds = IndexedDataMap<Shape, Shape, ShapeMapHasher>;

/// Glues faces into shells (`BRepTools_Quilt`): edges and vertices bound to new ones
/// are replaced in the faces added after the binding (the faces are copied), and
/// `shells` connects the faces sharing edges.
///
/// ```
/// use rustycad_tkbrep::brep_tools::Quilt;
/// let quilt = Quilt::new();
/// assert_eq!(quilt.shells()?.nb_children(), 0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct Quilt {
    bounds: Bounds,
    has_copy: bool,
}

impl Quilt {
    /// `BRepTools_Quilt()`; `Quilt::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Bind(Eold, Enew)`: `old` will be replaced by `new` (by its copy when it was
    /// copied), REVERSED when their orientations differ; the vertices of `old` are bound
    /// to the vertices of `new` with the same orientation. Nothing for a bound edge;
    /// `quilt.bind_edge(&old, &new)?`.
    pub fn bind_edge(&mut self, old: &Edge, new: &Edge) -> Result<(), Failure> {
        if self.bounds.contains(old) {
            return Ok(());
        }
        let mut target = new.clone();
        if self.is_copied(new) {
            target = topods::edge(&self.copy(new)?)?;
            target.set_orientation(new.orientation());
        }
        let orientation = if old.orientation() != target.orientation() {
            Orientation::Reversed
        } else {
            Orientation::Forward
        };
        self.bounds.add(
            old.oriented(Orientation::Forward),
            target.oriented(orientation),
        );
        for old_vertex in ShapeIterator::new(old, true, true) {
            if !self.bounds.contains(&old_vertex) {
                self.bind_matching_vertex(&old_vertex, &target)?;
            }
        }
        self.has_copy = true;
        Ok(())
    }

    /// Binds `old_vertex` to the first vertex of `edge` with its orientation (the copy of
    /// that vertex when it was copied).
    fn bind_matching_vertex(&mut self, old_vertex: &Shape, edge: &Edge) -> Result<(), Failure> {
        for new_vertex in ShapeIterator::new(edge, true, true) {
            if new_vertex.orientation() != old_vertex.orientation() {
                continue;
            }
            let mut vertex = topods::vertex(&new_vertex)?;
            if self.is_copied(&vertex) {
                vertex = topods::vertex(&self.copy(&vertex)?)?;
            }
            self.bounds.add(old_vertex.clone(), vertex.into_shape());
            return Ok(());
        }
        Ok(())
    }

    /// `Bind(Vold, Vnew)`: `old` will be replaced by `new`; nothing for a bound vertex;
    /// `quilt.bind_vertex(&old, &new)`.
    pub fn bind_vertex(&mut self, old: &Vertex, new: &Vertex) {
        if !self.bounds.contains(old) {
            self.bounds
                .add(old.clone().into_shape(), new.clone().into_shape());
        }
    }

    /// `Add(S)`: binds the free wires, edges and vertices of `shape` to themselves, then
    /// each face to itself or, when one of its edges is bound or has a bound vertex (after
    /// a binding), to a copy with the bound edges replaced (and the edges with bound
    /// vertices copied). Nothing for a shape already added; `quilt.add(&shape)?`.
    pub fn add(&mut self, shape: &Shape) -> Result<(), Failure> {
        if self.bounds.contains(shape) {
            return Ok(());
        }
        for (kind, avoid) in [
            (ShapeEnum::Wire, ShapeEnum::Face),
            (ShapeEnum::Edge, ShapeEnum::Wire),
            (ShapeEnum::Vertex, ShapeEnum::Edge),
        ] {
            for free in Explorer::new(shape, kind, avoid)? {
                self.bounds.add(free.clone(), free);
            }
        }
        for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            let face = topods::face(&face)?;
            let copy_face = self.has_copy && self.prepare_face_edges(&face)?;
            let new_face = if copy_face {
                self.copied_face(&face)?
            } else {
                face.clone()
            };
            self.bounds.add(face.into_shape(), new_face.into_shape());
        }
        Ok(())
    }

    /// Whether the face must be copied: one of its edges is bound, or has a bound vertex
    /// (that edge is then copied, `CopyShape`).
    fn prepare_face_edges(&mut self, face: &Face) -> Result<bool, Failure> {
        let mut copy_face = false;
        for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if self.bounds.contains(&edge) {
                copy_face = true;
            } else if self.needs_copy(&edge) {
                copy_face = true;
                self.copy_edge(&topods::edge(&edge)?)?;
            }
        }
        Ok(copy_face)
    }

    /// The static `NeedCopied`: a direct sub-shape of `shape` is bound.
    fn needs_copy(&self, shape: &Shape) -> bool {
        ShapeIterator::new(shape, true, true).any(|child| self.bounds.contains(&child))
    }

    /// The static `CopyShape(E, myBounds)`: an empty copy of the edge holding its vertices
    /// (the bound ones replaced) with its 3D range, bound FORWARD to the edge.
    fn copy_edge(&mut self, edge: &Edge) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut new_edge = edge.clone();
        new_edge.empty_copy()?;
        new_edge.set_orientation(Orientation::Forward);
        for vertex in ShapeIterator::new(edge, false, true) {
            let child = match self.bounds.find_from_key(&vertex) {
                Ok(bound) => bound.oriented(vertex.orientation()),
                Err(_) => vertex.clone(),
            };
            builder.add(&mut new_edge, &child)?;
        }
        let (first, last) = BRepTool::range(edge)?;
        builder.range(&new_edge, first, last, false)?;
        self.bounds.add(
            edge.clone().into_shape(),
            new_edge.oriented(Orientation::Forward),
        );
        Ok(())
    }

    /// The copy of a face: its wires rebuilt with the bound edges, each with the p-curve
    /// of the original edge (reversed with its range when the bound edge is REVERSED).
    fn copied_face(&self, face: &Face) -> Result<Face, Failure> {
        let builder = BRepBuilder::default();
        let mut new_face = face.clone();
        new_face.empty_copy()?;
        new_face.set_orientation(Orientation::Forward);
        for wire in ShapeIterator::new(face, false, true) {
            let wire = topods::wire(&wire)?;
            let new_wire = self.copied_wire(face, &wire)?;
            builder.add(&mut new_face, &new_wire)?;
        }
        new_face.set_orientation(face.orientation());
        Ok(new_face)
    }

    fn copied_wire(&self, face: &Face, wire: &Wire) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        let mut new_wire = Wire::default();
        builder.make_wire(&mut new_wire);
        for edge in ShapeIterator::new(wire, false, true) {
            let edge = topods::edge(&edge)?;
            let child = match self.bounds.find_from_key(&edge) {
                Ok(bound) => self.bound_edge_on_face(face, &edge, &topods::edge(bound)?)?,
                Err(_) => edge.clone().into_shape(),
            };
            builder.add(&mut new_wire, &child)?;
        }
        Ok(new_wire.oriented(wire.orientation()))
    }

    /// The bound edge `bound` of `edge` given the p-curve of `edge` on the face, oriented
    /// for the new wire.
    fn bound_edge_on_face(&self, face: &Face, edge: &Edge, bound: &Edge) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        let mut orientation = edge.orientation();
        let found = BRepTool::curve_on_face(edge, face)?;
        let (mut first, mut last) = (found.first, found.last);
        let tolerance = BRepTool::edge_tolerance(edge)?;
        if bound.orientation() == Orientation::Forward {
            builder.update_edge_pcurve_on_face(bound, &found.pcurve, face, tolerance)?;
        } else {
            // A REVERSED bound edge has its 3D curve the other way (the Bind convention):
            // the p-curve is reversed.
            orientation = top_abs::reverse(orientation);
            let Some(pcurve) = found.pcurve.get() else {
                return Err(crate::brep::errors::null_object(
                    "BRepTools_Quilt::Add: no p-curve of the edge on the face",
                ));
            };
            let reversed = pcurve.reversed()?;
            builder.update_edge_pcurve_on_face(bound, &reversed, face, tolerance)?;
            let previous_first = first;
            first = pcurve.reversed_parameter(last)?;
            last = pcurve.reversed_parameter(previous_first)?;
        }
        builder.range_on_face(bound, face, first, last)?;
        Ok(bound.oriented(orientation))
    }

    /// `IsCopied(S)`: `shape` is bound to another shape; `quilt.is_copied(&edge)`.
    pub fn is_copied(&self, shape: &Shape) -> bool {
        self.bounds
            .find_from_key(shape)
            .is_ok_and(|bound| !shape.is_same(bound))
    }

    /// `Copy(S)`: the shape `shape` is bound to; Standard_NoSuchObject when it is not
    /// copied; `quilt.copy(&edge)?`.
    pub fn copy(&self, shape: &Shape) -> Result<Shape, Failure> {
        if !self.is_copied(shape) {
            return Err(Failure::with_type(
                "Standard_NoSuchObject",
                "BRepTools_Quilt::Copy: the shape is not copied; expected a bound shape",
                "",
            ));
        }
        Ok(self.bounds.find_from_key(shape)?.clone())
    }
}
