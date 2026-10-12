//! Allocation receipts distinguish actual owned buffers from the native static terminator.
use super::AsciiString;

#[test]
fn ascii_buffer_receipts_preserve_copy_reuse_and_reject_release_cycles() {
    let mut text = AsciiString::new("abcdefghi");
    let receipt = text.buffer_identity();
    text.copy("123456789");
    assert!(receipt.matches(&text));
    text.assign_from(&AsciiString::new("abc"));
    assert!(receipt.matches(&text));
    assert_eq!(
        receipt.address(),
        text.as_bytes_with_nul().as_ptr() as usize
    );
    text.copy("replacement-longer-than-before");
    assert!(!receipt.matches(&text));
    let replacement = text.buffer_identity();
    text.clear();
    text.copy("replacement-longer-than-before");
    assert!(!replacement.matches(&text));
}

#[test]
fn ascii_empty_receipts_distinguish_static_storage_from_owned_truncation() {
    let empty = AsciiString::default().buffer_identity();
    assert!(empty.is_static_empty());
    assert!(empty.matches(AsciiString::empty_string()));
    let mut text = AsciiString::filled(8, b'x');
    let owned = text.buffer_identity();
    text.truncate(0).unwrap();
    assert!(owned.matches(&text));
    assert!(!owned.is_static_empty());
    assert!(!empty.matches(&text));
    text.copy("");
    assert!(empty.matches(&text));
    assert!(!owned.matches(&text));
}

#[test]
fn ascii_growth_paths_replace_receipts_and_nonallocating_edits_preserve_them() {
    for operation in 0..5 {
        let mut text = AsciiString::new("abc");
        let receipt = text.buffer_identity();
        grow_ascii_buffer(&mut text, operation);
        assert!(!receipt.matches(&text), "growth path {operation}");
        let grown = text.buffer_identity();
        text.upper_case();
        text.set_value_char(1, b'!').unwrap();
        text.remove(1, 1).unwrap();
        assert!(grown.matches(&text));
    }
}

fn grow_ascii_buffer(text: &mut AsciiString, operation: i32) {
    match operation {
        0 => text.append(&AsciiString::new("01234567890123456789")),
        1 => text.insert(1, "01234567890123456789").unwrap(),
        2 => text.set_value(2, "01234567890123456789").unwrap(),
        3 => text.left_justify(32, b'.').unwrap(),
        4 => text.right_justify(32, b'.').unwrap(),
        _ => unreachable!("operation {operation}; expected 0..5"),
    }
}

#[test]
fn ascii_move_swap_and_clone_receipts_track_physical_storage() {
    let mut first = AsciiString::new("first");
    let original = first.buffer_identity();
    let mut moved = AsciiString::move_from(&mut first);
    assert!(original.matches(&moved));
    assert_eq!(original.observe().unwrap().as_bytes(), b"first");
    assert!(!original.matches(&first));
    assert!(!original.matches(&moved.clone()));
    first.swap(&mut moved);
    assert!(original.matches(&first));
    assert_eq!(original.observe().unwrap().as_bytes(), b"first");
    let default = moved.buffer_identity();
    moved.insert(1, "new").unwrap();
    assert!(!default.matches(&moved));
    assert!(!moved.buffer_identity().is_static_empty());
}

#[test]
fn ascii_weak_observers_release_ownership_and_preserve_actual_reassigned_buffer() {
    let mut original = AsciiString::new("abcdefgh");
    let captured = original.buffer_identity();
    let mut owner = AsciiString::move_from(&mut original);
    owner.assign_from(&AsciiString::new("xyz"));
    assert_eq!(captured.observe().unwrap().as_bytes(), b"xyz");
    assert_eq!(
        captured.address(),
        owner.as_bytes_with_nul().as_ptr() as usize
    );
    let mut swapped = AsciiString::new("other");
    owner.swap(&mut swapped);
    assert_eq!(captured.observe().unwrap().as_bytes(), b"xyz");
    drop(swapped);
    assert!(captured.observe().is_err());
    assert_eq!(owner.as_bytes(), b"other");
}

#[test]
fn ascii_retired_allocation_cannot_be_observed_through_a_temporary_storage_lease() {
    let owner = AsciiString::new("owned");
    let captured = owner.buffer_identity();
    let lease = owner.terminated.allocation.as_ref().unwrap().clone();
    drop(owner);
    assert!(captured.observe().is_err());
    assert!(!*lease.gate.read().unwrap());
}

#[test]
fn ascii_weak_observation_serializes_with_bounded_same_size_owner_updates() {
    let mut owner = AsciiString::new("aaaaaaaa");
    let observer = owner.buffer_identity();
    let barrier = std::sync::Barrier::new(2);
    std::thread::scope(|threads| {
        let reader = threads.spawn(|| run_ascii_buffer_observer(&barrier, &observer));
        run_ascii_buffer_updates(&barrier, &mut owner);
        reader.join().unwrap();
    });
    assert_eq!(observer.observe().unwrap().as_bytes(), b"aaaaaaaa");
}
fn run_ascii_buffer_observer(
    barrier: &std::sync::Barrier,
    observer: &super::AsciiStringBufferIdentity,
) {
    let mut valid = true;
    for _ in 0..64 {
        barrier.wait();
        valid &= observed_ascii_buffer_pattern(observer);
        barrier.wait();
    }
    // Finish every barrier before asserting, so a failed read cannot strand the writer.
    assert!(
        valid,
        "received an invalid observed prefix; expected either complete eight-byte pattern"
    );
}
fn observed_ascii_buffer_pattern(observer: &super::AsciiStringBufferIdentity) -> bool {
    let Ok(observed) = observer.observe() else {
        return false;
    };
    let bytes = observed.as_bytes();
    bytes == b"aaaaaaaa" || bytes == b"bbbbbbbb"
}
fn run_ascii_buffer_updates(barrier: &std::sync::Barrier, owner: &mut AsciiString) {
    let patterns = ["bbbbbbbb", "aaaaaaaa"];
    for index in 0..64 {
        barrier.wait();
        owner.copy(patterns[index % 2]);
        barrier.wait();
    }
}
#[test]
#[cfg(debug_assertions)]
fn ascii_write_guard_rejects_unretired_vec_reallocation() {
    let mut owner = AsciiString::new("owned");
    let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        owner.terminated.write().reserve(128);
    }));
    assert!(rejected.is_err());
}
