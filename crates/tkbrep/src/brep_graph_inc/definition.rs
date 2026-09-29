// Copyright (c) 2026 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Definition.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The definition records of the incidence-table model (`BRepGraphInc::*Def`).
//!
//! Each record embeds the shared [`BaseDef`] fields (the native base class) and holds
//! the intrinsic geometry properties; ordered children live in the relation tables and
//! the placement of a use lives in the reference records.
use super::parity_orientation::ParityOrientation;
use crate::brep_graph::{
    CoEdgeCurve2DRepId, CoEdgePolygon2DRepId, CoEdgePolygonOnTriRepId, EdgeCurve3DRepId, EdgeId,
    EdgePolygon3DRepId, FaceId, FaceSurfaceRepId, FaceTriangulationRepId, NodeId, VertexRefId,
    WireId,
};
use rustycad_tkmath::gp::Point3;

/// `BRepGraphInc::BaseDef`: the fields shared by every definition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BaseDef {
    /// `UID`: the per-kind UID counter (0 until allocated).
    pub uid: u32,
    /// `OwnGen`: bumped when the definition's own fields change.
    pub own_gen: u32,
    /// `SubtreeGen`: bumped when the definition or a descendant changes.
    pub subtree_gen: u32,
    /// `LastPropWave`: the propagation wave that last visited the node.
    pub last_prop_wave: u32,
}

/// Access to the [`BaseDef`] part of a definition (the native base-class slice).
pub trait HasBaseDef {
    /// The base fields.
    fn base(&self) -> &BaseDef;
    /// The mutable base fields.
    fn base_mut(&mut self) -> &mut BaseDef;
}

impl HasBaseDef for BaseDef {
    fn base(&self) -> &BaseDef {
        self
    }
    fn base_mut(&mut self) -> &mut BaseDef {
        self
    }
}

macro_rules! base_def {
    ($($type:ident),*) => {$(
        impl HasBaseDef for $type {
            fn base(&self) -> &BaseDef {
                &self.base
            }
            fn base_mut(&mut self) -> &mut BaseDef {
                &mut self.base
            }
        }
    )*};
}

/// `BRepGraphInc::VertexDef`: a point in definition frame and a tolerance.
#[derive(Clone, Debug, Default)]
pub struct VertexDef {
    /// The base fields.
    pub base: BaseDef,
    /// `Point`: the raw `BRep_TVertex::Pnt` with the definition location applied.
    pub point: Point3,
    /// `Tolerance`.
    pub tolerance: f64,
}

/// `BRepGraphInc::EdgeDef`: the boundary vertices, the owned curve and polygon uses.
#[derive(Clone, Debug, Default)]
pub struct EdgeDef {
    /// The base fields.
    pub base: BaseDef,
    /// `Curve3DRepId`: the owned 3D curve use (invalid for degenerate edges).
    pub curve3d_rep_id: EdgeCurve3DRepId,
    /// `Tolerance`.
    pub tolerance: f64,
    /// `StartVertexRefId`.
    pub start_vertex_ref_id: VertexRefId,
    /// `EndVertexRefId`.
    pub end_vertex_ref_id: VertexRefId,
    /// `Polygon3DRepId`: the owned 3D polygon use.
    pub polygon3d_rep_id: EdgePolygon3DRepId,
}

/// `BRepGraphInc::CoEdgeDef`: the use of an edge on a face; owns the p-curve data.
#[derive(Clone, Debug, Default)]
pub struct CoEdgeDef {
    /// The base fields.
    pub base: BaseDef,
    /// `ParentWireId`.
    pub parent_wire_id: WireId,
    /// `ChildEdgeId`.
    pub child_edge_id: EdgeId,
    /// `FaceId`: the face of the use (invalid for free wires).
    pub face_id: FaceId,
    /// `Orientation`: relative to the edge.
    pub orientation: ParityOrientation,
    /// `Curve2DRepId`.
    pub curve2d_rep_id: CoEdgeCurve2DRepId,
    /// `Polygon2DRepId`.
    pub polygon2d_rep_id: CoEdgePolygon2DRepId,
    /// `PolygonOnTriRepId`.
    pub polygon_on_tri_rep_id: CoEdgePolygonOnTriRepId,
}

/// `BRepGraphInc::FaceDef`: the owned surface and triangulation uses and a tolerance.
#[derive(Clone, Debug, Default)]
pub struct FaceDef {
    /// The base fields.
    pub base: BaseDef,
    /// `SurfaceRepId`.
    pub surface_rep_id: FaceSurfaceRepId,
    /// `TriangulationRepId`.
    pub triangulation_rep_id: FaceTriangulationRepId,
    /// `Tolerance`.
    pub tolerance: f64,
}

/// `BRepGraphInc::OccurrenceDef`: the referenced child node (a topology root or a product).
#[derive(Clone, Debug, Default)]
pub struct OccurrenceDef {
    /// The base fields.
    pub base: BaseDef,
    /// `ChildNodeId`.
    pub child_node_id: NodeId,
}

macro_rules! plain_defs {
    ($($(#[$doc:meta])* $type:ident;)*) => {$(
        $(#[$doc])*
        #[derive(Clone, Debug, Default)]
        pub struct $type {
            /// The base fields.
            pub base: BaseDef,
        }
    )*};
}

plain_defs! {
    /// `BRepGraphInc::WireDef`: the ordered coedges live in the wire relations.
    WireDef;
    /// `BRepGraphInc::ShellDef`.
    ShellDef;
    /// `BRepGraphInc::SolidDef`.
    SolidDef;
    /// `BRepGraphInc::CompoundDef`.
    CompoundDef;
    /// `BRepGraphInc::CompSolidDef`.
    CompSolidDef;
    /// `BRepGraphInc::ProductDef`: a part or assembly; its occurrences are relations.
    ProductDef;
}

base_def!(
    VertexDef,
    EdgeDef,
    CoEdgeDef,
    WireDef,
    FaceDef,
    ShellDef,
    SolidDef,
    CompoundDef,
    CompSolidDef,
    ProductDef,
    OccurrenceDef
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definitions_default_to_invalid_links() {
        let edge = EdgeDef::default();
        assert!(!edge.curve3d_rep_id.is_valid());
        assert!(!edge.start_vertex_ref_id.is_valid());
        assert_eq!(edge.base().uid, 0);
        let mut coedge = CoEdgeDef::default();
        coedge.base_mut().own_gen = 2;
        assert_eq!(coedge.base.own_gen, 2);
        assert!(!coedge.orientation.is_reversed);
        assert!(!OccurrenceDef::default().child_node_id.is_valid());
    }
}
