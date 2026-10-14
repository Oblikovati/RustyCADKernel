// SPDX-License-Identifier: GPL-2.0-only
//! Authored finite cell words, not geometric tessellation or a modelling acceptance oracle.
use super::super::{
    graph::FrozenCoedgeIncidence, traversal_tests::ordered_projection as fresh_projection, *,
};
use rustycad_tkg3d::top_abs::Orientation;
use std::collections::BTreeSet;

pub(super) struct NamedCellComplex {
    pub(super) edges: Vec<(u64, u64)>,
    pub(super) faces: Vec<Vec<i32>>,
}
impl NamedCellComplex {
    pub(super) fn tetrahedron() -> Self {
        Self {
            edges: vec![(0, 1), (1, 2), (2, 0), (0, 3), (1, 3), (2, 3)],
            faces: vec![
                vec![1, 2, 3],
                vec![-1, 4, -5],
                vec![-2, 5, -6],
                vec![-3, 6, -4],
            ],
        }
    }
    pub(super) fn double_tetrahedron(pinched: bool) -> Self {
        let mut first = Self::tetrahedron();
        let second = Self::tetrahedron();
        let shift = |vertex| shifted_vertex(vertex, pinched);
        first
            .edges
            .extend(second.edges.iter().map(|(a, b)| (shift(*a), shift(*b))));
        first.faces.extend(
            second
                .faces
                .iter()
                .map(|word| word.iter().map(|edge| edge + 6 * edge.signum()).collect()),
        );
        first
    }
    pub(super) fn projection(&self) -> FrozenPrismProjection {
        let (mut projection, shell) = self.start_projection();
        for (position, word) in self.faces.iter().enumerate() {
            self.add_face(&mut projection, shell, position, word);
        }
        finish(projection)
    }
    pub(super) fn domain_projection(&self, faces: &[Vec<Vec<i32>>]) -> FrozenPrismProjection {
        let (mut projection, shell) = self.start_projection();
        let mut wire_identity = 0u64;
        for (position, loops) in faces.iter().enumerate() {
            let face = authored(4, position as u64);
            let face_use = add_use(&mut projection, Some(shell), face, false, 1);
            for word in loops {
                self.add_wire(&mut projection, face_use, face, wire_identity, word);
                wire_identity += 1;
            }
        }
        finish(projection)
    }
    fn start_projection(&self) -> (FrozenPrismProjection, usize) {
        let mut projection = fresh_projection();
        projection.nodes.clear();
        projection.uses.clear();
        projection.graph.endpoints.clear();
        projection.graph.coedges.clear();
        let root = add_use(&mut projection, None, authored(2, 1), false, 1);
        let shell = add_use(&mut projection, Some(root), authored(3, 1), false, 1);
        self.add_endpoints(&mut projection);
        (projection, shell)
    }
    fn add_endpoints(&self, projection: &mut FrozenPrismProjection) {
        for (position, (first, last)) in self.edges.iter().enumerate() {
            projection.graph.endpoints.push((
                authored(6, position as u64),
                authored(7, *first),
                authored(7, *last),
            ));
        }
    }
    fn add_face(
        &self,
        projection: &mut FrozenPrismProjection,
        shell: usize,
        position: usize,
        word: &[i32],
    ) {
        let face = authored(4, position as u64);
        let face_use = add_use(projection, Some(shell), face, false, 1);
        self.add_wire(projection, face_use, face, position as u64, word);
    }
    fn add_wire(
        &self,
        projection: &mut FrozenPrismProjection,
        face_use: usize,
        face: FrozenPrismKey,
        identity: u64,
        word: &[i32],
    ) {
        let wire = authored(5, identity);
        let wire_use = add_use(projection, Some(face_use), wire, false, 1);
        for (slot, signed) in word.iter().enumerate() {
            self.add_edge(projection, wire_use, [face, wire], slot, *signed);
        }
    }
    fn add_edge(
        &self,
        projection: &mut FrozenPrismProjection,
        wire: usize,
        containers: [FrozenPrismKey; 2],
        slot: usize,
        signed: i32,
    ) {
        let position = signed.unsigned_abs() as usize - 1;
        let edge = authored(6, position as u64);
        let use_index = add_use(projection, Some(wire), edge, signed < 0, slot as u64 + 1);
        let (first, last) = self.edges[position];
        add_use(projection, Some(use_index), authored(7, first), false, 1);
        add_use(projection, Some(use_index), authored(7, last), true, 2);
        append_coedge(projection, use_index, containers, edge, signed < 0);
    }
}
fn append_coedge(
    projection: &mut FrozenPrismProjection,
    use_index: usize,
    containers: [FrozenPrismKey; 2],
    edge: FrozenPrismKey,
    reversed: bool,
) {
    projection.graph.coedges.push(FrozenCoedgeIncidence {
        occurrence: projection.uses[use_index].key.clone(),
        face: containers[0],
        wire: containers[1],
        edge,
        reversed,
    });
}
fn shifted_vertex(vertex: u64, pinched: bool) -> u64 {
    if pinched && vertex == 0 {
        return 0;
    }
    vertex + 4
}

