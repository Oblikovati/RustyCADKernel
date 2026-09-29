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
// Rust translation of BRepGraphInc_Reconstruct::FaceWithCache and its aBuildWireForFace
// and aProcessCoEdgeForFace lambdas (BRepGraphInc_Reconstruct.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871).

//! The face of a face definition: its surface and triangulation, its wires in their
//! orientation, and the p-curves and polygons of its coedges installed on the shared
//! edges (both seam halves at once).
use super::edge::build_edge;
use super::{Cache, active_record, replay_supplement};
use crate::brep::{BRepBuilder, BRepTFace};
use crate::brep_graph::cache_derived_state;
use crate::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId, WireId};
use crate::brep_graph_inc::{CoEdgeDef, Storage};
use crate::topods::{Builder, Edge, Face, Shape, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::poly::Triangulation;
use rustycad_tkmath::toploc::Location;
use std::collections::HashSet;

// The graph, cache and face geometry shared by the wires of one face.
struct FaceBuild<'a> {
    graph: &'a BRepGraph,
    cache: &'a mut Cache,
    surface: Handle<dyn GeomSurface>,
    triangulation: Handle<Triangulation>,
}

/// `FaceWithCache(theGraph, theFaceId, theCache)`: the cached face, else a new FORWARD
/// face on the face's surface (a bare face without one) with its triangulation, its
/// wires and the supplement attachments; null for a face out of range.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, FaceId};
/// use rustycad_tkbrep::brep_graph_inc::reconstruct::{self, Cache};
/// let graph = BRepGraph::new();
/// let face = reconstruct::face_with_cache(&graph, FaceId::new(0), &mut Cache::new()).unwrap();
/// assert!(face.is_null());
/// ```
pub fn face_with_cache(
    graph: &BRepGraph,
    face: FaceId,
    cache: &mut Cache,
) -> Result<Shape, Failure> {
    let storage = graph.inc_storage();
    if !face.is_valid_for(storage.nb_faces()) {
        return Ok(Shape::default());
    }
    if let Some(cached) = cache.seek(face.into()) {
        return Ok(cached.clone());
    }
    let def = storage.face(face);
    let surface = active_record(storage, def.surface_rep_id)
        .map_or_else(Handle::default, |rep| rep.surface.clone());
    let mut result = Face::default();
    if surface.is_null() {
        BRepBuilder::default().make_face(&mut result);
    } else {
        BRepBuilder::default().make_face_on_surface(&mut result, &surface, def.tolerance)?;
    }
    let triangulation = attach_triangulation(storage, def.triangulation_rep_id, &result)?;
    let mut build = FaceBuild {
        graph,
        cache,
        surface,
        triangulation,
    };
    add_face_wires(&mut build, face, &mut result)?;
    result.set_orientation(Orientation::Forward);
    let mut shape: Shape = result.into();
    replay_supplement(graph.topo_supplement(), face.into(), &mut shape)?;
    build.cache.bind(face.into(), &shape);
    Ok(shape)
}

// The active triangulation record's triangulation, installed as the only (and active)
// triangulation of the TFace; the null handle without one.
fn attach_triangulation(
    storage: &Storage,
    rep: crate::brep_graph::FaceTriangulationRepId,
    face: &Face,
) -> Result<Handle<Triangulation>, Failure> {
    let Some(record) = active_record(storage, rep) else {
        return Ok(Handle::default());
    };
    let triangulation = record.triangulation.clone();
    if triangulation.is_null() {
        return Ok(triangulation);
    }
    let tface = face.tshape().cast::<BRepTFace>();
    if let Some(tface) = tface.get() {
        tface.set_triangulations(std::slice::from_ref(&triangulation), &triangulation)?;
    }
    Ok(triangulation)
}

// The active wire references' wires, in their orientation (set before adding).
fn add_face_wires(
    build: &mut FaceBuild<'_>,
    face: FaceId,
    result: &mut Face,
) -> Result<(), Failure> {
    let storage = build.graph.inc_storage();
    for &ref_id in &storage.face_relations(face).wire_ref_ids {
        if storage.is_removed(ref_id) {
            continue;
        }
        let reference = storage.wire_ref(ref_id);
        if !reference.child_wire_id.is_valid_for(storage.nb_wires()) {
            continue;
        }
        let mut wire = build_wire_for_face(build, reference.child_wire_id)?;
        wire.set_orientation(reference.orientation.orientation());
        Builder.add(result, &wire)?;
    }
    Ok(())
}

