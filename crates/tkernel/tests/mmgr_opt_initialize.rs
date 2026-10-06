//! Protected Initialize is callable over the actual optimized allocator state.
use rustycad_tkernel::standard::MMgrOpt;

#[test]
fn repeated_initialize_forgets_recycled_medium_blocks_without_freeing_live_blocks() {
    let mut allocator = MMgrOpt::new(true, false, 0, 100, 512);
    let live = allocator.allocate(24).unwrap();
    allocator.bytes_mut(live).unwrap().fill(7);
    let retired = allocator.allocate(40).unwrap();
    allocator.free(Some(retired)).unwrap();
    allocator.initialize_native_free_lists();
    assert_eq!(allocator.purge(false), 0);
    assert_eq!(allocator.bytes(live).unwrap(), [7; 24]);
    let replacement = allocator.allocate(40).unwrap();
    assert_ne!(retired, replacement);
    allocator.free(Some(replacement)).unwrap();
    assert_eq!(allocator.purge(false), 1);
}

#[test]
fn repeated_empty_initialize_keeps_constructor_normalization() {
    let mut allocator = MMgrOpt::new(false, false, 17, 7, 512);
    let initial = (
        allocator.cell_size(),
        allocator.nb_pages(),
        allocator.page_size(),
    );
    allocator.initialize_native_free_lists();
    assert_eq!(initial, (32, 1000, allocator.page_size()));
    assert_eq!(allocator.cell_size(), 32);
    assert_eq!(allocator.nb_pages(), 1000);
    assert_eq!(allocator.purge(false), 0);
}

#[path = "mmgr_opt_initialize/callback.rs"]
mod callback;
