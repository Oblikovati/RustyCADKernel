// Rust translation of FoundationClasses/TKMath/BVH/BVH_Object.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_ObjectTransient` state and the abstract `BVH_Object<T, N>`.
use super::bounding_box::BvhBox;
use super::properties::BvhProperties;
use super::triangulation::BvhTriangulation;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;
use std::sync::Arc;

/// Data of `BVH_ObjectTransient`: the dirty flag (initially false) and the
/// generic properties handle (initially null).
/// Example: `BvhObjectTransient::default().is_dirty == false`.
#[derive(Clone, Debug, Default)]
pub struct BvhObjectTransient {
    /// Marks internal object state as outdated (`myIsDirty`).
    pub is_dirty: bool,
    /// Generic properties assigned to the object (`myProperties`).
    pub properties: Option<Arc<dyn BvhProperties>>,
}

/// Abstract geometric object bounded by a BVH box.
///
/// The accessors are virtual upstream, so they are overridable defaults here.
/// Example: `object.mark_dirty(); assert!(object.is_dirty());`.
pub trait BvhObject<V: BvhVector> {
    /// Borrows the `BVH_ObjectTransient` data.
    fn object_transient(&self) -> &BvhObjectTransient;
    /// Mutably borrows the `BVH_ObjectTransient` data.
    fn object_transient_mut(&mut self) -> &mut BvhObjectTransient;
    /// Returns AABB of the geometric object, as the pure virtual `Box()`.
    fn object_box(&self) -> Result<BvhBox<V>, Failure>;
    /// Returns properties of the geometric object; `object.properties()`.
    fn properties(&self) -> Option<&Arc<dyn BvhProperties>> {
        self.object_transient().properties.as_ref()
    }
    /// Sets properties of the geometric object; `object.set_properties(Some(p))`.
    fn set_properties(&mut self, properties: Option<Arc<dyn BvhProperties>>) {
        self.object_transient_mut().properties = properties;
    }
    /// Returns TRUE if object state should be updated; `object.is_dirty()`.
    fn is_dirty(&self) -> bool {
        self.object_transient().is_dirty
    }
    /// Marks object state as outdated (needs BVH rebuilding); `object.mark_dirty()`.
    fn mark_dirty(&mut self) {
        self.object_transient_mut().is_dirty = true;
    }
    /// `dynamic_cast<BVH_Triangulation<T, N>*>`: the triangulation this object is,
    /// or derives from; `None` for other objects. A type that embeds a triangulation
    /// (a C++ subclass) returns it. Example: `object.triangulation_mut().is_some()`.
    fn triangulation_mut(&mut self) -> Option<&mut BvhTriangulation<V>> {
        None
    }
}
