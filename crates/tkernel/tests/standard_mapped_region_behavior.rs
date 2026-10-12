//! Checks that the mapped allocator really releases pages, beyond updating its token.
#![cfg(target_os = "linux")]
#![allow(unsafe_code)]

use rustycad_tkernel::standard::MMgrOpt;
use std::process::Command;

const MAPPED_REGION_CHILD: &str = "MARTENSITE_MAPPED_REGION_CHILD";

#[test]
fn mapped_prefix_free_unmaps_prefix_and_keeps_suffix_resident() {
    if std::env::var_os(MAPPED_REGION_CHILD).is_some() {
        assert_prefix_release();
        return;
    }
    assert_isolated_prefix_release();
}

fn isolated_prefix_release() -> std::process::Output {
    // Freed addresses are process-global: parallel allocations can reuse the prefix before mincore.
    Command::new(std::env::current_exe().expect("test executable path"))
        .args([
            "--exact",
            "standard_mapped_region_behavior::mapped_prefix_free_unmaps_prefix_and_keeps_suffix_resident",
            "--test-threads=1",
        ])
        .env(MAPPED_REGION_CHILD, "1")
        .output()
        .expect("spawn isolated mapped-region check")
}

fn assert_isolated_prefix_release() {
    let output = isolated_prefix_release();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "mapped-region child expected success: {output:?}"
    );
    assert!(
        stdout.contains("1 passed; 0 failed"),
        "expected exactly one checked test: {stdout}"
    );
}

fn assert_prefix_release() {
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
