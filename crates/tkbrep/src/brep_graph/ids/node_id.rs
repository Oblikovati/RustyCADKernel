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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_NodeId.hxx and the
// NodeId members of BRepGraph.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_NodeId`: the `(Kind, Index)` address of a definition node.
use super::hashing::{combine, hash_i32, hash_u32};
use super::typed::{CountProvider, IdFamily, KindTag, OwnedFamily, TypedId};
use crate::brep_graph::BRepGraph;
use crate::brep_graph_inc::NodeTag;

/// `BRepGraph_NodeId::Kind`: a transparent `int` that may hold any value, as the native
/// `enum class` does (the tests build the reserved kind 9); the enumerators are the
/// associated constants.
///
/// ```
/// use rustycad_tkbrep::brep_graph::NodeKind;
/// assert!(NodeKind::is_valid_kind(NodeKind::FACE));
/// assert!(!NodeKind::is_valid_kind(NodeKind(9)));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeKind(pub i32);

impl NodeKind {
    /// `Kind::Solid`.
    pub const SOLID: Self = Self(0);
    /// `Kind::Shell`.
    pub const SHELL: Self = Self(1);
    /// `Kind::Face`.
    pub const FACE: Self = Self(2);
    /// `Kind::Wire`.
    pub const WIRE: Self = Self(3);
    /// `Kind::Edge`.
    pub const EDGE: Self = Self(4);
    /// `Kind::Vertex`.
    pub const VERTEX: Self = Self(5);
    /// `Kind::Compound`: a `TopoDS_Compound` container.
    pub const COMPOUND: Self = Self(6);
    /// `Kind::CompSolid`: a `TopoDS_CompSolid` container.
    pub const COMP_SOLID: Self = Self(7);
    /// `Kind::CoEdge`: the use of an edge on a face (owns the p-curve data).
    pub const CO_EDGE: Self = Self(8);
    /// `Kind::Product`: a reusable shape definition (part or assembly).
    pub const PRODUCT: Self = Self(10);
    /// `Kind::Occurrence`: a placed instance of a product.
    pub const OCCURRENCE: Self = Self(11);

    /// `IsValidKind(theKind)`: one of the supported kinds (9 is reserved).
    pub fn is_valid_kind(kind: Self) -> bool {
        matches!(kind.0, 0..=8 | 10 | 11)
    }

    /// `IsTopologyKind(theKind)`: Solid through CoEdge.
    pub fn is_topology_kind(kind: Self) -> bool {
        Self::is_valid_kind(kind) && kind >= Self::SOLID && kind <= Self::CO_EDGE
    }

    /// `IsAssemblyKind(theKind)`: Product or Occurrence.
    pub fn is_assembly_kind(kind: Self) -> bool {
        Self::is_valid_kind(kind) && (kind == Self::PRODUCT || kind == Self::OCCURRENCE)
    }
}

/// `BRepGraph_NodeId`: a lightweight typed index into a per-kind node vector; the default
/// is invalid (`Kind::Solid`, index `u32::MAX`).
///
/// ```
/// use rustycad_tkbrep::brep_graph::{NodeId, NodeKind};
/// let id = NodeId::new(NodeKind::EDGE, 2);
/// assert!(id.is_valid_for(3) && !id.is_valid_for(2));
/// assert!(NodeId::start(NodeKind::FACE) < id);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId {
    /// `NodeKind`.
    pub kind: NodeKind,
    /// `Index`.
    pub index: u32,
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new(NodeKind::SOLID, Self::INVALID_INDEX)
    }
}

impl NodeId {
    /// `THE_KIND_COUNT`: the dense kind slots, with the reserved gap at 9.
    pub const KIND_COUNT: i32 = NodeKind::OCCURRENCE.0 + 1;
    /// `THE_START_INDEX`.
    pub const START_INDEX: u32 = 0;
    /// `THE_INVALID_INDEX`.
    pub const INVALID_INDEX: u32 = u32::MAX;

    /// `BRepGraph_NodeId(theKind, theIdx)`.
    pub const fn new(kind: NodeKind, index: u32) -> Self {
        Self { kind, index }
    }

    /// `Start(theKind)`: the first id of the kind.
    pub const fn start(kind: NodeKind) -> Self {
        Self::new(kind, Self::START_INDEX)
    }

    /// `Invalid(theKind)`.
    pub const fn invalid(kind: NodeKind) -> Self {
        Self::new(kind, Self::INVALID_INDEX)
    }

    /// `IsValid()`: a supported kind and an allocated index.
    pub fn is_valid(self) -> bool {
        NodeKind::is_valid_kind(self.kind) && self.index != Self::INVALID_INDEX
    }

