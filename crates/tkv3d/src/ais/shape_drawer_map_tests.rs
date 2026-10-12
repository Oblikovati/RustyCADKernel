//! Native owned specialization replay uses actual BRep keys and real colored drawer handles.
use super::shape_drawer_oracle_rows::shape_drawer_row;
use super::{
    AisShapeDrawerMap,
    shape_drawer_oracle_inputs::{ShapeDrawerOracleInputs, shape_drawer_vertex},
};
use rustycad_tkbrep::{topods::Shape, toptools::ShapeMapHasher};
use rustycad_tkernel::{ncollection::HashPolicy, standard::Handle};
use rustycad_tkmath::gp::Point3;
use std::fmt::Write;
fn map_counts(
    output: &mut String,
    label: &str,
    map: &AisShapeDrawerMap,
    input: &ShapeDrawerOracleInputs,
) {
    let extent = map.len();
    let key_refs = input.key.tshape().ref_count();
    let handles = format!(
        "{} {} {}",
        input.first.ref_count(),
        input.second.ref_count(),
        input.parent.ref_count()
    );
    writeln!(output, "{label} {extent} {key_refs} {handles}").unwrap();
}
fn record_key_hash_identity(output: &mut String, input: &ShapeDrawerOracleInputs) {
    let same = i32::from(input.key.is_same(&input.reversed));
    let equal = i32::from(input.key.is_equal(&input.reversed));
    let hash_equal =
        i32::from(ShapeMapHasher.hash(&input.key) == ShapeMapHasher.hash(&input.reversed));
    let located_same = i32::from(input.key.is_same(&input.located));
    shape_drawer_row(output, "identity", &[same, equal, hash_equal, located_same]);
}
fn map_key_identity(output: &mut String, map: &AisShapeDrawerMap, input: &ShapeDrawerOracleInputs) {
    record_key_hash_identity(output, input);
    let bound = i32::from(map.bind(&input.key, &input.first).unwrap());
    shape_drawer_row(output, "first-bind", &[bound]);
    map_replace_identity(output, map, input);
    map_counts(output, "replace-counts", map, input);
    map.change_value_ref(&input.key)
        .unwrap()
        .assign(&input.first)
        .unwrap();
    map_counts(output, "assign-counts", map, input);
    input.key.set_closed(true).unwrap();
    let closed = i32::from(map.node_ref(&input.key).unwrap().key_is_closed().unwrap());
    shape_drawer_row(output, "shared-key", &[closed]);
}
fn map_replace_identity(
    output: &mut String,
    map: &AisShapeDrawerMap,
    input: &ShapeDrawerOracleInputs,
) {
    let original = map.node_ref(&input.key).unwrap();
    let new = i32::from(map.bind(&input.reversed, &input.second).unwrap());
    let orientation = original.orientation().unwrap() as i32;
    let value = i32::from(original.drawer_address().unwrap() == input.second.object_address());
    let current = map.node_ref(&input.key).unwrap();
    let same_slot = i32::from(
        original.value_reference_address().unwrap() == current.value_reference_address().unwrap(),
    );
    shape_drawer_row(
        output,
        "replace-bind",
        &[new, orientation, value, same_slot],
    );
}
fn fill_shape_drawer_map(map: &AisShapeDrawerMap) -> Vec<Shape> {
    let mut keys = Vec::new();
    for index in 0..128 {
        let key = shape_drawer_vertex(Point3::new(f64::from(index), 8., 9.));
        map.bind(&key, &Handle::default()).unwrap();
        keys.push(key);
    }
    keys
}
fn map_growth(output: &mut String, map: &AisShapeDrawerMap, input: &ShapeDrawerOracleInputs) {
    let original = map.node_ref(&input.key).unwrap();
    let owning_value = original.clone_drawer().unwrap();
    map.bind(&input.reversed, &input.second).unwrap();
    let key_address = original.key_reference_address().unwrap();
    let value_address = original.value_reference_address().unwrap();
    map.resize(1000);
    let keys = fill_shape_drawer_map(map);
    record_growth_addresses(output, map, &original, key_address, value_address, input);
    map_counts(output, "growth-counts", map, input);
    remove_with_unused_aliases(output, map, input, &keys);
    drop(owning_value);
}
fn record_growth_addresses(
    output: &mut String,
    map: &AisShapeDrawerMap,
    original: &super::AisShapeDrawerNodeRef<'_>,
    key_address: *const (),
    value_address: *const (),
    input: &ShapeDrawerOracleInputs,
) {
    let buckets = map.bucket_count();
    let key_stable = i32::from(original.key_reference_address().unwrap() == key_address);
    let value_stable = i32::from(original.value_reference_address().unwrap() == value_address);
    let assigned = i32::from(original.drawer_address().unwrap() == input.second.object_address());
    writeln!(
        output,
        "growth {buckets} {key_stable} {value_stable} {assigned}"
    )
    .unwrap();
}
fn remove_with_unused_aliases(
    output: &mut String,
    map: &AisShapeDrawerMap,
    input: &ShapeDrawerOracleInputs,
    keys: &[Shape],
) {
    let unused_alias = map.node_ref(&input.key).unwrap();
    let storage_lease = unused_alias.live_node().unwrap();
    let last_key = keys.last().unwrap();
    let survivor = map.node_ref(last_key).unwrap();
    let original_address = survivor.key_reference_address().unwrap();
    assert!(map.unbind(&input.key));
    assert!(storage_lease.fields.read().is_none());
    let same = i32::from(survivor.is_same_key(last_key).unwrap());
    let stable = i32::from(survivor.key_reference_address().unwrap() == original_address);
    shape_drawer_row(output, "survivor", &[same, stable]);
    map_counts(output, "unbind-unused-alias", map, input);
    record_missing_and_clear(output, map, input, keys);
}
fn record_missing_and_clear(
    output: &mut String,
    map: &AisShapeDrawerMap,
    input: &ShapeDrawerOracleInputs,
    keys: &[Shape],
) {
    let failure = map.node_ref(&input.key).err().unwrap();
    writeln!(output, "missing-key {}", failure.exception_type()).unwrap();
    let last_key = keys.last().unwrap();
    let storage_lease = map.node_ref(last_key).unwrap().live_node().unwrap();
    map.clear(false);
    map_counts(output, "clear-unused-alias", map, input);
    assert!(storage_lease.fields.read().is_none());
    shape_drawer_row(output, "clear-last-key", &[last_key.tshape().ref_count()]);
}
fn map_locations_and_null(
    output: &mut String,
    map: &AisShapeDrawerMap,
    input: &mut ShapeDrawerOracleInputs,
) {
    map.bind(&input.key, &input.first).unwrap();
    map.bind(&input.located, &input.second).unwrap();
    let located = map.node_ref(&input.located).unwrap();
    let same = i32::from(located.is_same_key(&input.located).unwrap());
    let base_same = i32::from(located.is_same_key(&input.key).unwrap());
    writeln!(output, "location {} {same} {base_same}", map.len()).unwrap();
    let datum = location_ownership(output, map, input);
    map_counts(output, "location-counts", map, input);
    record_null_key(output, map, input);
    shape_drawer_row(output, "location-datum-released", &[datum.ref_count()]);
}
fn record_null_key(output: &mut String, map: &AisShapeDrawerMap, input: &ShapeDrawerOracleInputs) {
    let null_key = Shape::default();
    map.bind(&null_key, &input.first).unwrap();
    let node = map.node_ref(&null_key.reversed()).unwrap();
    let bound = i32::from(map.is_bound(&null_key.reversed()));
    let orientation = node.orientation().unwrap() as i32;
    shape_drawer_row(output, "null-key", &[bound, orientation]);
    map.clear(true);
    map_counts(output, "clear-release", map, input);
}

