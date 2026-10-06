//! Rust ownership checks supplement native bytes/identity rows without claiming native safe borrows.
use rustycad_tkernel::standard::memory::{self, AlignedMemory, StandardMemory};

fn require_owner_traits<T: Send + Sync + Unpin>() {
    // These were automatic for the prior Vec owners and must remain available after the move.
    let _: std::marker::PhantomData<T> = std::marker::PhantomData;
}

#[test]
fn initialized_byte_owners_retain_traits_content_equality_and_debug() {
    require_owner_traits::<StandardMemory>();
    require_owner_traits::<AlignedMemory>();
    let (left, right) = (memory::allocate(3).unwrap(), memory::allocate(3).unwrap());
    assert_eq!(left, right);
    assert_eq!(format!("{left:?}"), "StandardMemory { bytes: [0, 0, 0] }");
    assert_ne!(left.as_slice().as_ptr(), right.as_slice().as_ptr());
}

#[test]
fn payload_reborrows_do_not_replace_original_deallocation_provenance() {
    let mut slot = memory::allocate_optimal(1);
    for size in [17, 64, 8, 4096, 2] {
        slot.as_mut().unwrap().as_mut_slice()[0] = 41;
        memory::reallocate(&mut slot, size).unwrap();
        assert_eq!(slot.as_ref().unwrap().as_slice()[0], 41);
        assert!(
            slot.as_ref().unwrap().as_slice()[1..]
                .iter()
                .all(|&value| value == 0)
        );
    }
    memory::free_and_null(&mut slot);
    assert!(slot.is_none());
}

#[test]
fn aligned_payload_reborrows_free_their_original_layouts() {
    for alignment in [1, 2, 4, 8, 16, 64, 4096] {
        let mut owner = memory::allocate_aligned(17, alignment).unwrap();
        owner.as_mut_slice()[3..7].copy_from_slice(b"test");
        assert_eq!(&owner.as_slice()[3..7], b"test");
        assert_eq!(owner.len(), 17);
        memory::free_aligned(Some(owner));
    }
}

#[test]
fn shared_native_backend_supports_independent_thread_owners() {
    let jobs = (0..4).map(|_| std::thread::spawn(|| memory::allocate(0).unwrap()));
    let owners: Vec<StandardMemory> = jobs.map(|job| job.join().unwrap()).collect();
    let mut identities: Vec<usize> = owners
        .iter()
        .map(|owner| owner.as_slice().as_ptr().addr())
        .collect();
    identities.sort_unstable();
    identities.dedup();
    assert_eq!(identities.len(), 4);
}
