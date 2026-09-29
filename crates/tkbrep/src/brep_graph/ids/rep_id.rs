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
// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_RepId.hxx and the
// RepId member of BRepGraph.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_RepId`: the `(Kind, Index)` address of a representation use record.
use super::hashing::{combine, hash_i32, hash_u32};
use super::typed::{IdFamily, KindTag, TypedId};
use crate::brep_graph::BRepGraph;

/// `BRepGraph_RepId::Kind`: a transparent `int` with one constant per enumerator.
///
/// ```
/// use rustycad_tkbrep::brep_graph::RepKind;
/// assert!(RepKind::is_valid_kind(RepKind::FACE_TRIANGULATION));
/// assert!(!RepKind::is_valid_kind(RepKind(7)));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepKind(pub i32);

impl RepKind {
    /// `Kind::EdgeCurve3D`: a `Geom_Curve` use for edges.
    pub const EDGE_CURVE_3D: Self = Self(0);
    /// `Kind::EdgePolygon3D`: a `Poly_Polygon3D` use for edges.
    pub const EDGE_POLYGON_3D: Self = Self(1);
    /// `Kind::CoEdgeCurve2D`: a `Geom2d_Curve` use for coedges.
    pub const CO_EDGE_CURVE_2D: Self = Self(2);
    /// `Kind::CoEdgePolygon2D`: a `Poly_Polygon2D` use for coedges.
    pub const CO_EDGE_POLYGON_2D: Self = Self(3);
    /// `Kind::CoEdgePolygonOnTri`: a `Poly_PolygonOnTriangulation` use for coedges.
    pub const CO_EDGE_POLYGON_ON_TRI: Self = Self(4);
    /// `Kind::FaceSurface`: a `Geom_Surface` use for faces.
    pub const FACE_SURFACE: Self = Self(5);
    /// `Kind::FaceTriangulation`: a `Poly_Triangulation` use for faces.
    pub const FACE_TRIANGULATION: Self = Self(6);

    /// `IsValidKind(theKind)`.
    pub fn is_valid_kind(kind: Self) -> bool {
        (0..=6).contains(&kind.0)
    }
}

/// `BRepGraph_RepId`: an owner-scoped representation slot; the default is invalid
/// (`Kind::EdgeCurve3D`, index `u32::MAX`).
///
/// ```
/// use rustycad_tkbrep::brep_graph::{RepId, RepKind};
/// assert!(RepId::new(RepKind::FACE_SURFACE, 0) < RepId::new(RepKind::FACE_TRIANGULATION, 0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepId {
    /// `RepKind`.
    pub kind: RepKind,
    /// `Index`.
    pub index: u32,
}

impl Default for RepId {
    fn default() -> Self {
        Self::new(RepKind::EDGE_CURVE_3D, Self::INVALID_INDEX)
    }
}

impl RepId {
    /// `THE_START_INDEX`.
    pub const START_INDEX: u32 = 0;
    /// `THE_INVALID_INDEX`.
    pub const INVALID_INDEX: u32 = u32::MAX;

    /// `BRepGraph_RepId(theKind, theIdx)`.
    pub const fn new(kind: RepKind, index: u32) -> Self {
        Self { kind, index }
    }

    /// `IsValid()`.
    pub fn is_valid(self) -> bool {
        RepKind::is_valid_kind(self.kind) && self.index != Self::INVALID_INDEX
    }

    /// `IsValid(theMaxCount)`.
    pub fn is_valid_for(self, max_count: u32) -> bool {
        self.is_valid() && self.index < max_count
    }

    /// `std::hash<BRepGraph_RepId>`.
    pub fn hash_code(self) -> usize {
        combine(&[hash_i32(self.kind.0), hash_u32(self.index)])
    }

    /// `IsRemoved(theGraph)`: soft-removed; false for an invalid id.
    pub fn is_removed(self, graph: &BRepGraph) -> bool {
        if !self.is_valid() {
            return false;
        }
        graph.inc_storage().is_rep_removed(self)
    }
}

