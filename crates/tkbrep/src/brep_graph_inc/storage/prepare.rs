// Rust translation of PrepareForLoad, SetActiveCounts, Counts and ActiveCounts of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Storage.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The indexed-load preparation: final slot counts in, fixed-size tables out.
use super::Storage;
use super::rebuild::prepare_relation_table;
use crate::brep_graph_inc::LoadCounts;

impl Storage {
    /// `PrepareForLoad(theCounts)`: clears a non-empty storage, then sizes every table to
    /// its final count with all slots active and the relation tables initialised once.
    pub fn prepare_for_load(&mut self, counts: &LoadCounts) {
        if !self.is_empty() {
            self.clear();
        }
        self.prepare_definition_stores(counts);
        self.prepare_relation_tables_for(counts);
        self.prepare_reference_stores(counts);
        self.prepare_representation_stores(counts);
    }

    fn prepare_definition_stores(&mut self, counts: &LoadCounts) {
        self.vertices.prepare(counts.nb_vertices);
        self.edges.prepare(counts.nb_edges);
        self.co_edges.prepare(counts.nb_co_edges);
        self.wires.prepare(counts.nb_wires);
        self.faces.prepare(counts.nb_faces);
        self.shells.prepare(counts.nb_shells);
        self.solids.prepare(counts.nb_solids);
        self.compounds.prepare(counts.nb_compounds);
        self.comp_solids.prepare(counts.nb_comp_solids);
        self.products.prepare(counts.nb_products);
        self.occurrences.prepare(counts.nb_occurrences);
    }

    fn prepare_relation_tables_for(&mut self, counts: &LoadCounts) {
        prepare_relation_table(&mut self.vertex_relations, counts.nb_vertices);
        prepare_relation_table(&mut self.edge_relations, counts.nb_edges);
        prepare_relation_table(&mut self.wire_relations, counts.nb_wires);
        prepare_relation_table(&mut self.face_relations, counts.nb_faces);
        prepare_relation_table(&mut self.shell_relations, counts.nb_shells);
        prepare_relation_table(&mut self.solid_relations, counts.nb_solids);
        prepare_relation_table(&mut self.compound_relations, counts.nb_compounds);
        prepare_relation_table(&mut self.comp_solid_relations, counts.nb_comp_solids);
        prepare_relation_table(&mut self.product_relations, counts.nb_products);
        prepare_relation_table(&mut self.occurrence_relations, counts.nb_occurrences);
    }

    fn prepare_reference_stores(&mut self, counts: &LoadCounts) {
        self.shell_refs.prepare(counts.nb_shell_refs);
        self.face_refs.prepare(counts.nb_face_refs);
        self.wire_refs.prepare(counts.nb_wire_refs);
        self.vertex_refs.prepare(counts.nb_vertex_refs);
        self.solid_refs.prepare(counts.nb_solid_refs);
        self.child_refs.prepare(counts.nb_child_refs);
        self.occurrence_refs.prepare(counts.nb_occurrence_refs);
    }

    fn prepare_representation_stores(&mut self, counts: &LoadCounts) {
        self.face_surfaces.prepare(counts.nb_face_surface_reps);
        self.edge_curves3d.prepare(counts.nb_edge_curve3d_reps);
        self.co_edge_curves2d
            .prepare(counts.nb_co_edge_curve2d_reps);
        self.face_triangulations
            .prepare(counts.nb_face_triangulation_reps);
        self.edge_polygons3d.prepare(counts.nb_edge_polygon3d_reps);
        self.co_edge_polygons2d
            .prepare(counts.nb_co_edge_polygon2d_reps);
        self.co_edge_polygons_on_tri
            .prepare(counts.nb_co_edge_polygon_on_tri_reps);
    }

    /// `Counts()`: the allocated slot counts (`NbRootProducts` stays 0 as natively).
    pub fn counts(&self) -> LoadCounts {
        LoadCounts {
            nb_vertices: self.nb_vertices(),
            nb_edges: self.nb_edges(),
            nb_co_edges: self.nb_co_edges(),
            nb_wires: self.nb_wires(),
            nb_faces: self.nb_faces(),
            nb_shells: self.nb_shells(),
            nb_solids: self.nb_solids(),
            nb_compounds: self.nb_compounds(),
            nb_comp_solids: self.nb_comp_solids(),
            nb_products: self.nb_products(),
            nb_occurrences: self.nb_occurrences(),
            nb_shell_refs: self.nb_shell_refs(),
            nb_face_refs: self.nb_face_refs(),
            nb_wire_refs: self.nb_wire_refs(),
            nb_vertex_refs: self.nb_vertex_refs(),
            nb_solid_refs: self.nb_solid_refs(),
            nb_child_refs: self.nb_child_refs(),
            nb_occurrence_refs: self.nb_occurrence_refs(),
            nb_face_surface_reps: self.nb_face_surfaces(),
            nb_edge_curve3d_reps: self.nb_edge_curves3d(),
            nb_co_edge_curve2d_reps: self.nb_co_edge_curves2d(),
            nb_face_triangulation_reps: self.nb_face_triangulations(),
            nb_edge_polygon3d_reps: self.nb_edge_polygons3d(),
            nb_co_edge_polygon2d_reps: self.nb_co_edge_polygons2d(),
            nb_co_edge_polygon_on_tri_reps: self.nb_co_edge_polygons_on_tri(),
            nb_root_products: 0,
        }
    }

