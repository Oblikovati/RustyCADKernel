// SPDX-License-Identifier: GPL-2.0-only
//! Shared dependency checks over one join; this is not a substitute for ordered geometry/tolerance.
use super::super::{proof_join::operation_tests::JoinFixture, topology::*};
use crate::sketch_region_prism::region_certificate::embedding::check_prism_embedding_table;
use rustycad_tkernel::standard::Failure;

struct NamedJoinedBaselineProgress;
impl TopologyProgress for NamedJoinedBaselineProgress {
    fn checkpoint(&mut self, _: TopologyCheckpoint) -> Result<bool, Failure> {
        // This fixture proves dependency source binding without external activity.
        Ok(false)
    }
}

#[test]
fn joined_validation_dependencies_share_owned_source_and_do_not_promote_body() {
    let fixture = JoinFixture::new();
    let joined = fixture.compose(|| false).ok().unwrap();
    let before = joined.replay_bytes().to_vec();
    let FrozenTopologyOutcome::Certified(topology) =
        certify_frozen_topology(joined.projection(), &mut NamedJoinedBaselineProgress)
    else {
        panic!("expected owned cubic topology certificate");
    };
    assert!(topology.binds(joined.projection()));
    check_prism_embedding_table(joined.embedding(), |_| false).unwrap();
    assert_eq!(joined.replay_bytes(), before);
    assert!(!joined.projection().is_valid_body());
    assert!(fixture.rebind(&joined).is_ok());
}
