// Rust translation of BRepGraph_Transform::Perform and TransformNode of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Transform.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Perform` and `TransformNode`: the policy checks, the copy through
//! `BRepGraph_Copy` (default cache policy), then the in-graph steps on the copy.
use super::{
    CopiedRange, requires_geometry_modification, transform_copied_range, transform_whole_graph,
};
use crate::brep_graph::copy::{self, CachePolicy, GeomPolicy, MeshPolicy};
use crate::brep_graph::{BRepGraph, NodeId, NodeKind};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Transform3;

// The branch a transform takes after the policy checks: geometry-level (`useGeomModif`)
// or location-only, and whether the mesh is copied.
#[derive(Clone, Copy)]
struct Plan {
    use_geometry: bool,
    copy_mesh: bool,
}

// The native checks: a negative or scaling transformation, or the Copy geometry policy
// (or, for a node, a topology node), needs geometry-level modification, which refuses
// the Drop geometry policy and the Share mesh policy (stale handles).
fn plan(
    transform: &Transform3,
    geom: GeomPolicy,
    mesh: MeshPolicy,
    needs_geometry: bool,
) -> Option<Plan> {
    let use_geometry =
        needs_geometry || requires_geometry_modification(transform, geom == GeomPolicy::Copy);
    if use_geometry && (geom == GeomPolicy::Drop || mesh == MeshPolicy::Share) {
        return None;
    }
    Some(Plan {
        use_geometry,
        copy_mesh: mesh == MeshPolicy::Copy,
    })
}

// `TransformNode`'s plan: a product or occurrence carries its placement on occurrence
// references, which the geometry-level path never updates, so that combination is refused.
fn node_plan(
    node: NodeId,
    transform: &Transform3,
    geom: GeomPolicy,
    mesh: MeshPolicy,
) -> Option<Plan> {
    let is_placement = node.kind == NodeKind::PRODUCT || node.kind == NodeKind::OCCURRENCE;
    let plan = plan(transform, geom, mesh, !is_placement)?;
    (!(plan.use_geometry && is_placement)).then_some(plan)
}

/// `Perform(theSourceGraph, theTargetGraph, theTrsf, theGeomPolicy, theMeshPolicy)` into
/// another graph: copies the source (appending to a non-empty target), then transforms
/// the copied range. False for an empty source, a refused policy or a refused copy.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::copy::{GeomPolicy, MeshPolicy};
/// use rustycad_tkbrep::brep_graph::transform;
/// use rustycad_tkmath::gp::Transform3;
/// let (source, mut target) = (BRepGraph::new(), BRepGraph::new());
/// let done = transform::perform(&source, &mut target, &Transform3::default(),
///     GeomPolicy::Copy, MeshPolicy::Drop).unwrap();
/// assert!(!done);
/// ```
pub fn perform(
    source: &BRepGraph,
    target: &mut BRepGraph,
    transform: &Transform3,
    geom: GeomPolicy,
    mesh: MeshPolicy,
) -> Result<bool, Failure> {
    if source.is_empty() {
        return Ok(false);
    }
    let Some(plan) = plan(transform, geom, mesh, false) else {
        return Ok(false);
    };
    let range = CopiedRange::of(target);
    if !copy::perform(source, target, geom, mesh, CachePolicy::Drop)? {
        return Ok(false);
    }
    transform_copied_range(target, &range, transform, plan.use_geometry, plan.copy_mesh)?;
    Ok(true)
}

