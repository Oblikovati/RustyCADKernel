use rustycad_tkernel::ncollection::{
    BoundingTreeInsertion, BoundingVolume, EBTree, NativeBoundingTree, UBTree, UBTreeFiller,
};
#[derive(Clone, Default)]
struct Bounds;
impl BoundingVolume for Bounds {
    fn add(&mut self, _: &Self) {}
    fn is_out(&self, _: &Self) -> bool {
        false
    }
    fn square_extent(&self) -> f64 {
        0.0
    }
}
#[derive(Clone)]
struct CloneOnlyBounds;
impl BoundingVolume for CloneOnlyBounds {
    fn add(&mut self, _: &Self) {}
    fn is_out(&self, _: &Self) -> bool {
        false
    }
    fn square_extent(&self) -> f64 {
        0.0
    }
}
fn traits<T: Send + Sync + Unpin + std::panic::RefUnwindSafe>() {}
fn shorten<'long: 'short, 'short, Tree>(
    target: NativeBoundingTree<'long, Tree>,
) -> NativeBoundingTree<'short, Tree> {
    target
}
#[test]
fn borrowed_wrapper_preserves_conditional_thread_and_reference_traits() {
    traits::<NativeBoundingTree<'static, UBTree<i32, Bounds>>>();
    traits::<NativeBoundingTree<'static, EBTree<i32, Bounds>>>();
    traits::<UBTreeFiller<'static, i32, Bounds>>();
    traits::<UBTreeFiller<'static, i32, Bounds, NativeBoundingTree<'static, UBTree<i32, Bounds>>>>(
    );
    let mut tree = UBTree::default();
    let mut target = shorten(tree.native_insertion());
    std::thread::scope(|scope| {
        scope
            .spawn(move || assert!(target.insert_bound(1, Bounds)))
            .join()
            .unwrap();
    });
    assert_eq!(*tree.root().unwrap().object(), 1);
}
#[test]
fn owned_trait_entry_and_common_tree_view_use_the_actual_editable_tree() {
    let mut tree = EBTree::default();
    {
        let mut target = tree.native_insertion();
        assert!(target.insert_bound(1, Bounds));
        assert!(!target.insert_bound(1, Bounds));
        assert_eq!(*target.tree_view().root().unwrap().object(), 1);
    }
    assert!(tree.contains(&1));
}
#[test]
fn existing_filler_constructor_keeps_clone_only_bound_inference() {
    let mut tree = UBTree::default();
    {
        let mut filler = UBTreeFiller::new(&mut tree, None, true);
        filler.add(1, CloneOnlyBounds);
        assert_eq!(filler.fill(), 1);
    }
    assert_eq!(*tree.root().unwrap().object(), 1);
}
fn borrowed_fill<'a>(text: &'a str) {
    let mut tree = UBTree::<&'a str, Bounds>::default();
    {
        let mut target = tree.native_insertion();
        let target: &mut dyn BoundingTreeInsertion<&'a str, Bounds> = &mut target;
        let mut filler = UBTreeFiller::new(target, None, true);
        filler.add_ref(&text, &Bounds);
    }
    assert_eq!(*tree.root().unwrap().object(), text);
}
#[test]
fn nonstatic_borrowed_objects_support_erased_native_target_and_drop_fill() {
    let text = String::from("borrowed");
    borrowed_fill(text.as_str());
}
