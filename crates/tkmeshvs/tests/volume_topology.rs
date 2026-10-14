// SPDX-License-Identifier: GPL-2.0-only
//! Owned historical corpus and independent combinatorial checks; no live OCCT dependency.
use martensite_tkmeshvs::{
    MeshVSTopologyError, MeshVSTopologyOperation, MeshVSVolumeTopology, create_prism_topology,
    create_pyramid_topology,
};
use std::collections::BTreeMap;

type TopologyFactory = fn(i32) -> Result<MeshVSVolumeTopology, MeshVSTopologyError>;
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

fn topology_row(
    kind: &str,
    count: i32,
    topology: Result<MeshVSVolumeTopology, MeshVSTopologyError>,
) -> String {
    let prefix = format!("{kind} {count} {}", u8::from(topology.is_err()));
    let topology = match topology {
        Ok(topology) => topology,
        Err(MeshVSTopologyError::TooFewBasePoints { .. }) => return prefix,
        Err(cause) => panic!("unexpected construction failure: {cause}"),
    };
    let mut fields = vec![prefix, "1".to_owned(), topology.face_count().to_string()];
    for face in face_lists(topology) {
        fields.push(face.len().to_string());
        fields.extend(face.iter().map(i32::to_string));
    }
    fields.join(" ")
}

#[test]
fn minimum_prism_has_independently_stated_face_order() {
    assert_eq!(
        face_lists(create_prism_topology(3).unwrap()),
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
        face_lists(create_pyramid_topology(3).unwrap()),
        vec![vec![3, 2, 1], vec![0, 1, 2], vec![0, 2, 3], vec![0, 3, 1],]
    );
}

fn face_lists(topology: MeshVSVolumeTopology) -> Vec<Vec<i32>> {
    (1..=topology.face_count() as i32)
        .map(|face| topology.face(face).unwrap().to_vec())
        .collect()
}

#[test]
fn generated_connectivity_has_two_opposite_uses_of_every_edge() {
    for count in 3..=32 {
        for (_, factory) in FACTORIES {
            assert_closed_connectivity(factory(count).unwrap());
        }
    }
}

fn assert_closed_connectivity(topology: MeshVSVolumeTopology) {
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
        let previous = factory(4).unwrap();
        let expected = face_lists(previous.clone());
        for count in [i32::MIN, -1, 0, 1, 2] {
            assert!(
                matches!(factory(count), Err(MeshVSTopologyError::TooFewBasePoints { base_points, .. }) if base_points == count)
            );
            assert_eq!(face_lists(previous.clone()), expected);
        }
        let next = factory(4).unwrap();
        assert_eq!(previous, next);
        drop(previous);
        assert_eq!(face_lists(next), expected);
    }
}

#[test]
fn public_types_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MeshVSVolumeTopology>();
    assert_send_sync::<MeshVSTopologyError>();
    assert_send_sync::<MeshVSTopologyOperation>();
}

#[test]
fn face_lookup_rejects_invalid_one_based_indices() {
    for (_, factory) in FACTORIES {
        let topology = factory(3).unwrap();
        for face_id in [i32::MIN, -1, 0, topology.face_count() as i32 + 1, i32::MAX] {
            assert_eq!(topology.face(face_id), None);
        }
        assert!(topology.face(1).is_some());
        assert!(topology.face(topology.face_count() as i32).is_some());
    }
}

#[test]
fn overflow_inputs_fail_before_allocation() {
    for count in [i32::MAX / 2 + 2, i32::MAX - 1, i32::MAX] {
        assert!(
            matches!(create_prism_topology(count), Err(MeshVSTopologyError::IndexOverflow {
            operation: MeshVSTopologyOperation::Prism, base_points
        }) if base_points == count)
        );
    }
    assert!(matches!(
        create_pyramid_topology(i32::MAX),
        Err(MeshVSTopologyError::IndexOverflow {
            operation: MeshVSTopologyOperation::Pyramid,
            base_points: i32::MAX
        })
    ));
}

#[test]
fn generated_connectivity_has_expected_euler_characteristic_and_node_set() {
    for count in 3..=64 {
        check_combinatorial_counts(create_prism_topology(count).unwrap(), 2 * count, 3 * count);
        check_combinatorial_counts(
            create_pyramid_topology(count).unwrap(),
            count + 1,
            2 * count,
        );
    }
}

fn check_combinatorial_counts(topology: MeshVSVolumeTopology, vertices: i32, edges: i32) {
    let faces = face_lists(topology);
    let nodes: std::collections::BTreeSet<i32> = faces.iter().flatten().copied().collect();
    assert_eq!(nodes, (0..vertices).collect());
    let mut incidences = BTreeMap::new();
    for face in &faces {
        let unique: std::collections::BTreeSet<&i32> = face.iter().collect();
        assert_eq!(unique.len(), face.len());
        record_edges(face, &mut incidences);
    }
    assert_eq!(incidences.len(), edges as usize);
    assert_eq!(vertices - edges + faces.len() as i32, 2);
}

#[test]
fn concurrent_construction_and_shared_reads_match_serial_results() {
    for (_, factory) in FACTORIES {
        assert_threaded_connectivity(factory);
    }
}

fn assert_threaded_connectivity(factory: TopologyFactory) {
    let serial = factory(64).unwrap();
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| repeat_connectivity_reads(factory, &serial));
        }
    });
}

fn repeat_connectivity_reads(factory: TopologyFactory, serial: &MeshVSVolumeTopology) {
    for _ in 0..64 {
        assert_eq!(&factory(64).unwrap(), serial);
        assert_eq!(serial.face(1), serial.clone().face(1));
    }
}
