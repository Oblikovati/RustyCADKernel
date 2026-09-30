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
// Rust translation of BRepClass_Edge.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepClass_Edge`: an edge of a face as the classifier sees it, with the cached
//! pcurve, its range and its bounding box.
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;

/// `BRepClass_Edge::BndBoxState`: the state of the cached pcurve bounding box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BndBoxState {
    /// `NotBuilt`: no box is computed yet.
    NotBuilt,
    /// `Ready`: the cached box is the pcurve's box.
    Ready,
    /// `Unavailable`: the box could not be built (no pcurve, a failure or a void box).
    Unavailable,
}

/// `BRepClass_Edge` (docs/topalgo-design.md section 2): the edge, its face, the next
/// edge at its last vertex, the gap tolerance and bounding-box switch the intersector
/// reads, and the cached pcurve with its range and bounding box. Setting the edge or
/// the face drops everything derived from them.
///
/// ```
/// use rustycad_tktopalgo::brep_class::{BRepClassEdge, BndBoxState};
/// let edge = BRepClassEdge::new();
/// assert!(edge.curve().is_null());
/// assert_eq!(edge.bounding_box_state(), BndBoxState::NotBuilt);
/// assert!(edge.max_tolerance() >= 1e100);
/// ```
#[derive(Clone)]
pub struct BRepClassEdge {
    my_edge: Edge,
    my_face: Face,
    my_next_edge: Edge,
    my_curve: Handle<dyn Geom2dCurve>,
    my_bounding_box: BoundingBox2,
    my_first_parameter: f64,
    my_last_parameter: f64,
    my_max_tolerance: f64,
    my_bounding_box_state: BndBoxState,
    my_use_bnd_box: bool,
}

impl Default for BRepClassEdge {
    /// `BRepClass_Edge()`: null shapes, no pcurve, range 0, the maximum tolerance
    /// `Precision::Infinite()` and no bounding boxes.
    fn default() -> Self {
        Self::with_edge_face(Edge::default(), Face::default())
    }
}

impl BRepClassEdge {
    /// `BRepClass_Edge()`; `BRepClassEdge::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepClass_Edge(E, F)`: the edge `edge` of the face `face`, nothing cached;
    /// `BRepClassEdge::with_edge_face(edge, face)`.
    pub fn with_edge_face(edge: Edge, face: Face) -> Self {
        Self {
            my_edge: edge,
            my_face: face,
            my_next_edge: Edge::default(),
            my_curve: Handle::default(),
            my_bounding_box: BoundingBox2::default(),
            my_first_parameter: 0.0,
            my_last_parameter: 0.0,
            my_max_tolerance: precision::infinite(),
            my_bounding_box_state: BndBoxState::NotBuilt,
            my_use_bnd_box: false,
        }
    }

    /// `Edge()`: the current edge; `data.edge()`.
    pub fn edge(&self) -> &Edge {
        &self.my_edge
    }

    /// `Face()`: the face of the current edge; `data.face()`.
    pub fn face(&self) -> &Face {
        &self.my_face
    }

    /// `SetEdge(theEdge)`: drops the derived data (next edge, pcurve, range, box),
    /// then sets the edge; `data.set_edge(edge)`.
    pub fn set_edge(&mut self, edge: Edge) {
        self.invalidate_derived_data();
        self.my_edge = edge;
    }

    /// `SetFace(theFace)`: drops the derived data, then sets the face;
    /// `data.set_face(face)`.
    pub fn set_face(&mut self, face: Face) {
        self.invalidate_derived_data();
        self.my_face = face;
    }

    /// `NextEdge()`: the next edge at the last vertex, null when there is none;
    /// `data.next_edge()`.
    pub fn next_edge(&self) -> &Edge {
        &self.my_next_edge
    }

    /// `SetNextEdge(theEdge)`; `data.set_next_edge(edge)`.
    pub fn set_next_edge(&mut self, edge: Edge) {
        self.my_next_edge = edge;
    }

    /// `MaxTolerance()`: the vertex tolerance above which the intersector checks
    /// a gap at the last vertex; `data.max_tolerance()`.
    pub fn max_tolerance(&self) -> f64 {
        self.my_max_tolerance
    }

    /// `SetMaxTolerance(theValue)`; `data.set_max_tolerance(0.1)`.
    pub fn set_max_tolerance(&mut self, value: f64) {
        self.my_max_tolerance = value;
    }

    /// `UseBndBox()`: whether the intersector uses bounding boxes;
    /// `data.use_bnd_box()`.
    pub fn use_bnd_box(&self) -> bool {
        self.my_use_bnd_box
    }

    /// `SetUseBndBox(theValue)`; `data.set_use_bnd_box(true)`.
    pub fn set_use_bnd_box(&mut self, value: bool) {
        self.my_use_bnd_box = value;
    }

    /// `SetGeometry(theCurve, theFirst, theLast)`: caches the pcurve and its range
    /// and drops the box (NotBuilt); `data.set_geometry(pcurve, 0.0, 1.0)`.
    pub fn set_geometry(&mut self, curve: Handle<dyn Geom2dCurve>, first: f64, last: f64) {
        self.my_curve = curve;
        self.my_first_parameter = first;
        self.my_last_parameter = last;
        self.my_bounding_box.set_void();
        self.my_bounding_box_state = BndBoxState::NotBuilt;
    }

    /// `SetBoundingBox(theBox)`: caches a computed box (Ready); a void box makes the
    /// box unavailable instead; `data.set_bounding_box(&bounds)`.
    pub fn set_bounding_box(&mut self, bounds: &BoundingBox2) {
        if bounds.is_void() {
            self.set_bounding_box_unavailable();
            return;
        }
        self.my_bounding_box = bounds.clone();
        self.my_bounding_box_state = BndBoxState::Ready;
    }

    /// `SetBoundingBoxUnavailable()`: a void box after a failed build;
    /// `data.set_bounding_box_unavailable()`.
    pub fn set_bounding_box_unavailable(&mut self) {
        self.my_bounding_box.set_void();
        self.my_bounding_box_state = BndBoxState::Unavailable;
    }

    /// `Curve()`: the cached pcurve, null when none is cached; `data.curve()`.
    pub fn curve(&self) -> &Handle<dyn Geom2dCurve> {
        &self.my_curve
    }

    /// `FirstParameter()`: the cached first pcurve parameter; `data.first_parameter()`.
    pub fn first_parameter(&self) -> f64 {
        self.my_first_parameter
    }

    /// `LastParameter()`: the cached last pcurve parameter; `data.last_parameter()`.
    pub fn last_parameter(&self) -> f64 {
        self.my_last_parameter
    }

    /// `BoundingBox()`: the cached pcurve box; `data.bounding_box()`.
    pub fn bounding_box(&self) -> &BoundingBox2 {
        &self.my_bounding_box
    }

    /// `BoundingBoxState()`; `data.bounding_box_state()`.
    pub fn bounding_box_state(&self) -> BndBoxState {
        self.my_bounding_box_state
    }

    // `invalidateDerivedData()`: the next edge, the pcurve, its range and its box.
    fn invalidate_derived_data(&mut self) {
        self.my_next_edge.nullify();
        self.my_curve = Handle::default();
        self.my_bounding_box.set_void();
        self.my_first_parameter = 0.0;
        self.my_last_parameter = 0.0;
        self.my_bounding_box_state = BndBoxState::NotBuilt;
    }
}
