//! Standard_MMgrOpt / Standard_MMgrRoot.
use rustycad_tkernel::standard::{MMgrOpt, MMgrRoot, StorageAddress};
use std::cell::RefCell;

fn manager(clear: bool, mmap: bool) -> MMgrOpt {
    MMgrOpt::new(clear, mmap, 200, 1000, 40000)
}

#[test]
fn initialize_corrects_page_count_and_rounds_cell_size() {
    let small = MMgrOpt::new(true, true, 200, 99, 40000);
    assert_eq!(small.nb_pages(), 1000);
    assert_eq!(small.cell_size(), 208);
    assert_eq!(MMgrOpt::new(true, true, 17, 100, 1).nb_pages(), 100);
    assert!(small.page_size().is_power_of_two());
    assert_eq!(MMgrOpt::default().threshold(), 40000);
}

#[test]
fn mapped_system_regions_round_to_pages_and_are_zeroed() {
    let mut opt = manager(true, true);
    let mut size = opt.page_size() + 1;
    let block = opt.allocate_system_memory(&mut size).unwrap();
    assert_eq!(size, 2 * opt.page_size());
    assert_ne!(block.address().get(), 0);
    assert_eq!(block.allocated_size(), size);
    assert!(
        opt.system_memory_bytes(&block)
            .unwrap()
            .iter()
            .all(|b| *b == 0)
    );
    opt.system_memory_bytes_mut(&block).unwrap()[0] = 7;
    assert_eq!(opt.system_memory_bytes(&block).unwrap()[0], 7);
    opt.free_system_memory(&block, size).unwrap();
    assert!(opt.system_memory_bytes(&block).is_err());
}

#[test]
fn heap_system_regions_keep_size_and_reject_inexact_or_foreign_frees() {
    let mut opt = manager(true, false);
    let mut other = manager(true, false);
    let mut size = 37;
    let block = opt.allocate_system_memory(&mut size).unwrap();
    assert_eq!(size, 37);
    assert_eq!(block.allocated_size(), 37);
    assert!(
        opt.system_memory_bytes(&block)
            .unwrap()
            .iter()
            .all(|b| *b == 0)
    );
    assert_program_error(opt.free_system_memory(&block, 36).unwrap_err());
    assert_program_error(other.free_system_memory(&block, size).unwrap_err());
    opt.free_system_memory(&block, size).unwrap();
}

fn assert_program_error(failure: rustycad_tkernel::standard::Failure) {
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
}

#[test]
fn small_blocks_are_carved_contiguously_with_an_eight_byte_header() {
    let mut opt = manager(true, true);
    let first = opt.allocate(10).unwrap().get();
    let second = opt.allocate(10).unwrap().get();
    let third = opt.allocate(1).unwrap().get();
    assert_eq!(second - first, 16 + 8);
    assert_eq!(third - second, 16 + 8);
    assert_eq!(
        first % opt.page_size(),
        16,
        "pool link word, then the block header"
    );
}

#[test]
fn freed_blocks_are_reused_newest_first_by_rounded_size() {
    let mut opt = manager(false, false);
    let a = opt.allocate(300).unwrap();
    let b = opt.allocate(300).unwrap();
    opt.free(Some(a)).unwrap();
    opt.free(Some(b)).unwrap();
    assert_eq!(opt.allocate(297).unwrap(), b);
    assert_eq!(opt.allocate(304).unwrap(), a);
}

#[test]
fn purge_releases_medium_blocks_only() {
    let mut opt = manager(true, true);
    let small = opt.allocate(8).unwrap();
    let medium = [opt.allocate(300).unwrap(), opt.allocate(500).unwrap()];
    opt.free(Some(small)).unwrap();
    medium.iter().for_each(|m| opt.free(Some(*m)).unwrap());
    assert_eq!(opt.purge(false), 2);
    assert_eq!(opt.purge(true), 0);
    assert_eq!(
        opt.allocate(8).unwrap(),
        small,
        "small free lists survive a purge"
    );
}

