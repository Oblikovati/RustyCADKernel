// Rust translation of BRepGraph_ParentExplorer::nextParentFrame,
// nextCompoundOrOccurrenceParent, findNthOccurrenceWrapper and the find*Step members
// (ModelingData/TKBRep/BRepGraph/BRepGraph_ParentExplorer.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parent enumeration of one frame of [`ParentExplorer`](super::ParentExplorer).
//!
//! The step index of a reference parent counts the parent's active references (the
//! refs iterators), while the transform of the step reads `RefAtStep`, which indexes
//! the raw relation list; the two differ after a removed reference, as natively.
use super::StackFrame;
use crate::brep_graph::parent_traits::ParentTraits;
use crate::brep_graph::refs_iterator::{
    CoEdgesOfWire, RefsChildOfCompound, RefsFaceOfShell, RefsOccurrenceOfProduct, RefsOfParent,
    RefsShellOfSolid, RefsSolidOfCompSolid, RefsWireOfFace,
};
use crate::brep_graph::reverse_iterator::{
    CompSolidsOfSolid, CompoundsOfChild, FacesOfWire, OccurrencesOfProduct, ProductsOfOccurrence,
    ShellsOfFace, SolidsOfShell,
};
use crate::brep_graph::{
    BRepGraph, CoEdgeId, CompSolidId, CompoundId, EdgeId, FaceId, NodeId, NodeKind, OccurrenceId,
    OccurrenceRefId, ProductId, RefId, ShellId, SolidId, TypedId, VertexId, WireId,
};

/// `nextParentFrame(theChild, theParent)`: the next parent of the frame, advancing its
/// `NextParentIdx`; `None` when the parents are exhausted.
pub(super) fn next_parent_frame(graph: &BRepGraph, child: &mut StackFrame) -> Option<StackFrame> {
    loop {
        let index = child.next_parent_idx;
        child.next_parent_idx += 1;
        match parent_at(graph, child.node, index) {
            ParentLookup::Found(parent) => return Some(parent),
            ParentLookup::Skip => continue,
            ParentLookup::Exhausted => return None,
        }
    }
}

/// The outcome of one parent index: a parent, a skipped slot, or the end.
enum ParentLookup {
    Found(StackFrame),
    Skip,
    Exhausted,
}

impl ParentLookup {
    // A found parent with its step, or a skipped slot for a negative step.
    fn at_step(node: NodeId, step: i32) -> Self {
        if step < 0 {
            return Self::Skip;
        }
        Self::Found(parent_frame(node, step, RefId::default()))
    }

    fn from_option(parent: Option<StackFrame>) -> Self {
        parent.map_or(Self::Exhausted, Self::Found)
    }
}

fn parent_frame(node: NodeId, step_to_child: i32, ref_to_child: RefId) -> StackFrame {
    StackFrame {
        node,
        step_to_child,
        ref_to_child,
        ..StackFrame::default()
    }
}

// The switch of `nextParentFrame` for one parent index.
fn parent_at(graph: &BRepGraph, node: NodeId, index: u32) -> ParentLookup {
    // A start node out of its kind's range has no parent: the native Wire, Face,
    // Shell, Solid and Occurrence cases read its relations unchecked (undefined), and
    // every checked case finds nothing for it.
    if !graph.topo().generic().is_valid(node) {
        return ParentLookup::Exhausted;
    }
    match node.kind {
        NodeKind::VERTEX => vertex_parent(graph, VertexId::new(node.index), index),
        NodeKind::EDGE => edge_parent(graph, EdgeId::new(node.index), index),
        NodeKind::WIRE => {
            let refs = &graph
                .topo()
                .wires()
                .relations(WireId::new(node.index))
                .parent_wire_ref_ids;
            ref_parent(
                graph,
                node,
                index,
                FacesOfWire::new(graph, refs),
                find_face_child_step,
            )
        }
        NodeKind::FACE => {
            let refs = &graph
                .topo()
                .faces()
                .relations(FaceId::new(node.index))
                .parent_face_ref_ids;
            ref_parent(
                graph,
                node,
                index,
                ShellsOfFace::new(graph, refs),
                find_shell_child_step,
            )
        }
        NodeKind::SHELL => {
            let refs = &graph
                .topo()
                .shells()
                .relations(ShellId::new(node.index))
                .parent_shell_ref_ids;
            ref_parent(
                graph,
                node,
                index,
                SolidsOfShell::new(graph, refs),
                find_solid_child_step,
            )
        }
        NodeKind::SOLID => {
            let refs = &graph
                .topo()
                .solids()
                .relations(SolidId::new(node.index))
                .parent_solid_ref_ids;
            let parents = CompSolidsOfSolid::new(graph, refs);
            ref_parent(graph, node, index, parents, find_comp_solid_solid_step)
        }
        NodeKind::COMPOUND | NodeKind::COMP_SOLID => {
            ParentLookup::from_option(next_compound_or_occurrence_parent(graph, node, index))
        }
        NodeKind::CO_EDGE => co_edge_parent(graph, CoEdgeId::new(node.index), index),
        NodeKind::PRODUCT => ParentLookup::from_option(product_parent(graph, node, index)),
        NodeKind::OCCURRENCE => occurrence_parent(graph, OccurrenceId::new(node.index), index),
        _ => ParentLookup::Exhausted,
    }
}

