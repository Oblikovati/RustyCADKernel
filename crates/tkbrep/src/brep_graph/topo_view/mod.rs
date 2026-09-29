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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_TopoView.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph::TopoView`: read-only definitions, relations, representations and
//! adjacency, grouped by kind.
//!
//! Counts include soft-removed nodes; the definition accessors do not filter removed
//! nodes (callers check `is_removed` when needed). Every group is a `Copy` view that
//! borrows the graph.
mod assembly;
mod edge_parents;
mod generic;
mod geometry;

pub use generic::{GenOps, GeometryOps};

use crate::brep_graph::{BRepGraph, CountProvider};

/// `BRepGraph::TopoView`: obtained by [`BRepGraph::topo`].
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// let graph = BRepGraph::new();
/// assert_eq!(graph.topo().faces().nb(), 0);
/// assert_eq!(graph.topo().generic().nb_nodes(), 0);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct TopoView<'g> {
    graph: &'g BRepGraph,
}

impl BRepGraph {
    /// `Topo()`: the topology view.
    pub fn topo(&self) -> TopoView<'_> {
        TopoView { graph: self }
    }
}

macro_rules! kind_ops {
    ($($(#[$doc:meta])* $ops:ident, $accessor:ident, $id:ident, $def:ident, $get:ident,
       $nb:ident, $nb_active:ident $(, $relations:ident, $relations_type:ident)?;)*) => {
        impl<'g> TopoView<'g> {$(
            #[doc = concat!("The grouped `", stringify!($ops), "` queries.")]
            pub fn $accessor(self) -> $ops<'g> {
                $ops { graph: self.graph }
            }
        )*}
        $(
            $(#[$doc])*
            #[derive(Clone, Copy, Debug)]
            pub struct $ops<'g> {
                graph: &'g BRepGraph,
            }

            impl<'g> $ops<'g> {
                /// `Nb()`: the definitions, soft-removed ones included.
                pub fn nb(self) -> u32 {
                    self.graph.inc_storage().$nb()
                }

                /// `NbActive()`: the definitions that are not soft-removed.
                pub fn nb_active(self) -> u32 {
                    self.graph.inc_storage().$nb_active()
                }

                /// `StartId()`: the first id.
                pub fn start_id(self) -> crate::brep_graph::$id {
                    crate::brep_graph::$id::start()
                }

                /// `EndId()`: one past the last id.
                pub fn end_id(self) -> crate::brep_graph::$id {
                    crate::brep_graph::$id::new(self.nb())
                }

                /// `Definition(theId)`: the definition record (an id out of range panics).
                pub fn definition(self, id: crate::brep_graph::$id) -> &'g crate::brep_graph_inc::$def {
                    self.graph.inc_storage().$get(id)
                }

                $(
                    /// `Relations(theId)`: the relation record.
                    pub fn relations(
                        self,
                        id: crate::brep_graph::$id,
                    ) -> &'g crate::brep_graph_inc::$relations_type {
                        self.graph.inc_storage().$relations(id)
                    }
                )?
            }

            impl CountProvider for $ops<'_> {
                fn count(&self) -> u32 {
                    self.nb()
                }
            }
        )*
    };
}

kind_ops! {
    /// `TopoView::FaceOps`.
    FaceOps, faces, FaceId, FaceDef, face, nb_faces, nb_active_faces,
        face_relations, FaceRelations;
    /// `TopoView::EdgeOps`.
    EdgeOps, edges, EdgeId, EdgeDef, edge, nb_edges, nb_active_edges,
        edge_relations, EdgeRelations;
    /// `TopoView::VertexOps`.
    VertexOps, vertices, VertexId, VertexDef, vertex, nb_vertices, nb_active_vertices,
        vertex_relations, VertexRelations;
    /// `TopoView::WireOps`.
    WireOps, wires, WireId, WireDef, wire, nb_wires, nb_active_wires,
        wire_relations, WireRelations;
    /// `TopoView::ShellOps`.
    ShellOps, shells, ShellId, ShellDef, shell, nb_shells, nb_active_shells,
        shell_relations, ShellRelations;
    /// `TopoView::SolidOps`.
    SolidOps, solids, SolidId, SolidDef, solid, nb_solids, nb_active_solids,
        solid_relations, SolidRelations;
    /// `TopoView::CoEdgeOps`.
    CoEdgeOps, co_edges, CoEdgeId, CoEdgeDef, co_edge, nb_co_edges, nb_active_co_edges;
    /// `TopoView::CompoundOps`.
    CompoundOps, compounds, CompoundId, CompoundDef, compound, nb_compounds,
        nb_active_compounds, compound_relations, CompoundRelations;
    /// `TopoView::CompSolidOps`.
    CompSolidOps, comp_solids, CompSolidId, CompSolidDef, comp_solid, nb_comp_solids,
        nb_active_comp_solids, comp_solid_relations, CompSolidRelations;
    /// `TopoView::ProductOps`.
    ProductOps, products, ProductId, ProductDef, product, nb_products, nb_active_products,
        product_relations, ProductRelations;
    /// `TopoView::OccurrenceOps`.
    OccurrenceOps, occurrences, OccurrenceId, OccurrenceDef, occurrence, nb_occurrences,
        nb_active_occurrences, occurrence_relations, OccurrenceRelations;
}

impl<'g> TopoView<'g> {
    /// `Gen()`: the generic node queries.
    pub fn generic(self) -> GenOps<'g> {
        GenOps { graph: self.graph }
    }

    /// `Geometry()`: the representation counts.
    pub fn geometry(self) -> GeometryOps<'g> {
        GeometryOps { graph: self.graph }
    }
}
