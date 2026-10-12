//! Concrete native handle arguments project their actual slots without temporary ownership.
use super::{
    Graphic3dClipPlane, Graphic3dClipPlaneSequenceSource, Graphic3dClipPlaneSource,
    Graphic3dSequenceOfHClipPlane,
};
use rustycad_tkernel::standard::{Handle, HandleObserver, HandleReference};

macro_rules! borrowed_clip_source_argument {
    ($concrete:ty, $source:ty) => {
        impl HandleReference<$source> for Handle<$concrete> {
            fn observe(&self) -> HandleObserver<$source> {
                // A cast followed by observe/drop would add an observable native reference.
                self.observe_as::<$source>()
            }
            fn reference_address(&self) -> *const () {
                std::ptr::from_ref(self).cast()
            }
        }
    };
}
borrowed_clip_source_argument!(Graphic3dClipPlane, dyn Graphic3dClipPlaneSource);
borrowed_clip_source_argument!(
    Graphic3dSequenceOfHClipPlane,
    dyn Graphic3dClipPlaneSequenceSource
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plane_argument_borrows_actual_slot_without_temporary_ownership() {
        let plane = Handle::new(Graphic3dClipPlane::default());
        let reference: &dyn HandleReference<dyn Graphic3dClipPlaneSource> = &plane;
        let observed = reference.observe();
        assert_eq!(
            reference.reference_address(),
            std::ptr::from_ref(&plane).cast()
        );
        assert_eq!(observed.object_address(), plane.object_address());
        assert_eq!(plane.ref_count(), 1);
        let owned = reference.clone_handle();
        assert_eq!(owned.ref_count(), 2);
        drop(owned);
        assert_eq!(plane.ref_count(), 1);
    }

    #[test]
    fn collection_argument_borrows_actual_slot_and_preserves_null() {
        let collection = Handle::new(Graphic3dSequenceOfHClipPlane::default());
        let reference: &dyn HandleReference<dyn Graphic3dClipPlaneSequenceSource> = &collection;
        assert_eq!(
            reference.reference_address(),
            std::ptr::from_ref(&collection).cast()
        );
        assert_eq!(reference.observe().ref_count(), 1);
        assert_eq!(reference.object_address(), collection.object_address());
        let null = Handle::<Graphic3dSequenceOfHClipPlane>::default();
        let null_reference: &dyn HandleReference<dyn Graphic3dClipPlaneSequenceSource> = &null;
        assert!(null_reference.is_null());
        assert!(null_reference.observe().is_null());
    }
}
