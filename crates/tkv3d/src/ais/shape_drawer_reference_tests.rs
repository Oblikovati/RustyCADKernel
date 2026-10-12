//! Additional Rust safety regressions do not invent native behavior for erased/destroyed references.
use super::{AisShapeDrawerMap, shape_drawer_oracle_inputs::ShapeDrawerOracleInputs};
#[test]
fn shape_drawer_const_mutable_container_and_node_aliases_share_actual_storage() {
    let input = ShapeDrawerOracleInputs::new();
    let map = AisShapeDrawerMap::default();
    let read = map.as_const();
    let write = map.as_mut();
    assert!(write.bind(&input.key, &input.first).unwrap());
    let first = read.node_ref(&input.key).unwrap();
    let reversed = read.node_ref(&input.reversed).unwrap();
    assert!(first.is_same_reference(&reversed));
    assert!(read.is_same_reference(&write.as_const()));
    assert!(!write.unbind(&input.located));
    assert!(write.unbind(&input.key));
    assert!(read.is_empty());
    write.clear(true);
}
#[test]
fn shape_drawer_erased_aliases_fail_instead_of_following_a_rebound_equivalent_key() {
    let input = ShapeDrawerOracleInputs::new();
    let map = AisShapeDrawerMap::default();
    map.bind(&input.key, &input.first).unwrap();
    let old = map.node_ref(&input.key).unwrap();
    let mutable = map.change_value_ref(&input.key).unwrap();
    assert!(map.unbind(&input.key));
    map.bind(&input.reversed, &input.second).unwrap();
    let current = map.node_ref(&input.key).unwrap();
    assert!(!old.is_same_reference(&current));
    let read_failure = old.orientation().unwrap_err();
    let write_failure = mutable.assign(&input.first).unwrap_err();
    assert_eq!(read_failure.exception_type(), "Standard_NoSuchObject");
    assert_eq!(write_failure.exception_type(), "Standard_NoSuchObject");
    assert_eq!(
        current.drawer_address().unwrap(),
        input.second.object_address()
    );
}
#[test]
fn shape_drawer_removed_storage_debug_and_checked_read_have_no_owned_fields() {
    let input = ShapeDrawerOracleInputs::new();
    let map = AisShapeDrawerMap::default();
    map.bind(&input.key, &input.first).unwrap();
    let alias = map.node_ref(&input.key).unwrap();
    let storage_lease = alias.live_node().unwrap();
    assert!(format!("{storage_lease:?}").contains("Some(Shape"));
    map.clear(true);
    assert_eq!(
        format!("{storage_lease:?}"),
        "ShapeDrawerNode { key: None }"
    );
    let failure = alias.drawer_address().unwrap_err();
    assert!(failure.message().contains("fields=None"));
    assert_eq!(input.key.tshape().ref_count(), 3);
    assert_eq!(input.first.ref_count(), 1);
}
