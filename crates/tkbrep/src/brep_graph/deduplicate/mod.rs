// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_Deduplicate.hxx and
// BRepGraph_Deduplicate.cxx (`Perform`, `CanonicalizeWireOrders`) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Deduplicate`: canonicalizes shared geometry, then (on request) merges
//! equal vertices, edges, wires and faces.
//!
//! The native class has static members only: [`perform`] and [`perform_with`]. The run
//! holds a `BRepGraph_DeferredScope`, so the invalidations of every rewrite flush once
//! when it ends; the history layer records each rewrite and merge with the native
//! labels, in the iteration order of the native maps (docs/topology-design.md section
//! 11.12). C++ exceptions are [`Failure`] errors.
mod edges;
mod faces;
mod geometry;
mod redirect;
mod vertices;
mod wires;

use crate::brep_graph::deferred_scope::DeferredScope;
use crate::brep_graph::layer_history::LayerHistory;
use crate::brep_graph::{BRepGraph, EdgeId, FaceId, WireId};
use crate::brep_graph_inc::WireCoEdgeOrderStatus;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};

/// `BRepGraph_Deduplicate::Options`: the configuration of a run.
///
/// ```
/// use rustycad_tkbrep::brep_graph::deduplicate::Options;
/// let options = Options::default();
/// assert!(options.history_mode && !options.analyze_only && !options.merge_entities_when_safe);
/// assert_eq!(options.hash_tolerance, rustycad_tkernel::precision::confusion());
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Options {
    /// `AnalyzeOnly`: count without rewriting.
    pub analyze_only: bool,
    /// `HistoryMode`: record the rewrites and merges in the history layer.
    pub history_mode: bool,
    /// `MergeEntitiesWhenSafe`: also merge vertex, edge, wire and face definitions.
    pub merge_entities_when_safe: bool,
    /// `CompTolerance`: the geometry comparison and parameter-range tolerance.
    pub comp_tolerance: f64,
    /// `HashTolerance`: the geometry hash rounding and the vertex merge distance.
    pub hash_tolerance: f64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            analyze_only: false,
            history_mode: true,
            merge_entities_when_safe: false,
            comp_tolerance: precision::angular(),
            hash_tolerance: precision::confusion(),
        }
    }
}

/// `BRepGraph_Deduplicate::Result`: the counters of a run.
///
/// ```
/// use rustycad_tkbrep::brep_graph::deduplicate::DeduplicateResult;
/// let result = DeduplicateResult::default();
/// assert!(result.affected_faces.is_empty() && !result.is_entity_merge_applied);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeduplicateResult {
    /// `NbCanonicalSurfaces`: faces minus the faces whose surface is rewritten.
    pub nb_canonical_surfaces: u32,
    /// `NbCanonicalCurves`: edges minus the edges whose curve is rewritten.
    pub nb_canonical_curves: u32,
    /// `NbSurfaceRewrites`.
    pub nb_surface_rewrites: u32,
    /// `NbCurveRewrites`.
    pub nb_curve_rewrites: u32,
    /// `NbNullifiedSurfaces` (never written natively).
    pub nb_nullified_surfaces: u32,
    /// `NbNullifiedCurves` (never written natively).
    pub nb_nullified_curves: u32,
    /// `NbHistoryRecords`: the records the run asked for (also while history is off).
    pub nb_history_records: u32,
    /// `IsEntityMergeApplied`: a merge rewrote at least one definition.
    pub is_entity_merge_applied: bool,
    /// `NbMergedVertices`.
    pub nb_merged_vertices: u32,
    /// `NbMergedEdges`.
    pub nb_merged_edges: u32,
    /// `NbMergedWires`.
    pub nb_merged_wires: u32,
    /// `NbMergedFaces`.
    pub nb_merged_faces: u32,
    /// `NbReorderedWires`.
    pub nb_reordered_wires: u32,
    /// `NbToleranceOrderedWires`.
    pub nb_tolerance_ordered_wires: u32,
    /// `NbPartialOrderedWires`.
    pub nb_partial_ordered_wires: u32,
    /// `AffectedFaces`: faces whose surface changed.
    pub affected_faces: Vec<FaceId>,
    /// `AffectedEdges`: edges whose curve changed.
    pub affected_edges: Vec<EdgeId>,
}

