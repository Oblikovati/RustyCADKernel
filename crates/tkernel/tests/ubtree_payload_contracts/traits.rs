use rustycad_tkernel::ncollection::{BoundingNode, BoundingVolume, ObjBnd, UBTree};
use std::panic::{RefUnwindSafe, UnwindSafe};
fn all_traits<T: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe>() {}
fn node_covariant<'a>(
    node: BoundingNode<&'static str, &'static str>,
) -> BoundingNode<&'a str, &'a str> {
    node
}
fn pair_covariant<'a>(pair: ObjBnd<&'static str, &'static str>) -> ObjBnd<&'a str, &'a str> {
    pair
}
#[test]
fn public_field_types_partial_moves_and_positive_auto_traits_remain_available() {
    all_traits::<BoundingNode<i32, i32>>();
    all_traits::<ObjBnd<i32, i32>>();
    let pair = ObjBnd {
        object: String::from("object"),
        bounds: String::from("bounds"),
    };
    let object: String = pair.object;
    assert_eq!(object, "object");
    assert_eq!(pair.bounds, "bounds");
    let _ = node_covariant;
    let _ = pair_covariant;
}
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
#[test]
fn node_copy_and_assignment_preserve_arena_link_values() {
    let mut tree = UBTree::default();
    tree.add(1, Bounds);
    tree.add(2, Bounds);
    let root = tree.root().unwrap();
    let copy = root.clone();
    assert_eq!(copy.child_id(0), root.child_id(0));
    let child = tree.child(root, 0).unwrap().clone();
    assert_eq!(child.parent_id(), tree.root_id());
    let mut target = BoundingNode::new(0, Bounds);
    target.clone_from(root);
    assert_eq!(target.child_id(1), root.child_id(1));
    target.clone_from(&child);
    assert!(target.is_leaf());
    assert_eq!(target.parent_id(), tree.root_id());
}

#[test]
// Declaring the holder first forces its payload referent to drop before the holder.
#[allow(clippy::needless_late_init)]
fn borrowed_payloads_keep_prior_drop_check_acceptance() {
    let node;
    let value = String::from("borrowed");
    node = BoundingNode::new(&value, ());
    assert_eq!(node.object().as_str(), "borrowed");
    let pair;
    let other = String::from("pair");
    pair = ObjBnd::new(&other, ());
    assert_eq!(pair.object.as_str(), "pair");
}