fn authored(kind: u8, identity: u64) -> FrozenPrismKey {
    FrozenPrismKey {
        operation: AuthoredPrismId(700),
        source: AuthoredPrismId(900 + identity),
        role: FrozenSweepRole::Generated,
        kind,
    }
}
fn add_use(
    projection: &mut FrozenPrismProjection,
    parent: Option<usize>,
    key: FrozenPrismKey,
    reversed: bool,
    authored_use: u64,
) -> usize {
    let occurrence = child_key(projection, parent, key, authored_use);
    let orientation = child_orientation(projection, parent, reversed);
    if let Some(parent) = parent {
        projection.uses[parent].children.push(occurrence.clone());
    }
    projection.uses.push(FrozenPrismUse {
        key: occurrence,
        orientation,
        placement: fixture_placement(),
        children: Vec::new(),
    });
    projection.uses.len() - 1
}
fn child_key(
    projection: &FrozenPrismProjection,
    parent: Option<usize>,
    key: FrozenPrismKey,
    use_id: u64,
) -> FrozenUseKey {
    let mut ancestry = parent
        .map(|index| projection.uses[index].key.0.clone())
        .unwrap_or_default();
    ancestry.push((key, AuthoredPrismId(use_id)));
    FrozenUseKey(ancestry)
}
fn child_orientation(
    projection: &FrozenPrismProjection,
    parent: Option<usize>,
    reversed: bool,
) -> Orientation {
    let parent_reverse =
        parent.is_some_and(|index| projection.uses[index].orientation == Orientation::Reversed);
    if reversed ^ parent_reverse {
        return Orientation::Reversed;
    }
    Orientation::Forward
}
fn fixture_placement() -> rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding<3> {
    rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    }
}

fn finish(mut projection: FrozenPrismProjection) -> FrozenPrismProjection {
    let mut keys: BTreeSet<_> = projection
        .uses
        .iter()
        .map(|row| row.key.0.last().unwrap().0)
        .collect();
    for (edge, first, last) in &projection.graph.endpoints {
        keys.extend([*edge, *first, *last]);
    }
    projection.nodes = keys.into_iter().map(fixture_node).collect();
    projection.wire_traversals = Some(fixture_traversals(&projection));
    projection.graph.counts = counts(&projection);
    projection.uses.sort_by(|a, b| a.key.cmp(&b.key));
    projection.graph.coedges.sort();
    projection.graph.endpoints.sort();
    projection
}
fn fixture_node(key: FrozenPrismKey) -> FrozenPrismNode {
    FrozenPrismNode {
        key,
        point: None,
        raw_tolerance: None,
        effective_tolerance: None,
        curve: None,
    }
}
fn fixture_traversals(
    projection: &FrozenPrismProjection,
) -> Vec<super::super::traversal::FrozenWireTraversal> {
    projection
        .uses
        .iter()
        .filter(|row| row.key.0.len() == 4)
        .map(|row| super::super::traversal::FrozenWireTraversal {
            occurrence: row.key.clone(),
            coedges: row.children.clone(),
        })
        .collect()
}

fn counts(projection: &FrozenPrismProjection) -> [u32; 7] {
    let mut result = [0; 7];
    for node in &projection.nodes {
        let slot = match node.key.kind {
            7 => 0,
            6 => 1,
            5 => 3,
            4 => 4,
            3 => 5,
            _ => 6,
        };
        result[slot] += 1;
    }
    result[2] = projection.graph.coedges.len() as u32;
    result
}
