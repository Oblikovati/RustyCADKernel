use super::caught;
use rustycad_tkernel::ncollection::{BoundingVolume, ObjBnd, UBTree, UBTreeFiller};
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
#[test]
fn explicit_capabilities_keep_clone_only_bounds_and_nonstatic_object_borrows() {
    let text = String::from("borrowed");
    let mut tree = UBTree::default();
    {
        let mut filler = UBTreeFiller::new(&mut tree, None, true);
        filler.add_ref_with_constructor(&text.as_str(), &CloneOnlyBounds, |pair| pair.clone());
        let count = filler.fill_with_self_assignment(|pair| {
            pair.assign_self_with(|object| assert_eq!(*object, text), |_| {})
        });
        assert_eq!(count, 1);
    }
    assert_eq!(*tree.root().unwrap().object(), text);
}
#[test]
fn self_assignment_callbacks_receive_retained_fields_and_stop_after_object_failure() {
    let mut pair = ObjBnd::new(9, 90);
    let addresses = (
        std::ptr::from_ref(&pair.object).addr(),
        std::ptr::from_ref(&pair.bounds).addr(),
    );
    pair.assign_self_with(
        |object| assert_eq!(std::ptr::from_mut(object).addr(), addresses.0),
        |bounds| assert_eq!(std::ptr::from_mut(bounds).addr(), addresses.1),
    );
    assert!(caught(|| pair.assign_self_with(
        |object| {
            *object = 17;
            panic!("object self assignment rejected; expected permitted assignment");
        },
        |_| panic!("bounds callback must be skipped after object failure")
    )));
    assert_eq!((pair.object, pair.bounds), (17, 90));
}