fn vertex_parent(graph: &BRepGraph, vertex: VertexId, index: u32) -> ParentLookup {
    let edges = graph.topo().vertices().edges(vertex);
    let Some(edge) = edges.get(index as usize).copied() else {
        let remaining = index - edges.len() as u32;
        return ParentLookup::from_option(next_compound_or_occurrence_parent(
            graph,
            vertex.into(),
            remaining,
        ));
    };
    if edge.is_removed(graph) {
        return ParentLookup::Skip;
    }
    ParentLookup::at_step(edge.into(), find_edge_vertex_step(graph, edge, vertex))
}

fn edge_parent(graph: &BRepGraph, edge: EdgeId, index: u32) -> ParentLookup {
    let co_edges = graph.topo().edges().co_edges(edge);
    let Some(co_edge) = co_edges.get(index as usize).copied() else {
        let remaining = index - co_edges.len() as u32;
        return ParentLookup::from_option(next_compound_or_occurrence_parent(
            graph,
            edge.into(),
            remaining,
        ));
    };
    if co_edge.is_removed(graph) {
        return ParentLookup::Skip;
    }
    ParentLookup::Found(parent_frame(co_edge.into(), -1, RefId::default()))
}

fn co_edge_parent(graph: &BRepGraph, co_edge: CoEdgeId, index: u32) -> ParentLookup {
    let wire = graph.topo().co_edges().wire(co_edge);
    let nb_wires = u32::from(wire.is_valid());
    if index >= nb_wires {
        let parent = next_compound_or_occurrence_parent(graph, co_edge.into(), index - nb_wires);
        return ParentLookup::from_option(parent);
    }
    if wire.is_removed(graph) || find_wire_co_edge_step(graph, wire, co_edge) < 0 {
        return ParentLookup::Skip;
    }
    // The coedge is found by step, but the Wire to CoEdge link stays structural.
    ParentLookup::Found(parent_frame(wire.into(), -1, RefId::default()))
}

// The Wire, Face, Shell and Solid cases: the `index`-th active parent of the reverse
// reference list, else the compound and occurrence parents after them.
fn ref_parent<T: crate::brep_graph_inc::NodeTag>(
    graph: &BRepGraph,
    node: NodeId,
    index: u32,
    parents: impl Iterator<Item = TypedId<T>>,
    find_step: fn(&BRepGraph, TypedId<T>, NodeId) -> i32,
) -> ParentLookup {
    let mut remaining = index;
    match nth_parent_id(parents, &mut remaining) {
        Some(parent) => ParentLookup::at_step(parent.into(), find_step(graph, parent, node)),
        None => {
            ParentLookup::from_option(next_compound_or_occurrence_parent(graph, node, remaining))
        }
    }
}

/// `nthParentId`: the `remaining`-th yielded id; otherwise `remaining` is reduced by
/// the number of ids yielded.
fn nth_parent_id<I: Copy>(parents: impl Iterator<Item = I>, remaining: &mut u32) -> Option<I> {
    for parent in parents {
        if *remaining == 0 {
            return Some(parent);
        }
        *remaining -= 1;
    }
    None
}

fn product_parent(graph: &BRepGraph, node: NodeId, index: u32) -> Option<StackFrame> {
    let generic = graph.topo().generic();
    if !generic.has_occurrence_parents(node) {
        return None;
    }
    let occurrences = OccurrencesOfProduct::new(graph, generic.occurrence_ref_ids(node));
    let mut remaining = index;
    let occurrence = nth_parent_id(occurrences, &mut remaining)?;
    Some(parent_frame(occurrence.into(), -1, RefId::default()))
}

fn occurrence_parent(graph: &BRepGraph, occurrence: OccurrenceId, index: u32) -> ParentLookup {
    let refs = &graph
        .topo()
        .occurrences()
        .relations(occurrence)
        .parent_occurrence_ref_ids;
    let mut remaining = index;
    let Some(product) = nth_parent_id(ProductsOfOccurrence::new(graph, refs), &mut remaining)
    else {
        return ParentLookup::Exhausted;
    };
    match find_occurrence_step(graph, product, occurrence) {
        Some((step, reference)) => {
            ParentLookup::Found(parent_frame(product.into(), step, reference.into()))
        }
        None => ParentLookup::Skip,
    }
}

/// `nextCompoundOrOccurrenceParent`: the `remaining`-th active compound parent, then
/// the occurrences wrapping the node.
fn next_compound_or_occurrence_parent(
    graph: &BRepGraph,
    node: NodeId,
    remaining: u32,
) -> Option<StackFrame> {
    let generic = graph.topo().generic();
    let mut consumed = 0;
    if generic.has_compound_parents(node) {
        for compound in CompoundsOfChild::new(graph, generic.compound_ref_ids(node)) {
            if compound.is_removed(graph) {
                continue;
            }
            if consumed < remaining {
                consumed += 1;
                continue;
            }
            let step = find_compound_child_step(graph, compound, node);
            if step >= 0 {
                return Some(parent_frame(compound.into(), step, RefId::default()));
            }
        }
    }
    if !generic.has_occurrence_parents(node) {
        return None;
    }
    let (occurrence, reference) = find_nth_occurrence_wrapper(graph, node, remaining - consumed)?;
    Some(parent_frame(occurrence.into(), -1, reference.into()))
}

