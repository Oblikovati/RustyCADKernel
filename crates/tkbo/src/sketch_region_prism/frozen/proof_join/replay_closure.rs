// SPDX-License-Identifier: GPL-2.0-only
//! The original query and every semantic raw point are replayed without operation-local indices.
use super::*;

pub(super) fn append(
    closure: &JoinedClosureOperands,
    bindings: &PrismPublicationBindings,
    bytes: &mut Vec<u8>,
) -> CaptureResult<()> {
    let mut rows = Vec::new();
    for row in &closure.rows {
        let mut encoded = Vec::new();
        replay::text(
            &format!(
                "{:?}/{:?}/{}/{:?}",
                row.contact, row.original, row.stage, row.orientation
            ),
            &mut encoded,
        );
        encoding::numbers(row.local_point, &mut encoded);
        encoding::placement(row.placement, &mut encoded);
        rows.push(encoded);
    }
    replay_sources::sorted_blocks(rows, bytes);
    query(&closure.query, bindings, bytes)
}
fn query(
    query: &crate::sketch_region_contact_transport::ContactPointQuerySnapshot,
    bindings: &PrismPublicationBindings,
    bytes: &mut Vec<u8>,
) -> CaptureResult<()> {
    replay::text(
        &format!("{:?}", original_key(bindings, &query.original)?),
        bytes,
    );
    encoding::space(&query.representation, bytes);
    let construction = query;
    replay::text(
        &format!(
            "{:?}",
            parents::original_multiset(bindings, &construction.parents)?
        ),
        bytes,
    );
    replay_sources::occurrence(&construction.copied_occurrence, bytes)?;
    encoding::space(&construction.copied, bytes);
    encoding::curve(&construction.pcurve, bytes);
    encoding::plane(&construction.support, bytes);
    encoding::curve(&construction.captured, bytes);
    encoding::numbers(construction.native_interval, bytes);
    encoding::numbers(construction.basis_interval, bytes);
    encoding::numbers(query.interval, bytes);
    encoding::numbers([query.requested_world_limit], bytes);
    replay::text(
        &format!(
            "{:?}/{:?}/{}/{}",
            query.maps, query.roots, query.bounds[0], query.bounds[1]
        ),
        bytes,
    );
    Ok(())
}
