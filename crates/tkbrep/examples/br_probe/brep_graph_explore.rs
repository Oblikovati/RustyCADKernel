//! BRepGraph oracle, part 4, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_explore.cxx: the child and parent explorers
//! from every node under the probe configurations.
use super::brep::Scene;
use super::brep_graph::{node, reference};
use super::brep_graph_walk::nodes;
use super::common::{failure, flag, int, word};
use super::graph::chain;
use rustycad_tkbrep::brep_graph::child_explorer::{self, ChildExplorer};
use rustycad_tkbrep::brep_graph::parent_explorer::{self, ParentExplorer};
use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId, NodeKind};
use rustycad_tkbrep::brep_graph_inc::NodeInstance;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

fn instance(s: &Scene, i: &NodeInstance) {
    node(i.def_id);
    chain(&s.graph, &i.location);
    int(i.orientation as i32);
}
fn avoid(kind: Option<NodeKind>) {
    flag(kind.is_some());
    if let Some(kind) = kind {
        int(kind.0);
    }
}

// `bx_child`; an error of `Next` ends the report (the native exception leaves it).
fn child(s: &Scene, mut x: ChildExplorer<'_>) -> Result<(), Failure> {
    avoid(x.config().avoid_kind);
    while x.more() {
        instance(s, &x.current());
        node(x.current_parent());
        int(x.current_link_kind() as i32);
        reference(x.current_ref());
        let path = x.current_usage_path();
        int(path.size() as i32);
        for i in 0..path.size() {
            let step = path.value(i);
            node(step.node);
            reference(step.reference);
            int(step.step_index);
        }
        int(x.depth());
        chain(&s.graph, &x.location_of(NodeKind::FACE));
        node(x.node_of(NodeKind::EDGE));
        for level in -1..3 {
            chain(&s.graph, &x.location_at(level));
            node(x.node_at(level));
        }
        x.next()?;
    }
    instance(s, &x.current());
    word("|");
    Ok(())
}
fn child_report(s: &Scene, x: Result<ChildExplorer<'_>, Failure>) {
    if let Err(error) = x.and_then(|x| child(s, x)) {
        failure(&error);
    }
}

// `bx_child_configs`.
fn child_configs(s: &Scene, g: &BRepGraph, n: NodeId, l: &Location) {
    use child_explorer::TraversalMode::{DirectChildren, Recursive};
    let targets = [
        NodeKind::SOLID,
        NodeKind::SHELL,
        NodeKind::FACE,
        NodeKind::WIRE,
        NodeKind::CO_EDGE,
        NodeKind::EDGE,
        NodeKind::VERTEX,
        NodeKind::OCCURRENCE,
        NodeKind::PRODUCT,
    ];
    child_report(s, ChildExplorer::new(g, n));
    child_report(s, ChildExplorer::with_mode(g, n, DirectChildren));
    for k in targets {
        child_report(s, ChildExplorer::with_target(g, n, k));
    }
    for k in [NodeKind::FACE, NodeKind::EDGE, NodeKind::OCCURRENCE] {
        child_report(s, ChildExplorer::with_target_mode(g, n, k, DirectChildren));
    }
    child_report(
        s,
        ChildExplorer::with_avoid(g, n, Some(NodeKind::FACE), true, Recursive),
    );
    child_report(
        s,
        ChildExplorer::with_avoid(g, n, Some(NodeKind::WIRE), false, Recursive),
    );
    let vertex_avoid = ChildExplorer::with_target_avoid(
        g,
        n,
        NodeKind::VERTEX,
        Some(NodeKind::FACE),
        true,
        Recursive,
    );
    child_report(s, vertex_avoid);
    child_report(
        s,
        ChildExplorer::with_accumulation(g, n, NodeKind::VERTEX, false, false, Recursive),
    );
    let start = ChildExplorer::with_start(
        g,
        n,
        NodeKind::EDGE,
        l,
        Orientation::Reversed,
        DirectChildren,
    );
    child_report(s, start);
}

// `bx_parent`.
fn parent(s: &Scene, mut x: ParentExplorer<'_>) -> Result<(), Failure> {
    avoid(x.config().avoid_kind);
    while x.more() {
        instance(s, &x.current());
        node(x.current_child());
        int(x.current_link_kind() as i32);
        reference(x.current_ref());
        chain(&s.graph, &x.leaf_location());
        int(x.leaf_orientation() as i32);
        flag(x.is_current_branch_root());
        x.next()?;
    }
    instance(s, &x.current());
    word("|");
    Ok(())
}
fn parent_report(s: &Scene, x: Result<ParentExplorer<'_>, Failure>) {
    if let Err(error) = x.and_then(|x| parent(s, x)) {
        failure(&error);
    }
}

// `bx_parent_configs`.
fn parent_configs(s: &Scene, g: &BRepGraph, n: NodeId) {
    use parent_explorer::TraversalMode::{DirectParents, Recursive};
    let targets = [
        NodeKind::EDGE,
        NodeKind::WIRE,
        NodeKind::FACE,
        NodeKind::SHELL,
        NodeKind::SOLID,
        NodeKind::COMPOUND,
        NodeKind::PRODUCT,
    ];
    parent_report(s, ParentExplorer::new(g, n));
    parent_report(s, ParentExplorer::with_mode(g, n, DirectParents));
    for k in targets {
        parent_report(s, ParentExplorer::with_target(g, n, k));
    }
    for k in [NodeKind::FACE, NodeKind::OCCURRENCE] {
        parent_report(s, ParentExplorer::with_target_mode(g, n, k, DirectParents));
    }
    parent_report(
        s,
        ParentExplorer::with_avoid(g, n, Some(NodeKind::SHELL), true, Recursive),
    );
    parent_report(
        s,
        ParentExplorer::with_avoid(g, n, Some(NodeKind::FACE), false, Recursive),
    );
    let solid_avoid = ParentExplorer::with_target_avoid(
        g,
        n,
        NodeKind::SOLID,
        Some(NodeKind::FACE),
        true,
        Recursive,
    );
    parent_report(s, solid_avoid);
}

/// `bx_explore`: every node with both explorers, then the child explorer past each
/// kind's end.
pub fn explore(s: &Scene, g: &BRepGraph) {
    let l = s.graph.loc(s.graph.locs.len() as i32 - 1);
    for n in nodes(g) {
        child_configs(s, g, n, &l);
        parent_configs(s, g, n);
    }
    for k in 0..12 {
        let past = NodeId::new(NodeKind(k), g.topo().generic().nb(NodeKind(k)));
        child_configs(s, g, past, &l);
    }
    word("#");
}
