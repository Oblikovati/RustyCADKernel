// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_WireOrder.pxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraphInc_WireOrder` (private): orders the coedges of a wire into a connected
//! chain, first by exact vertex identity, then by tolerance-equivalent vertices, and
//! finally into best-effort connected runs.
use super::Storage;
use super::validate::WireCoEdgeOrderStatus;
use crate::brep_graph::{CoEdgeId, VertexId, WireId};
use crate::brep_graph_inc::{BitFlags, std_max};
use rustycad_tkernel::ncollection::KDTree;
use rustycad_tkernel::precision;

// `THE_KDTREE_THRESHOLD`: from this many vertex classes the proximity pass uses a tree.
const KDTREE_THRESHOLD: usize = 128;

// `CoEdgeEndpointData`.
#[derive(Clone, Copy)]
struct EndpointData {
    start: VertexId,
    end: VertexId,
    start_tolerance: f64,
    end_tolerance: f64,
}

/// `CoEdgeIdsAreUnique(theCoEdgeIds)`.
pub(crate) fn co_edge_ids_are_unique(ids: &[CoEdgeId]) -> bool {
    ids.iter()
        .enumerate()
        .all(|(index, id)| !ids[index + 1..].contains(id))
}

/// `coEdgeOrientedVertices`: the start and end vertices of an active coedge in its
/// orientation, when its edge, vertex references and vertices are active.
pub(crate) fn co_edge_oriented_vertices(
    storage: &Storage,
    co_edge: CoEdgeId,
) -> Option<(VertexId, VertexId)> {
    if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
        return None;
    }
    let def = storage.co_edge(co_edge);
    let edge_id = def.child_edge_id;
    if !edge_id.is_valid_for(storage.nb_edges()) || storage.is_removed(edge_id) {
        return None;
    }
    let edge = storage.edge(edge_id);
    let (start_ref, end_ref) = if def.orientation.is_reversed {
        (edge.end_vertex_ref_id, edge.start_vertex_ref_id)
    } else {
        (edge.start_vertex_ref_id, edge.end_vertex_ref_id)
    };
    let active = |id: crate::brep_graph::VertexRefId| {
        id.is_valid_for(storage.nb_vertex_refs()) && !storage.is_removed(id)
    };
    if !active(start_ref) || !active(end_ref) {
        return None;
    }
    let start = storage.vertex_ref(start_ref).child_vertex_id;
    let end = storage.vertex_ref(end_ref).child_vertex_id;
    let alive = |id: VertexId| id.is_valid_for(storage.nb_vertices()) && !storage.is_removed(id);
    (alive(start) && alive(end)).then_some((start, end))
}

// `collectCoEdgeEndpointData`: every coedge active, owned by the expected wire (when
// given) and with active endpoints; tolerances widened by the edge and Confusion.
fn collect_endpoint_data(
    storage: &Storage,
    expected_wire: WireId,
    input: &[CoEdgeId],
) -> Option<Vec<EndpointData>> {
    input
        .iter()
        .map(|id| endpoint_data(storage, expected_wire, *id))
        .collect()
}

fn endpoint_data(
    storage: &Storage,
    expected_wire: WireId,
    co_edge: CoEdgeId,
) -> Option<EndpointData> {
    if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
        return None;
    }
    let def = storage.co_edge(co_edge);
    let edge_id = def.child_edge_id;
    let wrong_wire = expected_wire.is_valid() && def.parent_wire_id != expected_wire;
    if wrong_wire || !edge_id.is_valid_for(storage.nb_edges()) || storage.is_removed(edge_id) {
        return None;
    }
    let (start, end) = co_edge_oriented_vertices(storage, co_edge)?;
    let edge_tolerance = storage.edge(edge_id).tolerance;
    let widen = |vertex: VertexId| {
        std_max(
            std_max(storage.vertex(vertex).tolerance, edge_tolerance),
            precision::confusion(),
        )
    };
    Some(EndpointData {
        start,
        end,
        start_tolerance: widen(start),
        end_tolerance: widen(end),
    })
}

