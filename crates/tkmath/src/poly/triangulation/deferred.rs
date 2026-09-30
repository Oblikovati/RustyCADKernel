// Rust translation of the late-load interface of FoundationClasses/TKMath/Poly/Poly_Triangulation.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The virtual late-load members of `Poly_Triangulation`.
//!
//! A native subclass (for example RWMesh_TriangulationSource) overrides `NbDeferredNodes`,
//! `NbDeferredTriangles`, `createNewEntity` and `loadDeferredData`. Here the subclass is a
//! [`DeferredTriangulation`] injected with `Triangulation::with_deferred`. OSD_FileSystem is
//! not ported, so the storage owns its input instead of taking a file system argument.
use super::{Triangulation, mesh_purpose};
use rustycad_tkernel::standard::Handle;
/// Deferred storage that can fill a triangulation later; `Triangulation::with_deferred(Box::new(storage))`.
pub trait DeferredTriangulation: Send + Sync {
    /// Estimated deferred node count; `storage.nb_deferred_nodes()`.
    fn nb_deferred_nodes(&self) -> i32 {
        0
    }
    /// Estimated deferred triangle count; `storage.nb_deferred_triangles()`.
    fn nb_deferred_triangles(&self) -> i32 {
        0
    }
    /// True when some data can be loaded; by default when triangles are pending.
    fn has_deferred_data(&self) -> bool {
        self.nb_deferred_triangles() > 0
    }
    /// Creates the object that `DetachedLoadDeferredData` fills; `storage.create_new_entity()`.
    fn create_new_entity(&self) -> Handle<Triangulation> {
        Handle::new(Triangulation::new())
    }
    /// Fills `destination` from the storage and reports success; `storage.load_deferred_data(&mesh)`.
    fn load_deferred_data(&self, destination: &Triangulation) -> bool;
}
impl Triangulation {
    /// Deferred node estimate, zero without storage; `triangulation.nb_deferred_nodes()`.
    pub fn nb_deferred_nodes(&self) -> i32 {
        self.deferred
            .as_ref()
            .map_or(0, |storage| storage.nb_deferred_nodes())
    }
    /// Deferred triangle estimate, zero without storage; `triangulation.nb_deferred_triangles()`.
    pub fn nb_deferred_triangles(&self) -> i32 {
        self.deferred
            .as_ref()
            .map_or(0, |storage| storage.nb_deferred_triangles())
    }
    /// True when the storage has something to load; `triangulation.has_deferred_data()`.
    pub fn has_deferred_data(&self) -> bool {
        self.deferred
            .as_ref()
            .is_some_and(|storage| storage.has_deferred_data())
    }
    /// Loads into this object and sets the `LOADED` purpose bit; `triangulation.load_deferred_data()`.
    pub fn load_deferred_data(&self) -> bool {
        let Some(storage) = self.deferred.as_ref().filter(|s| s.has_deferred_data()) else {
            return false;
        };
        if !storage.load_deferred_data(self) {
            return false;
        }
        self.set_mesh_purpose(self.mesh_purpose() | mesh_purpose::LOADED);
        true
    }
    /// Loads into a new object from `create_new_entity`, or returns a null handle;
    /// `triangulation.detached_load_deferred_data()`.
    pub fn detached_load_deferred_data(&self) -> Handle<Triangulation> {
        let Some(storage) = self.deferred.as_ref().filter(|s| s.has_deferred_data()) else {
            return Handle::default();
        };
        let result = storage.create_new_entity();
        if !storage.load_deferred_data(&result) {
            return Handle::default();
        }
        result.set_mesh_purpose(result.mesh_purpose() | mesh_purpose::LOADED);
        result
    }
    /// Clears the loaded data when a storage can reload it; `triangulation.unload_deferred_data()`.
    pub fn unload_deferred_data(&self) -> bool {
        if !self.has_deferred_data() {
            return false;
        }
        self.clear();
        self.set_mesh_purpose(self.mesh_purpose() & !mesh_purpose::LOADED);
        true
    }
}
