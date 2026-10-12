//! Retained Current slots are actual source storage, not values copied when a reader is acquired.
use super::{Explorer, ExplorerShapeSlot};
use crate::topods::{Builder, Compound, Edge, Shape, Wire};
use rustycad_tkg3d::top_abs::ShapeEnum;
#[test]
fn retained_explorer_slots_reset_and_reuse_with_actual_same_storage() {
    let (root, edges) = two_wire_reference_fixture();
    let mut explorer = Explorer::new(&root, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let slot = explorer.current_shape_slot();
    let address = explorer.reference_shape(slot).unwrap() as *const Shape;
    assert!(explorer.reference_shape(slot).unwrap().is_same(&edges[0]));
    explorer.next().unwrap();
    assert!(explorer.reference_shape(slot).unwrap().is_same(&edges[1]));
    assert_same_reference_address(&explorer, slot, address);
    explorer.next().unwrap();
    assert!(!explorer.more());
    assert!(explorer.reference_shape(slot).unwrap().is_null());
    explorer.re_init().unwrap();
    assert!(explorer.reference_shape(slot).unwrap().is_same(&edges[0]));
    explorer.clear();
    assert!(explorer.reference_shape(slot).unwrap().is_null());
    assert_retained_root_source(&explorer, &root);
}
#[test]
fn root_slot_and_checked_nonexistent_storage_are_independent_of_current_depth() {
    let (root, _) = two_wire_reference_fixture();
    let mut explorer = Explorer::default();
    assert_eq!(explorer.current_shape_slot(), ExplorerShapeSlot::Root);
    assert!(
        explorer
            .reference_shape(ExplorerShapeSlot::Root)
            .unwrap()
            .is_null()
    );
    let failure = explorer
        .reference_shape(ExplorerShapeSlot::Iterator(99))
        .unwrap_err();
    assert!(failure.to_string().contains("99"));
    explorer
        .init(&root, ShapeEnum::Compound, ShapeEnum::Shape)
        .unwrap();
    assert_eq!(explorer.current_shape_slot(), ExplorerShapeSlot::Root);
    assert_retained_root_source(&explorer, &root);
}
fn two_wire_reference_fixture() -> (Compound, [Edge; 2]) {
    let builder = crate::brep::BRepBuilder::default();
    let mut root = Compound::default();
    Builder.make_compound(&mut root);
    let mut edges = [Edge::default(), Edge::default()];
    for edge in &mut edges {
        builder.make_edge(edge).unwrap();
        let mut wire = Wire::default();
        Builder.make_wire(&mut wire);
        Builder.add(&mut wire, edge).unwrap();
        Builder.add(&mut root, &wire).unwrap();
    }
    (root, edges)
}

fn assert_retained_root_source(explorer: &Explorer, root: &Shape) {
    let source = explorer.reference_shape(ExplorerShapeSlot::Root).unwrap();
    assert!(source.is_same(root));
}
fn assert_same_reference_address(
    explorer: &Explorer,
    slot: ExplorerShapeSlot,
    address: *const Shape,
) {
    let actual = explorer.reference_shape(slot).unwrap() as *const Shape;
    assert_eq!(address, actual);
}