    /// `ActiveCounts()`: the active slot counts.
    pub fn active_counts(&self) -> LoadCounts {
        LoadCounts {
            nb_vertices: self.nb_active_vertices(),
            nb_edges: self.nb_active_edges(),
            nb_co_edges: self.nb_active_co_edges(),
            nb_wires: self.nb_active_wires(),
            nb_faces: self.nb_active_faces(),
            nb_shells: self.nb_active_shells(),
            nb_solids: self.nb_active_solids(),
            nb_compounds: self.nb_active_compounds(),
            nb_comp_solids: self.nb_active_comp_solids(),
            nb_products: self.nb_active_products(),
            nb_occurrences: self.nb_active_occurrences(),
            nb_shell_refs: self.nb_active_shell_refs(),
            nb_face_refs: self.nb_active_face_refs(),
            nb_wire_refs: self.nb_active_wire_refs(),
            nb_vertex_refs: self.nb_active_vertex_refs(),
            nb_solid_refs: self.nb_active_solid_refs(),
            nb_child_refs: self.nb_active_child_refs(),
            nb_occurrence_refs: self.nb_active_occurrence_refs(),
            nb_face_surface_reps: self.nb_active_face_surfaces(),
            nb_edge_curve3d_reps: self.nb_active_edge_curves3d(),
            nb_co_edge_curve2d_reps: self.nb_active_co_edge_curves2d(),
            nb_face_triangulation_reps: self.nb_active_face_triangulations(),
            nb_edge_polygon3d_reps: self.nb_active_edge_polygons3d(),
            nb_co_edge_polygon2d_reps: self.nb_active_co_edge_polygons2d(),
            nb_co_edge_polygon_on_tri_reps: self.nb_active_co_edge_polygons_on_tri(),
            nb_root_products: 0,
        }
    }

    /// `SetActiveCounts(theCounts)`: trusted active counts of a load path.
    pub fn set_active_counts(&mut self, counts: &LoadCounts) {
        self.vertices.nb_active = counts.nb_vertices;
        self.edges.nb_active = counts.nb_edges;
        self.co_edges.nb_active = counts.nb_co_edges;
        self.wires.nb_active = counts.nb_wires;
        self.faces.nb_active = counts.nb_faces;
        self.shells.nb_active = counts.nb_shells;
        self.solids.nb_active = counts.nb_solids;
        self.compounds.nb_active = counts.nb_compounds;
        self.comp_solids.nb_active = counts.nb_comp_solids;
        self.products.nb_active = counts.nb_products;
        self.occurrences.nb_active = counts.nb_occurrences;
        self.set_active_reference_counts(counts);
    }

    fn set_active_reference_counts(&mut self, counts: &LoadCounts) {
        self.shell_refs.nb_active = counts.nb_shell_refs;
        self.face_refs.nb_active = counts.nb_face_refs;
        self.wire_refs.nb_active = counts.nb_wire_refs;
        self.vertex_refs.nb_active = counts.nb_vertex_refs;
        self.solid_refs.nb_active = counts.nb_solid_refs;
        self.child_refs.nb_active = counts.nb_child_refs;
        self.occurrence_refs.nb_active = counts.nb_occurrence_refs;
        self.face_surfaces.nb_active = counts.nb_face_surface_reps;
        self.edge_curves3d.nb_active = counts.nb_edge_curve3d_reps;
        self.co_edge_curves2d.nb_active = counts.nb_co_edge_curve2d_reps;
        self.face_triangulations.nb_active = counts.nb_face_triangulation_reps;
        self.edge_polygons3d.nb_active = counts.nb_edge_polygon3d_reps;
        self.co_edge_polygons2d.nb_active = counts.nb_co_edge_polygon2d_reps;
        self.co_edge_polygons_on_tri.nb_active = counts.nb_co_edge_polygon_on_tri_reps;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_sizes_every_table() {
        let mut storage = Storage::new();
        storage.append_vertex();
        let counts = LoadCounts {
            nb_faces: 2,
            nb_wire_refs: 3,
            nb_face_surface_reps: 1,
            ..LoadCounts::default()
        };
        storage.prepare_for_load(&counts);
        assert_eq!(storage.counts(), counts);
        assert_eq!(storage.active_counts(), counts);
        assert_eq!(
            storage
                .face_relations(crate::brep_graph::FaceId::new(1))
                .wire_ref_ids
                .len(),
            0
        );
        let mut active = counts;
        active.nb_faces = 1;
        storage.set_active_counts(&active);
        assert_eq!(storage.nb_active_faces(), 1);
    }
}
