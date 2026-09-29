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
// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Load.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraphInc_Load::Counts`: the section counts of an indexed load.

/// `BRepGraphInc_Load::Counts`: the final (or active) slot count of every section.
///
/// ```
/// use rustycad_tkbrep::brep_graph_inc::LoadCounts;
/// let counts = LoadCounts { nb_faces: 6, ..LoadCounts::default() };
/// assert_eq!(counts.nb_edges, 0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoadCounts {
    /// `NbVertices`.
    pub nb_vertices: u32,
    /// `NbEdges`.
    pub nb_edges: u32,
    /// `NbCoEdges`.
    pub nb_co_edges: u32,
    /// `NbWires`.
    pub nb_wires: u32,
    /// `NbFaces`.
    pub nb_faces: u32,
    /// `NbShells`.
    pub nb_shells: u32,
    /// `NbSolids`.
    pub nb_solids: u32,
    /// `NbCompounds`.
    pub nb_compounds: u32,
    /// `NbCompSolids`.
    pub nb_comp_solids: u32,
    /// `NbProducts`.
    pub nb_products: u32,
    /// `NbOccurrences`.
    pub nb_occurrences: u32,
    /// `NbShellRefs`.
    pub nb_shell_refs: u32,
    /// `NbFaceRefs`.
    pub nb_face_refs: u32,
    /// `NbWireRefs`.
    pub nb_wire_refs: u32,
    /// `NbVertexRefs`.
    pub nb_vertex_refs: u32,
    /// `NbSolidRefs`.
    pub nb_solid_refs: u32,
    /// `NbChildRefs`.
    pub nb_child_refs: u32,
    /// `NbOccurrenceRefs`.
    pub nb_occurrence_refs: u32,
    /// `NbFaceSurfaceReps`.
    pub nb_face_surface_reps: u32,
    /// `NbEdgeCurve3DReps`.
    pub nb_edge_curve3d_reps: u32,
    /// `NbCoEdgeCurve2DReps`.
    pub nb_co_edge_curve2d_reps: u32,
    /// `NbFaceTriangulationReps`.
    pub nb_face_triangulation_reps: u32,
    /// `NbEdgePolygon3DReps`.
    pub nb_edge_polygon3d_reps: u32,
    /// `NbCoEdgePolygon2DReps`.
    pub nb_co_edge_polygon2d_reps: u32,
    /// `NbCoEdgePolygonOnTriReps`.
    pub nb_co_edge_polygon_on_tri_reps: u32,
    /// `NbRootProducts`: root product ids outside the storage tables.
    pub nb_root_products: u32,
}
