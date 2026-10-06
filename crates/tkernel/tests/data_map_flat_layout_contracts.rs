//! Regression matrix for native derived keys occupying TListNode base tail padding.
use rustycad_tkernel::ncollection::{BaseAllocator, DataMapNode, ListValueNode};
#[derive(Default)]
#[repr(C)]
struct ThreeBytes {
    bytes: [u8; 3],
}
#[derive(Default)]
#[repr(C, align(32))]
struct Align32 {
    byte: u8,
}
#[derive(Default)]
#[repr(C, align(64))]
struct Align64 {
    byte: u8,
}

fn layout_row<K: Default, V: Default>(label: &str) -> String {
    let node = DataMapNode::new(K::default(), V::default(), std::ptr::null_mut());
    let start = std::ptr::from_ref(&node).addr();
    format!(
        "{label} {} {} {} {} {} {} {}",
        size_of::<DataMapNode<K, V>>(),
        align_of::<DataMapNode<K, V>>(),
        std::ptr::from_ref(node.key()).addr() - start,
        std::ptr::from_ref(node.value()).addr() - start,
        std::ptr::from_ref(node.link()).addr() - start,
        size_of::<ListValueNode<V>>(),
        typed_allocation_span::<K, V>()
    )
}
// Warm the arena so its initial block padding cannot masquerade as node allocation bytes.
fn typed_allocation_span<K: Default, V: Default>() -> usize {
    let allocator = BaseAllocator::incremental(4096);
    let _prefix = allocator.allocate(16).unwrap();
    let make =
        || DataMapNode::allocate_with(&allocator, std::ptr::null_mut(), K::default, V::default);
    let (node, adjacent) = (make(), make());
    let start = std::ptr::from_ref(&*node).addr();
    let span = std::ptr::from_ref(&*adjacent).addr() - start;
    assert_eq!(start % align_of::<DataMapNode<K, V>>(), 0);
    assert_eq!(span, size_of::<DataMapNode<K, V>>());
    span
}
fn native_layout_row<K: Default, V: Default>(label: &str) {
    let rows =
        include_str!("../../../tools/oracle/tkernel_data_map_flat_layout_contracts.expected");
    let expected = rows
        .lines()
        .find(|row| row.starts_with(&format!("{label} ")))
        .unwrap();
    assert_eq!(layout_row::<K, V>(label), expected);
}
macro_rules! layout_contract {
    ($name:ident, $key:ty, $value:ty) => {
        #[test]
        fn $name() {
            native_layout_row::<$key, $value>(stringify!($name));
        }
    };
}
layout_contract!(int_int, i32, i32);
layout_contract!(char_int, i8, i32);
layout_contract!(int_char, i32, i8);
layout_contract!(char_char, i8, i8);
layout_contract!(double_int, f64, i32);
layout_contract!(int_double, i32, f64);
layout_contract!(long_char, i64, i8);
layout_contract!(char_long, i8, i64);
layout_contract!(char_three, i8, ThreeBytes);
layout_contract!(three_char, ThreeBytes, i8);
layout_contract!(three_three, ThreeBytes, ThreeBytes);
layout_contract!(char_align32, i8, Align32);
layout_contract!(align32_char, Align32, i8);
layout_contract!(align32_align32, Align32, Align32);
layout_contract!(align64_align32, Align64, Align32);
layout_contract!(align32_align64, Align32, Align64);

#[test]
fn changing_value_and_link_preserves_key_in_native_base_tail_padding() {
    let mut tail = rustycad_tkernel::ncollection::MapLinkNode::new(std::ptr::null_mut());
    let successor = std::ptr::from_mut(&mut tail);
    let mut node = DataMapNode::new(91_u8, 7_u8, successor);
    *node.change_value() = 8;
    *node.link_mut().next_mut() = std::ptr::null_mut();
    assert_eq!((*node.key(), *node.value()), (91, 8));
    assert!(node.link().next().is_null());
}
#[test]
fn typed_allocation_preserves_overaligned_native_payload_layout() {
    let allocator = rustycad_tkernel::ncollection::BaseAllocator::common();
    let owner = DataMapNode::allocate_with(
        &allocator,
        std::ptr::null_mut(),
        || Align64 { byte: 91 },
        || Align32 { byte: 7 },
    );
    assert_eq!(std::ptr::from_ref(owner.key()).addr() % 64, 0);
    assert_eq!(std::ptr::from_ref(owner.value()).addr() % 32, 0);
    assert_eq!((owner.key().byte, owner.value().byte), (91, 7));
}
