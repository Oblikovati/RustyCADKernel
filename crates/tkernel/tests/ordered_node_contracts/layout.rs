use super::fixture::native_node_row;
use rustycad_tkernel::ncollection::{
    BaseAllocator, OrderedDataMapNode, OrderedDataMapNodeOwner, OrderedMapNode, OrderedMapNodeOwner,
};
#[derive(Default)]
#[repr(C, align(32))]
struct Wide {
    value: u8,
}
trait LayoutNode: Sized {
    type Owner: std::ops::Deref<Target = Self>;
    fn allocate(arena: &BaseAllocator) -> Self::Owner;
    fn make() -> Self;
    fn offsets(&mut self) -> [usize; 4];
}
fn relative<T>(value: &T, start: usize) -> usize {
    std::ptr::from_ref(value).addr() - start
}
impl<K: Default> LayoutNode for OrderedMapNode<K> {
    type Owner = OrderedMapNodeOwner<K>;
    fn allocate(arena: &BaseAllocator) -> Self::Owner {
        Self::allocate_with(arena, std::ptr::null_mut(), K::default)
    }
    fn make() -> Self {
        Self::new(K::default(), std::ptr::null_mut())
    }
    fn offsets(&mut self) -> [usize; 4] {
        let start = std::ptr::from_ref(self).addr();
        assert_eq!(relative(self.link(), start), 0);
        [
            relative(self.key(), start),
            relative(self.value(), start),
            relative(self.order_previous_mut(), start),
            relative(self.order_next_mut(), start),
        ]
    }
}
impl<K: Default, V: Default> LayoutNode for OrderedDataMapNode<K, V> {
    type Owner = OrderedDataMapNodeOwner<K, V>;
    fn allocate(arena: &BaseAllocator) -> Self::Owner {
        Self::allocate_with(arena, std::ptr::null_mut(), K::default, V::default)
    }
    fn make() -> Self {
        Self::new(K::default(), V::default(), std::ptr::null_mut())
    }
    fn offsets(&mut self) -> [usize; 4] {
        let start = std::ptr::from_ref(self).addr();
        assert_eq!(relative(self.link(), start), 0);
        [
            relative(self.key(), start),
            relative(self.value(), start),
            relative(self.order_previous_mut(), start),
            relative(self.order_next_mut(), start),
        ]
    }
}
fn layout<N: LayoutNode>(label: &str) {
    let mut node = N::make();
    let [key, value, previous, next] = node.offsets();
    let arena = BaseAllocator::incremental(4096);
    let _prefix = arena.allocate(16).unwrap();
    let first = N::allocate(&arena);
    let second = N::allocate(&arena);
    let span = std::ptr::from_ref(&*second).addr() - std::ptr::from_ref(&*first).addr();
    assert_eq!(span, size_of::<N>());
    native_node_row(
        label,
        format!(
            "{label} {} {} {key} {value} {previous} {next} {span}",
            size_of::<N>(),
            align_of::<N>()
        ),
    );
}
#[test]
fn set_native_size_alignment_offsets_and_allocation_span() {
    layout::<OrderedMapNode<i32>>("set_int");
    layout::<OrderedMapNode<u8>>("set_u8");
    layout::<OrderedMapNode<Wide>>("set_wide");
}
#[test]
fn data_native_scalar_size_alignment_offsets_and_allocation_span() {
    layout::<OrderedDataMapNode<i32, i32>>("data_int_int");
    layout::<OrderedDataMapNode<u8, u8>>("data_u8_u8");
    layout::<OrderedDataMapNode<i32, u8>>("data_int_u8");
    layout::<OrderedDataMapNode<u8, i32>>("data_u8_int");
}
#[test]
fn data_native_aligned_size_alignment_offsets_and_allocation_span() {
    layout::<OrderedDataMapNode<Wide, u8>>("data_wide_u8");
    layout::<OrderedDataMapNode<u8, Wide>>("data_u8_wide");
    layout::<OrderedDataMapNode<Wide, Wide>>("data_wide_wide");
}
