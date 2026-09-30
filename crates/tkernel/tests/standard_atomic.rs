//! OCC22980 retained at its original 2^24 operation count.
#![allow(non_snake_case)]
use rustycad_tkernel::osd::Parallel;
use std::sync::atomic::{AtomicI32, Ordering};
#[test]
fn Standard_AtomicTest__OCC22980_AtomicOperations() {
    let sum = AtomicI32::new(0);
    assert_eq!(sum.fetch_sub(1, Ordering::SeqCst) - 1, -1);
    assert_eq!(sum.fetch_add(1, Ordering::SeqCst) + 1, 0);
    assert_eq!(sum.fetch_add(1, Ordering::SeqCst) + 1, 1);
    assert_eq!(sum.fetch_add(1, Ordering::SeqCst) + 1, 2);
    sum.store(0, Ordering::SeqCst);
    const COUNT: usize = 1 << 24;
    Parallel::for_range(
        0,
        COUNT as i32,
        |_| {
            sum.fetch_add(1, Ordering::SeqCst);
        },
        false,
    )
    .unwrap();
    assert_eq!(sum.load(Ordering::SeqCst), COUNT as i32);
    Parallel::for_range(
        0,
        COUNT as i32,
        |_| {
            sum.fetch_sub(1, Ordering::SeqCst);
        },
        false,
    )
    .unwrap();
    assert_eq!(sum.load(Ordering::SeqCst), 0);
}
