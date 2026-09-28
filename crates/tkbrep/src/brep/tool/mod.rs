// Created on: 1993-07-07
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_Tool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_Tool`: the geometric queries on BRep shapes (docs/topology-design.md section 9).
//!
//! The static members are associated functions of [`BRepTool`], split by what they
//! read: faces (`face.rs`), edge curves (`curves.rs`), the planar projection
//! (`curve_on_plane.rs`), edge polygons (`polygons.rs`), closedness (`closed.rs`), edge
//! flags, ranges and UV points (`edge.rs`), regularities (`continuity.rs`) and vertices
//! (`vertex.rs`). Output parameters become returned values.
mod closed;
mod continuity;
mod curve_on_plane;
mod curves;
mod edge;
mod face;
mod polygons;
mod vertex;

use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::toploc::Location;

/// `BRep_Tool`: static queries on the geometry of faces, edges and vertices.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
/// use rustycad_tkbrep::topods::Edge;
/// let mut edge = Edge::default();
/// BRepBuilder::default().make_edge(&mut edge).unwrap();
/// assert_eq!(BRepTool::edge_tolerance(&edge).unwrap(), 1e-7);
/// assert!(!BRepTool::is_geometric_edge(&edge).unwrap());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct BRepTool;

/// The 3D curve of an edge (`Curve(E, L, First, Last)`): the curve, its location and
/// its range; a null curve with the identity and `0, 0` when the edge has none.
#[derive(Clone, Default)]
pub struct EdgeCurve3D {
    /// The 3D curve, null when the edge has none.
    pub curve: Handle<dyn GeomCurve>,
    /// The edge location composed with the representation location.
    pub location: Location,
    /// The first parameter.
    pub first: f64,
    /// The last parameter.
    pub last: f64,
}

/// The p-curve of an edge on a surface (`CurveOnSurface(E, S, L, First, Last,
/// theIsStored)`): the curve, its range and whether it is stored in the edge.
#[derive(Clone, Default)]
pub struct EdgePCurve {
    /// The p-curve, null when the edge has none on the surface.
    pub pcurve: Handle<dyn Geom2dCurve>,
    /// The first parameter.
    pub first: f64,
    /// The last parameter.
    pub last: f64,
    /// False when the curve was not found in the edge and the planar projection ran.
    pub is_stored: bool,
}

/// A curve on surface of an edge (`CurveOnSurface(E, C, S, L, First, Last[, Index])`):
/// the p-curve, its surface, the composed location and the range; null handles, the
/// identity and `0, 0` when there is none.
#[derive(Clone, Default)]
pub struct EdgeCurveOnSurface {
    /// The p-curve.
    pub pcurve: Handle<dyn Geom2dCurve>,
    /// The surface of the p-curve.
    pub surface: Handle<dyn GeomSurface>,
    /// The edge location composed with the representation location.
    pub location: Location,
    /// The first parameter.
    pub first: f64,
    /// The last parameter.
    pub last: f64,
}
