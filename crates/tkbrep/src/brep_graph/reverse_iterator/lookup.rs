// Rust translation of BRepGraph_ReverseIterator::ParentRef, LookupParentRefsOf,
// WireOfCoEdgeUsageTraits and EdgeOfVertexRefTraits
// (ModelingData/TKBRep/BRepGraph/BRepGraph_ReverseIterator.hxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parents of a child with the reference each parent holds to it, found by lookup
//! where the reverse relation stores parent ids only.
use crate::brep_graph::iterator::NodeTraits;
use crate::brep_graph::parent_traits::SlotId;
use crate::brep_graph::refs_iterator::{CoEdgesOfWire, RefsVertexOfEdge};
use crate::brep_graph::{
    BRepGraph, CoEdgeId, EdgeId, EdgeTag, NodeId, TypedId, VertexId, VertexRefId, WireId, WireTag,
};
use crate::brep_graph_inc::NodeTag;

/// `ParentRef<ParentIdT, RefIdT>`: a parent id and the reference in that parent that
/// names the child.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParentRef<P, R> {
    /// `ParentId`.
    pub parent_id: P,
    /// `Ref`.
    pub ref_id: R,
}

/// The traits of [`LookupParentRefsOf`].
pub trait LookupTraits {
    /// `ParentId`.
    type Parent: NodeTag;
    /// `ChildId`.
    type Child: Copy;
    /// `RefId`.
    type Ref: SlotId;
    /// `FindRef(theGraph, theParent, theChild)`: the reference, invalid if not found.
    fn find_ref(graph: &BRepGraph, parent: TypedId<Self::Parent>, child: Self::Child) -> Self::Ref;
}

/// `WireOfCoEdgeUsageTraits`: a wire's use of a coedge is the coedge id itself.
#[derive(Clone, Copy, Debug, Default)]
pub struct WireOfCoEdgeUsageTraits;

impl LookupTraits for WireOfCoEdgeUsageTraits {
    type Parent = WireTag;
    type Child = CoEdgeId;
    type Ref = CoEdgeId;
    fn find_ref(graph: &BRepGraph, parent: WireId, child: CoEdgeId) -> CoEdgeId {
        CoEdgesOfWire::new(graph, parent)
            .find(|co_edge| *co_edge == child)
            .unwrap_or_default()
    }
}

/// `EdgeOfVertexRefTraits`: the active start or end vertex reference of an edge that
/// names the vertex.
#[derive(Clone, Copy, Debug, Default)]
pub struct EdgeOfVertexRefTraits;

impl LookupTraits for EdgeOfVertexRefTraits {
    type Parent = EdgeTag;
    type Child = VertexId;
    type Ref = VertexRefId;
    fn find_ref(graph: &BRepGraph, parent: EdgeId, child: VertexId) -> VertexRefId {
        RefsVertexOfEdge::new(graph, parent)
            .find(|reference| {
                graph.refs().generic().child_node((*reference).into()) == NodeId::from(child)
            })
            .unwrap_or_default()
    }
}

/// `LookupParentRefsOf<TraitsT>`: the active parents of a parent-id list that hold an
/// active reference to the child.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, EdgeId, VertexId};
/// use rustycad_tkbrep::brep_graph::reverse_iterator::RefsEdgesOfVertex;
/// let edges: Vec<EdgeId> = Vec::new();
/// assert!(!RefsEdgesOfVertex::new(&BRepGraph::new(), &edges, VertexId::new(0)).more());
/// ```
pub struct LookupParentRefsOf<'g, L: LookupTraits> {
    graph: &'g BRepGraph,
    parents: &'g [TypedId<L::Parent>],
    child: L::Child,
    current: ParentRef<TypedId<L::Parent>, L::Ref>,
    nb_parents: u32,
    nb_refs: u32,
    index: u32,
    has_current: bool,
}

impl<'g, L: LookupTraits> LookupParentRefsOf<'g, L> {
    /// `LookupParentRefsOf(theGraph, theParents, theChild)`.
    pub fn new(graph: &'g BRepGraph, parents: &'g [TypedId<L::Parent>], child: L::Child) -> Self {
        let mut iterator = Self {
            graph,
            parents,
            child,
            current: ParentRef::default(),
            nb_parents: L::Parent::count(graph),
            nb_refs: L::Ref::slot_count(graph),
            index: 0,
            has_current: false,
        };
        iterator.advance();
        iterator
    }

    /// `More()`.
    pub fn more(&self) -> bool {
        self.has_current
    }

    /// `Next()`.
    pub fn next(&mut self) {
        self.index += 1;
        self.advance();
    }

    /// `Current()`: the parent and its reference.
    pub fn current(&self) -> ParentRef<TypedId<L::Parent>, L::Ref> {
        self.current
    }

    /// `CurrentParentId()`.
    pub fn current_parent_id(&self) -> TypedId<L::Parent> {
        self.current.parent_id
    }

    /// `CurrentRefId()`.
    pub fn current_ref_id(&self) -> L::Ref {
        self.current.ref_id
    }

    /// `Index()`: the position in the parent list.
    pub fn index(&self) -> u32 {
        self.index
    }

    fn advance(&mut self) {
        self.has_current = false;
        while let Some(parent) = self.parents.get(self.index as usize).copied() {
            if parent.is_valid_for(self.nb_parents) && !parent.is_removed(self.graph) {
                let reference = L::find_ref(self.graph, parent, self.child);
                if reference.is_live(self.graph, self.nb_refs) {
                    self.current = ParentRef {
                        parent_id: parent,
                        ref_id: reference,
                    };
                    self.has_current = true;
                    return;
                }
            }
            self.index += 1;
        }
    }
}

/// The native `begin()`/`end()` range: yields each `Current()` pair.
impl<L: LookupTraits> Iterator for LookupParentRefsOf<'_, L> {
    type Item = ParentRef<TypedId<L::Parent>, L::Ref>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.more() {
            return None;
        }
        let current = self.current;
        LookupParentRefsOf::next(self);
        Some(current)
    }
}
