use super::oracle::{adopted, allocator, initialized};
use rustycad_tkernel::ncollection::Buffer;
#[test]
fn adopted_incremental_prefix_prevents_reset_until_freed() {
    let owner = allocator(2);
    let mut buffer = adopted(&owner, 4);
    assert!(owner.reset(false).is_err());
    assert!(owner.reset(true).is_err());
    assert_eq!(buffer.as_slice(), &[7; 4]);
    buffer.free().unwrap();
    owner.reset(false).unwrap();
    owner.reset(true).unwrap();
}
#[test]
fn adopted_zero_extent_still_retains_incremental_lease() {
    let owner = allocator(2);
    let buffer = adopted(&owner, 0);
    assert!(!buffer.is_empty());
    assert!(owner.reset(false).is_err());
    drop(buffer);
    owner.reset(true).unwrap();
}
#[test]
fn multiple_raw_adoptions_retain_independent_incremental_leases() {
    let owner = allocator(2);
    let first = adopted(&owner, 4);
    let second = adopted(&owner, 8);
    drop(first);
    assert!(owner.reset(true).is_err());
    assert_eq!(second.as_slice(), &[7; 8]);
    drop(second);
    owner.reset(true).unwrap();
}
#[test]
fn replacement_releases_old_arena_lease_before_selecting_new_allocator() {
    let owner = allocator(2);
    let replacement = allocator(2);
    let mut buffer = adopted(&owner, 4);
    buffer.set_allocator(Some(replacement.clone())).unwrap();
    owner.reset(true).unwrap();
    buffer.allocate(8).unwrap();
    assert!(replacement.reset(true).is_err());
    drop(buffer);
    replacement.reset(true).unwrap();
}
#[test]
fn raw_optimal_bytes_keep_contents_and_mutable_pointer_provenance() {
    for policy in 0..5 {
        let owner = allocator(policy);
        let pointer = initialized(&owner, true, 16);
        // SAFETY: the full initialized allocation transfers with its original allocator.
        let mut buffer = unsafe { Buffer::from_raw_data(owner, 16, pointer) }.unwrap();
        assert_eq!(buffer.as_slice(), &[7; 16]);
        buffer.as_mut_slice().fill(9);
        assert_eq!(buffer.data_mut_ptr(), pointer);
        assert_eq!(buffer.as_slice(), &[9; 16]);
    }
}
#[test]
fn raw_adoption_retains_allocator_after_external_handle_drops() {
    let owner = allocator(2);
    let buffer = adopted(&owner, 4);
    drop(owner);
    assert_eq!(buffer.as_slice(), &[7; 4]);
    assert!(buffer.allocator().unwrap().reset(true).is_err());
}
