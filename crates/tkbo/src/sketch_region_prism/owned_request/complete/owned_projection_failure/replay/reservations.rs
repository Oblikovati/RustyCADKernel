// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::owned_request::names::reservations::*;
pub(super) fn append(value: &PrismReservationFailure, bytes: &mut Vec<u8>) {
    words(&[value.operation.0], bytes);
    bytes.push(cause_tag(&value.cause));
    cause_fields(&value.cause, bytes);
}
fn cause_fields(value: &PrismReservationCause, bytes: &mut Vec<u8>) {
    use PrismReservationCause::*;
    match value {
        ForeignMap | EmptyUse => (),
        UnsupportedVersion { actual, expected } => versions(*actual, *expected, bytes),
        UnknownToken { actual } => words(&[*actual], bytes),
        MissingApplication { slot: value } => slot(*value, bytes),
        InapplicableApplied {
            slot: value,
            actual,
        } => {
            slot(*value, bytes);
            words(&[*actual as u64], bytes);
        }
    }
}
fn cause_tag(value: &PrismReservationCause) -> u8 {
    match value {
        PrismReservationCause::ForeignMap => 0,
        PrismReservationCause::UnsupportedVersion { .. } => 1,
        PrismReservationCause::UnknownToken { .. } => 2,
        PrismReservationCause::EmptyUse => 3,
        PrismReservationCause::MissingApplication { .. } => 4,
        PrismReservationCause::InapplicableApplied { .. } => 5,
    }
}
