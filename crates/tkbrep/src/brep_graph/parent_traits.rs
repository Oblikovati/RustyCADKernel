// Rust translation of the parent traits of ModelingData/TKBRep/BRepGraph/
// BRepGraph_DefsIterator.hxx and BRepGraph_RefsIterator.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parent traits shared by `BRepGraph_DefsIterator` and `BRepGraph_RefsIterator`.
//!
//! Both native namespaces declare the same `ShellOfSolidTraits`, `FaceOfShellTraits`...
//! (the defs versions add `Child`, and only the defs namespace has
//! `EdgeOfWireTraits`); the port declares each once. `THE_IS_DIRECT` selects a fast path
//! of `Next()` that checks exactly what the slow path checks, so the port keeps only the
//! slow path: the child of a direct trait is the reference id itself
//! ([`ParentTraits::resolve_child`]).
use crate::brep_graph::iterator::NodeTraits;
use crate::brep_graph::{
    BRepGraph, ChildRefId, CoEdgeId, CoEdgeTag, CompSolidTag, CompoundTag, EdgeId, EdgeTag,
    FaceRefId, FaceTag, NodeId, OccurrenceRefId, OccurrenceTag, ProductTag, ShellRefId, ShellTag,
    SolidRefId, SolidTag, TypedId, WireRefId, WireTag,
};
use crate::brep_graph_inc::{
    BaseDef, ChildRef, CoEdgeDef, FaceRef, NodeTag, OccurrenceRef, RemovalPlanes, ShellRef,
    SolidRef, StoredTag, WireRef,
};

/// A reference or child id slot the iterators test: `IsValid(count) && !IsRemoved`, or
/// `Topo().Gen().IsActive` for the untyped `NodeId` child of a compound.
pub trait SlotId: Copy + Default + PartialEq + 'static {
    /// `Topo().Gen().Nb(kind)` or `Refs().Gen().Nb(kind)` of the slot's kind (0 for the
    /// untyped `NodeId`, whose count the native iterators never read).
    fn slot_count(graph: &BRepGraph) -> u32;
    /// The liveness test of the slot against `count` (the precomputed slot count).
    fn is_live(self, graph: &BRepGraph, count: u32) -> bool;
}

impl<T: StoredTag> SlotId for TypedId<T> {
    fn slot_count(graph: &BRepGraph) -> u32 {
        T::store(graph.inc_storage()).count()
    }
    fn is_live(self, graph: &BRepGraph, count: u32) -> bool {
        self.is_valid_for(count) && !self.is_removed(graph)
    }
}

impl SlotId for NodeId {
    fn slot_count(_graph: &BRepGraph) -> u32 {
        0
    }
    fn is_live(self, graph: &BRepGraph, _count: u32) -> bool {
        graph.topo().generic().is_active(self)
    }
}

/// The traits of one parent-to-child relation (`ShellOfSolidTraits`...).
pub trait ParentTraits {
    /// The parent kind (`ParentId`).
    type Parent: NodeTag;
    /// The reference or coedge id stored in the parent's relation list (`RefId`).
    type Ref: SlotId;
    /// The entry that carries the relation (`RefEntry`).
    type Entry: 'static;
    /// The child id (`ChildId`).
    type Child: SlotId;
    /// The child definition (`ChildDef`, the defs traits only).
    type ChildDef: 'static;

    /// `IsParentValid(theGraph, theParent)`: the parent is in range and active.
    fn is_parent_valid(graph: &BRepGraph, parent: TypedId<Self::Parent>) -> bool {
        parent.is_valid_for(Self::Parent::count(graph)) && !parent.is_removed(graph)
    }
    /// `RefIds(theGraph, theParent)`: the relation list of a parent in range.
    fn ref_ids(graph: &BRepGraph, parent: TypedId<Self::Parent>) -> &[Self::Ref];
    /// `Ref(theGraph, theRefId)`: the entry of a reference in range.
    fn entry(graph: &BRepGraph, reference: Self::Ref) -> &Self::Entry;
    /// The child of a live reference: `ChildIdOf(graph, Ref(graph, ref))`, or the
    /// reference itself for a direct trait (`ChildId(aRefId)`).
    fn resolve_child(graph: &BRepGraph, reference: Self::Ref) -> Self::Child;
    /// `Child(theGraph, theChildId)`: the definition of a live child.
    fn child(graph: &BRepGraph, child: Self::Child) -> &Self::ChildDef;
}

// One trait struct whose relation list, entry and child definition are read from the
// storage accessors named by the macro arguments.
macro_rules! ref_parent_traits {
    ($(#[$doc:meta])* $name:ident: $parent:ty, $ref:ty => $entry:ty, $child:ty => $def:ty,
     $relations:ident . $list:ident, $entry_get:ident, $child_field:ident;) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $name;

        impl $name {
            /// `ChildIdOf(theGraph, theRef)`: the child the entry names.
            pub fn child_id_of(_graph: &BRepGraph, entry: &$entry) -> $child {
                entry.$child_field
            }
        }

        impl ParentTraits for $name {
            type Parent = $parent;
            type Ref = $ref;
            type Entry = $entry;
            type Child = $child;
            type ChildDef = $def;

            fn ref_ids(graph: &BRepGraph, parent: TypedId<$parent>) -> &[$ref] {
                &graph.inc_storage().$relations(parent).$list
            }
            fn entry(graph: &BRepGraph, reference: $ref) -> &$entry {
                graph.inc_storage().$entry_get(reference)
            }
            fn resolve_child(graph: &BRepGraph, reference: $ref) -> $child {
                Self::child_id_of(graph, Self::entry(graph, reference))
            }
            fn child(graph: &BRepGraph, child: $child) -> &$def {
                child_definition(graph, child)
            }
        }
    };
}