/// `Perform(theGraph)`: [`perform_with`] and the default options.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// let result = rustycad_tkbrep::brep_graph::deduplicate::perform(&mut BRepGraph::new()).unwrap();
/// assert_eq!(result.nb_canonical_surfaces, 0);
/// ```
pub fn perform(graph: &mut BRepGraph) -> Result<DeduplicateResult, Failure> {
    perform_with(graph, &Options::default())
}

/// `Perform(theGraph, theOptions)`: an empty graph is left alone. Otherwise, inside one
/// deferred scope and with history switched to `HistoryMode`: the geometry pass, then,
/// with `MergeEntitiesWhenSafe`, the vertex merge, the wire reordering (not when
/// analyzing), and the edge, wire and face merges. The history's enabled state is
/// restored on every return (not after a raise, as natively).
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::deduplicate::{perform_with, Options};
/// use rustycad_tkmath::gp::Point3;
/// let mut graph = BRepGraph::new();
/// graph.editor().vertices().add(Point3::new(0.0, 0.0, 0.0), 1e-7);
/// graph.editor().vertices().add(Point3::new(0.0, 0.0, 0.0), 1e-7);
/// let options = Options { merge_entities_when_safe: true, ..Options::default() };
/// assert_eq!(perform_with(&mut graph, &options).unwrap().nb_merged_vertices, 1);
/// ```
pub fn perform_with(
    graph: &mut BRepGraph,
    options: &Options,
) -> Result<DeduplicateResult, Failure> {
    let mut result = DeduplicateResult::default();
    if graph.is_empty() {
        return Ok(result);
    }
    let mut scope = DeferredScope::new(graph);
    let history = scope.layer_registry().ensure::<LayerHistory>()?;
    let was_enabled = history.is_enabled();
    history.set_enabled(options.history_mode);
    run_phases(&mut scope, &history, options, &mut result)?;
    // A raise leaves the history switched to `HistoryMode`: the native restore is a
    // plain statement on the return paths, which an exception skips.
    history.set_enabled(was_enabled);
    Ok(result)
}

// The phases after the history switch; an early exit is a plain return.
fn run_phases(
    graph: &mut BRepGraph,
    history: &Handle<LayerHistory>,
    options: &Options,
    result: &mut DeduplicateResult,
) -> Result<(), Failure> {
    geometry::canonicalize(graph, history, options, result)?;
    if options.analyze_only && !options.merge_entities_when_safe {
        return Ok(());
    }
    if !options.merge_entities_when_safe {
        result.is_entity_merge_applied = false;
        return Ok(());
    }
    vertices::merge(graph, history, options, result)?;
    if !options.analyze_only {
        canonicalize_wire_orders(graph, result);
    }
    edges::merge(graph, history, options, result)?;
    wires::merge(graph, history, options, result)?;
    faces::merge(graph, history, options, result)?;
    result.is_entity_merge_applied = !options.analyze_only
        && (result.nb_merged_vertices > 0
            || result.nb_merged_edges > 0
            || result.nb_merged_wires > 0
            || result.nb_merged_faces > 0);
    Ok(())
}

/// `CanonicalizeWireOrders(theGraph, theResult)` (private natively): reorders the
/// coedges of every active wire into connected chains, counting and marking modified
/// each reordered, tolerance-ordered and partially ordered wire.
fn canonicalize_wire_orders(graph: &mut BRepGraph, result: &mut DeduplicateResult) {
    for index in 0..graph.inc_storage().nb_wires() {
        let wire = WireId::new(index);
        if graph.inc_storage().is_removed(wire) {
            continue;
        }
        let counter = match graph
            .inc_storage_mut()
            .canonicalize_wire_co_edge_order_status(wire)
        {
            WireCoEdgeOrderStatus::Reordered => &mut result.nb_reordered_wires,
            WireCoEdgeOrderStatus::ToleranceOrdered => &mut result.nb_tolerance_ordered_wires,
            WireCoEdgeOrderStatus::Partial => &mut result.nb_partial_ordered_wires,
            _ => continue,
        };
        *counter += 1;
        graph.mark_modified(wire.into());
    }
}
