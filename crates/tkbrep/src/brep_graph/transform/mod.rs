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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_Transform.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Transform`: graph-to-graph transformation.
//!
//! A geometry-level transform (the `GeomPolicy::Copy` mode of `BRepBuilderAPI_Transform`,
//! or any negative or scaling transform) sets transformed copies of the vertex points,
//! surfaces and curves through the editor and never mutates a shared handle; a
//! location-only transform composes the transformation into the placements of the
//! root products' occurrence references. The mesh is dropped after a geometry-level
//! transform, or copied (and transformed with the geometry) on request.
//!
//! [`move_child_ref`] and [`move_occurrence_ref`] are the two `MoveRef` overloads. The
//! in-graph steps of `Perform` and `TransformNode` are the crate functions
//! `transform_whole_graph` (the self-transform branch) and `transform_copied_range`
//! (the steps after a copy into the target); the two public members take the
//! `BRepGraph_Copy` policies and copy into the target first, so they are added with
//! `BRepGraph_Copy` (docs/topology-design.md section 11.12).
// Until `Perform` and `TransformNode` land with BRepGraph_Copy (area C1), only the
// unit tests call the in-graph steps.
#![cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "Perform and TransformNode arrive with BRepGraph_Copy"
    )
)]
mod geometry;
mod mesh;
#[cfg(test)]
mod tests;

use crate::brep_graph::editor::OccurrenceOps;
use crate::brep_graph::iterator::RootProductIterator;
use crate::brep_graph::{BRepGraph, ChildRefId, OccurrenceRefId, TypedId};
use crate::brep_graph_inc::NodeTag;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Transform3;
use rustycad_tkmath::toploc::Location;

use geometry::transform_geometry;
use mesh::{copy_mesh, transform_existing_mesh};

// `std::abs(std::abs(theTrsf.ScaleFactor()) - 1.) > TopLoc_Location::ScalePrec()`: the
// transformation scales, so a location cannot carry it.
pub(crate) fn is_scaled(transform: &Transform3) -> bool {
    (transform.scale_factor().abs() - 1.0).abs() > Location::scale_prec()
}

/// `MoveRef(theGraph, theRefId, theTrsf)` for a child reference: composes the
/// transformation into the reference's placement, without copying geometry. False for
/// a scaling transformation or an out-of-range or removed reference; a locked
/// reference raises Standard_ProgramError, and composing the locations can raise.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, ChildRefId};
/// use rustycad_tkbrep::brep_graph::transform::move_child_ref;
/// use rustycad_tkmath::gp::Transform3;
/// let mut graph = BRepGraph::new();
/// assert!(!move_child_ref(&mut graph, ChildRefId::new(0), &Transform3::default()).unwrap());
/// ```
pub fn move_child_ref(
    graph: &mut BRepGraph,
    reference: ChildRefId,
    transform: &Transform3,
) -> Result<bool, Failure> {
    if is_scaled(transform) {
        return Ok(false);
    }
    let children = graph.refs().children();
    if !reference.is_valid_for(children.nb()) || graph.refs().generic().is_removed(reference.into())
    {
        return Ok(false);
    }
    let old = children.entry(reference).local_location.clone();
    let moved = (&Location::new(*transform) * &old)?;
    graph
        .editor()
        .generic()
        .set_child_ref_local_location(reference, &moved)?;
    Ok(true)
}

/// `MoveRef(theGraph, theRefId, theTrsf)` for an occurrence reference: as
/// [`move_child_ref`], on the occurrence reference's placement.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, OccurrenceRefId};
/// use rustycad_tkbrep::brep_graph::transform::move_occurrence_ref;
/// use rustycad_tkmath::gp::Transform3;
/// let mut graph = BRepGraph::new();
/// let refused = move_occurrence_ref(&mut graph, OccurrenceRefId::new(0), &Transform3::default());
/// assert!(!refused.unwrap());
/// ```
pub fn move_occurrence_ref(
    graph: &mut BRepGraph,
    reference: OccurrenceRefId,
    transform: &Transform3,
) -> Result<bool, Failure> {
    if is_scaled(transform) {
        return Ok(false);
    }
    let occurrences = graph.refs().occurrences();
    if !reference.is_valid_for(occurrences.nb())
        || graph.refs().generic().is_removed(reference.into())
    {
        return Ok(false);
    }
    let old = occurrences.entry(reference).local_location.clone();
    let moved = (&Location::new(*transform) * &old)?;
    graph
        .editor()
        .occurrences()
        .set_ref_local_location(reference, &moved)?;
    Ok(true)
}

/// `GraphCounts`: the definition counts of a target graph before a copy appends to it;
/// the ids at or past them are the copied range. The default (all zero) makes every id
/// part of the range, which is the whole-graph transform.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CopiedRange {
    nb_vertices: u32,
    nb_edges: u32,
    nb_co_edges: u32,
    nb_faces: u32,
    nb_products: u32,
}

