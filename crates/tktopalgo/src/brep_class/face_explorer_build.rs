// Created on: 1992-11-19
// Created by: Remi LEQUETTE
// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of the BRepClass_FaceExplorer constructor (BRepClass_FaceExplorer.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constructor body of `BRepClass_FaceExplorer`: the wires and edges of the face,
//! the next edge of each edge and the edges probed for classification segments.
use super::edge::BRepClassEdge;
use super::face_explorer::WireData;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The anonymous `VertexEdges` of the native file: the first two edges met at a
/// vertex of a wire and the number of edge occurrences at it.
#[derive(Default)]
struct VertexEdges {
    first: Edge,
    second: Edge,
    count: u32,
}

impl VertexEdges {
    fn add(&mut self, edge: &Edge) {
        if self.count == 0 {
            self.first = edge.clone();
        } else if self.count == 1 {
            self.second = edge.clone();
        }
        self.count += 1;
    }
}

/// The anonymous `EdgeOccurrences`: the first and last index of an edge (IsSame) in
/// the edge list; the other occurrences chain through `next_occurrence`.
#[derive(Clone, Copy, Debug, Default)]
struct EdgeOccurrences {
    first: usize,
    last: usize,
}

/// What the native constructor fills: the wires, the edges and the probe edges.
pub(super) struct ExplorerTopology {
    pub(super) wires: Vec<WireData>,
    pub(super) edges: Vec<BRepClassEdge>,
    pub(super) probe_edges: Vec<usize>,
    occurrences: DataMap<Shape, EdgeOccurrences, ShapeMapHasher>,
    // `aNextOccurrence`: the next index of the same edge, None for `THE_NO_EDGE`.
    next_occurrence: Vec<Option<usize>>,
}

impl ExplorerTopology {
    /// The constructor body over `face` (already FORWARD): each wire, then the probe
    /// edges of the face.
    pub(super) fn of_face(face: &Face) -> Result<Self, Failure> {
        let mut topology = Self {
            wires: Vec::new(),
            edges: Vec::new(),
            probe_edges: Vec::new(),
            occurrences: DataMap::default(),
            next_occurrence: Vec::new(),
        };
        for wire in Explorer::new(face, ShapeEnum::Wire, ShapeEnum::Shape)? {
            topology.add_wire(&wire, face)?;
        }
        for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge: Edge = topods::edge(&edge)?;
            let index: usize = topology.probe_edge_index(&edge, face)?;
            topology.probe_edges.push(index);
        }
        Ok(topology)
    }

    // One wire: its edges with their pcurves, their occurrences, the edges at each
    // vertex, then the next edge of each edge.
    fn add_wire(&mut self, wire: &Shape, face: &Face) -> Result<(), Failure> {
        let mut data = WireData {
            first_edge: self.edges.len(),
            nb_edges: 0,
        };
        let mut vertex_edges: DataMap<Shape, VertexEdges, ShapeMapHasher> = DataMap::default();
        for edge in Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge: Edge = topods::edge(&edge)?;
            self.edges.push(cached_edge(&edge, face)?);
            self.record_occurrence(&edge, self.edges.len() - 1);
            for vertex in Explorer::new(&edge, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                vertex_edges
                    .try_emplaced(vertex, VertexEdges::default)
                    .add(&edge);
            }
            data.nb_edges += 1;
        }
        for index in data.first_edge..data.first_edge + data.nb_edges {
            link_next_edge(&mut self.edges[index], &vertex_edges);
        }
        self.wires.push(data);
        Ok(())
    }

    // Chains the new index `index` of `edge` after its last occurrence.
    fn record_occurrence(&mut self, edge: &Edge, index: usize) {
        self.next_occurrence.push(None);
        let Some(occurrence) = self.occurrences.change_seek(edge) else {
            let first = EdgeOccurrences {
                first: index,
                last: index,
            };
            self.occurrences.bind(Shape::clone(edge), first);
            return;
        };
        self.next_occurrence[occurrence.last] = Some(index);
        occurrence.last = index;
    }

    // The index of the wire edge equal (IsEqual) to `edge`, or of a new edge
    // appended for it when no wire holds it.
    fn probe_edge_index(&mut self, edge: &Edge, face: &Face) -> Result<usize, Failure> {
        let mut candidate: Option<usize> = self.occurrences.seek(edge).map(|found| found.first);
        while let Some(index) = candidate {
            if self.edges[index].edge().is_equal(edge) {
                return Ok(index);
            }
            candidate = self.next_occurrence.get(index).copied().flatten();
        }
        self.edges.push(cached_edge(edge, face)?);
        Ok(self.edges.len() - 1)
    }
}

// `BRepClass_Edge(E, F)` with `cacheGeometry`: the pcurve of the edge on the face and
// its range, when the edge has one.
fn cached_edge(edge: &Edge, face: &Face) -> Result<BRepClassEdge, Failure> {
    let mut data = BRepClassEdge::with_edge_face(edge.clone(), face.clone());
    let pcurve = BRepTool::curve_on_face(data.edge(), data.face())?;
    if !pcurve.pcurve.is_null() {
        data.set_geometry(pcurve.pcurve, pcurve.first, pcurve.last);
    }
    Ok(data)
}

// The next edge of `data`: the other edge at its last vertex (FORWARD-oriented
// vertices of the wire), when exactly two edge occurrences meet there and the edge
// is not closed on one vertex.
fn link_next_edge(
    data: &mut BRepClassEdge,
    vertex_edges: &DataMap<Shape, VertexEdges, ShapeMapHasher>,
) {
    let (first_vertex, last_vertex) = topexp::vertices(data.edge(), true);
    if last_vertex.is_null() || last_vertex.is_same(&first_vertex) {
        return;
    }
    let Some(edges) = vertex_edges.seek(&last_vertex) else {
        return;
    };
    if edges.count != 2 {
        return;
    }
    let next: &Edge = if edges.first.is_same(data.edge()) {
        &edges.second
    } else {
        &edges.first
    };
    if !next.is_null() && !next.is_same(data.edge()) {
        data.set_next_edge(next.clone());
    }
}
