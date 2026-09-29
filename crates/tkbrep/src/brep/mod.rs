// Created on: 1991-07-02
// Created by: Remi LEQUETTE
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_Builder.hxx/.cxx, BRep_Tool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BRep: the boundary representation of the TopoDS shapes.
//!
//! The concrete TShapes [`BRepTVertex`], [`BRepTEdge`] and [`BRepTFace`] carry the
//! geometry: a point and a tolerance for a vertex, a list of curve representations
//! (3D curves, p-curves, regularities and polygons) for an edge, and a located
//! surface and triangulations for a face. [`BRepBuilder`] builds and updates them and
//! [`BRepTool`] queries them. The Rust mapping is in docs/topology-design.md section 9.
pub mod access;
mod builder;
mod curve3d;
mod curve_on_2_surfaces;
mod curve_on_closed_surface;
mod curve_on_surface;
mod curve_representation;
mod dump;
pub mod errors;
mod gcurve;
mod point_on_curve;
mod point_representation;
mod points_on_surface;
mod polygon3d;
mod polygon_on_surface;
mod polygon_on_triangulation;
mod tedge;
mod tface;
mod tool;
mod tvertex;

pub use builder::BRepBuilder;
pub use builder::update_curves;
pub use curve_on_2_surfaces::BRepCurveOn2Surfaces;
pub use curve_on_closed_surface::BRepCurveOnClosedSurface;
pub use curve_on_surface::BRepCurveOnSurface;
pub use curve_representation::{
    BRepCurveRepresentation, CurveRepresentationBase, dump_curve_representation,
};
pub use curve3d::BRepCurve3D;
pub use gcurve::{BRepGCurve, GCurveBase, dump_gcurve};
pub use point_on_curve::BRepPointOnCurve;
pub use point_representation::{
    BRepPointRepresentation, PointRepresentationBase, dump_point_representation,
};
pub use points_on_surface::{
    BRepPointOnCurveOnSurface, BRepPointOnSurface, BRepPointsOnSurface, PointsOnSurfaceBase,
    dump_points_on_surface,
};
pub use polygon_on_surface::{BRepPolygonOnClosedSurface, BRepPolygonOnSurface};
pub use polygon_on_triangulation::{BRepPolygonOnClosedTriangulation, BRepPolygonOnTriangulation};
pub use polygon3d::BRepPolygon3D;
pub use tedge::BRepTEdge;
pub use tface::BRepTFace;
pub use tool::{BRepTool, EdgeCurve3D, EdgeCurveOnSurface, EdgePCurve};
pub use tvertex::BRepTVertex;
