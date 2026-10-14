// SPDX-License-Identifier: GPL-2.0-only
//! Pin the reviewed complete join before introducing operation publication acceptance.
use crate::sketch_region_prism::{
    PrismCompletion, frozen::proof_join::operation_tests::JoinFixture,
};
use crate::sketch_region_validation::StageCoverage;

#[test]
fn reviewed_join_bytes_and_historical_incompleteness_survive_publication_work() {
    let fixture = JoinFixture::new();
    let joined = fixture.compose(|| false).ok().unwrap();
    assert_eq!(
        joined.replay_bytes(),
        include_bytes!("fixtures/join-v1.bin")
    );
    assert!(matches!(
        fixture.receipt.completion,
        PrismCompletion::Incomplete
    ));
    assert!(!joined.projection().is_valid_body());
    assert_eq!(
        joined.projection().coverage().current,
        [StageCoverage::Incomplete; 4]
    );
    assert!(joined.projection().limits().achieved_error.is_none());
    assert!(fixture.rebind(&joined).is_ok());
}
