// SPDX-License-Identifier: GPL-2.0-only
//! Replay stored native source parameters and physical endpoint coordinates without OCCT.
use super::fixtures::*;
use crate::sketch_region::OwnedPartitionSnapshot;
#[test]
fn actual_owned_vertices_and_pave_parameters_replay_both_native_root_families() {
    let snapshots = [
        actual_binding_snapshot(&two_root_edges()),
        actual_binding_snapshot(&single_root_edges()),
    ];
    let mut mode = 0;
    for line in include_str!("../../tests/fixtures/partition-binding-native.txt").lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        match fields[0] {
            "mode" => mode = fields[1].parse::<usize>().unwrap(),
            "vertex" => check_native_vertex(&snapshots[mode], &fields),
            "span" => check_native_span(&snapshots[mode], &fields),
            _ => panic!("row={line}; expected mode, vertex or span"),
        }
    }
}

fn check_native_vertex(snapshot: &OwnedPartitionSnapshot, fields: &[&str]) {
    assert_eq!(fields.len(), 5);
    let index = fields[1].parse::<i32>().unwrap();
    let vertex = snapshot
        .vertices
        .iter()
        .find(|vertex| vertex.shape_index == index)
        .unwrap();
    for axis in 0..3 {
        let expected = fields[axis + 2].parse::<f64>().unwrap();
        assert_eq!(
            vertex.point[axis].to_bits(),
            expected.to_bits(),
            "vertex={index}, axis={axis}"
        );
    }
}
fn check_native_span(snapshot: &OwnedPartitionSnapshot, fields: &[&str]) {
    assert_eq!(fields.len(), 6);
    let source = fields[1].parse::<i32>().unwrap();
    let ends = [
        fields[2].parse::<i32>().unwrap(),
        fields[3].parse::<i32>().unwrap(),
    ];
    let block = native_pave(snapshot, source, ends);
    for endpoint in 0..2 {
        let expected = fields[endpoint + 4].parse::<f64>().unwrap();
        assert_eq!(
            block.own_range[endpoint].to_bits(),
            expected.to_bits(),
            "source={source}, ends={ends:?}"
        );
    }
}

fn native_pave(
    snapshot: &OwnedPartitionSnapshot,
    source: i32,
    ends: [i32; 2],
) -> &crate::sketch_region::PavePartitionObservation {
    let row = snapshot
        .sources
        .iter()
        .find(|row| row.source_index == source)
        .unwrap();
    row.blocks
        .iter()
        .find(|block| block.own_vertices == ends)
        .unwrap()
}