// `aBuildWireForFace`: the cached wire gets only the face representations of its
// coedges; a new wire gets its edges too and is cached. The closure flag is applied
// last (BRep_Builder::Add may reset it).
fn build_wire_for_face(build: &mut FaceBuild<'_>, wire: WireId) -> Result<Wire, Failure> {
    let storage = build.graph.inc_storage();
    let cached = build.cache.seek(wire.into()).cloned();
    let add_to_wire = cached.is_none();
    let mut result = match cached {
        Some(shape) => Wire::unchecked(shape),
        None => {
            let mut fresh = Wire::default();
            Builder.make_wire(&mut fresh);
            fresh
        }
    };
    let mut installed = HashSet::new();
    for &co_edge in &storage.wire_relations(wire).co_edge_ids {
        process_co_edge(
            build,
            co_edge,
            add_to_wire.then_some(&mut result),
            &mut installed,
        )?;
    }
    if add_to_wire {
        build.cache.bind(wire.into(), &result);
    }
    if cache_derived_state::compute_wire_is_closed(build.graph, wire) {
        result.set_closed(true)?;
    }
    Ok(result)
}

// `aGetOrBuildEdge`: the cached edge, else the `FaceWithCache` edge variant with its
// supplement attachments, cached.
fn get_or_build_edge(build: &mut FaceBuild<'_>, edge: EdgeId) -> Result<Edge, Failure> {
    if let Some(cached) = build.cache.seek(edge.into()) {
        return Ok(Edge::unchecked(cached.clone()));
    }
    let result = build_edge(build.graph, edge, build.cache, false)?;
    let mut shape: Shape = result.into();
    replay_supplement(build.graph.topo_supplement(), edge.into(), &mut shape)?;
    build.cache.bind(edge.into(), &shape);
    Ok(Edge::unchecked(shape))
}

// `aProcessCoEdgeForFace`: the edge (added to the wire when asked), then its face
// representations, installed once per edge per wire.
fn process_co_edge(
    build: &mut FaceBuild<'_>,
    co_edge_id: CoEdgeId,
    wire: Option<&mut Wire>,
    installed: &mut HashSet<EdgeId>,
) -> Result<(), Failure> {
    let storage = build.graph.inc_storage();
    if !co_edge_id.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge_id) {
        return Ok(());
    }
    let co_edge = storage.co_edge(co_edge_id);
    if !co_edge.child_edge_id.is_valid_for(storage.nb_edges()) {
        return Ok(());
    }
    let edge = get_or_build_edge(build, co_edge.child_edge_id)?;
    if let Some(wire) = wire {
        let mut in_wire = edge.clone();
        in_wire.set_orientation(co_edge.orientation.orientation());
        Builder.add(wire, &in_wire)?;
    }
    if !installed.insert(co_edge.child_edge_id) {
        return Ok(());
    }
    let halves = seam_halves(storage, co_edge_id, co_edge);
    let tolerance = storage.edge(co_edge.child_edge_id).tolerance;
    install_pcurves(build, &edge, co_edge, halves, tolerance)?;
    install_polygons_on_surface(build, &edge, co_edge, halves)?;
    install_polygons_on_triangulation(build, &edge, co_edge, halves)
}

/// The (FORWARD, REVERSED) halves of a seam: the first active sibling coedge of the
/// same edge, face and wire with the other orientation.
type SeamHalves<'s> = Option<(&'s CoEdgeDef, &'s CoEdgeDef)>;

// For a seam, BRep_CurveOnClosedSurface takes PCurve() from the FORWARD half and
// PCurve2() from the REVERSED half, whichever half is visited.
fn seam_halves<'s>(
    storage: &'s Storage,
    co_edge_id: CoEdgeId,
    co_edge: &'s CoEdgeDef,
) -> SeamHalves<'s> {
    for &other_id in &storage.edge_relations(co_edge.child_edge_id).co_edge_ids {
        if other_id == co_edge_id || storage.is_removed(other_id) {
            continue;
        }
        let other = storage.co_edge(other_id);
        if other.face_id != co_edge.face_id || other.orientation == co_edge.orientation {
            continue;
        }
        // Only siblings of the same wire: orphaned coedges of dedup-merged wires may
        // carry p-curves on another surface.
        if other.parent_wire_id != co_edge.parent_wire_id {
            continue;
        }
        return Some(if co_edge.orientation.is_reversed {
            (other, co_edge)
        } else {
            (co_edge, other)
        });
    }
    None
}

