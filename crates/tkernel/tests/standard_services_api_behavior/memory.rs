//! Standard.hxx memory services (NATIVE manager semantics).
use rustycad_tkernel::standard::memory::{self, AllocatorType};

#[test]
fn native_allocator_is_reported_and_purge_returns_true() {
    assert_eq!(memory::get_allocator_type(), AllocatorType::Native);
    assert_eq!(AllocatorType::Jemalloc as i32, 3);
    assert_eq!(memory::purge(), 1);
}

#[test]
fn allocate_zero_fills_and_refuses_impossible_sizes() {
    let block = memory::allocate(33).unwrap();
    assert_eq!(block.len(), 33);
    assert!(block.as_slice().iter().all(|byte| *byte == 0));
    assert!(memory::allocate(0).unwrap().is_empty());
    let failure = memory::allocate(usize::MAX).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfMemory");
    assert_eq!(
        String::from_utf8_lossy(failure.what()),
        "Standard_MMgrRaw::Allocate(): malloc failed"
    );
    assert!(failure.is_kind("Standard_ProgramError"));
}

#[test]
fn allocate_optimal_returns_null_instead_of_raising() {
    assert_eq!(memory::allocate_optimal(5).map(|b| b.len()), Some(5));
    assert!(memory::allocate_optimal(1 << 62).is_none());
}

#[test]
fn free_and_null_empties_the_slot() {
    let mut slot = Some(memory::allocate(4).unwrap());
    memory::free_and_null(&mut slot);
    assert!(slot.is_none());
    memory::free(None);
    let mut aligned = memory::allocate_aligned(8, 64);
    memory::free_aligned_and_null(&mut aligned);
    assert!(aligned.is_none());
    memory::free_aligned(None);
}

#[test]
fn reallocate_keeps_the_prefix_and_allocates_from_null() {
    let mut slot = None;
    memory::reallocate(&mut slot, 3).unwrap();
    slot.as_mut()
        .unwrap()
        .as_mut_slice()
        .copy_from_slice(b"abc");
    memory::reallocate(&mut slot, 5).unwrap();
    assert_eq!(&slot.as_ref().unwrap().as_slice()[..3], b"abc");
    memory::reallocate(&mut slot, 2).unwrap();
    assert_eq!(slot.as_ref().unwrap().as_slice(), b"ab");
}

#[test]
fn reallocate_failure_keeps_the_block_but_zero_size_releases_it() {
    let mut slot = Some(memory::allocate(4).unwrap());
    let failure = memory::reallocate(&mut slot, usize::MAX).unwrap_err();
    assert_eq!(
        String::from_utf8_lossy(failure.what()),
        "Standard_MMgrRaw::Reallocate(): realloc failed"
    );
    assert_eq!(slot.as_ref().map(|b| b.len()), Some(4));
    let zero = memory::reallocate(&mut slot, 0);
    if cfg!(all(target_os = "linux", target_env = "gnu")) {
        assert!(
            zero.is_err() && slot.is_none(),
            "glibc realloc(p, 0) frees and returns NULL"
        );
    }
    let mut empty = None;
    memory::reallocate(&mut empty, 0).unwrap();
    assert!(empty.unwrap().is_empty());
}

#[test]
fn aligned_blocks_follow_the_runtime_alignment_rules() {
    for alignment in [8usize, 16, 64, 4096] {
        let block = memory::allocate_aligned(100, alignment).unwrap();
        assert_eq!(block.address() % alignment, 0);
        assert_eq!(block.len(), 100);
    }
    assert!(memory::allocate_aligned(16, 0).is_none());
    assert!(memory::allocate_aligned(16, 3).is_none());
    assert!(memory::allocate_aligned(16, 24).is_none());
    assert!(memory::allocate_aligned(usize::MAX, 16).is_none());
    assert!(memory::allocate_aligned(0, 32).unwrap().is_empty());
    let x86_gnu = cfg!(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        not(target_env = "msvc")
    ));
    if x86_gnu {
        // _mm_malloc: 1 is plain malloc, 2 and 4 widen to a pointer.
        assert!(memory::allocate_aligned(16, 1).is_some());
        assert_eq!(memory::allocate_aligned(16, 2).unwrap().address() % 8, 0);
    }
}
