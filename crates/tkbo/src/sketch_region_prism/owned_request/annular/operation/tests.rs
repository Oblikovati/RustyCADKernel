// SPDX-License-Identifier: GPL-2.0-only
//! One call from owned coefficients: independent closed forms (cap 9/5, volume 18/5), caller
//! names, complete validation and history, typed refusals at every stage and every checkpoint.
use super::super::tests::owned_request;
use super::*;
use crate::sketch_region_prism::frozen::FrozenSweepRole;
use crate::sketch_region_prism::frozen::ordered_validation::OrderedPrismStageStatus;

const _: () = {
    const fn owned<T: Send + Sync>() {}
    owned::<OwnedAnnularPrism>();
    owned::<OwnedAnnularPrismRefusal>();
};
/// Records every checkpoint; stops at `cutoff` (1-based) by cancel or native failure.
struct Script {
    seen: Vec<AnnularOperationCheckpoint>,
    cutoff: usize,
    fail: bool,
}
impl Script {
    fn at(cutoff: usize, fail: bool) -> Self {
        Self {
            seen: Vec::new(),
            cutoff,
            fail,
        }
    }
}
impl AnnularOperationProgress for Script {
    fn checkpoint(&mut self, point: AnnularOperationCheckpoint) -> Result<bool, Failure> {
        self.seen.push(point);
        match (self.seen.len() == self.cutoff, self.fail) {
            (true, true) => Err(Failure::new("caller stop")),
            (stop, _) => Ok(stop),
        }
    }
}
fn published(request: OwnedAnnularPrismRequest) -> Box<OwnedAnnularPrism> {
    match run_owned_annular_prism(request, &mut Script::at(0, false)) {
        OwnedAnnularPrismOutcome::Published(prism) => prism,
        OwnedAnnularPrismOutcome::Refused(r) => {
            panic!("refused at {:?} {:?}", r.stage(), r.class())
        }
    }
}
fn refused(
    request: OwnedAnnularPrismRequest,
    script: &mut Script,
) -> Box<OwnedAnnularPrismRefusal> {
    match run_owned_annular_prism(request, script) {
        OwnedAnnularPrismOutcome::Refused(refusal) => refusal,
        OwnedAnnularPrismOutcome::Published(_) => panic!("expected a refusal"),
    }
}
fn contains([lo, hi]: [f64; 2], value: f64) -> bool {
    lo <= value && value <= hi
}
#[test]
fn one_call_publishes_named_faces_with_certified_closed_form_measures() {
    for reversed in [false, true] {
        let prism = published(owned_request(reversed));
        let mut sides = Vec::new();
        let mut caps = Vec::new();
        for face in prism.faces() {
            match face.name {
                AnnularFaceName::Side { boundary, wire } => sides.push((boundary.0, wire)),
                AnnularFaceName::Cap { region, sweep } => {
                    assert_eq!(region, 301);
                    assert!(contains(face.area.enclosure, 1.8), "{:?}", face.area);
                    caps.push(sweep);
                }
            }
        }
        sides.sort();
        assert_eq!(sides, [(101, 401), (202, 402)]);
        caps.sort();
        assert_eq!(caps, [FrozenSweepRole::First, FrozenSweepRole::Last]);
        let volume = prism.volume();
        assert!(contains(volume.volume.enclosure, 3.6) && volume.volume.enclosure[0] >= 0.);
        assert_eq!(prism.validation(), [OrderedPrismStageStatus::Passed; 4]);
        let history = prism.history();
        let counts = (
            history.continued_originals,
            history.retired_wires,
            history.deleted_originals,
        );
        assert_eq!(counts, (6, 2, 0));
    }
}
#[test]
fn replay_is_identical_across_calls_and_threads() {
    let bytes = || {
        let prism = published(owned_request(false));
        [
            prism.publication_replay().to_vec(),
            prism.volume_replay().to_vec(),
        ]
    };
    let first = bytes();
    assert_eq!(first, bytes());
    assert_eq!(
        first,
        std::thread::spawn(bytes).join().expect("operation thread")
    );
}
#[test]
fn admission_and_owned_child_refusals_are_complete() {
    let mut request = owned_request(false);
    request.regions.inner_domain = request.regions.annulus;
    let refusal = refused(request, &mut Script::at(0, false));
    assert_eq!(refusal.stage(), AnnularOperationStage::Preparation);
    assert!(
        refusal.gaps().is_empty(),
        "{:?} {:?}",
        refusal.stage(),
        refusal.gaps()
    );
    assert!(matches!(
        refusal.diagnostic(),
        AnnularOperationDiagnostic::Preparation(_)
    ));
    let mut request = owned_request(false);
    request.policy.volume_version = 2;
    let refusal = refused(request, &mut Script::at(0, false));
    assert_eq!(
        (refusal.stage(), refusal.class()),
        (
            AnnularOperationStage::Volume,
            PrismDiagnosticClass::Incomplete
        )
    );
    assert!(
        refusal.gaps().is_empty(),
        "{:?} {:?}",
        refusal.stage(),
        refusal.gaps()
    );
    let mut request = owned_request(false);
    request.policy.publication.validation.version = 99;
    let refusal = refused(request, &mut Script::at(0, false));
    assert_eq!(refusal.stage(), AnnularOperationStage::Publication);
    assert!(refusal.gaps().is_empty(), "{:?}", refusal.gaps());
}
#[test]
fn borrowed_embedding_refusal_is_an_explicit_stage_gap() {
    let mut request = owned_request(false);
    request.policy.embedding.version = 9;
    let refusal = refused(request, &mut Script::at(0, false));
    assert_eq!(refusal.stage(), AnnularOperationStage::Embedding);
    assert!(matches!(
        refusal.diagnostic(),
        AnnularOperationDiagnostic::Unprojected
    ));
    let gap = AnnularOperationGap::StageFailure(AnnularOperationStage::Embedding);
    assert_eq!(refusal.gaps(), [gap]);
}
fn checkpoints() -> Vec<AnnularOperationCheckpoint> {
    let mut script = Script::at(0, false);
    let outcome = run_owned_annular_prism(owned_request(false), &mut script);
    assert!(matches!(outcome, OwnedAnnularPrismOutcome::Published(_)));
    script.seen
}
/// The first checkpoint of every stage plus the last one, by cancellation and by native failure.
#[test]
fn every_stage_stop_refuses_with_its_exact_checkpoint_and_class() {
    let seen = checkpoints();
    let mut cutoffs: Vec<usize> = Vec::new();
    for (index, point) in seen.iter().enumerate() {
        if index == 0 || seen[index - 1].stage != point.stage {
            cutoffs.push(index + 1);
        }
    }
    cutoffs.push(seen.len());
    let stages: std::collections::BTreeSet<_> = seen.iter().map(|p| p.stage).collect();
    assert!(stages.len() >= 5, "{stages:?}");
    for cutoff in cutoffs {
        for fail in [false, true] {
            let refusal = refused(owned_request(false), &mut Script::at(cutoff, fail));
            let stop = refusal.stop().expect("recorded caller stop");
            assert_eq!(stop.checkpoint, seen[cutoff - 1], "cutoff {cutoff}");
            let expected = match fail {
                true => PrismDiagnosticClass::Failed,
                false => PrismDiagnosticClass::Cancelled,
            };
            assert_eq!(
                refusal.class(),
                expected,
                "cutoff {cutoff} at {:?}",
                stop.checkpoint
            );
        }
    }
}
