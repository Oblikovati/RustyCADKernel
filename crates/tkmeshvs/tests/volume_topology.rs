// SPDX-License-Identifier: GPL-2.0-only
//! Permanent native corpus; these tests never load or execute OCCT.
use martensite_tkmeshvs::{MeshVSVolumeTopology, create_prism_topology, create_pyramid_topology};
use std::collections::BTreeMap;

type TopologyFactory = fn(i32) -> MeshVSVolumeTopology;
const NATIVE_TOPOLOGIES: &str = include_str!("fixtures/volume-topology-native.txt");
const FACTORIES: [(&str, TopologyFactory); 2] = [
    ("prism", create_prism_topology),
    ("pyramid", create_pyramid_topology),
];

#[test]
fn static_factories_match_native_null_bounds_and_every_face_index() {
    let rows: Vec<&str> = NATIVE_TOPOLOGIES.lines().take(24).collect();
    assert_eq!(rows.len(), 24);
    for (kind, factory) in FACTORIES {
        for row in rows.iter().filter(|row| row.starts_with(kind)) {
            let count = row.split_whitespace().nth(1).unwrap().parse().unwrap();
            assert_eq!(*row, topology_row(kind, count, factory(count)));
        }
    }
}

fn topology_row(kind: &str, count: i32, topology: MeshVSVolumeTopology) -> String {
    let prefix = format!("{kind} {count} {}", u8::from(topology.is_null()));
    if topology.is_null() {
        return prefix;
    }
    let faces = topology.array1();
    let mut fields = vec![prefix, faces.lower().to_string(), faces.upper().to_string()];
    for face in faces.lower()..=faces.upper() {
        let nodes = faces.value(face).unwrap();
        fields.push(nodes.len().to_string());
        fields.extend(nodes.iter().map(i32::to_string));
    }
    fields.join(" ")
}

#[test]
fn static_allocations_are_independent_and_handle_aliases_share_mutation() {
    let rows: Vec<&str> = NATIVE_TOPOLOGIES.lines().skip(24).collect();
    assert_eq!(rows.len(), 2);
    for ((kind, factory), expected) in FACTORIES.into_iter().zip(rows) {
        assert_eq!(expected, ownership_row(kind, factory));
    }
}

fn ownership_row(kind: &str, factory: TopologyFactory) -> String {
    let mut first = factory(3);
    let second = factory(3);
    let alias = first.clone();
    set_first_node(&alias, 99);
    let first_equal = u8::from(first == second);
    let alias_equal = u8::from(first == alias);
    let first_node = *first.array1().value(1).unwrap().value(1).unwrap();
    let second_node = *second.array1().value(1).unwrap().value(1).unwrap();
    first.nullify();
    let retained_node = *alias.array1().value(1).unwrap().value(1).unwrap();
    format!(
        "{kind}-ownership {first_equal} {alias_equal} {first_node} {second_node} {retained_node}"
    )
}

fn set_first_node(topology: &MeshVSVolumeTopology, node: i32) {
    let mut faces = topology.change_array1();
    let first_face = faces.change_value(1).unwrap();
    first_face.set_value(1, node).unwrap();
}

#[test]
fn minimum_prism_has_independently_stated_face_order() {
    assert_eq!(
        face_lists(create_prism_topology(3)),
        vec![
            vec![2, 1, 0],
            vec![3, 4, 5],
            vec![1, 4, 3, 0],
            vec![2, 5, 4, 1],
            vec![0, 3, 5, 2],
        ]
    );
}

#[test]
fn minimum_pyramid_has_independently_stated_face_order() {
    assert_eq!(
        face_lists(create_pyramid_topology(3)),
        vec![vec![3, 2, 1], vec![0, 1, 2], vec![0, 2, 3], vec![0, 3, 1],]
    );
}

fn face_lists(topology: MeshVSVolumeTopology) -> Vec<Vec<i32>> {
    let faces = topology.array1();
    (faces.lower()..=faces.upper())
        .map(|face| faces.value(face).unwrap().iter().copied().collect())
        .collect()
}

#[test]
fn generated_solids_have_two_opposite_uses_of_every_edge() {
    for count in 3..=32 {
        for (_, factory) in FACTORIES {
            assert_closed_solid(factory(count));
        }
    }
}

fn assert_closed_solid(topology: MeshVSVolumeTopology) {
    let mut incidences = BTreeMap::new();
    for face in face_lists(topology) {
        record_edges(&face, &mut incidences);
    }
    for (edge, (uses, orientation)) in incidences {
        assert_eq!((uses, orientation), (2, 0), "edge={edge:?}");
    }
}

fn record_edges(face: &[i32], incidences: &mut BTreeMap<(i32, i32), (i32, i32)>) {
    for (first, second) in face.iter().zip(face.iter().cycle().skip(1)) {
        let key = ((*first).min(*second), (*first).max(*second));
        let entry = incidences.entry(key).or_default();
        entry.0 += 1;
        entry.1 += if first < second { 1 } else { -1 };
    }
}

#[test]
fn invalid_calls_and_source_drops_preserve_previous_results() {
    for (_, factory) in FACTORIES {
        let previous = factory(4);
        let expected = face_lists(previous.clone());
        for count in [i32::MIN, -1, 0, 1, 2] {
            assert!(factory(count).is_null());
            assert_eq!(face_lists(previous.clone()), expected);
        }
        let next = factory(4);
        assert_ne!(previous, next);
        drop(previous);
        assert_eq!(face_lists(next), expected);
    }
}
