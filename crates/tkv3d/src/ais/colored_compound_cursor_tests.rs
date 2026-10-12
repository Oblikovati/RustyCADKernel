//! The actual indexed iterator freezes no extent and keeps node addresses through rehash.
use super::{
    AisColoredDrawer, AisColoredDrawerCompounds,
    colored_compound_cursor::ColoredCompoundNativeCursor,
};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::topods::{Builder, Compound};
use rustycad_tkernel::standard::Handle;
use std::fmt::Write;
#[test]
fn native_indexed_cursor_observes_add_resize_next_and_clear() {
    let parent = Handle::new(Prs3dDrawer::new());
    let first = Handle::new(AisColoredDrawer::new(&parent));
    let second = Handle::new(AisColoredDrawer::new(&parent));
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    let map = AisColoredDrawerCompounds::default();
    let mut rows = String::new();
    record_native_cursor_additions(&map, &first, &second, &compound, &mut rows);
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-compute/native-compound-cursor.txt")
    );
}
fn record_native_cursor_additions(
    map: &AisColoredDrawerCompounds,
    first: &Handle<AisColoredDrawer>,
    second: &Handle<AisColoredDrawer>,
    compound: &Compound,
    rows: &mut String,
) {
    let mut cursor = ColoredCompoundNativeCursor::new(map);
    writeln!(
        rows,
        "cursor-empty {}",
        i32::from(cursor.current().unwrap().is_some())
    )
    .unwrap();
    map.add(first, compound).unwrap();
    let current = cursor.current().unwrap().unwrap();
    let key = current.live_node().unwrap().drawer.object_address() == first.object_address();
    writeln!(rows, "cursor-added 1 {}", i32::from(key)).unwrap();
    let address = current.compound_address().unwrap();
    map.resize(1000);
    map.add(second, compound).unwrap();
    let stable = cursor
        .current()
        .unwrap()
        .unwrap()
        .compound_address()
        .unwrap()
        == address;
    writeln!(rows, "cursor-resized 1 {}", i32::from(stable)).unwrap();
    cursor.next();
    record_native_cursor_tail(map, &cursor, second, rows);
}
fn record_native_cursor_tail(
    map: &AisColoredDrawerCompounds,
    cursor: &ColoredCompoundNativeCursor<'_>,
    second: &Handle<AisColoredDrawer>,
    rows: &mut String,
) {
    let current = cursor.current().unwrap().unwrap();
    let key = current.live_node().unwrap().drawer.object_address() == second.object_address();
    writeln!(rows, "cursor-next 1 {}", i32::from(key)).unwrap();
    map.clear(false);
    writeln!(
        rows,
        "cursor-cleared {}",
        i32::from(cursor.current().unwrap().is_some())
    )
    .unwrap();
}
