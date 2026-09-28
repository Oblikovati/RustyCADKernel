//! Rust safety and ownership contracts not supplied by upstream legacy tests.
use rustycad_tkernel::{standard::Failure, tcollection::AsciiString};
#[test]
fn failure_printing_and_shared_copies() {
    let original = Failure::with_type("Standard_Failure", "message\0ignored", "\ntrace");
    let mut copy = original.clone();
    copy.set_message("changed");
    copy.set_stack_string("new trace");
    assert_eq!(original.message(), "message");
    assert_eq!(original.stack_string(), "\ntrace");
    assert_eq!(original.to_string(), "Standard_Failure: message\ntrace");
    assert_eq!(copy.message(), "changed");
    assert_eq!(copy.stack_string(), "new trace");
    assert_eq!(Failure::default().to_string(), "Standard_Failure");
    assert_eq!(Failure::default().exception_type(), "Standard_Failure");
}
#[test]
fn byte_string_terminators_and_owned_copies() {
    let original = AsciiString::from_bytes([255, b'A', 0, b'B']);
    let mut copy = original.clone();
    copy.append(&AsciiString::from_integer(-12));
    assert_eq!(original.as_bytes(), &[255, b'A']);
    assert_eq!(copy.as_bytes_with_nul(), &[255, b'A', b'-', b'1', b'2', 0]);
    assert_eq!(AsciiString::default().as_bytes_with_nul(), &[0]);
}

#[test]
fn parallel_pool_configuration_and_worker_ids() {
    use rustycad_tkernel::osd::ThreadPool;
    use std::sync::atomic::{AtomicUsize, Ordering};
    assert!(ThreadPool::new(0).is_err());
    let pool = ThreadPool::new(2).unwrap();
    assert_eq!(pool.thread_count(), 2);
    let count = AtomicUsize::new(0);
    pool.launcher().perform(0..100, |worker, index| {
        assert!(worker < 2);
        assert!(index < 100);
        count.fetch_add(1, Ordering::SeqCst);
    });
    assert_eq!(count.load(Ordering::SeqCst), 100);
}

#[test]
fn numeric_prefixes_and_occt_hex_boundary_behavior() {
    use rustycad_tkernel::standard::strtod;
    assert_eq!(strtod("  +1.25tail"), (1.25, 7));
    assert_eq!(strtod("1e+"), (1.0, 1));
    assert_eq!(strtod("."), (0.0, 0));
    assert_eq!(strtod("0x.8p2"), (2.0, 6));
    assert_eq!(strtod("0x1p-1074").0.to_bits(), 1);
    assert_eq!(strtod("0x1p-1075").0.to_bits(), 0);
    assert_eq!(strtod("0x1.0000000000001p-1075").0.to_bits(), 1);
    assert_eq!(strtod("0x1.fffffffffffffp-1023").0.to_bits(), 0);
    assert_eq!(strtod("nan(123 456)").0.to_bits(), 0x7ff0012300000456);
    assert_eq!(strtod("-0").0.to_bits(), 1_u64 << 63);
}
