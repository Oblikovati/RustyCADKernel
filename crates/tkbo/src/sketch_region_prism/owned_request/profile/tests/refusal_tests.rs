// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0068 refusals: typed admission causes, crossing and touching loops, seeds on
//! boundaries or outside every loop, duplicate selections and cancellation.
use super::*;

fn refusal(profile: Profile) -> LoopRegionFailure {
    prepared(profile).inspect_regions(|| false).unwrap_err()
}
fn admission(profile: Profile) -> ProfileAdmissionCause {
    let failure = prepare_owned_profile(profile.request(), &mut Continue)
        .err()
        .unwrap();
    let ProfilePreparationCause::Admission(cause) = failure.cause else {
        panic!("expected admission refusal at {:?}", failure.phase);
    };
    cause
}

#[test]
fn admission_refuses_bad_weights_open_loops_and_shared_vertex_mismatch() {
    let mut profile = Profile::default();
    profile.circle(10, [0., 0.], 1., false);
    profile.edges[1].world_weights = Some(vec![1., 0., 2.]);
    profile.select(1, [0., 0.]);
    let cause = admission(profile);
    let expected = ProfileEdgeCause::Weight {
        index: 1,
        actual: 0.,
    };
    assert_eq!(
        cause,
        ProfileAdmissionCause::Edge {
            source: CallerBoundaryId(11),
            cause: expected
        }
    );

    let mut open = Profile::default();
    open.circle(10, [0., 0.], 1., false);
    open.loops[0].uses.swap(1, 2);
    open.select(1, [0., 0.]);
    assert!(matches!(
        admission(open),
        ProfileAdmissionCause::Loop {
            name: 1010,
            cause: ProfileLoopCause::Disconnected { .. }
        }
    ));

    let mut moved = Profile::default();
    moved.circle(10, [0., 0.], 1., false);
    moved.edges[0].world_poles[0] = [1., 0.5, 0.];
    moved.select(1, [0., 0.]);
    assert!(matches!(
        admission(moved),
        ProfileAdmissionCause::Vertex { .. }
    ));

    let mut empty = Profile::default();
    empty.circle(10, [0., 0.], 1., false);
    assert_eq!(admission(empty), ProfileAdmissionCause::EmptySelection);
}
#[test]
fn crossing_circles_refuse_with_the_unexpected_contact() {
    let mut profile = Profile::default();
    profile.circle(10, [0., 0.], 1., false);
    profile.circle(20, [1., 0.], 1., false);
    profile.select(1, [-0.5, 0.]);
    let failure = refusal(profile);
    assert_eq!(failure.stage, LoopRegionStage::Joins);
    assert!(
        matches!(failure.cause, LoopRegionCause::UnexpectedContact { .. }),
        "{failure:?}"
    );
}
#[test]
fn externally_touching_circles_refuse() {
    let mut profile = Profile::default();
    profile.circle(10, [0., 0.], 1., false);
    profile.circle(20, [2., 0.], 1., false);
    profile.select(1, [0., 0.]);
    // The tangent contact of two distinct circles is not certified by the pair solver:
    // the proof names the incomplete pair instead of guessing a touching point.
    let failure = refusal(profile);
    assert_eq!(failure.stage, LoopRegionStage::Joins);
    assert!(
        matches!(failure.cause, LoopRegionCause::PairUnavailable { .. }),
        "{failure:?}"
    );
}
#[test]
fn seed_outside_everything_on_a_loop_and_duplicate_faces_refuse() {
    let mut outside = Profile::default();
    outside.circle(10, [0., 0.], 1., false);
    outside.select(1, [5., 5.]);
    assert_eq!(
        refusal(outside).cause,
        LoopRegionCause::UnboundedSelection { region: 1 }
    );

    let mut boundary = Profile::default();
    boundary.circle(10, [0., 0.], 1., false);
    boundary.select(1, [0., 1.]);
    assert_eq!(
        refusal(boundary).cause,
        LoopRegionCause::SeedOnBoundary {
            region: 1,
            loop_name: 1010
        }
    );

    let mut twice = Profile::default();
    twice.circle(10, [0., 0.], 1., false);
    twice.select(1, [0., 0.5]);
    twice.select(2, [0., -0.5]);
    assert_eq!(
        refusal(twice).cause,
        LoopRegionCause::DuplicateSelection { regions: [1, 2] }
    );
}
#[test]
fn every_proof_stage_cancels_and_reuse_is_clean() {
    let mut profile = Profile::default();
    profile.circle(10, [0., 0.], 1., false);
    profile.select(1, [0., 0.]);
    let prepared = prepared(profile);
    let complete = prepared.inspect_regions(|| false).unwrap();
    let mut polls = 0;
    prepared
        .inspect_regions(|| {
            polls += 1;
            false
        })
        .unwrap();
    // Every early checkpoint, then a deterministic stride through the numerical stages
    // (thousands of winding and area polls) and the final one.
    let stride = (polls / 64).max(1);
    let stops = (0..polls.min(64)).chain((64..polls).step_by(stride)).chain([polls - 1]);
    for stop in stops {
        let mut seen = 0;
        let failure = prepared
            .inspect_regions(|| {
                seen += 1;
                seen > stop
            })
            .unwrap_err();
        assert_eq!(failure.cause, LoopRegionCause::Cancelled, "stop {stop}");
    }
    assert_eq!(prepared.inspect_regions(|| false).unwrap(), complete);
}
#[test]
fn unsupported_region_version_refuses() {
    let mut profile = Profile::default();
    profile.circle(10, [0., 0.], 1., false);
    profile.select(1, [0., 0.]);
    let mut request = profile.request();
    request.policy.region.version = 2;
    let prepared = prepare_owned_profile(request, &mut Continue).ok().unwrap();
    let failure = prepared.inspect_regions(|| false).unwrap_err();
    assert_eq!(
        failure.cause,
        LoopRegionCause::Version {
            actual: 2,
            expected: 1
        }
    );
}
#[test]
fn single_edge_full_circle_is_improper_and_refuses_by_its_pair() {
    // ADR 0067: a closed one-edge rational circle is a 2:1 map; its self census refuses.
    let mut profile = Profile::default();
    let poles = [
        [-1., 0.],
        [-1., -4.],
        [3., -2.],
        [3., 2.],
        [-1., 4.],
        [-1., 0.],
    ];
    profile.edge(1, &poles, Some(&[5., 1., 1., 1., 1., 5.]), [7, 7]);
    profile.chain(100, &[1]);
    profile.select(1, [0., 0.]);
    let failure = refusal(profile);
    let sources = [crate::sketch_region::RegionSourceId(1); 2];
    assert_eq!(failure.cause, LoopRegionCause::PairUnavailable { sources });
}
