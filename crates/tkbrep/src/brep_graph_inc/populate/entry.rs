// SPDX-License-Identifier: GPL-2.0-only
//! Additive versioned entrypoints share the existing topology construction pipeline.
use super::{BuildStatus, Options, PCurveImportBehavior, TopologyBuildMode, build_topology};
use crate::{
    brep_graph::{BRepGraph, NodeId},
    topods::Shape,
};
use rustycad_tkernel::standard::Failure;

/// Clear and import with an explicit pcurve behavior; `perform_with_behavior(&mut graph, &shape, false, &Options, PCurveImportBehavior::LegacyLocalV1)`.
pub fn perform_with_behavior(
    graph: &mut BRepGraph,
    shape: &Shape,
    parallel: bool,
    options: &Options,
    behavior: PCurveImportBehavior,
) -> Result<BuildStatus, Failure> {
    graph.clear()?;
    append_with_behavior(graph, shape, parallel, options, behavior)
}
/// Append full hierarchy with an explicit pcurve behavior; `append_with_behavior(&mut graph, &shape, false, &Options, PCurveImportBehavior::LegacyLocalV1)`.
pub fn append_with_behavior(
    graph: &mut BRepGraph,
    shape: &Shape,
    parallel: bool,
    options: &Options,
    behavior: PCurveImportBehavior,
) -> Result<BuildStatus, Failure> {
    if shape.is_null() {
        return Ok(BuildStatus::Failed);
    }
    build_topology(
        graph,
        shape,
        parallel,
        TopologyBuildMode::FullHierarchy,
        options,
        None,
        behavior,
    )
}
/// Append flattened roots with an explicit pcurve behavior; `append_flattened_with_behavior(&mut graph, &shape, false, &mut roots, &Options, PCurveImportBehavior::LegacyLocalV1)`.
pub fn append_flattened_with_behavior(
    graph: &mut BRepGraph,
    shape: &Shape,
    parallel: bool,
    appended_roots: &mut Vec<NodeId>,
    options: &Options,
    behavior: PCurveImportBehavior,
) -> Result<BuildStatus, Failure> {
    if shape.is_null() {
        return Ok(BuildStatus::Failed);
    }
    build_topology(
        graph,
        shape,
        parallel,
        TopologyBuildMode::Flattened,
        options,
        Some(appended_roots),
        behavior,
    )
}
