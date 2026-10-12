//! Each native sequence node owns one distinct handle field, independent of alias storage leases.
use super::Graphic3dClipPlaneSource;
use rustycad_tkernel::standard::{
    Failure, HandleObserver, HandleReference, IntrusiveReferenceSlot, raise_native_failure,
};
use std::sync::Arc;

/// Actual protected clipping-node payload; `Graphic3dClipPlaneHandleField::from_reference(&plane)?`.
/// Cloning copies native handle ownership into a new field, rather than sharing its pointer slot.
pub struct Graphic3dClipPlaneHandleField {
    slot: Arc<IntrusiveReferenceSlot<dyn Graphic3dClipPlaneSource>>,
}
impl Default for Graphic3dClipPlaneHandleField {
    fn default() -> Self {
        Self {
            slot: Arc::new(IntrusiveReferenceSlot::default()),
        }
    }
}
impl Graphic3dClipPlaneHandleField {
    /// Construct after node allocation, rereading the caller's slot; `Self::from_reference(&plane)?`.
    pub fn from_reference(
        source: &dyn HandleReference<dyn Graphic3dClipPlaneSource>,
    ) -> Result<Self, Failure> {
        let field = Self::default();
        field.assign(source)?;
        Ok(field)
    }
    /// Replace the actual protected node field; `field.assign(&replacement)?`.
    pub fn assign(
        &self,
        source: &dyn HandleReference<dyn Graphic3dClipPlaneSource>,
    ) -> Result<(), Failure> {
        // Native handle Assign exposes its old pointer throughout the old pointee's destructor.
        self.slot.assign(source)
    }
    /// Keep field bytes without owning the plane; `field.storage_lease().object_address()`.
    /// The node still releases its native reference when this payload is destroyed.
    pub fn storage_lease(&self) -> Arc<IntrusiveReferenceSlot<dyn Graphic3dClipPlaneSource>> {
        // An alias retains field bytes only: the node's destructor still ends its native ownership.
        self.slot.clone()
    }
}
impl Clone for Graphic3dClipPlaneHandleField {
    fn clone(&self) -> Self {
        // NCollection node copying creates a different Handle field and acquires one reference.
        raise_native_failure(Self::from_reference(self))
    }
}
impl Drop for Graphic3dClipPlaneHandleField {
    fn drop(&mut self) {
        // Physical alias leases cannot defer native node-payload destruction and its callbacks.
        raise_native_failure(self.slot.clear());
    }
}
impl HandleReference<dyn Graphic3dClipPlaneSource> for Graphic3dClipPlaneHandleField {
    fn observe(&self) -> HandleObserver<dyn Graphic3dClipPlaneSource> {
        // A retained field alias rereads its target instead of keeping an owning snapshot.
        self.slot.observe()
    }
    fn reference_address(&self) -> *const () {
        // Target replacement cannot change the actual handle-field alias identity.
        self.slot.reference_address()
    }
    fn object_address(&self) -> *const () {
        // Destructor inspection reads the pointer without projecting a zero-count pointee.
        self.slot.object_address()
    }
    fn is_null(&self) -> bool {
        // Nullness remains readable throughout the old pointee's destructor callback.
        self.slot.is_null()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphic3d::Graphic3dClipPlane;
    use rustycad_tkernel::standard::Handle;

    #[test]
    fn node_field_copies_own_separately_and_storage_leases_do_not_extend_native_ownership() {
        let plane = Handle::new(Graphic3dClipPlane::default());
        let field = Graphic3dClipPlaneHandleField::from_reference(&plane).unwrap();
        let alias = field.storage_lease();
        let copied = field.clone();
        assert_eq!(plane.ref_count(), 3);
        assert_eq!(field.reference_address(), alias.reference_address());
        assert_ne!(field.reference_address(), copied.reference_address());
        assert_eq!(field.object_address(), copied.object_address());
        drop(field);
        assert!(alias.is_null());
        assert_eq!(plane.ref_count(), 2);
        drop(copied);
        assert_eq!(plane.ref_count(), 1);
    }
    #[test]
    fn actual_field_alias_observes_replacement_and_null_without_a_target_copy() {
        let old = Handle::new(Graphic3dClipPlane::default());
        let replacement = Handle::new(Graphic3dClipPlane::default());
        let field = Graphic3dClipPlaneHandleField::from_reference(&old).unwrap();
        let alias = field.storage_lease();
        field.assign(&replacement).unwrap();
        assert_eq!(old.ref_count(), 1);
        assert_eq!(replacement.ref_count(), 2);
        assert_eq!(alias.object_address(), replacement.object_address());
        field
            .assign(&Handle::<Graphic3dClipPlane>::default())
            .unwrap();
        assert!(field.is_null());
        assert!(alias.is_null());
        assert_eq!(replacement.ref_count(), 1);
        drop(field);
        assert!(alias.is_null());
    }
    #[test]
    fn protected_field_ownership_replays_native_count_columns_without_collection_credit() {
        let first = Handle::new(Graphic3dClipPlane::default());
        let second = Handle::new(Graphic3dClipPlane::default());
        let field = Graphic3dClipPlaneHandleField::from_reference(&first).unwrap();
        let alias = field.storage_lease();
        assert_native_field_counts("borrow", first.ref_count(), second.ref_count());
        field.assign(&second).unwrap();
        assert_native_field_counts("replace", first.ref_count(), second.ref_count());
        assert_eq!(alias.object_address(), second.object_address());
        assert_eq!(alias.reference_address(), field.reference_address());
    }
    fn assert_native_field_counts(phase: &str, first: i32, second: i32) {
        let fixture = include_str!("../../tests/fixtures/clip-collection-references-native.txt");
        let row = fixture.lines().find(|row| row.starts_with(phase)).unwrap();
        let columns: Vec<&str> = row.split_whitespace().collect();
        // Only native handle-field ownership is replayed here; collection/cursor bodies are next.
        assert_eq!(first, columns[3].parse::<i32>().unwrap());
        assert_eq!(second, columns[4].parse::<i32>().unwrap());
    }
}