/// The representation id family of [`TypedId`].
#[derive(Clone, Copy, Debug)]
pub struct RepFamily;

impl IdFamily for RepFamily {
    type Untyped = RepId;
    fn untyped(kind: i32, index: u32) -> RepId {
        RepId::new(RepKind(kind), index)
    }
    // The native typed `IsValid()` does not check the kind; every tag kind is valid, so
    // checking it gives the same answer.
    fn valid_kind(kind: i32) -> bool {
        RepKind::is_valid_kind(RepKind(kind))
    }
    // `std::hash<Typed<Kind>>` of the representation family is `std::hash<uint32_t>` of
    // the index, which is the identity in libstdc++ (unlike the node and reference ids).
    fn typed_hash(_kind: i32, index: u32) -> usize {
        index as usize
    }
    fn is_removed(graph: &BRepGraph, kind: i32, index: u32) -> bool {
        RepId::new(RepKind(kind), index).is_removed(graph)
    }
}

impl<T: KindTag<Family = RepFamily>> From<TypedId<T>> for RepId {
    fn from(id: TypedId<T>) -> Self {
        RepId::new(RepKind(T::RAW_KIND), id.index)
    }
}

macro_rules! rep_tags {
    ($($(#[$doc:meta])* $tag:ident, $alias:ident, $kind:ident, $name:literal;)*) => {$(
        #[doc = concat!("The kind tag of `", $name, "`.")]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $tag;
        impl KindTag for $tag {
            type Family = RepFamily;
            const RAW_KIND: i32 = RepKind::$kind.0;
            const NAME: &'static str = $name;
        }
        $(#[$doc])*
        pub type $alias = TypedId<$tag>;
    )*};
}

rep_tags! {
    /// `BRepGraph_EdgeCurve3DRepId`.
    EdgeCurve3DTag, EdgeCurve3DRepId, EDGE_CURVE_3D, "BRepGraph_EdgeCurve3DRepId";
    /// `BRepGraph_EdgePolygon3DRepId`.
    EdgePolygon3DTag, EdgePolygon3DRepId, EDGE_POLYGON_3D, "BRepGraph_EdgePolygon3DRepId";
    /// `BRepGraph_CoEdgeCurve2DRepId`.
    CoEdgeCurve2DTag, CoEdgeCurve2DRepId, CO_EDGE_CURVE_2D, "BRepGraph_CoEdgeCurve2DRepId";
    /// `BRepGraph_CoEdgePolygon2DRepId`.
    CoEdgePolygon2DTag, CoEdgePolygon2DRepId, CO_EDGE_POLYGON_2D, "BRepGraph_CoEdgePolygon2DRepId";
    /// `BRepGraph_CoEdgePolygonOnTriRepId`.
    CoEdgePolygonOnTriTag, CoEdgePolygonOnTriRepId, CO_EDGE_POLYGON_ON_TRI,
        "BRepGraph_CoEdgePolygonOnTriRepId";
    /// `BRepGraph_FaceSurfaceRepId`.
    FaceSurfaceTag, FaceSurfaceRepId, FACE_SURFACE, "BRepGraph_FaceSurfaceRepId";
    /// `BRepGraph_FaceTriangulationRepId`.
    FaceTriangulationTag, FaceTriangulationRepId, FACE_TRIANGULATION,
        "BRepGraph_FaceTriangulationRepId";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_rep_ids_convert_and_hash_by_index() {
        let surface = FaceSurfaceRepId::new(5);
        assert_eq!(RepId::from(surface), RepId::new(RepKind::FACE_SURFACE, 5));
        assert_eq!(surface.hash_code(), 5);
        assert_ne!(RepId::from(surface).hash_code(), 5);
        assert!(!RepId::default().is_valid_for(10));
    }
}
