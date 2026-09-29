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
// Rust translation of ModelingData/TKBRep/BRepGraphInc at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BRepGraphInc: the incidence-table backend of [`BRepGraph`](crate::brep_graph::BRepGraph).
//!
//! [`Storage`] holds the definition, reference and representation tables, the relation
//! tables, the UID indexes and the shape bindings; [`populate`] fills it from a
//! `TopoDS_Shape` and [`reconstruct`] rebuilds shapes from it. The records are the
//! `*Def`, `*Ref` and `*Rep` structs. docs/topology-design.md section 11 fixes the
//! mapping.
mod bit_flags;
mod definition;
mod instance;
mod load;
mod parity_orientation;
pub mod populate;
pub mod reconstruct;
mod reference;
mod relations;
mod representation;
mod shared_bit_flags;
mod storage;

pub use bit_flags::BitFlags;
pub use definition::{
    BaseDef, CoEdgeDef, CompSolidDef, CompoundDef, EdgeDef, FaceDef, HasBaseDef, OccurrenceDef,
    ProductDef, ShellDef, SolidDef, VertexDef, WireDef,
};
pub use instance::{
    CoEdgeInstance, CompSolidInstance, CompoundInstance, FaceInstance, Instance, InstanceId,
    NodeInstance, OccurrenceInstance, ProductInstance, ShellInstance, SolidInstance,
    VertexInstance, WireInstance,
};
pub use load::LoadCounts;
pub use parity_orientation::ParityOrientation;
pub use reference::{
    BaseRef, ChildRef, FaceRef, HasBaseRef, OccurrenceRef, ShellRef, SolidRef, VertexRef, WireRef,
};
pub use relations::{
    CompSolidRelations, CompoundRelations, EdgeRelations, FaceRelations, OccurrenceRelations,
    ProductRelations, ShellRelations, SolidRelations, VertexRelations, WireRelations,
};
pub use representation::{
    CoEdgeCurve2DRep, CoEdgePolygon2DRep, CoEdgePolygonOnTriRep, EdgeCurve3DRep, EdgePolygon3DRep,
    FaceSurfaceRep, FaceTriangulationRep,
};
pub use shared_bit_flags::SharedBitFlags;
pub use storage::{
    CachedShape, EntityStore, FlagPlane, NodeTag, OwnershipPlanes, RefTag, RemovalPlanes, RepStore,
    RepTag, ShapeBindings, Storage, StoredTag, WireCoEdgeOrderStatus,
};

/// `std::max(a, b)`: `b` only when `a < b`, so a NaN in `b` keeps `a` and a NaN in `a` is
/// kept, unlike `f64::max` (the ported code compares tolerances with `std::max`).
pub(crate) fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}
