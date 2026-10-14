// SPDX-License-Identifier: GPL-2.0-only
//! Existing graph population/audit supplies incidence; synthesis and alias storage never publish.
use super::{bindings::BoundProjection, types::*};
use rustycad_tkbrep::brep_graph::{
    shapes_view::{AddStatus, Options},
    validate, *,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct FrozenGraphIncidence {
    pub(super) counts: [u32; 7],
    pub(super) endpoints: Vec<(FrozenPrismKey, FrozenPrismKey, FrozenPrismKey)>,
    pub(super) coedges: Vec<FrozenCoedgeIncidence>,
}
pub(super) fn capture_graph(
    bound: &BoundProjection<'_>,
) -> Result<(BRepGraph, FrozenGraphIncidence), PrismFreezeCause> {
    let mut graph = BRepGraph::new();
    let options = Options {
        create_auto_product: false,
        track_added_nodes: true,
        ..Options::default()
    };
    let added = graph
        .shapes_mut()
        .add_with_options(&bound.diagnostic.body, &options)
        .map_err(PrismFreezeCause::Native)?;
    if added.status != AddStatus::Success {
        return Err(PrismFreezeCause::GraphStatus(added.status));
    }
    let names = mapped_nodes(bound, &graph)?;
    let audit = validate::perform_with_mode(&graph, validate::Mode::Audit);
    if !audit.issues.is_empty() {
        return Err(PrismFreezeCause::GraphAudit(project_issues(
            bound,
            &graph,
            &names,
            audit.issues,
        )));
    }
    let counts = graph_counts(&graph);
    if names.len() != counts.iter().sum::<u32>() as usize - counts[2] as usize {
        return Err(PrismFreezeCause::Unsupported {
            representation: "graph population",
            actual: "synthesized/unmapped definitions",
        });
    }
    let frozen = FrozenGraphIncidence {
        counts,
        endpoints: edge_endpoints(&graph, &names)?,
        coedges: directed_coedges(bound, &graph, &names)?,
    };
    Ok((graph, frozen))
}
fn mapped_nodes(
    bound: &BoundProjection<'_>,
    graph: &BRepGraph,
) -> Result<BTreeMap<NodeId, FrozenPrismKey>, PrismFreezeCause> {
    let mut names = BTreeMap::new();
    for (context, key) in bound.contexts.iter().zip(&bound.names) {
        let id = graph.shapes().find_node(&context.occurrence);
        if !id.is_valid() {
            return Err(PrismFreezeCause::Missing {
                required: "graph definition for actual occurrence",
                actual: 0,
            });
        }
        if names
            .insert(id, *key)
            .is_some_and(|previous| previous != *key)
        {
            return Err(PrismFreezeCause::Identity {
                required: "one stable key per graph definition",
                actual: 2,
            });
        }
    }
    Ok(names)
}
fn graph_counts(graph: &BRepGraph) -> [u32; 7] {
    let t = graph.topo();
    [
        t.vertices().nb(),
        t.edges().nb(),
        t.co_edges().nb(),
        t.wires().nb(),
        t.faces().nb(),
        t.shells().nb(),
        t.solids().nb(),
    ]
}
fn lookup(
    names: &BTreeMap<NodeId, FrozenPrismKey>,
    id: NodeId,
) -> Result<FrozenPrismKey, PrismFreezeCause> {
    names.get(&id).copied().ok_or(PrismFreezeCause::Missing {
        required: "bound graph incidence endpoint",
        actual: 0,
    })
}
fn edge_endpoints(
    graph: &BRepGraph,
    names: &BTreeMap<NodeId, FrozenPrismKey>,
) -> Result<Vec<(FrozenPrismKey, FrozenPrismKey, FrozenPrismKey)>, PrismFreezeCause> {
    let mut result = Vec::new();
    for index in 0..graph.topo().edges().nb() {
        let edge = EdgeId::new(index);
        let def = graph.topo().edges().definition(edge);
        let start = graph
            .refs()
            .vertices()
            .entry(def.start_vertex_ref_id)
            .child_vertex_id;
        let end = graph
            .refs()
            .vertices()
            .entry(def.end_vertex_ref_id)
            .child_vertex_id;
        result.push((
            lookup(names, edge.into())?,
            lookup(names, start.into())?,
            lookup(names, end.into())?,
        ));
    }
    result.sort();
    Ok(result)
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FrozenCoedgeIncidence {
    pub(super) occurrence: FrozenUseKey,
    pub(super) face: FrozenPrismKey,
    pub(super) wire: FrozenPrismKey,
    pub(super) edge: FrozenPrismKey,
    pub(super) reversed: bool,
}
#[derive(Debug)]
pub(crate) struct FrozenGraphIssue {
    pub entity: Option<FrozenPrismKey>,
    pub occurrence: Option<FrozenUseKey>,
    /// Explicit local diagnostic provenance only; never a stable output identity.
    pub unbound_local: Option<NodeId>,
    pub severity: validate::Severity,
    pub description: rustycad_tkernel::tcollection::AsciiString,
}
fn directed_coedges(
    bound: &BoundProjection<'_>,
    graph: &BRepGraph,
    names: &BTreeMap<NodeId, FrozenPrismKey>,
) -> Result<Vec<FrozenCoedgeIncidence>, PrismFreezeCause> {
    let mut result = Vec::new();
    for index in 0..graph.topo().co_edges().nb() {
        result.push(bind_coedge(bound, graph, names, CoEdgeId::new(index))?);
    }
    let unique: std::collections::BTreeSet<_> = result.iter().map(|c| &c.occurrence).collect();
    let actual = bound
        .contexts
        .iter()
        .filter(|c| {
            c.occurrence.shape_type().ok() == Some(rustycad_tkg3d::top_abs::ShapeEnum::Edge)
        })
        .count();
    if unique.len() != result.len() || actual != result.len() {
        return Err(PrismFreezeCause::Count {
            entity: "bound directed coedges",
            actual: unique.len(),
            expected: actual,
        });
    }
    result.sort();
    Ok(result)
}
fn bind_coedge(
    bound: &BoundProjection<'_>,
    graph: &BRepGraph,
    names: &BTreeMap<NodeId, FrozenPrismKey>,
    id: CoEdgeId,
) -> Result<FrozenCoedgeIncidence, PrismFreezeCause> {
    let def = graph.topo().co_edges().definition(id);
    let face = lookup(names, def.face_id.into())?;
    let wire = lookup(names, def.parent_wire_id.into())?;
    let edge = lookup(names, def.child_edge_id.into())?;
    let found: Vec<_> = (0..bound.contexts.len())
        .filter(|i| matches_coedge(bound, *i, [face, wire, edge], def.orientation.is_reversed))
        .collect();
    let [index] = found.as_slice() else {
        return Err(PrismFreezeCause::Identity {
            required: "one explicit directed use per graph coedge, including seam multiplicity",
            actual: found.len(),
        });
    };
    Ok(FrozenCoedgeIncidence {
        occurrence: bound.use_names[*index].clone(),
        face,
        wire,
        edge,
        reversed: def.orientation.is_reversed,
    })
}
fn matches_coedge(
    bound: &BoundProjection<'_>,
    index: usize,
    expected: [FrozenPrismKey; 3],
    reversed: bool,
) -> bool {
    let context = &bound.contexts[index];
    let ancestors = &context.ancestors;
    if ancestors.len() < 2 || bound.names[index] != expected[2] {
        return false;
    }
    let wire = &ancestors[ancestors.len() - 1];
    let face = &ancestors[ancestors.len() - 2];
    let local_reverse = context.occurrence.orientation() != wire.orientation();
    let name = |shape: &rustycad_tkbrep::topods::Shape| {
        bound
            .contexts
            .iter()
            .position(|c| c.occurrence.is_same(shape))
            .map(|i| bound.names[i])
    };
    name(wire) == Some(expected[1]) && name(face) == Some(expected[0]) && local_reverse == reversed
}
fn project_issues(
    bound: &BoundProjection<'_>,
    graph: &BRepGraph,
    names: &BTreeMap<NodeId, FrozenPrismKey>,
    issues: Vec<validate::Issue>,
) -> Vec<FrozenGraphIssue> {
    issues
        .into_iter()
        .map(|issue| {
            let occurrence = if issue.node_id.kind == NodeKind::CO_EDGE {
                bind_coedge(bound, graph, names, CoEdgeId::from_node_id(issue.node_id))
                    .ok()
                    .map(|c| c.occurrence)
            } else {
                None
            };
            let entity = names.get(&issue.node_id).copied();
            let unbound_local = (entity.is_none() && occurrence.is_none()).then_some(issue.node_id);
            FrozenGraphIssue {
                entity,
                occurrence,
                unbound_local,
                severity: issue.sev,
                description: issue.description,
            }
        })
        .collect()
}
