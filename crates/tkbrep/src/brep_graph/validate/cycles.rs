// Rust translation of the assembly and compound cycle detection of
// BRepGraph_Validate::Perform in ModelingData/TKBRep/BRepGraph/BRepGraph_Validate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The assembly-DAG and compound cycle checks of the audit.
//!
//! Both native loops are the same breadth-first search from each container's
//! children, skipping visited nodes (shared children are valid; only reaching the
//! root again is a cycle). They are one generic search over a [`CycleWalk`] that
//! lists a container's candidate children.
use super::{Issue, Severity, report};
use crate::brep_graph::iterator::DefIterator;
use crate::brep_graph::refs_iterator::{RefsChildOfCompound, RefsOccurrenceOfProduct};
use crate::brep_graph::*;
use std::collections::HashSet;

// One container kind of the cycle search.
trait CycleWalk: NodeTag {
    // The direct-cycle message.
    const DIRECT: &'static str;
    // The transitive-cycle message.
    const TRANSITIVE: &'static str;
    // The children of the same kind, in reference order, that the search follows.
    fn children(graph: &BRepGraph, parent: TypedId<Self>) -> Vec<TypedId<Self>>;
}

impl CycleWalk for ProductTag {
    const DIRECT: &'static str =
        "Assembly cycle: Product directly references itself via occurrence";
    const TRANSITIVE: &'static str =
        "Assembly cycle: Product reaches itself through occurrence chain";

    // The live occurrence references whose occurrence places an in-range product.
    fn children(graph: &BRepGraph, parent: ProductId) -> Vec<ProductId> {
        let nb_products = graph.topo().products().nb();
        let mut children = Vec::new();
        for reference in RefsOccurrenceOfProduct::new(graph, parent) {
            let occurrence = graph
                .refs()
                .occurrences()
                .entry(reference)
                .child_occurrence_id;
            let child = graph
                .topo()
                .occurrences()
                .definition(occurrence)
                .child_node_id;
            if reference.is_removed(graph) || child.kind != NodeKind::PRODUCT {
                continue;
            }
            let product = ProductId::from_node_id(child);
            if product.is_valid_for(nb_products) {
                children.push(product);
            }
        }
        children
    }
}

impl CycleWalk for CompoundTag {
    const DIRECT: &'static str = "Compound cycle: Compound directly references itself via ChildRef";
    const TRANSITIVE: &'static str =
        "Compound cycle: Compound reaches itself through ChildRef chain";

    // The live child references that name an in-range compound.
    fn children(graph: &BRepGraph, parent: CompoundId) -> Vec<CompoundId> {
        let mut children = Vec::new();
        for reference in RefsChildOfCompound::new(graph, parent) {
            let child = graph.refs().children().entry(reference).child_node_id;
            let is_removed = graph.refs().generic().is_removed(reference.into());
            if is_removed || child.kind != NodeKind::COMPOUND || !child.is_valid() {
                continue;
            }
            let compound = CompoundId::from_node_id(child);
            if compound.is_valid_in(&graph.topo().compounds()) {
                children.push(compound);
            }
        }
        children
    }
}

/// Assembly DAG cycle detection: no product reaches itself through its occurrences.
pub(super) fn check_assembly_cycles(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    check_cycles::<ProductTag>(graph, issues);
}

/// Compound structural cycle detection: no compound reaches itself through its
/// compound children (distinct from the assembly-level cycles).
pub(super) fn check_compound_cycles(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    check_cycles::<CompoundTag>(graph, issues);
}

fn check_cycles<T: CycleWalk>(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut roots = DefIterator::<T>::new(graph);
    while roots.more() {
        search_from::<T>(graph, roots.current_id(), issues);
        roots.next();
    }
}

// The seed loop reports a direct cycle and stops seeding, but the search still runs
// over the children seeded before it, as natively, so a root can report both texts.
fn search_from<T: CycleWalk>(graph: &BRepGraph, root: TypedId<T>, issues: &mut Vec<Issue>) {
    let mut visited: HashSet<TypedId<T>> = HashSet::new();
    let mut queue: Vec<TypedId<T>> = Vec::new();
    for child in T::children(graph, root) {
        if child == root {
            report(issues, Severity::Error, root, T::DIRECT);
            break;
        }
        if visited.insert(child) {
            queue.push(child);
        }
    }
    if reaches_root(graph, root, &mut visited, &mut queue) {
        report(issues, Severity::Error, root, T::TRANSITIVE);
    }
}

// The breadth-first search over the queue; true when a descendant is the root.
fn reaches_root<T: CycleWalk>(
    graph: &BRepGraph,
    root: TypedId<T>,
    visited: &mut HashSet<TypedId<T>>,
    queue: &mut Vec<TypedId<T>>,
) -> bool {
    let mut head = 0;
    while head < queue.len() {
        let node = queue[head];
        head += 1;
        if node.is_removed(graph) {
            continue;
        }
        for descendant in T::children(graph, node) {
            if descendant == root {
                return true;
            }
            if visited.insert(descendant) {
                queue.push(descendant);
            }
        }
    }
    false
}