#[test]
fn free_pools_invalidates_small_handles_and_keeps_medium_blocks() {
    let mut opt = manager(true, false);
    let small = opt.allocate(16).unwrap();
    let medium = opt.allocate(300).unwrap();
    opt.bytes_mut(medium).unwrap()[..4].copy_from_slice(b"keep");
    opt.free_pools().unwrap();
    assert!(opt.bytes(small).is_err());
    assert_eq!(&opt.bytes(medium).unwrap()[..4], b"keep");
    opt.free_pools().unwrap();
    let next_small = opt.allocate(16).unwrap();
    assert!(opt.bytes(next_small).unwrap().iter().all(|byte| *byte == 0));
}

#[test]
fn free_pools_prunes_recycled_pool_blocks_before_reuse() {
    let mut opt = manager(true, false);
    let released = opt.allocate(16).unwrap();
    opt.free(Some(released)).unwrap();
    opt.free_pools().unwrap();
    let next = opt.allocate(16).unwrap();
    assert!(opt.bytes(next).unwrap().iter().all(|byte| *byte == 0));
}

#[test]
fn a_fully_free_pool_is_never_released() {
    // 100 pages hold (P - 8) / 24 blocks of 16 bytes; the tally of a completely free pool
    // is P - 8 (or P - 16), never the pool size P that Purge compares it with.
    let mut opt = MMgrOpt::new(true, true, 16, 100, 40000);
    let per_pool = (opt.page_size() * 100 - 8) / 24;
    let blocks: Vec<StorageAddress> = (0..per_pool + 1)
        .map(|_| opt.allocate(16).unwrap())
        .collect();
    blocks[..per_pool]
        .iter()
        .for_each(|b| opt.free(Some(*b)).unwrap());
    assert_eq!(opt.purge(false), 0);
    assert_eq!(opt.allocate(16).unwrap(), blocks[per_pool - 1]);
}

#[test]
fn sizes_near_usize_max_wrap_like_the_macros() {
    let mut opt = manager(true, true);
    let zero = opt.allocate(usize::MAX).unwrap();
    assert!(opt.bytes(zero).unwrap().is_empty());
    let failure = opt.allocate(usize::MAX - 8).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfMemory");
    assert_eq!(
        String::from_utf8_lossy(failure.what()),
        "Invalid argument",
        "mmap of the wrapped zero length"
    );
    let huge = opt.allocate(1 << 62).unwrap_err();
    assert_eq!(
        String::from_utf8_lossy(huge.what()),
        "Cannot allocate memory"
    );
    let unmapped = manager(true, false).allocate(1 << 62).unwrap_err();
    assert_eq!(
        String::from_utf8_lossy(unmapped.what()),
        "Standard_MMgrOpt::Allocate(): malloc failed"
    );
}

#[test]
fn large_blocks_are_released_immediately() {
    let mut opt = manager(true, true);
    let large = opt.allocate(50000).unwrap();
    assert_eq!(large.get() % opt.page_size(), 8);
    assert_eq!(opt.bytes(large).unwrap().len(), 50000);
    opt.free(Some(large)).unwrap();
    assert!(opt.bytes(large).is_err());
    assert_eq!(opt.purge(false), 0);
}

#[test]
fn reallocate_keeps_fitting_blocks_and_moves_growing_ones() {
    let mut opt = manager(true, true);
    let block = opt.reallocate(None, 3).unwrap();
    opt.bytes_mut(block).unwrap()[..3].copy_from_slice(b"xyz");
    assert_eq!(opt.reallocate(Some(block), 8).unwrap(), block);
    let moved = opt.reallocate(Some(block), 30).unwrap();
    assert_ne!(moved, block);
    assert_eq!(&opt.bytes(moved).unwrap()[..4], b"xyz\0");
    assert!(
        opt.bytes(moved).unwrap()[8..30]
            .iter()
            .all(|byte| *byte == 0)
    );
    assert_eq!(
        opt.allocate(8).unwrap(),
        block,
        "the old block went to its free list"
    );
}

