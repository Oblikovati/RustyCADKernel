use rustycad_tkernel::ncollection::{BoundingVolume, UBTree, UBTreeFiller};
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
fn inherited_traits<T: Send + Sync + Unpin>(_: &T) {}
#[test]
fn policy_can_borrow_local_state_and_object_without_default_bounds() {
    let text = String::from("borrowed");
    let mut visits = 0;
    let mut tree = UBTree::default();
    {
        let mut filler =
            UBTreeFiller::new(&mut tree, None, true).with_self_assignment_policy(|pair| {
                assert_eq!(pair.object, text.as_str());
                visits += 1;
            });
        filler.add(text.as_str(), CloneOnlyBounds);
        assert_eq!(filler.fill(), 1);
    }
    assert_eq!(visits, 1);
    assert_eq!(*tree.root().unwrap().object(), text);
}
#[test]
fn inferred_ordinary_and_owning_filler_keep_thread_and_pin_traits() {
    let mut tree = UBTree::<i32, CloneOnlyBounds>::default();
    let ordinary = UBTreeFiller::new(&mut tree, None, true);
    inherited_traits(&ordinary);
    let mut filler = ordinary.with_self_assignment_policy(|_| {});
    inherited_traits(&filler);
    filler.add(1, CloneOnlyBounds);
    std::thread::scope(|scope| scope.spawn(move || drop(filler)).join().unwrap());
    assert_eq!(*tree.root().unwrap().object(), 1);
}
