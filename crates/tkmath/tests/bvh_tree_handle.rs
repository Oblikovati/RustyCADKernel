//! Retained native tree identity across builds, traversal adapters and failure.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::*;
use std::sync::Arc;
#[derive(Debug)]
struct PartialTreeBuilder {
    parameters: BvhBuilderTransient,
}
impl BvhBuilder<BvhVec3d> for PartialTreeBuilder {
    fn builder_transient(&self) -> &BvhBuilderTransient {
        &self.parameters
    }
    fn build(
        &self,
        _set: &mut dyn BvhSet<BvhVec3d>,
        tree: &mut BvhTree<BvhVec3d>,
        bounds: &BvhBox<BvhVec3d>,
    ) -> Result<(), Failure> {
        tree.clear();
        tree.add_leaf_node_box(bounds, 0, 0);
        Err(Failure::with_message(
            "Standard_ProgramError",
            "injected partial build; expected caller to observe failure",
        ))
    }
}
#[test]
fn retained_tree_survives_rebuild_and_set_destruction() {
    let mut set = BvhBoxSet::<BvhVec3d>::default();
    let retained = set.bvh_handle().unwrap();
    assert_eq!(retained.read().length(), 0);
    set.add(1, BvhBox::from_point(BvhVec3d::new(1., 2., 3.)));
    let rebuilt = set.bvh_handle().unwrap();
    assert!(retained.ptr_eq(&rebuilt));
    assert_eq!(retained.read().length(), 1);
    drop(set);
    assert_eq!(retained.read().min_point(0).unwrap().x, 1.);
    rebuilt.write().clear();
    assert_eq!(retained.read().length(), 0);
    assert!(!retained.ptr_eq(&BvhTreeHandle::default()));
}
#[test]
fn traversal_adapter_does_not_empty_a_retained_tree() {
    let mut set = BvhBoxSet::<BvhVec3d>::default();
    set.add(1, BvhBox::from_point(BvhVec3d::new(1., 2., 3.)));
    let retained = set.bvh_handle().unwrap();
    let tree = set.take_primitive_bvh().unwrap();
    assert_eq!(retained.read().length(), 1);
    set.restore_primitive_bvh(tree);
    assert_eq!(retained.read().length(), 1);
    assert!(retained.ptr_eq(&set.bvh_handle().unwrap()));
}
#[test]
fn partial_build_failure_preserves_identity_and_dirty_state() {
    let mut set = BvhBoxSet::<BvhVec3d>::new(Some(Arc::new(PartialTreeBuilder {
        parameters: BvhBuilderTransient::new(1, 8),
    })));
    let retained = set.bvh_handle().unwrap();
    set.add(1, BvhBox::from_point(BvhVec3d::new(1., 2., 3.)));
    assert_eq!(
        set.bvh_handle().unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
    assert_eq!(retained.read().length(), 1);
    assert!(set.is_dirty());
    set.set_builder(None);
    assert_eq!(
        set.bvh_handle().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    assert_eq!(retained.read().length(), 1);
}
