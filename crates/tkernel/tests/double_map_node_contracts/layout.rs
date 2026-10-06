use super::fixture::native_node_row;
use rustycad_tkernel::ncollection::{BaseAllocator, DoubleMapNode};
#[derive(Default)]
#[repr(C, align(32))]
struct DoubleWide {
    value: u8,
}
fn allocation_span<K: Default, V: Default>() -> usize {
    let arena = BaseAllocator::incremental(4096);
    let _prefix = arena.allocate(16).unwrap();
    let make = || {
        DoubleMapNode::allocate_with(
            &arena,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            K::default,
            V::default,
        )
    };
    let (node, adjacent) = (make(), make());
    let span = std::ptr::from_ref(&*adjacent).addr() - std::ptr::from_ref(&*node).addr();
    assert_eq!(span, size_of::<DoubleMapNode<K, V>>());
    span
}
fn empty_node<K: Default, V: Default>() -> DoubleMapNode<K, V> {
    DoubleMapNode::new(
        K::default(),
        V::default(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    )
}
fn layout_row<K: Default, V: Default>(label: &str) {
    let mut node = empty_node::<K, V>();
    let start = std::ptr::from_ref(&node).addr();
    assert_eq!(std::ptr::from_ref(node.link()).addr(), start);
    let key = std::ptr::from_ref(node.key1()).addr() - start;
    let value = std::ptr::from_ref(node.key2()).addr() - start;
    let next = std::ptr::from_mut(node.next2()).addr() - start;
    native_node_row(
        label,
        format!(
            "{label} {} {} {key} {value} {next} {}",
            size_of_val(&node),
            align_of_val(&node),
            allocation_span::<K, V>()
        ),
    );
}
#[test]
fn scalar_node_layout_and_allocation_bytes_match_native() {
    layout_row::<i32, i32>("int_int");
    layout_row::<u8, u8>("u8_u8");
    layout_row::<i32, u8>("int_u8");
    layout_row::<u8, i32>("u8_int");
}
#[test]
fn aligned_node_layout_and_allocation_bytes_match_native() {
    layout_row::<DoubleWide, u8>("wide_u8");
    layout_row::<u8, DoubleWide>("u8_wide");
    layout_row::<DoubleWide, DoubleWide>("wide_wide");
}