/// `Perform(theGraph, theGraph, theTrsf, ...)`: the self-transform, in place, with no
/// copy (a geometry-level or location-only transform, the mesh copied on request).
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::copy::{GeomPolicy, MeshPolicy};
/// use rustycad_tkbrep::brep_graph::transform;
/// use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
/// let mut graph = BRepGraph::new();
/// let vertex = graph.editor().vertices().add(Point3::new(1.0, 0.0, 0.0), 1e-7);
/// let shift = Transform3::translation_vector(Vector3::new(0.0, 2.0, 0.0));
/// assert!(transform::perform_in_place(&mut graph, &shift, GeomPolicy::Copy, MeshPolicy::Drop).unwrap());
/// assert_eq!(graph.topo().vertices().definition(vertex).point, Point3::new(1.0, 2.0, 0.0));
/// ```
pub fn perform_in_place(
    graph: &mut BRepGraph,
    transform: &Transform3,
    geom: GeomPolicy,
    mesh: MeshPolicy,
) -> Result<bool, Failure> {
    if graph.is_empty() {
        return Ok(false);
    }
    let Some(plan) = plan(transform, geom, mesh, false) else {
        return Ok(false);
    };
    transform_whole_graph(graph, transform, plan.use_geometry, plan.copy_mesh)?;
    Ok(true)
}

/// `TransformNode(theSourceGraph, theTargetGraph, theNodeId, theTrsf, ...)` into another
/// graph: copies the node's sub-graph, then transforms the copied range; the mapped
/// root, or the invalid id for an empty source, a refused policy or a failed copy.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, NodeId};
/// use rustycad_tkbrep::brep_graph::copy::{GeomPolicy, MeshPolicy};
/// use rustycad_tkbrep::brep_graph::transform;
/// use rustycad_tkmath::gp::Transform3;
/// let (source, mut target) = (BRepGraph::new(), BRepGraph::new());
/// let root = transform::transform_node(&source, &mut target, NodeId::default(),
///     &Transform3::default(), GeomPolicy::Copy, MeshPolicy::Drop).unwrap();
/// assert!(!root.is_valid());
/// ```
pub fn transform_node(
    source: &BRepGraph,
    target: &mut BRepGraph,
    node: NodeId,
    transform: &Transform3,
    geom: GeomPolicy,
    mesh: MeshPolicy,
) -> Result<NodeId, Failure> {
    if source.is_empty() {
        return Ok(NodeId::default());
    }
    let Some(plan) = node_plan(node, transform, geom, mesh) else {
        return Ok(NodeId::default());
    };
    let range = CopiedRange::of(target);
    let root = copy::copy_node(source, target, node, geom, mesh, CachePolicy::Drop)?;
    finish_node(target, root, &range, transform, plan)
}

/// `TransformNode(theGraph, theGraph, theNodeId, theTrsf, ...)`: duplicates the node's
/// sub-graph with new ids in the same graph, then transforms the duplicate.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::copy::{GeomPolicy, MeshPolicy};
/// use rustycad_tkbrep::brep_graph::transform;
/// use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
/// let mut graph = BRepGraph::new();
/// let vertex = graph.editor().vertices().add(Point3::new(1.0, 0.0, 0.0), 1e-7);
/// let shift = Transform3::translation_vector(Vector3::new(0.0, 2.0, 0.0));
/// let copy = transform::transform_node_in_place(&mut graph, vertex.into(), &shift,
///     GeomPolicy::Copy, MeshPolicy::Drop).unwrap();
/// assert_eq!(copy.index, 1);
/// ```
pub fn transform_node_in_place(
    graph: &mut BRepGraph,
    node: NodeId,
    transform: &Transform3,
    geom: GeomPolicy,
    mesh: MeshPolicy,
) -> Result<NodeId, Failure> {
    if graph.is_empty() {
        return Ok(NodeId::default());
    }
    let Some(plan) = node_plan(node, transform, geom, mesh) else {
        return Ok(NodeId::default());
    };
    let range = CopiedRange::of(graph);
    let root = copy::copy_node_in_place(graph, node, geom, mesh, CachePolicy::Drop)?;
    finish_node(graph, root, &range, transform, plan)
}

// The steps after a node copy: nothing when the copy failed.
fn finish_node(
    graph: &mut BRepGraph,
    root: NodeId,
    range: &CopiedRange,
    transform: &Transform3,
    plan: Plan,
) -> Result<NodeId, Failure> {
    if !root.is_valid() {
        return Ok(NodeId::default());
    }
    transform_copied_range(graph, range, transform, plan.use_geometry, plan.copy_mesh)?;
    Ok(root)
}