impl CopiedRange {
    /// `graphCounts(theGraph)`.
    pub(crate) fn of(graph: &BRepGraph) -> Self {
        let topo = graph.topo();
        Self {
            nb_vertices: topo.vertices().nb(),
            nb_edges: topo.edges().nb(),
            nb_co_edges: topo.co_edges().nb(),
            nb_faces: topo.faces().nb(),
            nb_products: topo.products().nb(),
        }
    }

    /// `isInCopiedRange(theId, theCounts)` for any of the five counted kinds.
    pub(crate) fn contains<T: RangeKind>(&self, id: TypedId<T>) -> bool {
        id.index >= T::bound(self)
    }
}

/// A node kind whose copied range [`CopiedRange`] counts.
pub(crate) trait RangeKind: NodeTag {
    fn bound(range: &CopiedRange) -> u32;
}

macro_rules! range_kinds {
    ($($tag:ty => $field:ident;)*) => {$(
        impl RangeKind for $tag {
            fn bound(range: &CopiedRange) -> u32 {
                range.$field
            }
        }
    )*};
}

range_kinds! {
    crate::brep_graph::VertexTag => nb_vertices;
    crate::brep_graph::EdgeTag => nb_edges;
    crate::brep_graph::CoEdgeTag => nb_co_edges;
    crate::brep_graph::FaceTag => nb_faces;
    crate::brep_graph::ProductTag => nb_products;
}

// The live ids of a kind in the range, collected before the editor writes (the native
// loops iterate the graph they edit; no edit adds or removes a node of the kind).
fn ids_in_range<T: RangeKind>(graph: &BRepGraph, range: &CopiedRange) -> Vec<TypedId<T>> {
    (0..graph.inc_storage().nb::<T>())
        .map(TypedId::new)
        .filter(|id| !id.is_removed(graph) && range.contains(*id))
        .collect()
}

/// `applyLocationTransform` (whole graph, the default range) and
/// `applyLocationTransformInCopiedRange`: composes the transformation into every live
/// occurrence reference of each live root product in the range, through a mutation
/// guard (parts carry their shape-root occurrence, assemblies their sub-products).
pub(crate) fn apply_location_transform(
    graph: &mut BRepGraph,
    range: &CopiedRange,
    transform: &Transform3,
) -> Result<(), Failure> {
    let location = Location::new(*transform);
    for product in RootProductIterator::new(graph).collect::<Vec<_>>() {
        let is_live =
            product.is_valid_for(graph.topo().products().nb()) && !product.is_removed(graph);
        if !is_live || !range.contains(product) {
            continue;
        }
        let references = graph
            .topo()
            .products()
            .relations(product)
            .occurrence_ref_ids
            .clone();
        for reference in references {
            compose_occurrence_ref(graph, reference, &location)?;
        }
    }
    Ok(())
}

// One occurrence reference of a root product, skipped when removed.
fn compose_occurrence_ref(
    graph: &mut BRepGraph,
    reference: OccurrenceRefId,
    location: &Location,
) -> Result<(), Failure> {
    if graph.refs().generic().is_removed(reference.into()) {
        return Ok(());
    }
    let mut guard = graph.editor().occurrences().mutate_ref(reference)?;
    let moved = (location * &guard.local_location)?;
    OccurrenceOps::set_ref_local_location_guarded(&mut guard, &moved);
    Ok(())
}

/// Whether a transform must rewrite the geometry (`useGeomModif`): the geometry policy
/// asks for copies (`geometry_copy`), or the transformation is negative or scales, which
/// no location can carry (the `BRepBuilderAPI_Transform` rule).
pub(crate) fn requires_geometry_modification(transform: &Transform3, geometry_copy: bool) -> bool {
    geometry_copy || transform.is_negative() || is_scaled(transform)
}

/// The self-transform branch of `Perform` (`theSourceGraph == theTargetGraph`):
/// `applyGeometryTransform` with the mesh copied and transformed on request, or
/// `applyLocationTransform` with the mesh copied as-is on request.
pub(crate) fn transform_whole_graph(
    graph: &mut BRepGraph,
    transform: &Transform3,
    use_geometry: bool,
    copy_the_mesh: bool,
) -> Result<(), Failure> {
    let whole = CopiedRange::default();
    if use_geometry {
        transform_geometry(graph, &whole, transform, copy_the_mesh)?;
    } else {
        apply_location_transform(graph, &whole, transform)?;
    }
    if copy_the_mesh {
        copy_mesh(graph, transform, use_geometry)?;
    }
    Ok(())
}

/// The steps of `Perform` and `TransformNode` after the copy into the target:
/// `applyGeometryTransformInCopiedRange` or `applyLocationTransformInCopiedRange`, then,
/// on request, the copied mesh transformed with the geometry or only its cache cleared.
pub(crate) fn transform_copied_range(
    graph: &mut BRepGraph,
    range: &CopiedRange,
    transform: &Transform3,
    use_geometry: bool,
    copy_the_mesh: bool,
) -> Result<(), Failure> {
    if use_geometry {
        transform_geometry(graph, range, transform, copy_the_mesh)?;
    } else {
        apply_location_transform(graph, range, transform)?;
    }
    if copy_the_mesh {
        transform_existing_mesh(graph, range, transform, use_geometry)?;
    }
    Ok(())
}