    /// `IsValid(theMaxCount)`.
    pub fn is_valid_for(self, max_count: u32) -> bool {
        self.is_valid() && self.index < max_count
    }

    /// `IsValidIn(theProvider)`.
    pub fn is_valid_in(self, provider: &(impl CountProvider + ?Sized)) -> bool {
        self.is_valid_for(provider.count())
    }

    /// `++id`: the index wraps as `uint32_t` (the assertion is debug-only natively).
    pub fn pre_increment(&mut self) -> Self {
        self.index = self.index.wrapping_add(1);
        *self
    }

    /// `id++`.
    pub fn post_increment(&mut self) -> Self {
        let previous = *self;
        self.index = self.index.wrapping_add(1);
        previous
    }

    /// `operator+(theOffset)`.
    pub fn offset(self, offset: u32) -> Self {
        Self::new(self.kind, self.index.wrapping_add(offset))
    }

    /// `operator-(theOffset)`.
    pub fn retreat(self, offset: u32) -> Self {
        Self::new(self.kind, self.index.wrapping_sub(offset))
    }

    /// `std::hash<BRepGraph_NodeId>`: `hashBytes({hash(int kind), hash(index)})`.
    pub fn hash_code(self) -> usize {
        combine(&[hash_i32(self.kind.0), hash_u32(self.index)])
    }

    /// `Visit(theNodeId, theFunc)`: calls the visitor with the typed id of the kind; an
    /// unsupported kind reaches the native fallback, the invalid vertex id.
    pub fn visit<V: NodeVisitor>(self, visitor: V) -> V::Output {
        use crate::brep_graph as g;
        match self.kind {
            NodeKind::VERTEX => visitor.visit(g::VertexId::from_node_id(self)),
            NodeKind::EDGE => visitor.visit(g::EdgeId::from_node_id(self)),
            NodeKind::CO_EDGE => visitor.visit(g::CoEdgeId::from_node_id(self)),
            NodeKind::WIRE => visitor.visit(g::WireId::from_node_id(self)),
            NodeKind::FACE => visitor.visit(g::FaceId::from_node_id(self)),
            NodeKind::SHELL => visitor.visit(g::ShellId::from_node_id(self)),
            NodeKind::SOLID => visitor.visit(g::SolidId::from_node_id(self)),
            NodeKind::COMPOUND => visitor.visit(g::CompoundId::from_node_id(self)),
            NodeKind::COMP_SOLID => visitor.visit(g::CompSolidId::from_node_id(self)),
            NodeKind::PRODUCT => visitor.visit(g::ProductId::from_node_id(self)),
            NodeKind::OCCURRENCE => visitor.visit(g::OccurrenceId::from_node_id(self)),
            _ => visitor.visit(g::VertexId::invalid()),
        }
    }

    /// `IsRemoved(theGraph)`: soft-removed; false for an invalid id.
    pub fn is_removed(self, graph: &BRepGraph) -> bool {
        if !self.is_valid() {
            return false;
        }
        self.visit(RemovedProbe(graph))
    }

    /// `IsOwned(theGraph)`: the node has an active owner; false for an invalid id.
    pub fn is_owned(self, graph: &BRepGraph) -> bool {
        if !self.is_valid() {
            return false;
        }
        self.visit(OwnedProbe(graph))
    }
}

/// The generic callable of `BRepGraph_NodeId::Visit`: one method generic over the kind.
pub trait NodeVisitor {
    /// The callable's result.
    type Output;
    /// Called with the typed id of the visited kind.
    fn visit<T: NodeTag>(self, id: TypedId<T>) -> Self::Output;
}

struct RemovedProbe<'g>(&'g BRepGraph);

impl NodeVisitor for RemovedProbe<'_> {
    type Output = bool;
    fn visit<T: NodeTag>(self, id: TypedId<T>) -> bool {
        self.0.inc_storage().is_removed(id)
    }
}

struct OwnedProbe<'g>(&'g BRepGraph);

impl NodeVisitor for OwnedProbe<'_> {
    type Output = bool;
    fn visit<T: NodeTag>(self, id: TypedId<T>) -> bool {
        self.0.inc_storage().is_owned(id)
    }
}

/// The node id family of [`TypedId`].
#[derive(Clone, Copy, Debug)]
pub struct NodeFamily;

impl IdFamily for NodeFamily {
    type Untyped = NodeId;
    fn untyped(kind: i32, index: u32) -> NodeId {
        NodeId::new(NodeKind(kind), index)
    }
    fn valid_kind(kind: i32) -> bool {
        NodeKind::is_valid_kind(NodeKind(kind))
    }
    fn typed_hash(kind: i32, index: u32) -> usize {
        NodeId::new(NodeKind(kind), index).hash_code()
    }
    fn is_removed(graph: &BRepGraph, kind: i32, index: u32) -> bool {
        NodeId::new(NodeKind(kind), index).is_removed(graph)
    }
}

