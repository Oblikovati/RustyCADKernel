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
// Rust translation of the record accessors of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Storage.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The typed record accessors (`Vertex`, `ChangeVertex`, `AppendVertex`, `NbVertices`,
//! `NbActiveVertices`...) and the relation-table accessors.
//!
//! The record accessors index the store without a check natively; an index out of range
//! panics here with the index.
use super::Storage;
use super::tags::{NodeTag, RefTag, RepTag, StoredTag};
use crate::brep_graph::*;
use crate::brep_graph_inc::definition::*;
use crate::brep_graph_inc::reference::*;
use crate::brep_graph_inc::relations::*;
use crate::brep_graph_inc::representation::*;

impl Storage {
    /// The definition of any node kind (the generic body of `Vertex()`, `Edge()`...).
    pub fn def<T: NodeTag>(&self, id: TypedId<T>) -> &T::Def {
        &T::store(self).items[id.index as usize]
    }

    /// The mutable definition of any node kind (`ChangeVertex()`...).
    pub fn change_def<T: NodeTag>(&mut self, id: TypedId<T>) -> &mut T::Def {
        &mut T::store_mut(self).items[id.index as usize]
    }

    /// The reference entry of any reference kind (`ShellRef()`...).
    pub fn entry<T: RefTag>(&self, id: TypedId<T>) -> &T::Entry {
        &T::store(self).items[id.index as usize]
    }

    /// The mutable reference entry of any reference kind (`ChangeShellRef()`...).
    pub fn change_entry<T: RefTag>(&mut self, id: TypedId<T>) -> &mut T::Entry {
        &mut T::store_mut(self).items[id.index as usize]
    }

    /// The use record of any representation kind (`EdgeCurve3DRep()`...).
    pub fn record<T: RepTag>(&self, id: TypedId<T>) -> &T::Record {
        &T::store(self).items[id.index as usize]
    }

    /// The mutable use record of any representation kind (`ChangeEdgeCurve3DRep()`...).
    pub fn change_record<T: RepTag>(&mut self, id: TypedId<T>) -> &mut T::Record {
        &mut T::store_mut(self).items[id.index as usize]
    }

    /// The slot count of any stored kind (`NbVertices()`, `NbShellRefs()`...).
    pub fn nb<T: StoredTag>(&self) -> u32 {
        use super::stores::RemovalPlanes;
        T::store(self).count()
    }

    /// `NbActiveEdgeCurves3D()`.
    pub fn nb_active_edge_curves3d(&self) -> u32 {
        self.edge_curves3d.nb_active
    }

    /// `NbActiveCoEdgeCurves2D()`.
    pub fn nb_active_co_edge_curves2d(&self) -> u32 {
        self.co_edge_curves2d.nb_active
    }

    /// `NbActiveFaceSurfaces()`.
    pub fn nb_active_face_surfaces(&self) -> u32 {
        self.face_surfaces.nb_active
    }

    /// `NbActiveFaceTriangulations()`.
    pub fn nb_active_face_triangulations(&self) -> u32 {
        self.face_triangulations.nb_active
    }

    /// `NbActiveEdgePolygons3D()`.
    pub fn nb_active_edge_polygons3d(&self) -> u32 {
        self.edge_polygons3d.nb_active
    }

    /// `NbActiveCoEdgePolygons2D()`.
    pub fn nb_active_co_edge_polygons2d(&self) -> u32 {
        self.co_edge_polygons2d.nb_active
    }

    /// `NbActiveCoEdgePolygonsOnTri()`.
    pub fn nb_active_co_edge_polygons_on_tri(&self) -> u32 {
        self.co_edge_polygons_on_tri.nb_active
    }
}

macro_rules! def_accessors {
    ($($tag:ty, $id:ty, $def:ty, $store:ident, $get:ident, $change:ident, $nb:ident,
       $nb_active:ident, $append:ident $(, $relations:ident)?;)*) => {
        impl Storage {$(
            #[doc = concat!("`", stringify!($get), "`: the definition at the typed id.")]
            pub fn $get(&self, id: $id) -> &$def {
                self.def(id)
            }
            #[doc = concat!("The mutable definition at the typed id (`Change` accessor).")]
            pub fn $change(&mut self, id: $id) -> &mut $def {
                self.change_def(id)
            }
            #[doc = "The slot count, removed slots included."]
            pub fn $nb(&self) -> u32 {
                self.$store.nb()
            }
            #[doc = "The active (not removed) slot count."]
            pub fn $nb_active(&self) -> u32 {
                self.$store.nb_active
            }
            #[doc = "Appends a default definition (and its relation slot) and returns its id."]
            pub fn $append(&mut self) -> $id {
                let id = <$id>::new(self.$store.append());
                $(self.$relations.push(Default::default());)?
                id
            }
        )*}
    };
}

