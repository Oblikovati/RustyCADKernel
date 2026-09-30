// Rust translation of CopyDerivedRelationsFrom, CopyRemovedFlagsFrom and
// RecountActiveCounts of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Storage.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The identity-copy helpers of the storage (relations, parent bits, removed planes)
//! and the active-count recount.
use super::Storage;

impl Storage {
    /// `CopyDerivedRelationsFrom(theSource)`: the relation lists, the sparse maps and the
    /// parent bits of an identity copy with the same slot counts.
    pub fn copy_derived_relations_from(&mut self, source: &Storage) {
        self.prepare_relation_tables();
        copy_prefix(&mut self.vertex_relations, &source.vertex_relations);
        copy_prefix(&mut self.edge_relations, &source.edge_relations);
        copy_prefix(&mut self.wire_relations, &source.wire_relations);
        copy_prefix(&mut self.face_relations, &source.face_relations);
        copy_prefix(&mut self.shell_relations, &source.shell_relations);
        copy_prefix(&mut self.solid_relations, &source.solid_relations);
        copy_prefix(&mut self.comp_solid_relations, &source.comp_solid_relations);
        copy_prefix(&mut self.compound_relations, &source.compound_relations);
        copy_prefix(&mut self.product_relations, &source.product_relations);
        copy_prefix(&mut self.occurrence_relations, &source.occurrence_relations);
        self.node_to_compounds = source.node_to_compounds.clone();
        self.node_to_occurrences = source.node_to_occurrences.clone();
        self.copy_parent_flags_from(source);
    }

    fn copy_parent_flags_from(&mut self, source: &Storage) {
        macro_rules! copy {
            ($($store:ident),*) => {$(
                self.$store.has_compound_parent = source.$store.has_compound_parent.clone();
                self.$store.has_occurrence_parent = source.$store.has_occurrence_parent.clone();
            )*};
        }
        copy!(
            vertices,
            edges,
            co_edges,
            wires,
            faces,
            shells,
            solids,
            compounds,
            comp_solids,
            products,
            occurrences
        );
    }

    /// `CopyRemovedFlagsFrom(theSource)`: the removed planes of every definition and
    /// reference store.
    pub fn copy_removed_flags_from(&mut self, source: &Storage) {
        macro_rules! copy {
            ($($store:ident),*) => {$(
                self.$store.removed = source.$store.removed.clone();
            )*};
        }
        copy!(
            vertices,
            edges,
            co_edges,
            wires,
            faces,
            shells,
            solids,
            compounds,
            comp_solids,
            products,
            occurrences,
            shell_refs,
            face_refs,
            wire_refs,
            vertex_refs,
            solid_refs,
            child_refs,
            occurrence_refs
        );
    }

    /// `RecountActiveCounts()`: every active count from the removed planes.
    pub fn recount_active_counts(&mut self) {
        macro_rules! recount {
            ($($store:ident),*) => {$(
                self.$store.recount_active();
            )*};
        }
        recount!(
            vertices,
            edges,
            co_edges,
            wires,
            faces,
            shells,
            solids,
            compounds,
            comp_solids,
            products,
            occurrences,
            shell_refs,
            face_refs,
            wire_refs,
            vertex_refs,
            solid_refs,
            child_refs,
            occurrence_refs,
            face_surfaces,
            edge_curves3d,
            co_edge_curves2d,
            face_triangulations,
            edge_polygons3d,
            co_edge_polygons2d,
            co_edge_polygons_on_tri
        );
    }
}

// The slots that exist in both tables take the source's value (the loops over the
// destination count with `IsValid(theSource.Nb...)` natively).
fn copy_prefix<R: Clone>(target: &mut [R], source: &[R]) {
    let count = target.len().min(source.len());
    target[..count].clone_from_slice(&source[..count]);
}
