// SPDX-License-Identifier: GPL-2.0-only
use super::{
    admission, candidates, failure::*, fixture::*, fixture_bindings, nesting, outcome::*,
    request::*, sources, work::*,
};
use rustycad_tkernel::standard::Failure;
struct CandidateProgress;
impl TwoLoopRegionProgress for CandidateProgress {
    fn checkpoint(&mut self, _: TwoLoopRegionStage) -> Result<bool, Failure> {
        Ok(false)
    }
}
fn capture<'a>(
    fixture: &'a TwoLoopFixture,
    identities: &TwoLoopRegionIdentities,
    inner_interior: [f64; 2],
    material_seed: [f64; 2],
) -> TwoLoopCheck<'a, Vec<TwoLoopCandidateProof>> {
    let originals = fixture.sources();
    let request = TwoLoopRegionRequest {
        bridge: &fixture.bridge,
        sources: &originals,
        identities,
        inner_interior,
        material_seed,
        requested: fixture_bindings::options(),
    };
    let mut progress = CandidateProgress;
    let mut work = TwoLoopWork::new(77, request.requested, &mut progress);
    admission::inspect(&request, &mut work)?;
    let source = sources::capture(&request, &mut work)?;
    let proof = nesting::capture(&request, &source, &mut work)?;
    candidates::capture(&request, &source, &proof, &mut work)
}
#[test]
fn complete_nested_partition_binds_annulus_and_both_complementary_dispositions() {
    for reversed in [false, true] {
        let orientation = if reversed {
            rustycad_tkg3d::top_abs::Orientation::Reversed
        } else {
            rustycad_tkg3d::top_abs::Orientation::Forward
        };
        let fixture = TwoLoopFixture::for_edges(
            cubic(1., [0., 0.]).oriented(orientation),
            cubic(0.5, [0.5, 0.5]).oriented(orientation),
        );
        let identities = fixture_bindings::identities(&fixture);
        let captured = capture(&fixture, &identities, [1., 1.], [0.25, 0.25])
            .unwrap_or_else(|e| panic!("candidate: {:?}", e.cause));
        check_dispositions(&captured);
    }
}
#[test]
fn duplicate_candidate_or_wire_bindings_and_wrong_selection_are_refused() {
    let fixture = TwoLoopFixture::nested();
    let mut names = fixture_bindings::identities(&fixture);
    names.selected = 302;
    assert!(capture(&fixture, &names, [1., 1.], [0.25, 0.25]).is_err());
    let mut names = fixture_bindings::identities(&fixture);
    names.candidates[1].actual = names.candidates[0].actual.clone();
    assert!(capture(&fixture, &names, [1., 1.], [0.25, 0.25]).is_err());
    let mut names = fixture_bindings::identities(&fixture);
    let ring = names.candidates.iter_mut().find(|c| c.name == 301).unwrap();
    ring.wires[1].actual = ring.wires[0].actual.clone();
    assert!(capture(&fixture, &names, [1., 1.], [0.25, 0.25]).is_err());
}
#[test]
fn private_inner_and_material_witnesses_are_hypotheses_not_authority() {
    let fixture = TwoLoopFixture::nested();
    let names = fixture_bindings::identities(&fixture);
    for (inner, material) in [
        ([0.25, 0.25], [0.25, 0.25]),
        ([1., 1.], [1., 1.]),
        ([1., 1.], [0.5, 0.5]),
    ] {
        assert!(matches!(
            capture(&fixture, &names, inner, material)
                .err()
                .unwrap()
                .cause,
            TwoLoopRegionCause::WindingLocation { .. }
        ));
    }
}

fn check_dispositions(captured: &[TwoLoopCandidateProof]) {
    assert_eq!(
        captured.iter().map(|r| r.name).collect::<Vec<_>>(),
        [301, 302, 303]
    );
    assert_eq!(
        captured[0].disposition,
        TwoLoopCandidateDisposition::SelectedAnnulus
    );
    assert_eq!(
        captured[1].disposition,
        TwoLoopCandidateDisposition::RejectedInnerDomain
    );
    assert_eq!(
        captured[2].disposition,
        TwoLoopCandidateDisposition::RejectedOuterComplement
    );
}
