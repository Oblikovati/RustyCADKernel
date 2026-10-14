// SPDX-License-Identifier: GPL-2.0-only
use super::super::replay::word;
use super::*;

pub(super) fn encode(
    names: &ResolvedPrismNames,
    rows: &[PrismReservationRow],
) -> (Vec<u8>, std::ops::Range<usize>) {
    let mut bytes = b"Martensite.SemanticReservationCoverage\0".to_vec();
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    word(&mut bytes, names.replay_bytes().len() as u64);
    let start = bytes.len();
    bytes.extend_from_slice(names.replay_bytes());
    let map_range = start..bytes.len();
    word(&mut bytes, rows.len() as u64);
    for row in rows {
        encode_row(&mut bytes, row);
    }
    (bytes, map_range)
}
fn encode_row(bytes: &mut Vec<u8>, row: &PrismReservationRow) {
    bytes.extend_from_slice(&[row.slot.stage as u8, row.slot.role as u8]);
    match row.status {
        PrismReservationStatus::Applied { named_fields } => {
            bytes.push(1);
            word(bytes, named_fields as u64);
        }
        PrismReservationStatus::Inapplicable(reason) => {
            bytes.push(2);
            bytes.push(reason_tag(reason));
        }
    }
}
fn reason_tag(reason: PrismReservationInapplicable) -> u8 {
    match reason {
        PrismReservationInapplicable::PartitionHasNoGeneratedRegions => 1,
        PrismReservationInapplicable::BoundedSelectionExcludesComplementRoles => 2,
        PrismReservationInapplicable::ComplementExcludesBoundedRoles => 3,
    }
}
