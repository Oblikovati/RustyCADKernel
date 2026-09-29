// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Populate.hxx and
// the runTopologyBuild, buildTopology, Perform, AppendFlattened and Append functions of
// BRepGraphInc_Populate.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraphInc_Populate`: fills the incidence storage of a graph from a shape.
//!
//! Population runs in three phases: the hierarchy traversal creates containers, wires,
//! edges and vertices and queues the faces; the face extraction reads every queued face
//! (in parallel when allowed); the registration appends the face geometry, wires and
//! coedges in queue order, then synthesises natural-restriction boundaries.
mod boundary_builder;
mod context;
mod extract;
mod face;
mod locations;
mod register;
mod supplement;
mod synthesize;
mod traverse;

use crate::brep_graph::hooks::TopoSupplementHooks;
use crate::brep_graph::parallel_policy::{ParallelPolicy, Workload};
use crate::brep_graph::{BRepGraph, NodeId};
use crate::topods::Shape;
use context::{BuildContext, TopologyBuildMode};
use face::{FaceBuildData, extract_face_data};
use rustycad_tkernel::osd::Parallel;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::toploc::Location;
use std::sync::Mutex;

/// `BuildStatus`: the result of a build operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildStatus {
    /// `Success`: all faces built successfully.
    Success,
    /// `SuccessWithWarnings`: diagnostics such as unbounded natural faces or tolerance
    /// wire orders.
    SuccessWithWarnings,
    /// `Failed`: a null shape.
    Failed,
}

/// `Options`: the population controls (none yet, as natively).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options;

/// `Perform(theGraph, theShape, theParallel, theOptions)`: clears the graph, then
/// builds the storage of the full hierarchy of the shape; `Failed` for a null shape.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph_inc::populate::{self, BuildStatus, Options};
/// use rustycad_tkbrep::topods::Shape;
/// let mut graph = BRepGraph::new();
/// let status = populate::perform(&mut graph, &Shape::default(), false, &Options).unwrap();
/// assert_eq!(status, BuildStatus::Failed);
/// ```
pub fn perform(
    graph: &mut BRepGraph,
    shape: &Shape,
    parallel: bool,
    options: &Options,
) -> Result<BuildStatus, Failure> {
    graph.clear()?;
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
    )
}

/// `AppendFlattened(theGraph, theShape, theParallel, theAppendedRoots, theOptions)`:
/// extends the storage without the containers; the non-container roots are appended
/// to `appended_roots` once each.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph_inc::populate::{self, BuildStatus, Options};
/// use rustycad_tkbrep::topods::Shape;
/// let (mut graph, mut roots) = (BRepGraph::new(), Vec::new());
/// let status = populate::append_flattened(&mut graph, &Shape::default(), false, &mut roots, &Options);
/// assert_eq!(status.unwrap(), BuildStatus::Failed);
/// ```
pub fn append_flattened(
    graph: &mut BRepGraph,
    shape: &Shape,
    parallel: bool,
    appended_roots: &mut Vec<NodeId>,
    options: &Options,
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
    )
}

/// `Append(theGraph, theShape, theParallel, theOptions)`: extends the storage with the
/// full hierarchy of the shape; definitions already bound (same TShape and location)
/// are reused.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph_inc::populate::{self, BuildStatus, Options};
/// use rustycad_tkbrep::topods::Shape;
/// let mut graph = BRepGraph::new();
/// let status = populate::append(&mut graph, &Shape::default(), false, &Options).unwrap();
/// assert_eq!(status, BuildStatus::Failed);
/// ```
pub fn append(
    graph: &mut BRepGraph,
    shape: &Shape,
    parallel: bool,
    options: &Options,
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
    )
}

// `buildTopology`: the build context over the storage, with the registered topology
// supplement layer, if any. The native captured counts and options are unused.
fn build_topology(
    graph: &mut BRepGraph,
    shape: &Shape,
    parallel: bool,
    mode: TopologyBuildMode,
    _options: &Options,
    appended_roots: Option<&mut Vec<NodeId>>,
) -> Result<BuildStatus, Failure> {
    let supplement = graph.topo_supplement();
    let mut build = BuildContext::new(
        graph.inc_storage_mut(),
        parallel,
        mode,
        supplement
            .get()
            .map(|layer| layer as &dyn TopoSupplementHooks),
        appended_roots,
    );
    run_topology_build(&mut build, shape)
}

// `runTopologyBuild`: traversal, face extraction, face registration and boundary
// synthesis; warnings from unsupported natural boundaries or wire orders.
fn run_topology_build(build: &mut BuildContext<'_>, root: &Shape) -> Result<BuildStatus, Failure> {
    traverse::traverse_topology(build, root, &Location::default())?;
    let warning_status = |warn: bool| {
        if warn {
            BuildStatus::SuccessWithWarnings
        } else {
            BuildStatus::Success
        }
    };
    if build.pending_faces.is_empty() {
        return Ok(warning_status(build.has_wire_order_warnings));
    }
    let mut faces = std::mem::take(&mut build.pending_faces);
    extract_all_faces(&mut faces, build.parallel)?;
    let unsupported = faces.iter().any(|face| face.unsupported_natural_boundary);
    register::register_face_data(build, &faces)?;
    synthesize::synthesize_face_boundaries(build, &faces)?;
    build.pending_faces = faces;
    Ok(warning_status(unsupported || build.has_wire_order_warnings))
}

// The `OSD_Parallel::For` over the queued faces; the error of the lowest failing face
// is returned, so that parallel and sequential runs report the same failure.
fn extract_all_faces(faces: &mut [FaceBuildData], parallel: bool) -> Result<(), Failure> {
    let workload = Workload {
        primary_items: faces.len() as u32,
        ..Workload::default()
    };
    let is_parallel = ParallelPolicy::should_run(parallel, &workload);
    let first_error: Mutex<Option<(usize, Failure)>> = Mutex::new(None);
    let extract = |(index, face): (usize, &mut FaceBuildData)| {
        let Err(error) = extract_face_data(face) else {
            return;
        };
        let mut slot = first_error
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if slot.as_ref().is_none_or(|(first, _)| index < *first) {
            *slot = Some((index, error));
        }
    };
    Parallel::for_each(faces.iter_mut().enumerate(), extract, !is_parallel, -1)?;
    let error = first_error
        .into_inner()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    error.map_or(Ok(()), |(_, error)| Err(error))
}
