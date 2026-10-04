//! Defined nullable triangle-pointer lifecycle checked against the native Poly_CoherentTriPtr.
use rustycad_tkernel::ncollection::BaseAllocator;
use rustycad_tkmath::poly::CoherentTriPtr;

const NATIVE_ROWS: &str =
    include_str!("../../../tools/oracle/tkmath_coherent_nullable_triptr.expected");

#[test]
fn protected_null_constructor_allows_linking_without_triangle_reads() {
    let mut ring = null_ring(BaseAllocator::common());
    let head = ring.head_member().unwrap();
    ring.append_member(head, 4, BaseAllocator::common())
        .unwrap();
    ring.prepend_member(head, 7, BaseAllocator::common())
        .unwrap();
    assert_native_row(
        "protected_null_constructor",
        format!("protected_null_constructor {}", ring.len()),
    );
    assert_eq!(ring.try_get_triangle().unwrap(), None);
    assert_eq!(
        ring.triangle(head).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn null_members_keep_ring_links_and_clone_state_without_dereference() {
    let allocator = BaseAllocator::incremental(4096);
    let mut ring = CoherentTriPtr::new_from_nullable_triangle_in(None, allocator.clone());
    let head = ring.head_member().unwrap();
    let second = ring.append_member(head, 4, allocator.clone()).unwrap();
    ring.prepend_member(head, 7, allocator).unwrap();
    ring.set_member_triangle_nullable(second, None).unwrap();
    assert_eq!(ring.iter_from_member(head).value(), None);
    let mut cursor = ring.iter_from_member(head);
    cursor.advance();
    assert_eq!(cursor.value(), None);
    cursor.advance();
    assert_eq!(cursor.value(), Some(7));
    assert_debug_lists_null_and_later_members(&ring);
    assert_nullable_clone_matches(&ring);
    assert_ne!(ring, CoherentTriPtr::new_from_nullable_triangle(None));
}

#[test]
fn ordinary_iterator_fails_explicitly_on_null_member() {
    let ring = CoherentTriPtr::new_from_nullable_triangle(None);
    let failure = std::panic::catch_unwind(|| ring.iter().collect::<Vec<_>>());
    assert!(failure.is_err());
}

#[test]
fn removing_a_null_member_keeps_remaining_nodes_linked() {
    let allocator = BaseAllocator::incremental(4096);
    let mut ring = CoherentTriPtr::new_from_nullable_triangle_in(None, allocator.clone());
    let head = ring.head_member().unwrap();
    ring.append_member(head, 4, allocator.clone()).unwrap();
    ring.prepend_member(head, 7, allocator.clone()).unwrap();
    ring.remove_member(head, &allocator).unwrap();
    assert_eq!(ring.try_get_triangle().unwrap(), Some(4));
    assert_eq!(ring.iter().collect::<Vec<_>>(), [4, 7]);
    ring.remove_list_in(&allocator).unwrap();
    allocator.reset(false).unwrap();
}

#[test]
fn set_triangle_reset_and_null_member_removal_match_native_order() {
    let allocator = BaseAllocator::incremental(4096);
    let mut ring = null_ring(allocator.clone());
    let head = ring.head_member().unwrap();
    ring.append_member(head, 4, allocator.clone()).unwrap();
    ring.prepend_member(head, 7, allocator.clone()).unwrap();
    ring.set_triangle_nullable(Some(1)).unwrap();
    assert_native_row("restored_order", format_values("restored_order", &ring));
    ring.set_triangle_nullable(None).unwrap();
    let next = ring.next_member(head).unwrap();
    ring.remove_member(head, &allocator).unwrap();
    assert_eq!(ring.head_member(), Some(next));
    assert_native_row(
        "remove_null_member",
        format_values("remove_null_member", &ring),
    );
}

#[test]
fn empty_nullable_ring_returns_checked_head_errors() {
    let mut ring = CoherentTriPtr::new_from_nullable_triangle(None);
    ring.remove(0);
    assert_eq!(
        ring.try_get_triangle().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert!(ring.set_triangle_nullable(Some(8)).is_err());
}

fn null_ring(allocator: BaseAllocator) -> CoherentTriPtr {
    CoherentTriPtr::new_from_nullable_triangle_in(None, allocator)
}

fn format_values(label: &str, ring: &CoherentTriPtr) -> String {
    let values = ring
        .iter()
        .map(|value| format!(" {value}"))
        .collect::<String>();
    format!("{label} {}{values}", ring.len())
}

fn assert_native_row(label: &str, actual: String) {
    let expected = NATIVE_ROWS
        .lines()
        .find(|row| row.starts_with(label))
        .unwrap();
    assert_eq!(actual, expected);
}

fn assert_nullable_clone_matches(ring: &CoherentTriPtr) {
    let clone = ring.clone();
    assert_ne!(clone.head_member(), ring.head_member());
    assert_eq!(
        clone
            .triangle_nullable(clone.head_member().unwrap())
            .unwrap(),
        None
    );
    assert_eq!(clone, *ring);
}

fn assert_debug_lists_null_and_later_members(ring: &CoherentTriPtr) {
    let rendered = format!("{ring:?}");
    assert!(rendered.contains("None"));
    assert!(rendered.contains("Some(7)"));
}
