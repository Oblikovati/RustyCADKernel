// SPDX-License-Identifier: GPL-2.0-only
//! A symbolic checked map is explicitly insufficient for a real completed operation.
use super::super::fixture::{PublicationFixture, PublicationObserver};
use super::*;
#[test]
fn unbound_symbolic_names_cannot_publish_a_real_body_read_view() {
    let result = PublicationFixture::new().publish(&mut PublicationObserver::default());
    let before = result.replay_bytes().to_vec();
    let names = test_fixture::names();
    let error = project_named_prism_body(&result, &names).err().unwrap();
    assert_eq!(error.class, PrismBodyProjectionClass::Incomplete);
    assert_eq!(error.cause, PrismBodyProjectionCause::PublicationBinding);
    assert_eq!(error.operation, names.plan().identities().operation);
    assert_eq!(result.replay_bytes(), before);
}
#[test]
fn real_bound_metadata_and_incidence_coverage_reuses_existing_rows() {
    let result = PublicationFixture::new().publish(&mut PublicationObserver::default());
    references::check(&result, &test_fixture::names()).unwrap();
    assert_eq!(result.join().projection().nodes().len(), 13);
    assert_eq!(result.join().projection().uses().len(), 26);
}
#[test]
fn read_model_and_failures_are_send_sync_without_mutable_shape_owners() {
    static_assertions::assert_impl_all!(OwnedNamedPrismBody: Send, Sync);
    static_assertions::assert_impl_all!(PrismBodyProjectionFailure: Send, Sync);
    static_assertions::assert_impl_all!(PrismBodyMember: Send, Sync);
    static_assertions::assert_impl_all!(PrismBodyOccurrence: Send, Sync);
    static_assertions::assert_impl_all!(PrismBodyFace: Send, Sync);
}
