//! Standalone layout comparison probe; differences remain explicit and receive no parity credit.
use rustycad_tkernel::ncollection::{DataMapNode, ListValueNode, MapNode};
fn data_layout<K: Default, V: Default>(label: &str) {
    let node = DataMapNode::new(K::default(), V::default(), std::ptr::null_mut());
    let start = std::ptr::from_ref(&node).addr();
    println!(
        "{label} {} {} {} {} {}",
        size_of::<DataMapNode<K, V>>(),
        align_of::<DataMapNode<K, V>>(),
        std::ptr::from_ref(node.key()).addr() - start,
        std::ptr::from_ref(node.value()).addr() - start,
        size_of::<ListValueNode<V>>()
    );
}
fn map_layout<K: Default>(label: &str) {
    let node = MapNode::new(K::default(), std::ptr::null_mut());
    let start = std::ptr::from_ref(&node).addr();
    println!(
        "{label} {} {} {} {}",
        size_of::<MapNode<K>>(),
        align_of::<MapNode<K>>(),
        std::ptr::from_ref(node.key()).addr() - start,
        size_of::<ListValueNode<K>>()
    );
}
fn main() {
    data_layout::<i32, i32>("data_int_int");
    data_layout::<i8, i32>("data_char_int");
    data_layout::<i32, i8>("data_int_char");
    data_layout::<i8, i8>("data_char_char");
    data_layout::<f64, i32>("data_double_int");
    data_layout::<i32, f64>("data_int_double");
    data_layout::<i64, i8>("data_long_char");
    data_layout::<i8, i64>("data_char_long");
    map_layout::<i32>("map_int");
    map_layout::<i8>("map_char");
    map_layout::<f64>("map_double");
}