#[test]
fn foreign_or_freed_storage_is_refused() {
    let mut opt = manager(true, true);
    let block = opt.allocate(8).unwrap();
    opt.free(Some(block)).unwrap();
    let twice = opt.free(Some(block)).unwrap_err();
    assert!(twice.is_kind("Standard_ProgramError"));
    assert!(twice.message().contains(&format!("{:#x}", block.get())));
    assert!(opt.bytes(StorageAddress::new(8).unwrap()).is_err());
    opt.free(None).unwrap();
}

#[test]
fn clearing_zeroes_reused_blocks() {
    let mut opt = manager(true, true);
    let block = opt.allocate(16).unwrap();
    opt.bytes_mut(block).unwrap().fill(0xAB);
    opt.free(Some(block)).unwrap();
    let reused = opt.allocate(16).unwrap();
    assert!(opt.bytes(reused).unwrap().iter().all(|b| *b == 0));
    let mut keep = manager(false, true);
    let block = keep.allocate(16).unwrap();
    keep.bytes_mut(block).unwrap().fill(0xAB);
    keep.free(Some(block)).unwrap();
    let reused = keep.allocate(16).unwrap();
    assert!(keep.bytes(reused).unwrap().iter().all(|b| *b == 0xAB));
}

#[test]
fn the_root_interface_dispatches_and_defaults_purge() {
    struct CountingRoot;
    impl MMgrRoot for CountingRoot {
        fn allocate(
            &mut self,
            _: usize,
        ) -> Result<StorageAddress, rustycad_tkernel::standard::Failure> {
            Ok(StorageAddress::new(16).unwrap())
        }
        fn reallocate(
            &mut self,
            _: Option<StorageAddress>,
            _: usize,
        ) -> Result<StorageAddress, rustycad_tkernel::standard::Failure> {
            Ok(StorageAddress::new(32).unwrap())
        }
        fn free(
            &mut self,
            _: Option<StorageAddress>,
        ) -> Result<(), rustycad_tkernel::standard::Failure> {
            Ok(())
        }
        fn bytes(&self, _: StorageAddress) -> Result<&[u8], rustycad_tkernel::standard::Failure> {
            Ok(&[])
        }
        fn bytes_mut(
            &mut self,
            _: StorageAddress,
        ) -> Result<&mut [u8], rustycad_tkernel::standard::Failure> {
            Ok(&mut [])
        }
    }
    assert_eq!(
        CountingRoot.purge(true),
        0,
        "Standard_MMgrRoot::Purge returns 0"
    );
    let mut dynamic: Box<dyn MMgrRoot> = Box::new(manager(true, true));
    let block = dynamic.allocate(24).unwrap();
    assert_eq!(dynamic.bytes(block).unwrap().len(), 24);
    dynamic.free(Some(block)).unwrap();
    assert_eq!(dynamic.purge(false), 0);
}

thread_local! {
    static OBSERVED: RefCell<Vec<(bool, usize, usize)>> = const { RefCell::new(Vec::new()) };
}

fn observe(is_alloc: bool, _: StorageAddress, round_size: usize, size: usize) {
    OBSERVED.with(|log| log.borrow_mut().push((is_alloc, round_size, size)));
}

#[test]
fn the_callback_sees_rounded_and_requested_sizes() {
    // The callback is process-wide; only this test installs one, and each test thread
    // logs into its own thread-local list.
    MMgrOpt::set_call_back_function(Some(observe));
    let mut opt = manager(true, true);
    let block = opt.allocate(13).unwrap();
    let moved = opt.reallocate(Some(block), 17).unwrap();
    opt.free(Some(moved)).unwrap();
    MMgrOpt::set_call_back_function(None);
    opt.allocate(1).unwrap();
    let log = OBSERVED.with(|log| log.borrow().clone());
    assert_eq!(
        log,
        [
            (true, 16, 13),
            (true, 24, 17),
            (false, 16, 0),
            (false, 24, 0)
        ]
    );
}
