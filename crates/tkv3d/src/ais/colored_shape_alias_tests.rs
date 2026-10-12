//! Native same-map insertion is live and keeps key/value reference counts and slots.
use super::*;
use super::{
    colored_shape_alias_inputs::ColoredAliasInputs, shape_drawer_cursor::ShapeDrawerNativeCursor,
};
use rustycad_tkernel::standard::Handle;
use std::fmt::Write;
#[test]
fn real_geometry_same_map_expansion_replays_native_live_aliases() {
    let input = ColoredAliasInputs::new();
    let object = AisColoredShape::new(&input.base);
    let drawer = object.custom_aspects(&input.wire).unwrap();
    let map = &object.colored_shape_base().shape_colors;
    let mut output = String::new();
    replay_live_alias(&input, map, &drawer, &mut output);
    map.clear(false);
    map.bind(&input.wire, &drawer).unwrap();
    object.fill_subshape_drawer_map(map).unwrap();
    let same = i32::from(
        map.node_ref(&input.edge).unwrap().drawer_address().unwrap() == drawer.object_address(),
    );
    writeln!(output, "same-output {} {same}", map.len()).unwrap();
    assert_eq!(
        output,
        include_str!("../../tests/fixtures/colored-shape-expansion/native-alias.txt")
    );
}
fn replay_live_alias(
    input: &ColoredAliasInputs,
    map: &AisShapeDrawerMap,
    drawer: &Handle<AisColoredDrawer>,
    output: &mut String,
) {
    let mut cursor = ShapeDrawerNativeCursor::new(map).unwrap();
    let reference = map.node_ref(&input.wire).unwrap();
    let addresses = (
        reference.key_reference_address().unwrap(),
        reference.value_reference_address().unwrap(),
    );
    record_alias_borrowed(input, drawer, output);
    insert_alias_current(&cursor, map);
    record_alias_inserted(input, &reference, drawer, addresses, output);
    record_cursor_remainder(&mut cursor, input, output);
}
fn record_alias_borrowed(
    input: &ColoredAliasInputs,
    drawer: &Handle<AisColoredDrawer>,
    output: &mut String,
) {
    let keys = input.wire.tshape().ref_count();
    let values = drawer.ref_count();
    writeln!(output, "borrowed {keys} {values}").unwrap();
}
fn insert_alias_current(cursor: &ShapeDrawerNativeCursor<'_>, map: &AisShapeDrawerMap) {
    let node = cursor.current().unwrap().unwrap();
    node.with_entry(|key, value| {
        colored_shape_expansion::bind_colored_shape_leaves(map, key, value)
    })
    .unwrap()
    .unwrap();
}
fn record_cursor_remainder(
    cursor: &mut ShapeDrawerNativeCursor<'_>,
    input: &ColoredAliasInputs,
    output: &mut String,
) {
    cursor.next().unwrap();
    let current = cursor.current().unwrap().unwrap();
    let same = current
        .read_fields(|entry| entry.key.is_same(&input.edge))
        .unwrap();
    writeln!(output, "visited-new 1 {}", i32::from(same)).unwrap();
    cursor.next().unwrap();
    let more = i32::from(cursor.current().unwrap().is_some());
    writeln!(output, "end {more}").unwrap();
}

fn record_alias_inserted(
    input: &ColoredAliasInputs,
    reference: &AisShapeDrawerNodeRef<'_>,
    drawer: &Handle<AisColoredDrawer>,
    addresses: (*const (), *const ()),
    output: &mut String,
) {
    let keys = input.wire.tshape().ref_count();
    let values = drawer.ref_count();
    let key_same = i32::from(reference.key_reference_address().unwrap() == addresses.0);
    let value_same = i32::from(reference.value_reference_address().unwrap() == addresses.1);
    writeln!(output, "inserted {keys} {values} {key_same} {value_same}").unwrap();
}
