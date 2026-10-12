//! Checked cursor invalidation is a Martensite safety boundary, not a claim about dangling OCCT iterators.
use super::shape_drawer_oracle_inputs::ShapeDrawerOracleInputs;
use super::{AisShapeDrawerMap, shape_drawer_cursor::ShapeDrawerNativeCursor};
use rustycad_tkbrep::topods::{Builder, Compound};

#[test]
fn empty_and_exhausted_cursor_remain_exhausted() {
    let input = ShapeDrawerOracleInputs::new();
    let map = AisShapeDrawerMap::default();
    let mut empty = ShapeDrawerNativeCursor::new(&map).unwrap();
    assert!(empty.current().unwrap().is_none());
    empty.next().unwrap();
    map.bind(&input.key, &input.first).unwrap();
    let mut populated = ShapeDrawerNativeCursor::new(&map).unwrap();
    assert!(populated.current().unwrap().is_some());
    populated.next().unwrap();
    populated.next().unwrap();
    assert!(populated.current().unwrap().is_none());
    assert!(empty.current().unwrap().is_none());
}

#[test]
fn removed_current_node_fails_checked_traversal() {
    let input = ShapeDrawerOracleInputs::new();
    let map = AisShapeDrawerMap::default();
    map.bind(&input.key, &input.first).unwrap();
    let mut removed = ShapeDrawerNativeCursor::new(&map).unwrap();
    let lease = removed.current().unwrap().unwrap();
    map.unbind(&input.key);
    assert_eq!(
        removed.next().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    drop(lease);
    assert!(removed.current().is_err());
}

#[test]
fn replaced_bucket_table_fails_checked_traversal() {
    let input = ShapeDrawerOracleInputs::new();
    let map = AisShapeDrawerMap::default();
    map.bind(&input.key, &input.first).unwrap();
    let mut replaced = ShapeDrawerNativeCursor::new(&map).unwrap();
    grow_cursor_table(&map, &input);
    assert_eq!(
        replaced.next().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
}

fn grow_cursor_table(map: &AisShapeDrawerMap, input: &ShapeDrawerOracleInputs) {
    for _ in 0..103 {
        let mut compound = Compound::default();
        Builder.make_compound(&mut compound);
        map.bind(&compound.into_shape(), &input.first).unwrap();
    }
}
