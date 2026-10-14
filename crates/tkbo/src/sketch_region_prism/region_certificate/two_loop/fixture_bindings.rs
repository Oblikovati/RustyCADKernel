// SPDX-License-Identifier: GPL-2.0-only
//! Names are authored by semantic boundary roles; actual identities are lookup values only.
use super::{fixture::*, request::*};
use crate::sketch_region::{PartitionOccurrenceIdentity, RegionSourceId};
use rustycad_tkbrep::topods::ShapeIterator;

pub(super) fn identities(fixture: &TwoLoopFixture) -> TwoLoopRegionIdentities {
    let batch = super::super::borrowed_batch(&fixture.bridge).unwrap();
    let candidates = batch
        .candidates
        .iter()
        .map(|candidate| {
            let annulus = candidate.wires.len() == 2;
            let only_source = candidate.wires[0].segments[0].parents[0].source;
            let name = if annulus {
                301
            } else if only_source == INNER {
                302
            } else {
                303
            };
            let mut iterator = ShapeIterator::new(&candidate.face, true, true);
            let mut wires = Vec::new();
            while iterator.more() {
                let wire = iterator.value().unwrap();
                let edge = ShapeIterator::new(wire, true, true)
                    .value()
                    .unwrap()
                    .clone();
                let metadata = candidate
                    .wires
                    .iter()
                    .flat_map(|w| &w.segments)
                    .find(|s| {
                        PartitionOccurrenceIdentity::capture(&s.edge)
                            == PartitionOccurrenceIdentity::capture(&edge)
                    })
                    .unwrap();
                let source = metadata.parents[0].source;
                wires.push(TwoLoopWireBinding {
                    name: wire_name(name, source),
                    actual: PartitionOccurrenceIdentity::capture(wire),
                    source,
                });
                iterator.next();
            }
            TwoLoopCandidateBinding {
                name,
                actual: PartitionOccurrenceIdentity::capture(&candidate.face),
                wires,
            }
        })
        .collect();
    TwoLoopRegionIdentities {
        operation: 77,
        outer: OUTER,
        inner: INNER,
        selected: 301,
        candidates,
    }
}
fn wire_name(candidate: u64, source: RegionSourceId) -> u64 {
    match (candidate, source) {
        (301, OUTER) => 401,
        (301, INNER) => 402,
        (302, INNER) => 403,
        (303, OUTER) => 404,
        _ => panic!(
            "unexpected fixture role {candidate}/{source:?}; expected authored annular partition"
        ),
    }
}
pub(super) fn options() -> TwoLoopRegionOptions {
    TwoLoopRegionOptions {
        behavior: TwoLoopRegionBehavior::NestedPolynomialV2,
        winding: rustycad_tkmath::predicates::roots::RootIsolationOptions::new(0.001),
        maximum_retained_units: 8192,
        maximum_poles: 33,
    }
}
