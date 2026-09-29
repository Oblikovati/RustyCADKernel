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
// Rust translation of the layer and cache registry calls made by BRepGraph.cxx,
// BRepGraphInc_Populate.cxx and BRepGraphInc_Reconstruct.cxx, and of
// BRepGraph_LayerTopoSupplement::AttachmentKind, at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The attachment points of the layer and cache registries (phase 2 of the port).
//!
//! The graph core calls the registries in a few places only; these traits are those
//! calls. A graph without hooks behaves as a graph with no registered layer or cache
//! (docs/topology-design.md section 11.6).
use crate::brep_graph::{BRepGraph, NodeId, RefId};
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
    /// `AddAttachment(theOwner, theKind, theShape)`: the attachment uid, 0 when refused.
    fn add_attachment(&self, owner: NodeId, kind: AttachmentKind, shape: &Shape) -> u64;
    /// The shapes of `AttachedTo(theOwner)` resolved through `FindByUid`, in attachment
    /// order, skipping missing or null entries (what `replaySupplementAttachments` adds).
    fn attached_shapes(&self, owner: NodeId) -> Vec<Shape>;
}

/// The `BRepGraph_LayerRegistry` calls of the graph core.
pub trait LayerHooks: Send + Sync {
    /// `HasModificationSubscribers()`.
    fn has_modification_subscribers(&self) -> bool;
    /// `DispatchNodeModified(theNode)`.
    fn dispatch_node_modified(&self, graph: &BRepGraph, node: NodeId);
    /// `HasRefModificationSubscribers()`.
    fn has_ref_modification_subscribers(&self) -> bool;
    /// `DispatchRefModified(theRef)`.
    fn dispatch_ref_modified(&self, graph: &BRepGraph, reference: RefId);
    /// `ClearAll()`.
    fn clear_all(&mut self);
    /// `Find<BRepGraph_LayerTopoSupplement>()`: the registered supplement layer, if any.
    fn topo_supplement(&self) -> Option<&dyn TopoSupplementHooks>;
}

/// The `BRepGraph_CacheRegistry` calls of the graph core.
pub trait CacheHooks: Send + Sync {
    /// `ClearAll()`.
    fn clear_all(&mut self);
}