def_accessors! {
    VertexTag, VertexId, VertexDef, vertices, vertex, change_vertex, nb_vertices,
        nb_active_vertices, append_vertex, vertex_relations;
    EdgeTag, EdgeId, EdgeDef, edges, edge, change_edge, nb_edges, nb_active_edges,
        append_edge, edge_relations;
    CoEdgeTag, CoEdgeId, CoEdgeDef, co_edges, co_edge, change_co_edge, nb_co_edges,
        nb_active_co_edges, append_co_edge;
    WireTag, WireId, WireDef, wires, wire, change_wire, nb_wires, nb_active_wires,
        append_wire, wire_relations;
    FaceTag, FaceId, FaceDef, faces, face, change_face, nb_faces, nb_active_faces,
        append_face, face_relations;
    ShellTag, ShellId, ShellDef, shells, shell, change_shell, nb_shells, nb_active_shells,
        append_shell, shell_relations;
    SolidTag, SolidId, SolidDef, solids, solid, change_solid, nb_solids, nb_active_solids,
        append_solid, solid_relations;
    CompoundTag, CompoundId, CompoundDef, compounds, compound, change_compound, nb_compounds,
        nb_active_compounds, append_compound, compound_relations;
    CompSolidTag, CompSolidId, CompSolidDef, comp_solids, comp_solid, change_comp_solid,
        nb_comp_solids, nb_active_comp_solids, append_comp_solid, comp_solid_relations;
    ProductTag, ProductId, ProductDef, products, product, change_product, nb_products,
        nb_active_products, append_product, product_relations;
    OccurrenceTag, OccurrenceId, OccurrenceDef, occurrences, occurrence, change_occurrence,
        nb_occurrences, nb_active_occurrences, append_occurrence, occurrence_relations;
}

macro_rules! ref_accessors {
    ($($id:ty, $entry:ty, $store:ident, $get:ident, $change:ident, $nb:ident,
       $nb_active:ident, $append:ident;)*) => {
        impl Storage {$(
            #[doc = "The reference entry at the typed id."]
            pub fn $get(&self, id: $id) -> &$entry {
                self.entry(id)
            }
            #[doc = "The mutable reference entry at the typed id."]
            pub fn $change(&mut self, id: $id) -> &mut $entry {
                self.change_entry(id)
            }
            #[doc = "The entry count, removed entries included."]
            pub fn $nb(&self) -> u32 {
                self.$store.nb()
            }
            #[doc = "The active (not removed) entry count."]
            pub fn $nb_active(&self) -> u32 {
                self.$store.nb_active
            }
            #[doc = "Appends a default reference entry and returns its id."]
            pub fn $append(&mut self) -> $id {
                <$id>::new(self.$store.append())
            }
        )*}
    };
}

ref_accessors! {
    ShellRefId, ShellRef, shell_refs, shell_ref, change_shell_ref, nb_shell_refs,
        nb_active_shell_refs, append_shell_ref;
    FaceRefId, FaceRef, face_refs, face_ref, change_face_ref, nb_face_refs,
        nb_active_face_refs, append_face_ref;
    WireRefId, WireRef, wire_refs, wire_ref, change_wire_ref, nb_wire_refs,
        nb_active_wire_refs, append_wire_ref;
    VertexRefId, VertexRef, vertex_refs, vertex_ref, change_vertex_ref, nb_vertex_refs,
        nb_active_vertex_refs, append_vertex_ref;
    SolidRefId, SolidRef, solid_refs, solid_ref, change_solid_ref, nb_solid_refs,
        nb_active_solid_refs, append_solid_ref;
    ChildRefId, ChildRef, child_refs, child_ref, change_child_ref, nb_child_refs,
        nb_active_child_refs, append_child_ref;
    OccurrenceRefId, OccurrenceRef, occurrence_refs, occurrence_ref, change_occurrence_ref,
        nb_occurrence_refs, nb_active_occurrence_refs, append_occurrence_ref;
}

macro_rules! rep_accessors {
    ($($id:ty, $record:ty, $store:ident, $get:ident, $change:ident, $nb:ident, $append:ident;)*) => {
        impl Storage {$(
            #[doc = "The use record at the typed id."]
            pub fn $get(&self, id: $id) -> &$record {
                self.record(id)
            }
            #[doc = "The mutable use record at the typed id."]
            pub fn $change(&mut self, id: $id) -> &mut $record {
                self.change_record(id)
            }
            #[doc = "The record count, removed records included."]
            pub fn $nb(&self) -> u32 {
                self.$store.nb()
            }
            #[doc = "Appends a default use record and returns its id."]
            pub fn $append(&mut self) -> $id {
                <$id>::new(self.$store.append())
            }
        )*}
    };
}

