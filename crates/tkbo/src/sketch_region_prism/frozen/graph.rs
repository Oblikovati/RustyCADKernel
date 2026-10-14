// SPDX-License-Identifier: GPL-2.0-only
//! Existing graph population/audit supplies incidence; synthesis and alias storage never publish.
use super::bindings::{identity, missing};
use super::{bindings::BoundProjection, types::*};
use rustycad_tkbrep::brep_graph::{
    shapes_view::{AddStatus, Options},
    validate, *,
};
use rustycad_tkbrep::brep_graph_inc::populate::PCurveImportBehavior;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct FrozenGraphIncidence {
    pub(crate) import_behavior: PCurveImportBehavior,
    pub(crate) counts: [u32; 7],
    pub(crate) endpoints: Vec<(FrozenPrismKey, FrozenPrismKey, FrozenPrismKey)>,
    pub(crate) coedges: Vec<FrozenCoedgeIncidence>,
}
pub(super) fn capture_graph(bound: &BoundProjection<'_>) -> CaptureResult<FrozenGraphIncidence> {
    let graph = populate_graph(bound)?;
    let names = mapped_nodes(bound, &graph)?;
    audit_graph(bound, &graph, &names)?;
    Ok(FrozenGraphIncidence {
        import_behavior: PCurveImportBehavior::CumulativeOccurrenceV2,
        counts: checked_counts(&graph, names.len())?,
        endpoints: edge_endpoints(&graph, &names)?,
        coedges: directed_coedges(bound, &graph, &names)?,
    })
}
fn populate_graph(bound: &BoundProjection<'_>) -> CaptureResult<BRepGraph> {
    let mut graph = BRepGraph::new();
    let options = Options {
        create_auto_product: false,
        track_added_nodes: true,
        ..Options::default()
    };
    let added = graph
        .shapes_mut()
        .add_with_options_and_behavior(
            &bound.diagnostic.body,
            &options,
            PCurveImportBehavior::CumulativeOccurrenceV2,
        )
        .map_err(PrismFreezeCause::Native)?;
    if added.status != AddStatus::Success {
        return Err(PrismFreezeCause::GraphStatus(added.status));
    }
    Ok(graph)
}
fn audit_graph(
    bound: &BoundProjection<'_>,
    graph: &BRepGraph,
    names: &BTreeMap<NodeId, FrozenPrismKey>,
) -> CaptureResult<()> {
    let audit = validate::perform_with_mode(graph, validate::Mode::Audit);
    if !audit.issues.is_empty() {
        return Err(PrismFreezeCause::GraphAudit(project_issues(
            bound,
            graph,
            names,
            audit.issues,
        )));
    }
    Ok(())
}
fn checked_counts(graph: &BRepGraph, named: usize) -> CaptureResult<[u32; 7]> {
    let counts = graph_counts(graph);
    if named != counts.iter().sum::<u32>() as usize - counts[2] as usize {
        return Err(PrismFreezeCause::Unsupported {
            representation: "graph population",
            actual: "synthesized/unmapped definitions",
        });
    }
    Ok(counts)
}
fn mapped_nodes(
    bound: &BoundProjection<'_>,
    graph: &BRepGraph,
) -> CaptureResult<BTreeMap<NodeId, FrozenPrismKey>> {
    let mut names = BTreeMap::new();
    for (context, key) in bound.contexts.iter().zip(&bound.names) {
        let id = graph.shapes().find_node(&context.occurrence);
        if !id.is_valid() {
            return Err(missing("graph definition for actual occurrence"));
        }
        if names
            .insert(id, *key)
            .is_some_and(|previous| previous != *key)
        {
            return Err(identity("one stable key per graph definition", 2));
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
fn lookup(names: &BTreeMap<NodeId, FrozenPrismKey>, id: NodeId) -> CaptureResult<FrozenPrismKey> {
    names.get(&id).copied().ok_or(PrismFreezeCause::Missing {
        required: "bound graph incidence endpoint",
        actual: 0,
    })
}
fn edge_endpoints(
    graph: &BRepGraph,
    names: &BTreeMap<NodeId, FrozenPrismKey>,
) -> CaptureResult<Vec<(FrozenPrismKey, FrozenPrismKey, FrozenPrismKey)>> {
    let mut result = (0..graph.topo().edges().nb())
        .map(|index| endpoints_for(graph, names, EdgeId::new(index)))
        .collect::<Result<Vec<_>, _>>()?;
    result.sort();
    Ok(result)
}
fn endpoints_for(
    graph: &BRepGraph,
    names: &BTreeMap<NodeId, FrozenPrismKey>,
    edge: EdgeId,
) -> CaptureResult<(FrozenPrismKey, FrozenPrismKey, FrozenPrismKey)> {
    let def = graph.topo().edges().definition(edge);
    let vertices = graph.refs().vertices();
    let start = vertices.entry(def.start_vertex_ref_id).child_vertex_id;
    let end = vertices.entry(def.end_vertex_ref_id).child_vertex_id;
    Ok((
        lookup(names, edge.into())?,
        lookup(names, start.into())?,
        lookup(names, end.into())?,
    ))
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FrozenCoedgeIncidence {
    pub(crate) occurrence: FrozenUseKey,
    pub(crate) face: FrozenPrismKey,
    pub(crate) wire: FrozenPrismKey,
    pub(crate) edge: FrozenPrismKey,
    pub(crate) reversed: bool,
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
) -> CaptureResult<Vec<FrozenCoedgeIncidence>> {
    let mut result = Vec::new();
    for index in 0..graph.topo().co_edges().nb() {
        result.push(bind_coedge(bound, graph, names, CoEdgeId::new(index))?);
    }
    check_coedge_multiplicity(bound, &result)?;
    result.sort();
    Ok(result)
}
fn check_coedge_multiplicity(
    bound: &BoundProjection<'_>,
    result: &[FrozenCoedgeIncidence],
) -> CaptureResult<()> {
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
    Ok(())
}
fn bind_coedge(
    bound: &BoundProjection<'_>,
    graph: &BRepGraph,
    names: &BTreeMap<NodeId, FrozenPrismKey>,
    id: CoEdgeId,
) -> CaptureResult<FrozenCoedgeIncidence> {
    let def = graph.topo().co_edges().definition(id);
    let face = lookup(names, def.face_id.into())?;
    let wire = lookup(names, def.parent_wire_id.into())?;
    let edge = lookup(names, def.child_edge_id.into())?;
    let occurrence = coedge_occurrence(bound, [face, wire, edge], def.orientation.is_reversed)?;
    Ok(FrozenCoedgeIncidence {
        occurrence,
        face,
        wire,
        edge,
        reversed: def.orientation.is_reversed,
    })
}
fn coedge_occurrence(
    bound: &BoundProjection<'_>,
    keys: [FrozenPrismKey; 3],
    reversed: bool,
) -> CaptureResult<FrozenUseKey> {
    let found: Vec<_> = (0..bound.contexts.len())
        .filter(|i| matches_coedge(bound, *i, keys, reversed))
        .collect();
    let [index] = found.as_slice() else {
        return Err(identity(
            "one explicit directed use per graph coedge, including seam multiplicity",
            found.len(),
        ));
    };
    Ok(bound.use_names[*index].clone())
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
    definition_name(bound, wire) == Some(expected[1])
        && definition_name(bound, face) == Some(expected[0])
        && local_reverse == reversed
}
fn definition_name(
    bound: &BoundProjection<'_>,
    shape: &rustycad_tkbrep::topods::Shape,
) -> Option<FrozenPrismKey> {
    bound
        .contexts
        .iter()
        .position(|c| c.occurrence.is_same(shape))
        .map(|i| bound.names[i])
}
fn project_issues(
    bound: &BoundProjection<'_>,
    graph: &BRepGraph,
    names: &BTreeMap<NodeId, FrozenPrismKey>,
    issues: Vec<validate::Issue>,
) -> Vec<FrozenGraphIssue> {
    issues
        .into_iter()
        .map(|issue| project_issue(bound, graph, names, issue))
        .collect()
}
fn project_issue(
    bound: &BoundProjection<'_>,
    graph: &BRepGraph,
    names: &BTreeMap<NodeId, FrozenPrismKey>,
    issue: validate::Issue,
) -> FrozenGraphIssue {
    let occurrence = issue_occurrence(bound, graph, names, issue.node_id);
    let entity = names.get(&issue.node_id).copied();
    let unbound_local = (entity.is_none() && occurrence.is_none()).then_some(issue.node_id);
    FrozenGraphIssue {
        entity,
        occurrence,
        unbound_local,
        severity: issue.sev,
        description: issue.description,
    }
}
fn issue_occurrence(
    bound: &BoundProjection<'_>,
    graph: &BRepGraph,
    names: &BTreeMap<NodeId, FrozenPrismKey>,
    id: NodeId,
) -> Option<FrozenUseKey> {
    if id.kind != NodeKind::CO_EDGE {
        return None;
    }
    bind_coedge(bound, graph, names, CoEdgeId::from_node_id(id))
        .ok()
        .map(|c| c.occurrence)
}