// `buildConnectedWireOrderByClasses`: Hierholzer's circuit over the vertex classes,
// from the unique open start class (or the first start), taking the first unused
// outgoing coedge in input order.
fn connected_order(
    input: &[CoEdgeId],
    starts: &[VertexId],
    ends: &[VertexId],
) -> Option<Vec<CoEdgeId>> {
    if input.is_empty() {
        return Some(Vec::new());
    }
    let start_class = open_start_class(starts, ends)?;
    let mut used = BitFlags::new();
    used.resize(input.len());
    let (mut class_stack, mut edge_stack, mut circuit) =
        (vec![start_class], Vec::new(), Vec::new());
    while let Some(current) = class_stack.last().copied() {
        let next =
            (0..input.len()).find(|index| !used.test(*index as u32) && starts[*index] == current);
        if let Some(index) = next {
            used.set(index as u32);
            edge_stack.push(index);
            class_stack.push(ends[index]);
            continue;
        }
        class_stack.pop();
        if let Some(index) = edge_stack.pop() {
            circuit.push(index);
        }
    }
    (circuit.len() == input.len())
        .then(|| circuit.iter().rev().map(|index| input[*index]).collect())
}

// The start class of the circuit: the class with one more outgoing than incoming use,
// or the first start when every class is balanced; `None` when the degrees forbid a
// single chain.
fn open_start_class(starts: &[VertexId], ends: &[VertexId]) -> Option<VertexId> {
    let mut classes: Vec<VertexId> = Vec::with_capacity(starts.len() * 2);
    for (start, end) in starts.iter().zip(ends) {
        for class in [*start, *end] {
            if !classes.contains(&class) {
                classes.push(class);
            }
        }
    }
    let (mut start_class, mut open_starts, mut open_ends) = (starts[0], 0, 0);
    for class in classes {
        let outgoing = starts.iter().filter(|start| **start == class).count();
        let incoming = ends.iter().filter(|end| **end == class).count();
        if outgoing == incoming + 1 {
            open_starts += 1;
            start_class = class;
        } else if incoming == outgoing + 1 {
            open_ends += 1;
        } else if outgoing != incoming {
            return None;
        }
    }
    let balanced = open_starts == 0 && open_ends == 0;
    (balanced || (open_starts == 1 && open_ends == 1)).then_some(start_class)
}

// `classOrderIsConnected`: each start equals the previous end.
fn class_order_is_connected(starts: &[VertexId], ends: &[VertexId]) -> bool {
    starts.len() == ends.len()
        && starts
            .iter()
            .skip(1)
            .zip(ends)
            .all(|(start, end)| start == end)
}

// The union-find over the vertex classes of the tolerance pass (`addOrUpdateVertexClass`,
// `rootClass`, `uniteClasses`); classes are searched linearly as natively.
struct VertexClasses {
    vertices: Vec<VertexId>,
    tolerances: Vec<f64>,
    parents: Vec<VertexId>,
}

impl VertexClasses {
    fn add_or_update(&mut self, storage: &Storage, vertex: VertexId, tolerance: f64) -> VertexId {
        if let Some(index) = self.index_of(vertex) {
            self.tolerances[index] = std_max(self.tolerances[index], tolerance);
            return vertex;
        }
        self.vertices.push(vertex);
        self.tolerances
            .push(std_max(tolerance, storage.vertex(vertex).tolerance));
        self.parents.push(vertex);
        vertex
    }

    fn index_of(&self, vertex: VertexId) -> Option<usize> {
        self.vertices.iter().position(|item| *item == vertex)
    }

    // `rootClass`: the root of the class with path compression, bounded by the class
    // count; an unknown class or a cycle returns the class itself.
    fn root(&mut self, class: VertexId) -> VertexId {
        let count = self.vertices.len();
        let (mut root, mut root_index, mut hops) = (class, self.index_of(class), 0);
        while let Some(index) = root_index.filter(|i| self.parents[*i] != root && hops < count) {
            root = self.parents[index];
            root_index = self.index_of(root);
            hops += 1;
        }
        if root_index.is_none() || hops == count {
            return class;
        }
        let (mut current_index, mut hops) = (self.index_of(class), 0);
        while let Some(index) = current_index.filter(|i| self.parents[*i] != root && hops < count) {
            let next = self.parents[index];
            self.parents[index] = root;
            current_index = self.index_of(next);
            hops += 1;
        }
        root
    }

    // `uniteClasses`: the second root joins the first.
    fn unite(&mut self, first: VertexId, second: VertexId) {
        let (first_root, second_root) = (self.root(first), self.root(second));
        if first_root != second_root
            && let Some(index) = self.index_of(second_root)
        {
            self.parents[index] = first_root;
        }
    }
}

