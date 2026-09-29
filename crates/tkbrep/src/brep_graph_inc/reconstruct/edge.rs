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
// Rust translation of the Edge case of BRepGraphInc_Reconstruct::Node and the
// aGetOrBuildEdge lambda of BRepGraphInc_Reconstruct::FaceWithCache
// (BRepGraphInc_Reconstruct.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871).

//! The edge of an edge definition: its 3D curve and range, the SameRange and
//! SameParameter flags of its coedges, its boundary vertices and its 3D polygon.
use super::{Cache, active_record, node_with_cache};
use crate::brep::BRepBuilder;
use crate::brep_graph::cache_derived_state;
use crate::brep_graph::{BRepGraph, EdgeId, VertexRefId};
use crate::brep_graph_inc::{EdgeDef, Storage};
use crate::topods::Edge;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

/// The new edge of `edge`. `always_range` selects the `Node` variant (the range of the
/// curve record, 0..0 without one, is set even on degenerated edges) over the
/// `FaceWithCache` variant (the range is set only with an active curve record).
pub(super) fn build_edge(
    graph: &BRepGraph,
    edge: EdgeId,
    cache: &mut Cache,
    always_range: bool,
) -> Result<Edge, Failure> {
    let storage = graph.inc_storage();
    let def = storage.edge(edge);
    let (degenerated, closed) =
        cache_derived_state::compute_edge_properties(graph, edge).unwrap_or((false, false));
    let mut result = make_edge_geometry(storage, def, degenerated, always_range)?;
    set_same_flags(graph, edge, &result)?;
    add_vertex(
        graph,
        def.start_vertex_ref_id,
        Orientation::Forward,
        &mut result,
        cache,
    )?;
    add_vertex(
        graph,
        def.end_vertex_ref_id,
        Orientation::Reversed,
        &mut result,
        cache,
    )?;
    attach_polygon3d(storage, def, &result)?;
    if closed {
        result.set_closed(true)?;
    }
    Ok(result)
}

// The TEdge with its curve (none when degenerated) and range.
fn make_edge_geometry(
    storage: &Storage,
    def: &EdgeDef,
    degenerated: bool,
    always_range: bool,
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut result = Edge::default();
    let curve_rep = active_record(storage, def.curve3d_rep_id);
    if degenerated {
        builder.make_edge(&mut result)?;
        builder.degenerated(&result, true)?;
    } else if let Some(rep) = curve_rep.filter(|rep| !rep.curve.is_null()) {
        builder.make_edge_with_curve(&mut result, &rep.curve, def.tolerance)?;
    } else {
        builder.make_edge(&mut result)?;
    }
    let range = curve_rep.map_or((0.0, 0.0), |rep| (rep.param_first, rep.param_last));
    if always_range || (!degenerated && curve_rep.is_some()) {
        builder.range(&result, range.0, range.1, false)?;
    }
    Ok(result)
}

// SameRange/SameParameter are per-coedge properties: the edge flag is true only when
// every active coedge agrees (every coedge is evaluated, as natively).
fn set_same_flags(graph: &BRepGraph, edge: EdgeId, result: &Edge) -> Result<(), Failure> {
    let storage = graph.inc_storage();
    let (mut same_range, mut same_parameter) = (true, true);
    for &co_edge in &storage.edge_relations(edge).co_edge_ids {
        if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
            continue;
        }
        same_range &= cache_derived_state::compute_same_range(graph, co_edge);
        same_parameter &= cache_derived_state::compute_same_parameter(graph, co_edge)?;
    }
    let builder = BRepBuilder::default();
    builder.same_parameter(result, same_parameter)?;
    builder.same_range(result, same_range)
}

// A boundary vertex (shared through the cache) in the given orientation.
fn add_vertex(
    graph: &BRepGraph,
    reference: VertexRefId,
    orientation: Orientation,
    result: &mut Edge,
    cache: &mut Cache,
) -> Result<(), Failure> {
    if !reference.is_valid() {
        return Ok(());
    }
    let vertex = graph.inc_storage().vertex_ref(reference).child_vertex_id;
    let mut shape = node_with_cache(graph, vertex.into(), cache)?;
    if shape.is_null() {
        return Ok(());
    }
    shape.set_orientation(orientation);
    BRepBuilder::default().add(result, &shape)
}

// The active 3D polygon record's polygon, when not null.
fn attach_polygon3d(storage: &Storage, def: &EdgeDef, result: &Edge) -> Result<(), Failure> {
    let Some(record) = active_record(storage, def.polygon3d_rep_id) else {
        return Ok(());
    };
    let polygon = &record.polygon;
    if polygon.is_null() {
        return Ok(());
    }
    BRepBuilder::default().update_edge_polygon3d(result, polygon, &Location::default())
}