impl OwnedFamily for NodeFamily {
    fn is_owned(graph: &BRepGraph, kind: i32, index: u32) -> bool {
        NodeId::new(NodeKind(kind), index).is_owned(graph)
    }
}

impl<T: KindTag<Family = NodeFamily>> TypedId<T> {
    /// `FromNodeId(theId)`: the typed id of a node of this kind; another kind (a debug
    /// assertion natively) or an invalid id gives the invalid typed id.
    pub fn from_node_id(id: NodeId) -> Self {
        if id.kind.0 != T::RAW_KIND || !id.is_valid() {
            return Self::invalid();
        }
        Self::new(id.index)
    }
}

impl<T: KindTag<Family = NodeFamily>> From<TypedId<T>> for NodeId {
    fn from(id: TypedId<T>) -> Self {
        NodeId::new(NodeKind(T::RAW_KIND), id.index)
    }
}

impl<T: KindTag<Family = NodeFamily>> PartialEq<NodeId> for TypedId<T> {
    fn eq(&self, other: &NodeId) -> bool {
        other.kind.0 == T::RAW_KIND && other.index == self.index
    }
}

impl<T: KindTag<Family = NodeFamily>> PartialEq<TypedId<T>> for NodeId {
    fn eq(&self, other: &TypedId<T>) -> bool {
        other == self
    }
}

macro_rules! node_tags {
    ($($(#[$doc:meta])* $tag:ident, $alias:ident, $kind:ident, $name:literal;)*) => {$(
        #[doc = concat!("The kind tag of `", $name, "`.")]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $tag;
        impl KindTag for $tag {
            type Family = NodeFamily;
            const RAW_KIND: i32 = NodeKind::$kind.0;
            const NAME: &'static str = $name;
        }
        $(#[$doc])*
        pub type $alias = TypedId<$tag>;
    )*};
}

node_tags! {
    /// `BRepGraph_SolidId`.
    SolidTag, SolidId, SOLID, "BRepGraph_SolidId";
    /// `BRepGraph_ShellId`.
    ShellTag, ShellId, SHELL, "BRepGraph_ShellId";
    /// `BRepGraph_FaceId`.
    FaceTag, FaceId, FACE, "BRepGraph_FaceId";
    /// `BRepGraph_WireId`.
    WireTag, WireId, WIRE, "BRepGraph_WireId";
    /// `BRepGraph_EdgeId`.
    EdgeTag, EdgeId, EDGE, "BRepGraph_EdgeId";
    /// `BRepGraph_VertexId`.
    VertexTag, VertexId, VERTEX, "BRepGraph_VertexId";
    /// `BRepGraph_CompoundId`.
    CompoundTag, CompoundId, COMPOUND, "BRepGraph_CompoundId";
    /// `BRepGraph_CompSolidId`.
    CompSolidTag, CompSolidId, COMP_SOLID, "BRepGraph_CompSolidId";
    /// `BRepGraph_CoEdgeId`.
    CoEdgeTag, CoEdgeId, CO_EDGE, "BRepGraph_CoEdgeId";
    /// `BRepGraph_ProductId`.
    ProductTag, ProductId, PRODUCT, "BRepGraph_ProductId";
    /// `BRepGraph_OccurrenceId`.
    OccurrenceTag, OccurrenceId, OCCURRENCE, "BRepGraph_OccurrenceId";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_classify_as_natively() {
        assert!(NodeKind::is_topology_kind(NodeKind::CO_EDGE));
        assert!(!NodeKind::is_topology_kind(NodeKind::PRODUCT));
        assert!(NodeKind::is_assembly_kind(NodeKind::OCCURRENCE));
        assert!(!NodeKind::is_assembly_kind(NodeKind(9)));
        assert_eq!(NodeId::KIND_COUNT, 12);
    }

    #[test]
    fn typed_and_untyped_ids_convert() {
        let face = FaceId::new(4);
        let node: NodeId = face.into();
        assert_eq!(face, node);
        assert_eq!(node, face);
        assert_eq!(FaceId::from_node_id(node), face);
        assert!(!EdgeId::from_node_id(node).is_valid());
        assert_eq!(face.hash_code(), node.hash_code());
        assert!(!NodeId::default().is_valid());
        let mut id = NodeId::start(NodeKind::EDGE);
        assert_eq!(id.post_increment().index, 0);
        assert_eq!(id.pre_increment().index, 2);
        assert_eq!(id.offset(2).retreat(1).index, 3);
    }
}
