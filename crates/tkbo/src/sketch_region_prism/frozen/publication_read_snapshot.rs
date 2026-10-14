// SPDX-License-Identifier: GPL-2.0-only
//! Test-only fresh observations reuse encoders; cached publication/join/history bytes are excluded.
use super::{encoding, proof_join::*};
use crate::sketch_region_prism::{
    publication::PublishedNamedPrism, surface_charts::CapturedPrismSources,
};

pub(in crate::sketch_region_prism) fn read_snapshot(result: &PublishedNamedPrism) -> Vec<u8> {
    let mut bytes = result.join().projection().replay_bytes();
    bytes.extend(result.validation().replay_bytes());
    bytes.extend(result.measures().replay_bytes());
    raw_operands(result.join(), &mut bytes);
    bound_geometry(result.join(), &mut bytes);
    closure_query(result.join(), &mut bytes);
    history_views(result, &mut bytes);
    bytes
}
fn raw_operands(join: &JoinedPrismProof, bytes: &mut Vec<u8>) {
    for point in join.raw_vertices() {
        encoding::numbers(point.local_point, bytes);
        encoding::placement(point.placement, bytes);
    }
    for point in join.raw_closure_points() {
        encoding::numbers(point.local_point, bytes);
        encoding::placement(point.placement, bytes);
    }
}
fn bound_geometry(join: &JoinedPrismProof, bytes: &mut Vec<u8>) {
    let binding = join.embedding().binding();
    let region = &binding.region;
    for source in [
        &region.source_candidate,
        &region.complementary_candidate,
        &region.copied,
    ] {
        source_geometry(source, bytes);
    }
    for chart in &binding.charts {
        bytes.extend(encoding::chart(&chart.captured));
        occurrence(&chart.source, bytes);
        occurrence(&chart.output, bytes);
    }
    for point in &region.closure_points {
        encoding::numbers(point.point, bytes);
    }
}
fn source_geometry(source: &CapturedPrismSources, bytes: &mut Vec<u8>) {
    bytes.extend(encoding::sources(source));
    occurrence(&source.face, bytes);
    for boundary in &source.boundaries {
        occurrence(&boundary.occurrence, bytes);
        for (endpoint, _) in &boundary.endpoints {
            occurrence(endpoint, bytes);
        }
    }
}
fn occurrence(value: &crate::sketch_region::PartitionOccurrenceIdentity, bytes: &mut Vec<u8>) {
    bytes.push(value.orientation as u8);
    encoding::placement(
        super::history::freeze_placement(&value.location).unwrap(),
        bytes,
    );
}
fn closure_query(join: &JoinedPrismProof, bytes: &mut Vec<u8>) {
    let query = join.closure_query();
    encoding::space(&query.representation, bytes);
    encoding::space(&query.copied, bytes);
    encoding::curve(&query.pcurve, bytes);
    encoding::curve(&query.captured, bytes);
    encoding::plane(&query.support, bytes);
    encoding::numbers(query.native_interval, bytes);
    encoding::numbers(query.basis_interval, bytes);
    encoding::numbers(query.interval, bytes);
    encoding::numbers([query.requested_world_limit], bytes);
    occurrence(&query.copied_occurrence, bytes);
    encode_joined_root_options(query.roots, bytes);
}
fn history_views(result: &PublishedNamedPrism, bytes: &mut Vec<u8>) {
    let history = result.history();
    bytes.extend(history.lineage().replay_bytes());
    encode_joined_text(
        &format!(
            "{:?}/{:?}/{:?}/{:?}/{:?}/{:?}",
            history.source_binding(),
            history.requested(),
            history.continued(),
            history.deleted_originals(),
            history.intermediate(),
            history.sweep_queries()
        ),
        bytes,
    );
    let partition = history.partition();
    encode_joined_text(
        &format!(
            "{:?}/{:?}/{:?}/{:?}",
            partition.source, partition.parents, partition.endpoints, partition.vertices
        ),
        bytes,
    );
    if let Some(space) = &partition.space {
        encoding::space(space, bytes);
    }
    encoding::numbers(partition.range_bits.map(f64::from_bits), bytes);
}