/// `findNthOccurrenceWrapper`: the `ordinal`-th active occurrence (through an active
/// reference) whose child is the node.
fn find_nth_occurrence_wrapper(
    graph: &BRepGraph,
    node: NodeId,
    ordinal: u32,
) -> Option<(OccurrenceId, OccurrenceRefId)> {
    let storage = graph.inc_storage();
    graph
        .topo()
        .generic()
        .occurrence_ref_ids(node)
        .iter()
        .filter(|reference| !reference.is_removed(graph))
        .map(|reference| {
            (
                storage.occurrence_ref(*reference).child_occurrence_id,
                *reference,
            )
        })
        .filter(|(occurrence, _)| {
            let child = storage.occurrence(*occurrence).child_node_id;
            !occurrence.is_removed(graph) && child == node
        })
        .nth(ordinal as usize)
}

// The position among the parent's active references of the first one whose child is
// `child`, with that reference; -1 when none names it.
fn active_ref_step<P>(
    graph: &BRepGraph,
    refs: RefsOfParent<'_, P>,
    child: NodeId,
) -> Option<(i32, P::Ref)>
where
    P: ParentTraits,
    P::Ref: Into<RefId>,
{
    refs.enumerate()
        .find(|(_, reference)| graph.refs().generic().child_node((*reference).into()) == child)
        .map(|(step, reference)| (step as i32, reference))
}

fn step_or_missing<R>(found: Option<(i32, R)>) -> i32 {
    found.map_or(-1, |(step, _)| step)
}

/// `findOccurrenceStep`: the step and reference of the occurrence in the product.
fn find_occurrence_step(
    graph: &BRepGraph,
    product: ProductId,
    occurrence: OccurrenceId,
) -> Option<(i32, OccurrenceRefId)> {
    active_ref_step(
        graph,
        RefsOccurrenceOfProduct::new(graph, product),
        occurrence.into(),
    )
}

/// `findCompoundChildStep`.
fn find_compound_child_step(graph: &BRepGraph, compound: CompoundId, child: NodeId) -> i32 {
    step_or_missing(active_ref_step(
        graph,
        RefsChildOfCompound::new(graph, compound),
        child,
    ))
}

/// `findCompSolidSolidStep`.
fn find_comp_solid_solid_step(graph: &BRepGraph, comp_solid: CompSolidId, child: NodeId) -> i32 {
    step_or_missing(active_ref_step(
        graph,
        RefsSolidOfCompSolid::new(graph, comp_solid),
        child,
    ))
}

/// `findSolidChildStep`: shells only.
fn find_solid_child_step(graph: &BRepGraph, solid: SolidId, child: NodeId) -> i32 {
    if child.kind != NodeKind::SHELL {
        return -1;
    }
    step_or_missing(active_ref_step(
        graph,
        RefsShellOfSolid::new(graph, solid),
        child,
    ))
}

/// `findShellChildStep`: faces only.
fn find_shell_child_step(graph: &BRepGraph, shell: ShellId, child: NodeId) -> i32 {
    if child.kind != NodeKind::FACE {
        return -1;
    }
    step_or_missing(active_ref_step(
        graph,
        RefsFaceOfShell::new(graph, shell),
        child,
    ))
}

/// `findFaceChildStep`: wires only.
fn find_face_child_step(graph: &BRepGraph, face: FaceId, child: NodeId) -> i32 {
    if child.kind != NodeKind::WIRE {
        return -1;
    }
    step_or_missing(active_ref_step(
        graph,
        RefsWireOfFace::new(graph, face),
        child,
    ))
}

/// `findWireCoEdgeStep`: the position among the wire's active coedges.
fn find_wire_co_edge_step(graph: &BRepGraph, wire: WireId, co_edge: CoEdgeId) -> i32 {
    CoEdgesOfWire::new(graph, wire)
        .position(|candidate| candidate == co_edge)
        .map_or(-1, |step| step as i32)
}

/// `findEdgeVertexStep`: 0 when the start reference names the vertex, 1 for the end
/// reference, -1 otherwise (the references' removal is not checked).
fn find_edge_vertex_step(graph: &BRepGraph, edge: EdgeId, vertex: VertexId) -> i32 {
    let def = graph.inc_storage().edge(edge);
    let refs = graph.refs().generic();
    let names = |reference: crate::brep_graph::VertexRefId| {
        reference.is_valid() && refs.child_node(reference.into()) == NodeId::from(vertex)
    };
    if names(def.start_vertex_ref_id) {
        return 0;
    }
    if names(def.end_vertex_ref_id) {
        return 1;
    }
    -1
}
