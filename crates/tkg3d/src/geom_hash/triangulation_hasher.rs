// Rust translation of ModelingData/TKG3d/GeomHash/GeomHash_TriangulationHasher.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomHash_TriangulationHasher`: a triangulation hashed by its counts, UV presence,
//! deflection, end nodes, end triangles and end UV nodes, and compared node by node,
//! triangle by triangle and UV node by UV node.
use super::polygon_hashers::{none_beyond, null_decides, write_rounded, xy_of, xyz_of};
use rustycad_tkernel::ncollection::HashPolicy;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_hash::hash_key::{combined_hash, int_hash, rounded_hash};
use rustycad_tkg2d::hasher_tolerances;
use rustycad_tkmath::poly::Triangulation;

hasher_tolerances!(
    /// `GeomHash_TriangulationHasher`: hashes and compares `Poly_Triangulation` handles;
    /// a null handle hashes to 0; `GeomHashTriangulationHasher::default().hash(&mesh)`.
    GeomHashTriangulationHasher,
    precision::computational(),
    precision::computational()
);

impl GeomHashTriangulationHasher {
    /// The 20-slot native hash (`operator()(const occ::handle<Poly_Triangulation>&)`);
    /// `hasher.hash(&triangulation)`.
    pub fn hash(&self, triangulation: &Handle<Triangulation>) -> usize {
        let Some(mesh) = triangulation.get() else {
            return 0;
        };
        let mut slots = [0usize; 20];
        // Indices 1 and NbNodes()/NbTriangles() always exist when the counts are
        // positive, so the accessors cannot fail here.
        self.fill_slots(mesh, &mut slots).unwrap_or(());
        combined_hash(&slots)
    }
    fn fill_slots(&self, mesh: &Triangulation, slots: &mut [usize; 20]) -> Result<(), Failure> {
        let factor: f64 = 1.0 / self.hash_tolerance;
        let (nodes, triangles) = (mesh.nb_nodes(), mesh.nb_triangles());
        slots[0] = int_hash(nodes);
        slots[1] = int_hash(triangles);
        slots[2] = int_hash(i32::from(mesh.has_uv_nodes()));
        slots[3] = rounded_hash(mesh.deflection(), factor);
        if nodes > 0 {
            write_rounded(&mut slots[4..7], &xyz_of(mesh.node(1)?), factor);
            write_rounded(&mut slots[7..10], &xyz_of(mesh.node(nodes)?), factor);
        }
        if triangles > 0 {
            let ends = [mesh.triangle(1)?.get(), mesh.triangle(triangles)?.get()];
            for (offset, (n1, n2, n3)) in [10, 13].into_iter().zip(ends) {
                slots[offset..offset + 3].copy_from_slice(&[n1, n2, n3].map(int_hash));
            }
        }
        if mesh.has_uv_nodes() && nodes > 0 {
            write_rounded(&mut slots[16..18], &xy_of(mesh.uv_node(1)?), factor);
            write_rounded(&mut slots[18..20], &xy_of(mesh.uv_node(nodes)?), factor);
        }
        Ok(())
    }

    /// Same counts, UV presence and deflection, then every node, triangle and UV node
    /// (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Handle<Triangulation>, second: &Handle<Triangulation>) -> bool {
        if let Some(decided) = null_decides(first, second) {
            return decided;
        }
        let (Some(a), Some(b)) = (first.get(), second.get()) else {
            return false;
        };
        if a.nb_nodes() != b.nb_nodes()
            || a.nb_triangles() != b.nb_triangles()
            || a.has_uv_nodes() != b.has_uv_nodes()
        {
            return false;
        }
        if (a.deflection() - b.deflection()).abs() > self.comp_tolerance {
            return false;
        }
        self.same_mesh(a, b).unwrap_or(false)
    }
    fn same_mesh(&self, first: &Triangulation, second: &Triangulation) -> Result<bool, Failure> {
        let comp: f64 = self.comp_tolerance;
        for index in 1..=first.nb_nodes() {
            let (p1, p2) = (xyz_of(first.node(index)?), xyz_of(second.node(index)?));
            if !none_beyond(comp, &p1, &p2) {
                return Ok(false);
            }
        }
        for index in 1..=first.nb_triangles() {
            if first.triangle(index)?.get() != second.triangle(index)?.get() {
                return Ok(false);
            }
        }
        if !first.has_uv_nodes() {
            return Ok(true);
        }
        for index in 1..=first.nb_nodes() {
            let (u1, u2) = (xy_of(first.uv_node(index)?), xy_of(second.uv_node(index)?));
            if !none_beyond(comp, &u1, &u2) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl HashPolicy<Handle<Triangulation>> for GeomHashTriangulationHasher {
    fn hash(&self, key: &Handle<Triangulation>) -> usize {
        GeomHashTriangulationHasher::hash(self, key)
    }
    fn equal(&self, first: &Handle<Triangulation>, second: &Handle<Triangulation>) -> bool {
        GeomHashTriangulationHasher::equal(self, first, second)
    }
}
