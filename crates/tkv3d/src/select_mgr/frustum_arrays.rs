//! Fixed-shape access to the native protected frustum arrays.
use super::SelectMgrFrustumCore;
use rustycad_tkmath::gp::{Point3, Xyz};
impl<const N: usize> Default for SelectMgrFrustumCore<N> {
    /// Zero native projection caches; `SelectMgrFrustumCore::<4>::default()`.
    fn default() -> Self {
        Self::new()
    }
}
impl<const N: usize> SelectMgrFrustumCore<N> {
    /// Borrow the native `planes` array; `core.planes()`.
    pub fn planes(&self) -> &[Xyz] {
        &self.planes
    }
    /// Mutate entries without changing array dimensions; `core.planes_mut()[0] = value`.
    pub fn planes_mut(&mut self) -> &mut [Xyz] {
        &mut self.planes
    }
    /// Borrow the native `vertices` array; `core.vertices()`.
    pub fn vertices(&self) -> &[Point3] {
        &self.vertices
    }
    /// Mutate entries without changing array dimensions; `core.vertices_mut()[0] = value`.
    pub fn vertices_mut(&mut self) -> &mut [Point3] {
        &mut self.vertices
    }
    /// Borrow the native `max_verts_projections` array; `core.max_verts_projections()`.
    pub fn max_verts_projections(&self) -> &[f64] {
        &self.max_verts_projections
    }
    /// Mutate entries without changing array dimensions; `core.max_verts_projections_mut()[0] = value`.
    pub fn max_verts_projections_mut(&mut self) -> &mut [f64] {
        &mut self.max_verts_projections
    }
    /// Borrow the native `min_verts_projections` array; `core.min_verts_projections()`.
    pub fn min_verts_projections(&self) -> &[f64] {
        &self.min_verts_projections
    }
    /// Mutate entries without changing array dimensions; `core.min_verts_projections_mut()[0] = value`.
    pub fn min_verts_projections_mut(&mut self) -> &mut [f64] {
        &mut self.min_verts_projections
    }
    /// Borrow the native `max_ortho_verts_projections` array; `core.max_ortho_verts_projections()`.
    pub fn max_ortho_verts_projections(&self) -> &[f64] {
        &self.max_ortho_verts_projections
    }
    /// Mutate entries without changing array dimensions; `core.max_ortho_verts_projections_mut()[0] = value`.
    pub fn max_ortho_verts_projections_mut(&mut self) -> &mut [f64] {
        &mut self.max_ortho_verts_projections
    }
    /// Borrow the native `min_ortho_verts_projections` array; `core.min_ortho_verts_projections()`.
    pub fn min_ortho_verts_projections(&self) -> &[f64] {
        &self.min_ortho_verts_projections
    }
    /// Mutate entries without changing array dimensions; `core.min_ortho_verts_projections_mut()[0] = value`.
    pub fn min_ortho_verts_projections_mut(&mut self) -> &mut [f64] {
        &mut self.min_ortho_verts_projections
    }
    /// Borrow the native `edge_dirs` array; `core.edge_dirs()`.
    pub fn edge_dirs(&self) -> &[Xyz] {
        &self.edge_dirs
    }
    /// Mutate entries without changing array dimensions; `core.edge_dirs_mut()[0] = value`.
    pub fn edge_dirs_mut(&mut self) -> &mut [Xyz] {
        &mut self.edge_dirs
    }
}