// `Topo().<Kind>().Definition(child)`, or `*Topo().Gen().TopoEntity(child)` for a
// compound child (a live child always has an entity).
trait ChildDefinition<D> {
    fn definition(graph: &BRepGraph, child: Self) -> &D;
}

fn child_definition<C: ChildDefinition<D>, D>(graph: &BRepGraph, child: C) -> &D {
    C::definition(graph, child)
}

impl<T: NodeTag> ChildDefinition<T::Def> for TypedId<T> {
    fn definition(graph: &BRepGraph, child: Self) -> &T::Def {
        T::get(graph, child)
    }
}

impl ChildDefinition<BaseDef> for NodeId {
    fn definition(graph: &BRepGraph, child: Self) -> &BaseDef {
        graph
            .topo_entity(child)
            .unwrap_or_else(|| panic!("ChildOfCompoundTraits::Child: no entity for {child:?}"))
    }
}

ref_parent_traits! {
    /// `ShellOfSolidTraits`: the shells of a solid.
    ShellOfSolidTraits: SolidTag, ShellRefId => ShellRef, TypedId<ShellTag> =>
        crate::brep_graph_inc::ShellDef, solid_relations.shell_ref_ids, shell_ref, child_shell_id;
}
ref_parent_traits! {
    /// `FaceOfShellTraits`: the faces of a shell.
    FaceOfShellTraits: ShellTag, FaceRefId => FaceRef, TypedId<FaceTag> =>
        crate::brep_graph_inc::FaceDef, shell_relations.face_ref_ids, face_ref, child_face_id;
}
ref_parent_traits! {
    /// `WireOfFaceTraits`: the wires of a face.
    WireOfFaceTraits: FaceTag, WireRefId => WireRef, TypedId<WireTag> =>
        crate::brep_graph_inc::WireDef, face_relations.wire_ref_ids, wire_ref, child_wire_id;
}
ref_parent_traits! {
    /// `SolidOfCompSolidTraits`: the solids of a compsolid.
    SolidOfCompSolidTraits: CompSolidTag, SolidRefId => SolidRef, TypedId<SolidTag> =>
        crate::brep_graph_inc::SolidDef, comp_solid_relations.solid_ref_ids, solid_ref,
        child_solid_id;
}
ref_parent_traits! {
    /// `ChildOfCompoundTraits`: the children of a compound, of any kind.
    ChildOfCompoundTraits: CompoundTag, ChildRefId => ChildRef, NodeId => BaseDef,
        compound_relations.child_ref_ids, child_ref, child_node_id;
}
ref_parent_traits! {
    /// `OccurrenceOfProductTraits`: the occurrences of a product.
    OccurrenceOfProductTraits: ProductTag, OccurrenceRefId => OccurrenceRef,
        TypedId<OccurrenceTag> => crate::brep_graph_inc::OccurrenceDef,
        product_relations.occurrence_ref_ids, occurrence_ref, child_occurrence_id;
}

/// `CoEdgeOfWireTraits`: the coedges of a wire, stored directly (no reference entry).
#[derive(Clone, Copy, Debug, Default)]
pub struct CoEdgeOfWireTraits;

impl ParentTraits for CoEdgeOfWireTraits {
    type Parent = WireTag;
    type Ref = CoEdgeId;
    type Entry = CoEdgeDef;
    type Child = CoEdgeId;
    type ChildDef = CoEdgeDef;

    fn ref_ids(graph: &BRepGraph, parent: TypedId<WireTag>) -> &[CoEdgeId] {
        &graph.inc_storage().wire_relations(parent).co_edge_ids
    }
    fn entry(graph: &BRepGraph, reference: CoEdgeId) -> &CoEdgeDef {
        CoEdgeTag::get(graph, reference)
    }
    fn resolve_child(_graph: &BRepGraph, reference: CoEdgeId) -> CoEdgeId {
        reference
    }
    fn child(graph: &BRepGraph, child: CoEdgeId) -> &CoEdgeDef {
        CoEdgeTag::get(graph, child)
    }
}

/// `EdgeOfWireTraits` (defs only): the edges of a wire through its coedges.
#[derive(Clone, Copy, Debug, Default)]
pub struct EdgeOfWireTraits;

impl EdgeOfWireTraits {
    /// `ChildIdOf(theGraph, theCoEdge)`: the coedge's edge when it is in range and
    /// active, invalid otherwise.
    pub fn child_id_of(graph: &BRepGraph, co_edge: &CoEdgeDef) -> EdgeId {
        let edge = co_edge.child_edge_id;
        if !edge.is_valid_for(EdgeTag::count(graph)) || edge.is_removed(graph) {
            return EdgeId::default();
        }
        edge
    }
}

impl ParentTraits for EdgeOfWireTraits {
    type Parent = WireTag;
    type Ref = CoEdgeId;
    type Entry = CoEdgeDef;
    type Child = EdgeId;
    type ChildDef = crate::brep_graph_inc::EdgeDef;

    fn ref_ids(graph: &BRepGraph, parent: TypedId<WireTag>) -> &[CoEdgeId] {
        CoEdgeOfWireTraits::ref_ids(graph, parent)
    }
    fn entry(graph: &BRepGraph, reference: CoEdgeId) -> &CoEdgeDef {
        CoEdgeTag::get(graph, reference)
    }
    fn resolve_child(graph: &BRepGraph, reference: CoEdgeId) -> EdgeId {
        Self::child_id_of(graph, Self::entry(graph, reference))
    }
    fn child(graph: &BRepGraph, child: EdgeId) -> &crate::brep_graph_inc::EdgeDef {
        EdgeTag::get(graph, child)
    }
}
