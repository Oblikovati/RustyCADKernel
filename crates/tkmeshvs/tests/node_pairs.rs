// SPDX-License-Identifier: GPL-2.0-only
//! Permanent native fixtures exercise both key types without any OCCT runtime.
use martensite_tkmeshvs::{MeshVSNodePair, MeshVSSymmetricPairHasher, MeshVSTwoNodes};
use rustycad_tkernel::ncollection::{HashKey, HashPolicy, Map};

#[test]
fn defaults_and_all_native_signed_hashes_preserve_original_node_order() {
    let lines = native_lines();
    assert_eq!(lines[0], "defaults 0 0 -7 0");
    assert_eq!(MeshVSTwoNodes::default(), MeshVSTwoNodes::new(0, 0));
    assert_eq!(MeshVSTwoNodes::new(-7, 0).first, -7);
    assert_eq!(MeshVSTwoNodes::new(-7, 0).second, 0);
    for line in &lines[1..122] {
        let columns: Vec<&str> = line.split_whitespace().collect();
        let pair = parse_pair(&columns);
        let key = MeshVSTwoNodes::new(pair.0, pair.1);
        assert_eq!((key.first, key.second), pair);
        assert_eq!(key.occt_hash(), columns[3].parse::<usize>().unwrap());
        assert_eq!(
            MeshVSSymmetricPairHasher.hash(&pair),
            columns[4].parse::<usize>().unwrap()
        );
    }
}

#[test]
fn complete_native_equality_matrices_keep_tuple_equality_separate() {
    let lines = native_lines();
    let cases: Vec<MeshVSNodePair> = lines[1..122]
        .iter()
        .map(|line| parse_pair(&line.split_whitespace().collect::<Vec<_>>()))
        .collect();
    for line in &lines[122..243] {
        check_equality_row(line, &cases);
    }
    assert_ne!((7, 9), (9, 7));
    assert!(MeshVSSymmetricPairHasher.equal(&(7, 9), &(9, 7)));
    assert_eq!(MeshVSTwoNodes::new(7, 9), MeshVSTwoNodes::new(9, 7));
}

fn check_equality_row(line: &str, cases: &[MeshVSNodePair]) {
    let columns: Vec<&str> = line.split_whitespace().collect();
    assert_eq!(columns[0], "equal");
    let first = cases[columns[1].parse::<usize>().unwrap()];
    assert_eq!(columns[2].len(), cases.len());
    assert_eq!(columns[3].len(), cases.len());
    for (index, second) in cases.iter().enumerate() {
        let nodes_equal =
            MeshVSTwoNodes::new(first.0, first.1) == MeshVSTwoNodes::new(second.0, second.1);
        assert_eq!(nodes_equal, columns[2].as_bytes()[index] == b'1');
        assert_eq!(
            MeshVSSymmetricPairHasher.equal(&first, second),
            columns[3].as_bytes()[index] == b'1'
        );
    }
}

#[test]
fn actual_maps_replay_native_duplicate_extent_and_first_orientation() {
    let lines = native_lines();
    let mut nodes = NodeLinkMap::default();
    let mut pairs = SymmetricLinkMap::default();
    for (source, expected) in lines[1..122].iter().zip(&lines[243..364]) {
        let pair = parse_pair(&source.split_whitespace().collect::<Vec<_>>());
        assert_eq!(*expected, map_insert_row(&mut nodes, &mut pairs, pair));
    }
    assert_eq!(lines[364], orientation_row(&mut nodes, &mut pairs));
    assert_eq!(lines[365], removal_row(&mut nodes, &mut pairs));
}

type NodeLinkMap = Map<MeshVSTwoNodes>;
type SymmetricLinkMap = Map<MeshVSNodePair, MeshVSSymmetricPairHasher>;

fn map_insert_row(
    nodes: &mut NodeLinkMap,
    pairs: &mut SymmetricLinkMap,
    pair: MeshVSNodePair,
) -> String {
    let added = u8::from(nodes.add(MeshVSTwoNodes::new(pair.0, pair.1)));
    let pair_added = u8::from(pairs.add(pair));
    format!(
        "map {added} {pair_added} {} {}",
        nodes.extent(),
        pairs.extent()
    )
}

fn orientation_row(nodes: &mut NodeLinkMap, pairs: &mut SymmetricLinkMap) -> String {
    let key = nodes.added(MeshVSTwoNodes::new(1, -1));
    let pair = pairs.added((1, -1));
    format!(
        "orientation {} {} {} {}",
        key.first, key.second, pair.0, pair.1
    )
}

fn removal_row(nodes: &mut NodeLinkMap, pairs: &mut SymmetricLinkMap) -> String {
    let removed = u8::from(nodes.remove(&MeshVSTwoNodes::new(1, -1)));
    let pair_removed = u8::from(pairs.remove(&(1, -1)));
    let contains = u8::from(nodes.contains(&MeshVSTwoNodes::new(-1, 1)));
    let pair_contains = u8::from(pairs.contains(&(-1, 1)));
    format!("remove {removed} {pair_removed} {contains} {pair_contains}")
}

fn parse_pair(columns: &[&str]) -> MeshVSNodePair {
    let first = columns[1].parse().unwrap();
    let second = columns[2].parse().unwrap();
    (first, second)
}

fn native_lines() -> Vec<&'static str> {
    let lines: Vec<&str> = include_str!("fixtures/node-pairs-native.txt")
        .lines()
        .collect();
    assert_eq!(lines.len(), 366);
    lines
}

#[test]
fn input_mutation_never_retargets_the_stored_link_copy() {
    let mut key = MeshVSTwoNodes::new(7, -9);
    let mut nodes = NodeLinkMap::default();
    assert!(nodes.add(key));
    key.first = 13;
    let original = nodes.contained(&MeshVSTwoNodes::new(-9, 7)).unwrap();
    assert_eq!((original.first, original.second), (7, -9));
    assert!(!nodes.contains(&key));
    assert_eq!((key.first, key.second), (13, -9));
}