// The p-curves and their range: the FORWARD half's first and the REVERSED half's
// second (with its range when the FORWARD p-curve is null), else the coedge's own.
fn select_pcurves(
    storage: &Storage,
    co_edge: &CoEdgeDef,
    halves: SeamHalves<'_>,
) -> SelectedPCurves {
    let mut selected = SelectedPCurves::default();
    let Some((forward, reversed)) = halves else {
        if let Some(record) = active_record(storage, co_edge.curve2d_rep_id) {
            selected = SelectedPCurves {
                first: record.curve.clone(),
                range: (record.param_first, record.param_last),
                ..selected
            };
        }
        return selected;
    };
    if let Some(record) = active_record(storage, forward.curve2d_rep_id) {
        selected.first = record.curve.clone();
        selected.range = (record.param_first, record.param_last);
    }
    if let Some(record) = active_record(storage, reversed.curve2d_rep_id) {
        selected.second = record.curve.clone();
        if selected.first.is_null() {
            selected.range = (record.param_first, record.param_last);
        }
    }
    selected
}

// `aPC1`, `aPC2`, `aPCFirst`, `aPCLast`.
#[derive(Default)]
struct SelectedPCurves {
    first: Handle<dyn Geom2dCurve>,
    second: Handle<dyn Geom2dCurve>,
    range: (f64, f64),
}

// The seam pair (with the UV points of the first p-curve), the single first or the
// single second p-curve, each followed by its range on the surface.
fn install_pcurves(
    build: &FaceBuild<'_>,
    edge: &Edge,
    co_edge: &CoEdgeDef,
    halves: SeamHalves<'_>,
    tolerance: f64,
) -> Result<(), Failure> {
    let selected = select_pcurves(build.graph.inc_storage(), co_edge, halves);
    let (builder, identity, surface) =
        (BRepBuilder::default(), Location::default(), &build.surface);
    let (first, last) = selected.range;
    if let Some(curve) = selected.first.get() {
        let uv = Some((curve.eval_d0(first)?, curve.eval_d0(last)?));
        if selected.second.is_null() {
            builder.update_edge_pcurve(edge, &selected.first, surface, &identity, tolerance, uv)?;
        } else {
            let pair = (&selected.first, &selected.second);
            builder.update_edge_pcurves(edge, pair, surface, &identity, tolerance, uv)?;
        }
    } else if !selected.second.is_null() {
        builder.update_edge_pcurve(edge, &selected.second, surface, &identity, tolerance, None)?;
    } else {
        return Ok(());
    }
    builder.range_on_surface(edge, surface, &identity, first, last)
}

// The 2D polygons: both seam halves', else the coedge's own.
fn install_polygons_on_surface(
    build: &FaceBuild<'_>,
    edge: &Edge,
    co_edge: &CoEdgeDef,
    halves: SeamHalves<'_>,
) -> Result<(), Failure> {
    let storage = build.graph.inc_storage();
    let polygon = |def: &CoEdgeDef| {
        active_record(storage, def.polygon2d_rep_id).map(|rep| rep.polygon.clone())
    };
    let (first, second) = match halves {
        Some((forward, reversed)) => (polygon(forward), polygon(reversed)),
        None => (polygon(co_edge), None),
    };
    let (first, second) = (first.unwrap_or_default(), second.unwrap_or_default());
    let (builder, identity, surface) =
        (BRepBuilder::default(), Location::default(), &build.surface);
    match (first.is_null(), second.is_null()) {
        (false, false) => {
            builder.update_edge_polygons_on_surface(edge, (&first, &second), surface, &identity)
        }
        (false, true) => builder.update_edge_polygon_on_surface(edge, &first, surface, &identity),
        (true, false) => builder.update_edge_polygon_on_surface(edge, &second, surface, &identity),
        (true, true) => Ok(()),
    }
}

// The polygons on the face triangulation: both seam halves', else the coedge's own;
// nothing without a face triangulation.
fn install_polygons_on_triangulation(
    build: &FaceBuild<'_>,
    edge: &Edge,
    co_edge: &CoEdgeDef,
    halves: SeamHalves<'_>,
) -> Result<(), Failure> {
    let storage = build.graph.inc_storage();
    let nodes = |def: &CoEdgeDef| {
        active_record(storage, def.polygon_on_tri_rep_id).map(|rep| rep.polygon.clone())
    };
    let (first, second) = match halves {
        Some((forward, reversed)) => (nodes(forward), nodes(reversed)),
        None => (nodes(co_edge), None),
    };
    let triangulation = &build.triangulation;
    if triangulation.is_null() {
        return Ok(());
    }
    let (first, second) = (first.unwrap_or_default(), second.unwrap_or_default());
    let (builder, identity) = (BRepBuilder::default(), Location::default());
    match (first.is_null(), second.is_null()) {
        (false, false) => builder.update_edge_polygons_on_triangulation(
            edge,
            (&first, &second),
            triangulation,
            &identity,
        ),
        (false, true) => {
            builder.update_edge_polygon_on_triangulation(edge, &first, triangulation, &identity)
        }
        (true, false) => {
            builder.update_edge_polygon_on_triangulation(edge, &second, triangulation, &identity)
        }
        (true, true) => Ok(()),
    }
}
