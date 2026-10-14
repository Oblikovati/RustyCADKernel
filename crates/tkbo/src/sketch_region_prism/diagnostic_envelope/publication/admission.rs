// SPDX-License-Identifier: GPL-2.0-only
//! Sealed provenance is necessary; equal reusable internal tokens alone never issue context.
use super::*;
use crate::sketch_region_prism::owned_request::request_replay::encode_publication_request_options;
pub(super) fn gaps(packet: &InvokedPrismPublicationFailure<'_>) -> Vec<PublicationDiagnosticGap> {
    let mut gaps = Vec::new();
    let failure = packet.failure();
    if failure.operation != packet.bindings().operation {
        gaps.push(PublicationDiagnosticGap::Operation {
            actual: failure.operation,
            expected: packet.bindings().operation,
        });
    }
    if !same_policy(failure.requested, packet.request().policy.publication) {
        gaps.push(PublicationDiagnosticGap::RequestedPolicy);
    }
    if !same_dictionary(packet) {
        gaps.push(PublicationDiagnosticGap::CallerDictionary);
    }
    gaps
}
pub(super) fn same_policy(
    actual: PrismPublicationOptions,
    expected: PrismPublicationOptions,
) -> bool {
    let actual = encode_publication_request_options(actual);
    let expected = encode_publication_request_options(expected);
    actual == expected
}
fn same_dictionary(packet: &InvokedPrismPublicationFailure<'_>) -> bool {
    let request = packet.request();
    let [source] = request.sources.as_slice() else {
        return false;
    };
    let actual = packet.plan().identities();
    actual == source.names(request.operation)
}
