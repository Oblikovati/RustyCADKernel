// Created on: 1993-01-19
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
// Rust translation of TopExp::FirstVertex, LastVertex, Vertices and CommonVertex
// (ModelingData/TKBRep/TopExp/TopExp.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertices of edges and wires. The casts are unchecked, as in the native file
//! (`No_Standard_TypeMismatch`): a child of another type is returned as a vertex.
use crate::topods::{Edge, Shape, ShapeIterator, Vertex, Wire};
use crate::toptools::MapOfShape;
use rustycad_tkg3d::top_abs::Orientation;

/// The first direct sub-shape of `edge` with the given orientation, or a null vertex.
fn oriented_vertex(edge: &Shape, cum_ori: bool, wanted: Orientation) -> Vertex {
    let mut iterator = ShapeIterator::new(edge, cum_ori, true);
    while let Ok(child) = iterator.value() {
        if child.orientation() == wanted {
            return Vertex::unchecked(child.clone());
        }
        iterator.next();
    }
    Vertex::default()
}

/// `FirstVertex(E, CumOri)`: the first FORWARD vertex of `edge` (composed with the
/// edge orientation when `cum_ori`), or a null vertex; the native default of `CumOri`
/// is false; `topexp::first_vertex(&edge, false)`.
pub fn first_vertex(edge: &Edge, cum_ori: bool) -> Vertex {
    oriented_vertex(edge, cum_ori, Orientation::Forward)
}

/// `LastVertex(E, CumOri)`: the first REVERSED vertex of `edge`, or a null vertex;
/// `topexp::last_vertex(&edge, false)`.
pub fn last_vertex(edge: &Edge, cum_ori: bool) -> Vertex {
    oriented_vertex(edge, cum_ori, Orientation::Reversed)
}

/// `Vertices(E, Vfirst, Vlast, CumOri)`: the FORWARD and REVERSED vertices of `edge`,
/// null when absent. Unlike [`first_vertex`], the whole list is read, so with two
/// FORWARD vertices the last one is returned (native behaviour);
/// `let (first, last) = topexp::vertices(&edge, false);`.
pub fn vertices(edge: &Edge, cum_ori: bool) -> (Vertex, Vertex) {
    let (mut first, mut last) = (Vertex::default(), Vertex::default());
    let mut iterator = ShapeIterator::new(edge, cum_ori, true);
    while let Ok(child) = iterator.value() {
        match child.orientation() {
            Orientation::Forward => first = Vertex::unchecked(child.clone()),
            Orientation::Reversed => last = Vertex::unchecked(child.clone()),
            _ => {}
        }
        iterator.next();
    }
    (first, last)
}

/// `Vertices(W, Vfirst, Vlast)`: the free ends of an open wire; the same vertex
/// (FORWARD, then REVERSED) for a closed wire; two null vertices for a non-manifold
/// wire. The ends are read from a hash map, so with two ends of one orientation the
/// choice follows the hash order (TShape addresses), natively as here;
/// `let (first, last) = topexp::wire_vertices(&wire);`.
pub fn wire_vertices(wire: &Wire) -> (Vertex, Vertex) {
    let mut ends = MapOfShape::default();
    let mut last_end = Vertex::default();
    let mut iterator = ShapeIterator::new(wire, true, true);
    while let Ok(child) = iterator.value() {
        let edge = Edge::unchecked(child.clone());
        let (mut start, mut end) = vertices(&edge, false);
        if edge.orientation() == Orientation::Reversed {
            std::mem::swap(&mut start, &mut end);
        }
        start.set_orientation(Orientation::Forward);
        end.set_orientation(Orientation::Reversed);
        toggle_end(&mut ends, &start);
        toggle_end(&mut ends, &end);
        last_end = end;
        iterator.next();
    }
    if ends.is_empty() {
        let first = Vertex::unchecked(last_end.oriented(Orientation::Forward));
        return (
            first,
            Vertex::unchecked(last_end.oriented(Orientation::Reversed)),
        );
    }
    if ends.extent() != 2 {
        return (Vertex::default(), Vertex::default());
    }
    (
        map_end(&ends, Orientation::Forward),
        map_end(&ends, Orientation::Reversed),
    )
}

/// `if (!vmap.Add(V)) vmap.Remove(V);`.
fn toggle_end(ends: &mut MapOfShape, vertex: &Shape) {
    if !ends.add(vertex.clone()) {
        ends.remove(vertex);
    }
}

/// The first key of `ends`, in map order, with the orientation `wanted`.
fn map_end(ends: &MapOfShape, wanted: Orientation) -> Vertex {
    let found = ends.iter().find(|shape| shape.orientation() == wanted);
    found.map_or_else(Vertex::default, |shape| Vertex::unchecked(shape.clone()))
}

/// `CommonVertex(E1, E2, V)`: a vertex of `first` that `second` also has (same TShape
/// and location), trying the first vertex of `first`, then its last one; `None` when
/// there is none; `topexp::common_vertex(&e1, &e2)`.
pub fn common_vertex(first: &Edge, second: &Edge) -> Option<Vertex> {
    let (first1, last1) = vertices(first, false);
    let (first2, last2) = vertices(second, false);
    if first1.is_same(&first2) || first1.is_same(&last2) {
        return Some(first1);
    }
    if last1.is_same(&first2) || last1.is_same(&last2) {
        return Some(last1);
    }
    None
}
