// Rust translation of the late-load interface of FoundationClasses/TKMath/Poly/Poly_Triangulation.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Native virtual extension points of `Poly_Triangulation`.
//!
//! A native subclass (for example RWMesh_TriangulationSource) overrides deferred methods and
//! may override `computeBoundingBox`. Rust supplies that behavior through
//! [`TriangulationHooks`] and `Triangulation::with_hooks`; the optional filesystem handle is
//! forwarded unchanged, including native null/default arguments. `DeferredTriangulation` stays
//! as a compatibility alias for existing implementors.
use super::{Triangulation, mesh_purpose};
use crate::{bnd::BoundingBox3, gp::Transform3};
use rustycad_tkernel::{osd::FileSystemHandle, standard::Handle};
/// Rust override hooks for native `Poly_Triangulation` virtual members.
/// `Triangulation::with_hooks(Box::new(storage))`.
pub trait TriangulationHooks: Send + Sync {
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
    /// Overrides protected virtual `computeBoundingBox`; return `None` to call the base implementation.
    /// `Some(self.compute_bounding_box_base(mesh, transform))` explicitly delegates to the native base.
    fn compute_bounding_box(
        &self,
        _base: &Triangulation,
        _transform: Transform3,
    ) -> Option<BoundingBox3> {
        None
    }
    /// Creates the object that `DetachedLoadDeferredData` fills; `storage.create_new_entity()`.
    fn create_new_entity(&self) -> Handle<Triangulation> {
        Handle::new(Triangulation::new())
    }
    /// Fills `destination` from the storage and reports success; `storage.load_deferred_data(&mesh)`.
    fn load_deferred_data(&self, _destination: &Triangulation) -> bool {
        false
    }
    /// Loads with the caller's optional filesystem; `storage.load_deferred_data_with_file_system(None, &mesh)`.
    /// The fallback preserves existing storage implementations that own their input.
    fn load_deferred_data_with_file_system(
        &self,
        _file_system: Option<&FileSystemHandle>,
        destination: &Triangulation,
    ) -> bool {
        self.load_deferred_data(destination)
    }
}
/// Backwards-compatible name for implementations that only supply deferred data hooks.
pub use TriangulationHooks as DeferredTriangulation;
impl Triangulation {
    /// Deferred node estimate, zero without storage; `triangulation.nb_deferred_nodes()`.
    pub fn nb_deferred_nodes(&self) -> i32 {
        self.hooks
            .as_ref()
            .map_or(0, |storage| storage.nb_deferred_nodes())
    }
    /// Deferred triangle estimate, zero without storage; `triangulation.nb_deferred_triangles()`.
    pub fn nb_deferred_triangles(&self) -> i32 {
        self.hooks
            .as_ref()
            .map_or(0, |storage| storage.nb_deferred_triangles())
    }
    /// True when the storage has something to load; `triangulation.has_deferred_data()`.
    pub fn has_deferred_data(&self) -> bool {
        self.hooks
            .as_ref()
            .is_some_and(|storage| storage.has_deferred_data())
    }
    /// Loads into this object and sets the `LOADED` purpose bit; `triangulation.load_deferred_data()`.
    pub fn load_deferred_data(&self) -> bool {
        self.load_deferred_data_with_file_system(None)
    }
    /// Forwards the filesystem unchanged and marks successful loads; `mesh.load_deferred_data_with_file_system(Some(&fs))`.
    pub fn load_deferred_data_with_file_system(
        &self,
        file_system: Option<&FileSystemHandle>,
    ) -> bool {
        let Some(storage) = self.hooks.as_ref().filter(|s| s.has_deferred_data()) else {
            return false;
        };
        if !storage.load_deferred_data_with_file_system(file_system, self) {
            return false;
        }
        self.set_mesh_purpose(self.mesh_purpose() | mesh_purpose::LOADED);
        true
    }
    /// Loads into a new object from `create_new_entity`, or returns a null handle;
    /// `triangulation.detached_load_deferred_data()`.
    pub fn detached_load_deferred_data(&self) -> Handle<Triangulation> {
        self.detached_load_deferred_data_with_file_system(None)
    }
    /// Forwards the filesystem to the detached load; `mesh.detached_load_deferred_data_with_file_system(Some(&fs))`.
    pub fn detached_load_deferred_data_with_file_system(
        &self,
        file_system: Option<&FileSystemHandle>,
    ) -> Handle<Triangulation> {
        let Some(storage) = self.hooks.as_ref().filter(|s| s.has_deferred_data()) else {
            return Handle::default();
        };
        let result = storage.create_new_entity();
        if !storage.load_deferred_data_with_file_system(file_system, &result) {
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