#[test]
fn shape_drawer_native_map_identity_growth_replacement_and_ownership() {
    let mut input = ShapeDrawerOracleInputs::new();
    let map = AisShapeDrawerMap::default();
    let mut output = String::new();
    map_counts(&mut output, "initial", &map, &input);
    map_key_identity(&mut output, &map, &input);
    explicit_key_snapshot(&mut output, &map, &input);
    map_growth(&mut output, &map, &input);
    map_locations_and_null(&mut output, &map, &mut input);
    destruction_with_storage_lease(&mut output, &input);
    replacement_rehash(&mut output);
    assert_eq!(
        output,
        include_str!("../../tests/fixtures/shape-drawer-map/native-map.txt")
    );
}

fn explicit_key_snapshot(
    output: &mut String,
    map: &AisShapeDrawerMap,
    input: &ShapeDrawerOracleInputs,
) {
    let alias = map.node_ref(&input.key).unwrap();
    let owned_key = alias.key_snapshot().unwrap();
    let same = i32::from(owned_key.is_same(&input.key));
    let orientation = owned_key.orientation() as i32;
    let key_refs = input.key.tshape().ref_count();
    let value_refs = alias.drawer_ref_count().unwrap();
    writeln!(
        output,
        "explicit-copy {same} {orientation} {key_refs} {value_refs}"
    )
    .unwrap();
}

