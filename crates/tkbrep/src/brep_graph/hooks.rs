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
// Rust translation of the supplement-layer calls made by BRepGraphInc_Populate.cxx and
// BRepGraphInc_Reconstruct.cxx, and of
// BRepGraph_LayerTopoSupplement::AttachmentKind, at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The supplement-layer calls of the BRepGraphInc pipelines.
//!
//! Population and reconstruction reach the topology supplement layer through this
//! trait (docs/topology-design.md sections 11.6 and 11.9); the graph owns the layer
//! and cache registries themselves.
use crate::brep_graph::NodeId;
use crate::topods::Shape;

/// `BRepGraph_LayerTopoSupplement::AttachmentKind`: why a supplement shape is attached.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttachmentKind {
    /// `VertexSupplementShape`.
    VertexSupplementShape,
    /// `EdgeInternalVertex`: an edge vertex other than its boundary vertices.
    EdgeInternalVertex,
    /// `FaceDirectVertex`: a vertex child of a face.
    FaceDirectVertex,
    /// `SolidAuxShape`.
    SolidAuxShape,
    /// `ShellAuxShape`.
    ShellAuxShape,
    /// `CompSolidAuxShape`.
    CompSolidAuxShape,
    /// `CompoundAuxShape`.
    CompoundAuxShape,
    /// `GenericSupplementShape`.
    GenericSupplementShape,
}

/// The members of `BRepGraph_LayerTopoSupplement` that population and reconstruction
/// call. The layer keeps its records behind its own lock, as the native layer is
/// reached through a handle from `const` code.
pub trait TopoSupplementHooks: Send + Sync {
    /// `AddAttachment(theOwner, theKind, theShape)` during population: the attachment
    /// uid, 0 when refused. The `Shapes().ClearCached(theOwner)` of the native member is
    /// a no-op there, since an owner created by the running build has no cached
    /// reconstruction (the cache is only filled by `Shapes().Shape`).
    fn add_attachment(&self, owner: NodeId, kind: AttachmentKind, shape: &Shape) -> u64;
    /// The shapes of `AttachedTo(theOwner)` resolved through `FindByUid`, in attachment
    /// order, skipping missing or null entries (what `replaySupplementAttachments` adds).
    fn attached_shapes(&self, owner: NodeId) -> Vec<Shape>;
}