rep_accessors! {
    EdgeCurve3DRepId, EdgeCurve3DRep, edge_curves3d, edge_curve3d_rep,
        change_edge_curve3d_rep, nb_edge_curves3d, append_edge_curve3d_rep;
    EdgePolygon3DRepId, EdgePolygon3DRep, edge_polygons3d, edge_polygon3d_rep,
        change_edge_polygon3d_rep, nb_edge_polygons3d, append_edge_polygon3d_rep;
    CoEdgeCurve2DRepId, CoEdgeCurve2DRep, co_edge_curves2d, co_edge_curve2d_rep,
        change_co_edge_curve2d_rep, nb_co_edge_curves2d, append_co_edge_curve2d_rep;
    CoEdgePolygon2DRepId, CoEdgePolygon2DRep, co_edge_polygons2d, co_edge_polygon2d_rep,
        change_co_edge_polygon2d_rep, nb_co_edge_polygons2d, append_co_edge_polygon2d_rep;
    CoEdgePolygonOnTriRepId, CoEdgePolygonOnTriRep, co_edge_polygons_on_tri,
        co_edge_polygon_on_tri_rep, change_co_edge_polygon_on_tri_rep,
        nb_co_edge_polygons_on_tri, append_co_edge_polygon_on_tri_rep;
    FaceSurfaceRepId, FaceSurfaceRep, face_surfaces, face_surface_rep,
        change_face_surface_rep, nb_face_surfaces, append_face_surface_rep;
    FaceTriangulationRepId, FaceTriangulationRep, face_triangulations,
        face_triangulation_rep, change_face_triangulation_rep, nb_face_triangulations,
        append_face_triangulation_rep;
}

macro_rules! relation_accessors {
    ($($id:ty, $relations:ty, $table:ident, $get:ident, $change:ident;)*) => {
        impl Storage {$(
            #[doc = "The relation record of the typed id (an index past the table panics)."]
            pub fn $get(&self, id: $id) -> &$relations {
                &self.$table[id.index as usize]
            }
            #[doc = "The mutable relation record (`Change...RelationsInternal`)."]
            pub(crate) fn $change(&mut self, id: $id) -> &mut $relations {
                &mut self.$table[id.index as usize]
            }
        )*}
    };
}

relation_accessors! {
    FaceId, FaceRelations, face_relations, face_relations, change_face_relations_internal;
    WireId, WireRelations, wire_relations, wire_relations, change_wire_relations_internal;
    EdgeId, EdgeRelations, edge_relations, edge_relations, change_edge_relations_internal;
    ShellId, ShellRelations, shell_relations, shell_relations, change_shell_relations_internal;
    SolidId, SolidRelations, solid_relations, solid_relations, change_solid_relations_internal;
    CompoundId, CompoundRelations, compound_relations, compound_relations,
        change_compound_relations_internal;
    CompSolidId, CompSolidRelations, comp_solid_relations, comp_solid_relations,
        change_comp_solid_relations_internal;
    VertexId, VertexRelations, vertex_relations, vertex_relations,
        change_vertex_relations_internal;
    ProductId, ProductRelations, product_relations, product_relations,
        change_product_relations_internal;
    OccurrenceId, OccurrenceRelations, occurrence_relations, occurrence_relations,
        change_occurrence_relations_internal;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_grow_records_and_relations() {
        let mut storage = Storage::new();
        let edge = storage.append_edge();
        storage.change_edge(edge).tolerance = 0.5;
        assert_eq!(storage.edge(edge).tolerance, 0.5);
        assert!(storage.edge_relations(edge).co_edge_ids.is_empty());
        let co_edge = storage.append_co_edge();
        assert_eq!(
            (storage.nb_co_edges(), storage.nb_active_co_edges()),
            (1, 1)
        );
        storage.change_co_edge(co_edge).child_edge_id = edge;
        assert_eq!(storage.co_edge(co_edge).child_edge_id, edge);
        let reference = storage.append_wire_ref();
        storage.change_wire_ref(reference).parent_face_id = FaceId::new(2);
        assert_eq!(storage.wire_ref(reference).parent_face_id, FaceId::new(2));
        let curve = storage.append_edge_curve3d_rep();
        storage.change_edge_curve3d_rep(curve).param_last = 3.0;
        assert_eq!(storage.edge_curve3d_rep(curve).param_last, 3.0);
        assert_eq!(storage.nb::<EdgeCurve3DTag>(), 1);
        assert_eq!(storage.nb_active_edge_curves3d(), 1);
    }
}
