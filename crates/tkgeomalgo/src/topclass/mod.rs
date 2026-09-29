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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/TopClass at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package TopClass: the generic 2d classification of a point against a face
//! bounded by edges, by intersecting a half-line from the point with the edges.
//!
//! The package is two header-only templates (`TopClass_Classifier2d.pxx` and
//! `TopClass_FaceClassifier.pxx`); docs/geomalgo-design.md section 6 fixes their
//! port: [`Classifier2d`] over a [`Classifier2dIntersector`] and [`FaceClassifier`]
//! over a [`FaceExplorer`], instantiated by Geom2dHatch.
mod classifier2d;
mod face_classifier;

pub use classifier2d::Classifier2d;
pub use face_classifier::FaceClassifier;

use crate::int_res2d::Intersection;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Direction2, Line2, Point2};

/// The `TheIntersector` operations of `TopClass_Classifier2d.pxx`: the intersection
/// of the segment `[0, param]` of a line with an edge, its IntRes2d results and the
/// local geometry of the edge.
pub trait Classifier2dIntersector: Default {
    /// The `TheEdge` type the intersector reads.
    type Edge;
    /// `Perform(L, P, Tol, E)`: intersects the segment of `lin` of parameter range
    /// `[0, param]` (`param` may be RealLast) with `edge`; the segment comes first.
    fn perform_segment(
        &mut self,
        lin: &Line2,
        param: f64,
        tolerance: f64,
        edge: &Self::Edge,
    ) -> Result<(), Failure>;
    /// The IntRes2d results of the last Perform (`IsDone`, `Point`, `Segment`...).
    fn results(&self) -> &Intersection;
    /// `LocalGeometry(E, U, T, N, C)`: the tangent, normal and curvature of `edge`
    /// at `u`.
    fn local_geometry(
        &self,
        edge: &Self::Edge,
        u: f64,
    ) -> Result<(Direction2, Direction2, f64), Failure>;
}

/// The `TheFaceExplorer` operations of `TopClass_FaceClassifier.pxx`: the choice of
/// the classification segments and the iteration over the wires and edges of the
/// face. The C++ output references are returned tuples.
pub trait FaceExplorer {
    /// The `TheEdge` type: an edge of the face, a copy of which the classifier keeps.
    type Edge: Default;
    /// The intersector of the classifier's `FClass2d`.
    type Intersector: Classifier2dIntersector<Edge = Self::Edge>;
    /// `CheckPoint(P)`: may move `point`; the classifier calls it until it is true.
    fn check_point(&mut self, point: &mut Point2) -> bool;
    /// `Reject(P)`: the point is outside the face without computation.
    fn reject(&self, point: &Point2) -> bool;
    /// `Segment(P, L, Par)`: the first classification segment from `point`: whether
    /// it is valid, its line and its length.
    fn segment(&mut self, point: &Point2) -> Result<(bool, Line2, f64), Failure>;
    /// `OtherSegment(P, L, Par)`: the next classification segment.
    fn other_segment(&mut self, point: &Point2) -> Result<(bool, Line2, f64), Failure>;
    /// `InitWires()`.
    fn init_wires(&mut self);
    /// `MoreWires()`.
    fn more_wires(&self) -> bool;
    /// `NextWire()`.
    fn next_wire(&mut self);
    /// `RejectWire(L, Par)`: the current wire cannot cut the segment.
    fn reject_wire(&self, line: &Line2, param: f64) -> bool;
    /// `InitEdges()`.
    fn init_edges(&mut self);
    /// `MoreEdges()`.
    fn more_edges(&self) -> bool;
    /// `NextEdge()`.
    fn next_edge(&mut self);
    /// `RejectEdge(L, Par)`: the current edge cannot cut the segment.
    fn reject_edge(&self, line: &Line2, param: f64) -> bool;
    /// `CurrentEdge(E, Or)`: the current edge and its orientation.
    fn current_edge(&self) -> Result<(Self::Edge, Orientation), Failure>;
}
