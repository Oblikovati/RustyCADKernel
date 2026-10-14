// SPDX-License-Identifier: GPL-2.0-only
//! Raw native point/copy/context rows are observations, never algebraic contact certificates.
use crate::sketch_region::*;
use crate::sketch_region_candidates::*;
use crate::sketch_region_partition_bindings::*;

#[derive(Debug, PartialEq)]
struct NativeContactPoint {
    role: String,
    source: i64,
    copy_index: i64,
    orientation: i64,
    point: [f64; 3],
}
#[test]
fn actual_point_snapshots_replay_native_ds_copy_and_directed_contexts() {
    let raw = include_str!("../../tests/fixtures/contact-vertex-transport-native.txt");
    for mode in 0..18 {
        let outputs = observed_mode(mode);
        let expected: Vec<_> = raw
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .filter(|fields| {
                fields[1].parse::<usize>().unwrap() == mode
                    && matches!(fields[0], "native" | "copied" | "directed")
            })
            .map(|fields| parse_native_point(&fields))
            .collect();
        assert_eq!(
            observed_native_points(&outputs),
            expected,
            "native point mode={mode}"
        );
    }
}
fn parse_native_point(fields: &[&str]) -> NativeContactPoint {
    assert_eq!(fields.len(), 8);
    NativeContactPoint {
        role: fields[0].into(),
        source: fields[2].parse().unwrap(),
        copy_index: fields[3].parse().unwrap(),
        orientation: fields[4].parse().unwrap(),
        point: std::array::from_fn(|axis| fields[axis + 5].parse().unwrap()),
    }
}
fn observed_native_points(outputs: &PartitionBoundOutputs) -> Vec<NativeContactPoint> {
    let PartitionObservation::Captured(snapshot) = &outputs.arrangement.partition else {
        panic!("expected actual DS");
    };
    let copied = outputs.copied.as_ref().unwrap();
    let mut rows = Vec::new();
    for vertex in &snapshot.vertices {
        rows.push(NativeContactPoint {
            role: "native".into(),
            source: vertex.shape_index.into(),
            copy_index: -1,
            orientation: vertex.occurrence.orientation as i64,
            point: vertex.point,
        });
        append_vertex_copy_points(vertex, copied, &mut rows);
    }
    append_directed_points(copied, &mut rows);
    rows
}
fn append_vertex_copy_points(
    vertex: &PartitionVertexObservation,
    copied: &CandidateCopyInspection,
    rows: &mut Vec<NativeContactPoint>,
) {
    let CandidateCopyTreeObservation::Captured(members) = &copied.copied_tree else {
        panic!("expected actual copied tree");
    };
    let mut seen = Vec::new();
    // The pinned Modifier map ignores query orientation; copied index plus stored orientation is a full image key.
    for link in linked_vertex_copies(copied, vertex) {
        let member = copied_vertex_member(members, link);
        if seen.contains(&(member.index, link.copied.orientation)) {
            continue;
        }
        seen.push((member.index, link.copied.orientation));
        rows.push(copied_point_row(vertex.shape_index, member, link));
    }
}
fn copied_vertex_member<'a>(
    members: &'a [CandidateCopiedMember],
    link: &CandidateImageCopyLink,
) -> &'a CandidateCopiedMember {
    members
        .iter()
        .find(|member| member.occurrence.same_physical(&link.copied))
        .unwrap()
}
fn copied_point_row(
    source: i32,
    member: &CandidateCopiedMember,
    link: &CandidateImageCopyLink,
) -> NativeContactPoint {
    NativeContactPoint {
        role: "copied".into(),
        source: source.into(),
        copy_index: member.index as i64,
        orientation: link.copied.orientation as i64,
        point: *member.vertex_point.as_ref().unwrap().as_ref().unwrap(),
    }
}

fn append_directed_points(copied: &CandidateCopyInspection, rows: &mut Vec<NativeContactPoint>) {
    let GeneratedTopologyObservation::Observed(areas) = &copied.generated_topology else {
        panic!("expected actual generated areas");
    };
    for area in areas {
        for member in area.members.iter().filter(|row| row.vertex_point.is_some()) {
            assert_eq!(member.copy_indices.len(), 1);
            rows.push(NativeContactPoint {
                role: "directed".into(),
                source: area.ordinal as i64,
                copy_index: member.copy_indices[0] as i64,
                orientation: member.occurrence.orientation as i64,
                point: *member.vertex_point.as_ref().unwrap().as_ref().unwrap(),
            });
        }
    }
}

#[derive(Debug, PartialEq)]
struct NativeContactBlock {
    edge_indices: [i32; 5],
    ranges: [f64; 4],
    vertices: [i32; 4],
}
#[test]
fn actual_native_own_and_representative_parameter_values_remain_unmodified() {
    let raw = include_str!("../../tests/fixtures/contact-vertex-transport-native.txt");
    for mode in 0..18 {
        let outputs = observed_mode(mode);
        let expected: Vec<_> = raw
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .filter(|fields| fields[0] == "block" && fields[1].parse::<usize>().unwrap() == mode)
            .map(|fields| parse_native_block(&fields))
            .collect();
        assert_eq!(
            observed_native_blocks(&outputs),
            expected,
            "native block mode={mode}"
        );
    }
}
fn parse_native_block(fields: &[&str]) -> NativeContactBlock {
    assert_eq!(fields.len(), 15);
    NativeContactBlock {
        edge_indices: std::array::from_fn(|i| fields[i + 2].parse().unwrap()),
        ranges: std::array::from_fn(|i| fields[i + 7].parse().unwrap()),
        vertices: std::array::from_fn(|i| fields[i + 11].parse().unwrap()),
    }
}
fn observed_native_blocks(outputs: &PartitionBoundOutputs) -> Vec<NativeContactBlock> {
    let PartitionObservation::Captured(snapshot) = &outputs.arrangement.partition else {
        panic!("expected actual DS");
    };
    snapshot
        .sources
        .iter()
        .flat_map(|source| {
            source
                .blocks
                .iter()
                .map(|block| native_contact_block(source.source_index, block))
        })
        .collect()
}

fn observed_mode(mode: usize) -> PartitionBoundOutputs {
    let sources = super::general_tests::transport_fixture_sources(mode);
    if mode == 17 {
        return super::test_fixture::mapped_outputs_on_plane(
            &binding_sources(&sources),
            super::declared_plane_tests::translated_support(),
        );
    }
    inspect_corpus_sources(&sources)
}

fn native_contact_block(source: i32, block: &PavePartitionObservation) -> NativeContactBlock {
    let [oa, ob] = block.own_range;
    let [ra, rb] = block.real_range;
    let [ov, ow] = block.own_vertices;
    let [rv, rw] = block.real_vertices;
    NativeContactBlock {
        edge_indices: contact_block_edge_indices(source, block),
        ranges: [oa, ob, ra, rb],
        vertices: [ov, ow, rv, rw],
    }
}
fn contact_block_edge_indices(source: i32, block: &PavePartitionObservation) -> [i32; 5] {
    [
        source,
        block.original_edge,
        block.own_edge,
        block.real_original_edge,
        block.real_edge,
    ]
}

fn linked_vertex_copies<'a>(
    copied: &'a CandidateCopyInspection,
    vertex: &'a PartitionVertexObservation,
) -> impl Iterator<Item = &'a CandidateImageCopyLink> {
    copied
        .links
        .iter()
        .filter(move |link| link.original.same_physical(&vertex.occurrence))
}
