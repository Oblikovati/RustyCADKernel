// SPDX-License-Identifier: GPL-2.0-only
//! Metadata-only readers retain the original slot, not an observed/projected pointee.
use martensite_tkmeshvs::{MeshVSDrawer, MeshVSDrawerSource};
use rustycad_tkernel::standard::{Handle, HandleObserver, HandleReference};
struct MetadataOnlyDrawerReference<'a> {
    original: &'a dyn HandleReference<dyn MeshVSDrawerSource>,
}
impl HandleReference<dyn MeshVSDrawerSource> for MetadataOnlyDrawerReference<'_> {
    fn observe(&self) -> HandleObserver<dyn MeshVSDrawerSource> {
        panic!("metadata-only reader expected original slot queries, received observation")
    }
    fn reference_address(&self) -> *const () {
        self.original.reference_address()
    }
    fn object_address(&self) -> *const () {
        self.original.object_address()
    }
    fn is_null(&self) -> bool {
        self.original.is_null()
    }
}
#[test]
fn metadata_only_reference_preserves_live_and_null_original_slot_identity() {
    let owner = Handle::new(MeshVSDrawer::default());
    assert_original_metadata(&owner);
    assert_eq!(owner.ref_count(), 1);
    let empty = Handle::<MeshVSDrawer>::default();
    assert_original_metadata(&empty);
}
fn assert_original_metadata(owner: &Handle<MeshVSDrawer>) {
    let reference = MetadataOnlyDrawerReference { original: owner };
    assert_eq!(
        reference.reference_address(),
        std::ptr::from_ref(owner).cast()
    );
    assert_eq!(reference.object_address(), owner.object_address());
    assert_eq!(reference.is_null(), owner.is_null());
}