fn destruction_with_storage_lease(output: &mut String, input: &ShapeDrawerOracleInputs) {
    let storage_lease;
    {
        let local = AisShapeDrawerMap::default();
        local.bind(&input.key, &input.first).unwrap();
        storage_lease = local.node_ref(&input.key).unwrap().live_node().unwrap();
    }
    assert!(storage_lease.fields.read().is_none());
    let key_refs = input.key.tshape().ref_count();
    let value_refs = input.first.ref_count();
    shape_drawer_row(
        output,
        "destruction-unused-alias",
        &[1, key_refs, value_refs],
    );
}

fn replacement_rehash(output: &mut String) {
    let map = AisShapeDrawerMap::default();
    let keys: Vec<_> = (0..102)
        .map(|index| shape_drawer_vertex(Point3::new(f64::from(index), 8., 9.)))
        .collect();
    for key in &keys {
        map.bind(key, &Handle::default()).unwrap();
    }
    let alias = map.node_ref(&keys[0]).unwrap();
    let address = alias.value_reference_address().unwrap();
    let before = shape_drawer_size(&map);
    shape_drawer_row(output, "hit-before", &before);
    let inserted = i32::from(map.bind(&keys[0], &Handle::default()).unwrap());
    let [extent, buckets] = shape_drawer_size(&map);
    let stable = i32::from(alias.value_reference_address().unwrap() == address);
    let after = [inserted, extent, buckets, stable];
    shape_drawer_row(output, "hit-after", &after);
}

fn location_ownership(
    output: &mut String,
    map: &AisShapeDrawerMap,
    input: &mut ShapeDrawerOracleInputs,
) -> Handle<rustycad_tkmath::toploc::Datum3> {
    let original = input.located.location().clone();
    let datum = original.first_datum().unwrap().clone();
    let node = map.node_ref(&input.located).unwrap();
    record_location_identity(output, &node, input, &original);
    input
        .located
        .set_location(&rustycad_tkmath::toploc::Location::default(), false)
        .unwrap();
    record_stored_location(output, &node, input, &original);
    drop(original);
    shape_drawer_row(output, "location-datum-owned", &[datum.ref_count()]);
    datum
}
fn record_stored_location(
    output: &mut String,
    node: &super::AisShapeDrawerNodeRef<'_>,
    input: &ShapeDrawerOracleInputs,
    original: &rustycad_tkmath::toploc::Location,
) {
    let old = input.key.located(original, false).unwrap();
    let same = i32::from(node.is_same_key(&old).unwrap());
    let moved = i32::from(node.is_same_key(&input.located).unwrap());
    drop(old);
    shape_drawer_row(output, "stored-location", &[same, moved]);
}
fn record_location_identity(
    output: &mut String,
    node: &super::AisShapeDrawerNodeRef<'_>,
    input: &ShapeDrawerOracleInputs,
    original: &rustycad_tkmath::toploc::Location,
) {
    let mut transform = rustycad_tkmath::gp::Transform3::default();
    transform.set_translation(rustycad_tkmath::gp::Vector3::new(7., -3., 2.));
    let equivalent = rustycad_tkmath::toploc::Location::new(transform);
    let same = i32::from(
        node.is_same_key(&input.key.located(original, false).unwrap())
            .unwrap(),
    );
    let numerical_same = i32::from(
        node.is_same_key(&input.key.located(&equivalent, false).unwrap())
            .unwrap(),
    );
    shape_drawer_row(output, "location-identity", &[same, numerical_same]);
}

fn shape_drawer_size(map: &AisShapeDrawerMap) -> [i32; 2] {
    let extent = i32::try_from(map.len()).unwrap();
    let buckets = i32::try_from(map.bucket_count()).unwrap();
    [extent, buckets]
}