// `buildToleranceVertexClasses`: vertices within the larger of their tolerances share a
// class; the classes replace the exact start and end vertices.
fn tolerance_classes(
    storage: &Storage,
    data: &[EndpointData],
) -> Option<(Vec<VertexId>, Vec<VertexId>)> {
    let mut classes = VertexClasses {
        vertices: Vec::new(),
        tolerances: Vec::new(),
        parents: Vec::new(),
    };
    let (mut starts, mut ends) = (
        Vec::with_capacity(data.len()),
        Vec::with_capacity(data.len()),
    );
    for item in data {
        let start = classes.add_or_update(storage, item.start, item.start_tolerance);
        let end = classes.add_or_update(storage, item.end, item.end_tolerance);
        if !start.is_valid_for(storage.nb_vertices()) || !end.is_valid_for(storage.nb_vertices()) {
            return None;
        }
        starts.push(start);
        ends.push(end);
    }
    unite_close_vertices(storage, &mut classes);
    for (start, end) in starts.iter_mut().zip(ends.iter_mut()) {
        *start = classes.root(*start);
        *end = classes.root(*end);
    }
    Some((starts, ends))
}

// The proximity pass: a KD tree from `THE_KDTREE_THRESHOLD` classes, else all pairs.
// Both unite the same pairs (a pair within the larger tolerance is found from the class
// with that tolerance), so the classes do not depend on the path.
fn unite_close_vertices(storage: &Storage, classes: &mut VertexClasses) {
    if classes.vertices.len() >= KDTREE_THRESHOLD {
        unite_with_tree(storage, classes);
    } else {
        unite_all_pairs(storage, classes);
    }
}

// Two classes whose points lie within the larger of their tolerances.
fn classes_are_close(
    storage: &Storage,
    classes: &VertexClasses,
    outer: usize,
    inner: usize,
) -> bool {
    let tolerance = std_max(classes.tolerances[outer], classes.tolerances[inner]);
    let outer_point = storage.vertex(classes.vertices[outer]).point;
    outer_point.distance(&storage.vertex(classes.vertices[inner]).point) <= tolerance
}

fn unite_close_pair(storage: &Storage, classes: &mut VertexClasses, outer: usize, inner: usize) {
    if classes_are_close(storage, classes, outer, inner) {
        let (first, second) = (classes.vertices[outer], classes.vertices[inner]);
        classes.unite(first, second);
    }
}

fn unite_all_pairs(storage: &Storage, classes: &mut VertexClasses) {
    let count = classes.vertices.len();
    for outer in 0..count {
        for inner in outer + 1..count {
            unite_close_pair(storage, classes, outer, inner);
        }
    }
}

// `ForEachInRange` of `NCollection_KDTree<gp_Pnt, 3>` over the class points, with its
// one-based result indices.
fn unite_with_tree(storage: &Storage, classes: &mut VertexClasses) {
    let points: Vec<[f64; 3]> = classes
        .vertices
        .iter()
        .map(|vertex| {
            let point = storage.vertex(*vertex).point;
            [point.x(), point.y(), point.z()]
        })
        .collect();
    let mut tree: KDTree<[f64; 3], 3> = KDTree::default();
    tree.build(&points);
    for (outer, point) in points.iter().enumerate() {
        let mut found = Vec::new();
        tree.for_each_in_range(point, classes.tolerances[outer], |index| {
            found.push(index - 1)
        });
        for inner in found {
            if inner != outer && inner < points.len() {
                unite_close_pair(storage, classes, outer, inner);
            }
        }
    }
}

// `buildBestEffortRunsByClasses`: maximal connected runs (each starting at a coedge with
// no unused predecessor, else the first unused one), emitted longest first, ties by the
// first coedge's input index.
fn best_effort_runs(input: &[CoEdgeId], starts: &[VertexId], ends: &[VertexId]) -> Vec<CoEdgeId> {
    let count = input.len();
    let mut used = BitFlags::new();
    used.resize(count);
    let mut runs: Vec<Vec<usize>> = Vec::new();
    let mut nb_used = 0;
    while nb_used < count {
        let Some(first) = run_start(&used, starts, ends) else {
            break;
        };
        let mut run = Vec::new();
        let mut current = Some(first);
        while let Some(index) = current {
            used.set(index as u32);
            nb_used += 1;
            run.push(index);
            current = (0..count).find(|i| !used.test(*i as u32) && starts[*i] == ends[index]);
        }
        runs.push(run);
    }
    let mut order: Vec<usize> = (0..runs.len()).collect();
    order.sort_by(|a, b| {
        runs[*b]
            .len()
            .cmp(&runs[*a].len())
            .then(runs[*a][0].cmp(&runs[*b][0]))
    });
    order
        .iter()
        .flat_map(|run| runs[*run].iter().map(|index| input[*index]))
        .collect()
}

