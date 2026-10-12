//! Actual map ownership and aliases replay the pinned native specialization; erased-node reads are checked extensions.
use super::*;
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::topods::{Builder, Compound};
use rustycad_tkernel::standard::Handle;
use std::fmt::Write;
#[test]
fn actual_compound_group_map_replays_native_borrowed_key_value_lifetimes() {
    let parent = Handle::new(Prs3dDrawer::new());
    let drawer = Handle::new(AisColoredDrawer::new(&parent));
    let mut first = Compound::default();
    let mut second = Compound::default();
    Builder.make_compound(&mut first);
    Builder.make_compound(&mut second);
    let map = AisColoredDrawerCompounds::default();
    let mut rows = String::new();
    writeln!(
        rows,
        "add {} {} {}",
        map.add(&drawer, &first).unwrap(),
        drawer.ref_count(),
        first.tshape().ref_count()
    )
    .unwrap();
    let alias = map.node_ref(1).unwrap();
    record_compound_aliases(&mut rows, &map, &alias, &drawer, &first, &second);
    map.clear(false);
    writeln!(
        rows,
        "cleared {} {} {}",
        drawer.ref_count(),
        second.tshape().ref_count(),
        map.len()
    )
    .unwrap();
    let native = include_str!("../../tests/fixtures/colored-shape-compute/native-compound-map.txt");
    assert_eq!(
        rows,
        native.lines().take(5).collect::<Vec<_>>().join("\n") + "\n"
    );
    assert!(alias.clone_compound().is_err());
}
fn record_compound_aliases(
    rows: &mut String,
    map: &AisColoredDrawerCompounds,
    alias: &AisColoredCompoundRef<'_>,
    drawer: &Handle<AisColoredDrawer>,
    first: &Compound,
    second: &Compound,
) {
    let key_address = alias.drawer_address().unwrap();
    let value_address = alias.compound_address().unwrap();
    let index = map.add(drawer, second).unwrap();
    let preserved = alias.clone_compound().unwrap().is_same(first);
    writeln!(rows, "duplicate {index} {}", i32::from(preserved)).unwrap();
    map.resize(1000);
    writeln!(
        rows,
        "rehash {} {}",
        i32::from(alias.drawer_address().unwrap() == key_address),
        i32::from(alias.compound_address().unwrap() == value_address)
    )
    .unwrap();
    alias.assign_compound(second).unwrap();
    let changed = alias.clone_compound().unwrap().is_same(second);
    writeln!(
        rows,
        "changed {} {} {}",
        i32::from(changed),
        first.tshape().ref_count(),
        second.tshape().ref_count()
    )
    .unwrap();
}

#[test]
fn native_clear_keeps_value_reference_live_through_key_destructor_then_releases_all_ownership() {
    use super::colored_compound_drop_probe::ColoredCompoundKeyDropFake;
    use std::sync::{Arc, Mutex};
    let parent = Handle::new(Prs3dDrawer::new());
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    let map = Arc::new(AisColoredDrawerCompounds::default());
    let rows = Arc::new(Mutex::new(String::new()));
    let drawer = ColoredCompoundKeyDropFake::new(&parent, &map, &rows);
    let key: Handle<AisColoredDrawer> = drawer.cast();
    map.add(&key, &compound).unwrap();
    drop(drawer);
    drop(key);
    map.clear(false);
    writeln!(
        rows.lock().unwrap(),
        "release-complete {} {} {}",
        map.len(),
        parent.ref_count(),
        compound.tshape().ref_count()
    )
    .unwrap();
    let native = include_str!("../../tests/fixtures/colored-shape-compute/native-compound-map.txt");
    assert_eq!(
        *rows.lock().unwrap(),
        native.lines().skip(5).collect::<Vec<_>>().join("\n") + "\n"
    );
}
#[test]
fn native_value_assignment_keeps_old_pointer_at_zero_before_retaining_incoming() {
    use super::colored_compound_value_drop_probe::ColoredCompoundValueDropFake;
    use rustycad_tkbrep::topods::Shape;
    use rustycad_tkg3d::top_abs::Orientation;
    use rustycad_tkmath::toploc::Location;
    use std::sync::{Arc, Mutex};
    let parent = Handle::new(Prs3dDrawer::new());
    let drawer = Handle::new(AisColoredDrawer::new(&parent));
    let map = Arc::new(AisColoredDrawerCompounds::default());
    let mut replacement = Compound::default();
    Builder.make_compound(&mut replacement);
    let rows = Arc::new(Mutex::new(String::new()));
    let old = ColoredCompoundValueDropFake::new(&map, replacement.tshape(), &rows);
    let old = Compound::unchecked(Shape::from_parts(
        old.into_cast(),
        Location::default(),
        Orientation::External,
    ));
    map.add(&drawer, &old).unwrap();
    drop(old);
    record_native_compound_assignment(&map, &replacement, &rows);
    assert_eq!(
        *rows.lock().unwrap(),
        include_str!("../../tests/fixtures/colored-shape-compute/native-compound-assignment.txt")
    );
}
fn record_native_compound_assignment(
    map: &AisColoredDrawerCompounds,
    replacement: &Compound,
    rows: &std::sync::Mutex<String>,
) {
    let alias = map.node_ref(1).unwrap();
    alias.assign_compound(replacement).unwrap();
    let same = alias.clone_compound().unwrap().is_same(replacement);
    writeln!(
        rows.lock().unwrap(),
        "assigned {} {}",
        i32::from(same),
        replacement.tshape().ref_count()
    )
    .unwrap();
    map.clear(false);
    writeln!(
        rows.lock().unwrap(),
        "assignment-cleared {}",
        replacement.tshape().ref_count()
    )
    .unwrap();
}
#[test]
fn native_const_compound_projection_is_uncounted_and_explicit_copy_is_owning() {
    let parent = Handle::new(Prs3dDrawer::new());
    let drawer = Handle::new(AisColoredDrawer::new(&parent));
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    let map = AisColoredDrawerCompounds::default();
    map.add(&drawer, &compound).unwrap();
    let mut rows = String::new();
    record_compound_reference(&map, &compound, &mut rows);
    map.clear(false);
    writeln!(rows, "reference-finished {}", compound.tshape().ref_count()).unwrap();
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-compute/native-compound-reference.txt")
    );
}
fn record_compound_reference(
    map: &AisColoredDrawerCompounds,
    compound: &Compound,
    rows: &mut String,
) {
    map.node_ref(1)
        .unwrap()
        .with_shape(|shape| {
            writeln!(
                rows,
                "referenced {} {}",
                i32::from(shape.is_equal(compound)),
                shape.tshape().ref_count()
            )
            .unwrap();
            let copy = shape.clone();
            writeln!(
                rows,
                "copied {} {}",
                i32::from(copy.is_equal(compound)),
                shape.tshape().ref_count()
            )
            .unwrap();
            drop(copy);
            writeln!(rows, "copy-released {}", shape.tshape().ref_count()).unwrap();
            Ok(())
        })
        .unwrap();
}
