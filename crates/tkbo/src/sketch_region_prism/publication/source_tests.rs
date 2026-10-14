// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_partition_bindings::binding_sources;
use crate::sketch_region_prism::{
    frozen::proof_join::operation_tests::JoinFixture,
    region_certificate::embedding::PrismEmbeddingOptions,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions, metric_bounds::BezierRegularityOptions,
};

#[derive(Default)]
struct SourceProbeProgress {
    stopped: bool,
    seen: Vec<PrismPublicationCheckpoint>,
}
impl PrismPublicationProgress for SourceProbeProgress {
    fn checkpoint(&mut self, point: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        self.seen.push(point);
        Ok(self.stopped)
    }
}
fn options() -> PrismProofJoinOptions {
    PrismProofJoinOptions::v1(PrismEmbeddingOptions::v1(BezierRegularityOptions::v1(
        RootIsolationOptions::new(1e-6),
    )))
}
fn source<'a, 's>(
    fixture: &'a JoinFixture,
    sources: &'a [ArrangementSource<'s>],
) -> PrismPublicationSource<'a, 's> {
    PrismPublicationSource {
        bridge: &fixture.bridge,
        sources,
        receipt: &fixture.receipt,
        bindings: &fixture.authored,
        contact: &fixture.contact,
    }
}
#[test]
fn actual_source_join_reuses_complete_baseline_and_callback_free_rebind() {
    let fixture = JoinFixture::new();
    let sources = binding_sources(&fixture.shapes);
    let source = source(&fixture, &sources);
    let mut progress = SourceProbeProgress::default();
    let join = source
        .compose(options(), &mut PublicationCallbacks::new(&mut progress))
        .ok()
        .unwrap();
    assert_eq!(join.replay_bytes(), include_bytes!("fixtures/join-v1.bin"));
    let calls = progress.seen.len();
    assert!(source.rebind(&join).is_ok());
    assert_eq!(progress.seen.len(), calls);
    assert!(
        progress
            .seen
            .iter()
            .all(|point| point.phase == PrismPublicationPhase::Join)
    );
}
#[test]
fn unsupported_join_version_refuses_before_any_user_callback() {
    let fixture = JoinFixture::new();
    let sources = binding_sources(&fixture.shapes);
    let source = source(&fixture, &sources);
    let mut progress = SourceProbeProgress::default();
    let mut requested = options();
    requested.version = 29;
    let failure = source
        .compose(requested, &mut PublicationCallbacks::new(&mut progress))
        .err()
        .unwrap();
    assert_eq!(failure.class, PrismPublicationClass::Incomplete);
    assert!(progress.seen.is_empty());
}
#[test]
fn join_cancellation_returns_no_proof_and_allows_fresh_reuse() {
    let fixture = JoinFixture::new();
    let sources = binding_sources(&fixture.shapes);
    let source = source(&fixture, &sources);
    let mut progress = SourceProbeProgress {
        stopped: true,
        seen: Vec::new(),
    };
    let failure = source
        .compose(options(), &mut PublicationCallbacks::new(&mut progress))
        .err()
        .unwrap();
    assert_eq!(failure.class, PrismPublicationClass::Cancelled);
    let mut progress = SourceProbeProgress::default();
    assert!(
        source
            .compose(options(), &mut PublicationCallbacks::new(&mut progress))
            .is_ok()
    );
}
#[test]
fn authored_source_change_fails_final_rebind_without_modifying_owned_join() {
    let mut fixture = JoinFixture::new();
    let sources = binding_sources(&fixture.shapes);
    let join = fixture.compose(|| false).ok().unwrap();
    let bytes = join.replay_bytes().to_vec();
    fixture.authored.operation.0 = 801;
    let failure = source(&fixture, &sources).rebind(&join).err().unwrap();
    assert_eq!(failure.phase, PrismPublicationPhase::Commit);
    assert_eq!(failure.class, PrismPublicationClass::Incomplete);
    assert_eq!(join.replay_bytes(), bytes);
}
