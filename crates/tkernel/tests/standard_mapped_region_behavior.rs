//! Checks that the mapped allocator really releases pages, beyond updating its token.
#![cfg(target_os = "linux")]
#![allow(unsafe_code)]

use rustycad_tkernel::standard::MMgrOpt;

#[test]
fn mapped_prefix_free_unmaps_prefix_and_keeps_suffix_resident() {
    let mut manager = MMgrOpt::default();
    let (page, mut block) = allocate_three_pages(&mut manager);
    let prefix = block.address().get();
    let suffix = prefix + 2 * page;
    manager.free_system_memory(&mut block, page + 1).unwrap();
    assert_unmapped(prefix, page);
    assert_resident(suffix, page);
    manager.free_system_memory(&mut block, page).unwrap();
}

#[test]
fn allocator_keeps_send_and_sync_capabilities() {
    assert_send_sync::<MMgrOpt>();
}

fn assert_send_sync<T: Send + Sync>() {}

fn allocate_three_pages(
    manager: &mut MMgrOpt,
) -> (usize, rustycad_tkernel::standard::MMgrOptSystemBlock) {
    let page = manager.page_size();
    let mut size = 3 * page;
    let block = manager.allocate_system_memory(&mut size).unwrap();
    assert_eq!(size, 3 * page);
    (page, block)
}

fn assert_unmapped(address: usize, page: usize) {
    let mut residency = 0;
    // SAFETY: mincore only inspects residency metadata for one aligned page.
    let result = unsafe { libc::mincore(address as *mut libc::c_void, page, &mut residency) };
    assert_eq!(result, -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ENOMEM)
    );
}

fn assert_resident(address: usize, page: usize) {
    let mut residency = 0;
    // SAFETY: the checked address belongs to the still-live mapped suffix.
    let result = unsafe { libc::mincore(address as *mut libc::c_void, page, &mut residency) };
    assert_eq!(result, 0);
}
