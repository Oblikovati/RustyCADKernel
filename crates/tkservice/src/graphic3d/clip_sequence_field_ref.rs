//! Captured actual clipping handle slots; alias reads never repeat a virtual getter.
use super::Graphic3dClipPlaneSequenceSource;
use rustycad_tkernel::standard::{Handle, HandleObserver, HandleReference, IntrusiveReferenceSlot};
use std::sync::{Arc, Weak};
/// Nonowning alias of one selected native field; `Graphic3dClipSequenceFieldRef::new(&actual_slot)`.
#[derive(Clone)]
pub struct Graphic3dClipSequenceFieldRef {
    slot: Weak<IntrusiveReferenceSlot<dyn Graphic3dClipPlaneSequenceSource>>,
    address: usize,
}
impl Graphic3dClipSequenceFieldRef {
    /// Capture this field once, without retaining its owner/target; `Self::new(&actual_slot)`.
    pub fn new(slot: &Arc<IntrusiveReferenceSlot<dyn Graphic3dClipPlaneSequenceSource>>) -> Self {
        Self {
            slot: Arc::downgrade(slot),
            address: slot.reference_address() as usize,
        }
    }
}
impl HandleReference<dyn Graphic3dClipPlaneSequenceSource> for Graphic3dClipSequenceFieldRef {
    fn observe(&self) -> HandleObserver<dyn Graphic3dClipPlaneSequenceSource> {
        self.slot.upgrade().map_or_else(
            || Handle::<dyn Graphic3dClipPlaneSequenceSource>::default().observe(),
            |slot| slot.observe(),
        )
    }
    fn reference_address(&self) -> *const () {
        // The actual selected field keeps its identity when its target is replaced or cleared.
        self.address as *const ()
    }
    fn object_address(&self) -> *const () {
        // Identity/null inspection remains valid during native pointee retirement.
        self.slot
            .upgrade()
            .map_or(std::ptr::null(), |slot| slot.object_address())
    }
    fn is_null(&self) -> bool {
        // Expiration after field disposal is safe-null beyond C++'s valid reference lifetime.
        self.slot.upgrade().is_none_or(|slot| slot.is_null())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphic3d::Graphic3dSequenceOfHClipPlane;
    #[test]
    fn captured_clipping_field_keeps_identity_and_observes_replacement_without_ownership() {
        let first = Handle::new(Graphic3dSequenceOfHClipPlane::default());
        let second = Handle::new(Graphic3dSequenceOfHClipPlane::default());
        let slot = Arc::new(IntrusiveReferenceSlot::default());
        slot.assign(&first).unwrap();
        let alias = Graphic3dClipSequenceFieldRef::new(&slot);
        assert_eq!(first.ref_count(), 2);
        assert_eq!(alias.observe().ref_count(), 2);
        slot.assign(&second).unwrap();
        assert_eq!(alias.object_address(), second.object_address());
        assert_eq!(alias.reference_address(), slot.reference_address());
        assert_eq!(alias.clone().reference_address(), alias.reference_address());
        slot.clear().unwrap();
        assert!(alias.is_null());
        assert_eq!((first.ref_count(), second.ref_count()), (1, 1));
    }
    #[test]
    fn disposing_clipping_field_does_not_keep_collection_alive_through_alias_storage() {
        let owner = Handle::new(Graphic3dSequenceOfHClipPlane::default());
        let slot = Arc::new(IntrusiveReferenceSlot::default());
        slot.assign(&owner).unwrap();
        let alias = Graphic3dClipSequenceFieldRef::new(&slot);
        drop(slot);
        assert!(alias.is_null());
        assert!(alias.observe().is_null());
        assert!(alias.object_address().is_null());
        assert_eq!(owner.ref_count(), 1);
    }
}