fn run_start(used: &BitFlags, starts: &[VertexId], ends: &[VertexId]) -> Option<usize> {
    let count = starts.len();
    let unused = |index: usize| !used.test(index as u32);
    let no_incoming = |index: usize| {
        !(0..count).any(|other| unused(other) && other != index && ends[other] == starts[index])
    };
    (0..count)
        .find(|index| unused(*index) && no_incoming(*index))
        .or_else(|| (0..count).find(|index| unused(*index)))
}

/// `BuildCoEdgeOrder(theStorage, theExpectedWireId, theInput, theOrdered)`: the ordered
/// coedges and how the order was reached.
pub(crate) fn build_co_edge_order(
    storage: &Storage,
    expected_wire: WireId,
    input: &[CoEdgeId],
) -> (WireCoEdgeOrderStatus, Vec<CoEdgeId>) {
    use WireCoEdgeOrderStatus as Status;
    if !co_edge_ids_are_unique(input) {
        return (Status::InvalidInput, Vec::new());
    }
    if input.is_empty() {
        return (Status::Connected, Vec::new());
    }
    let Some(data) = collect_endpoint_data(storage, expected_wire, input) else {
        return (Status::InvalidInput, Vec::new());
    };
    let starts: Vec<VertexId> = data.iter().map(|item| item.start).collect();
    let ends: Vec<VertexId> = data.iter().map(|item| item.end).collect();
    if class_order_is_connected(&starts, &ends) {
        return (Status::Connected, input.to_vec());
    }
    if let Some(order) = connected_order(input, &starts, &ends) {
        return (Status::Reordered, order);
    }
    tolerance_order(storage, input, &data)
}

// The tolerance and best-effort tail of `BuildCoEdgeOrder`.
fn tolerance_order(
    storage: &Storage,
    input: &[CoEdgeId],
    data: &[EndpointData],
) -> (WireCoEdgeOrderStatus, Vec<CoEdgeId>) {
    use WireCoEdgeOrderStatus as Status;
    let Some((starts, ends)) = tolerance_classes(storage, data) else {
        return (Status::InvalidInput, Vec::new());
    };
    if class_order_is_connected(&starts, &ends) {
        return (Status::ToleranceOrdered, input.to_vec());
    }
    if let Some(order) = connected_order(input, &starts, &ends) {
        return (Status::ToleranceOrdered, order);
    }
    let order = best_effort_runs(input, &starts, &ends);
    let status = if order.len() == input.len() {
        Status::Partial
    } else {
        Status::InvalidInput
    };
    (status, order)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(index: u32) -> VertexId {
        VertexId::new(index)
    }

    #[test]
    fn circuit_orders_a_shuffled_loop() {
        let input: Vec<CoEdgeId> = (0..3).map(CoEdgeId::new).collect();
        let (starts, ends) = (vec![v(1), v(0), v(2)], vec![v(2), v(1), v(0)]);
        assert!(!class_order_is_connected(&starts, &ends));
        let order = connected_order(&input, &starts, &ends).unwrap();
        assert_eq!(
            order,
            vec![CoEdgeId::new(0), CoEdgeId::new(2), CoEdgeId::new(1)]
        );
        assert!(co_edge_ids_are_unique(&input));
        assert!(!co_edge_ids_are_unique(&[
            CoEdgeId::new(1),
            CoEdgeId::new(1)
        ]));
    }

    #[test]
    fn runs_emit_the_longest_first() {
        let input: Vec<CoEdgeId> = (0..3).map(CoEdgeId::new).collect();
        let (starts, ends) = (vec![v(5), v(0), v(1)], vec![v(6), v(1), v(2)]);
        assert!(connected_order(&input, &starts, &ends).is_none());
        let order = best_effort_runs(&input, &starts, &ends);
        assert_eq!(
            order,
            vec![CoEdgeId::new(1), CoEdgeId::new(2), CoEdgeId::new(0)]
        );
    }
}
