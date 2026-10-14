// SPDX-License-Identifier: GPL-2.0-only
//! Independent authored roles exercise the original retained construction and certificate together.
use super::super::two_loop::{fixture::TwoLoopFixture, fixture_bindings};
use super::*;
use rustycad_tkernel::standard::Failure;
struct ContinueRegion;
impl TwoLoopRegionProgress for ContinueRegion {
    fn checkpoint(&mut self, _: TwoLoopRegionStage) -> Result<bool, Failure> {
        Ok(false)
    }
}
fn with_actual<R>(
    run: impl FnOnce(
        &TwoLoopRegionRequest<'_, '_>,
        Box<TwoLoopRegionCertificate>,
        &mut PrismReceipt,
    ) -> R,
) -> R {
    let fixture = TwoLoopFixture::nested();
    let sources = fixture.sources();
    let identities = fixture_bindings::identities(&fixture);
    let request = TwoLoopRegionRequest {
        bridge: &fixture.bridge,
        sources: &sources,
        identities: &identities,
        inner_interior: [1., 1.],
        material_seed: [0.25, 0.25],
        requested: fixture_bindings::options(),
    };
    let TwoLoopRegionOutcome::Inspected(region) =
        inspect_two_loop_region(&request, &mut ContinueRegion)
    else {
        panic!("expected independent nested region proof");
    };
    let mut receipt = super::super::super::annular_construction::retained_annular_sweep(&fixture);
    run(&request, region, &mut receipt)
}
#[test]
fn actual_copy_has_nine_source_keyed_directed_members_and_two_full_boundaries() {
    with_actual(|request, region, receipt| {
        let captured = capture::copy_inputs(request, &region, receipt)
            .unwrap_or_else(|_| panic!("expected actual copy binding"));
        assert_eq!(captured.members.len(), 9);
        assert_eq!(captured.selected.boundaries.len(), 2);
        assert_eq!(captured.copied.boundaries.len(), 2);
        for wire in [401, 402] {
            let roles: Vec<_> = captured
                .members
                .iter()
                .filter(|m| m.key.wire == Some(wire))
                .map(|m| m.key.role)
                .collect();
            assert_eq!(
                roles,
                [
                    TwoLoopMemberRole::Wire,
                    TwoLoopMemberRole::Boundary,
                    TwoLoopMemberRole::NativeStart,
                    TwoLoopMemberRole::NativeEnd
                ]
            );
        }
    });
}
#[test]
fn duplicate_copy_answer_cannot_replace_an_equal_count_directed_member() {
    with_actual(|request, region, receipt| {
        let copied = receipt.copied.as_mut().unwrap();
        copied.observations[1] = copied.observations[0].clone();
        assert!(capture::copy_inputs(request, &region, receipt).is_err());
    });
}
#[test]
fn source_role_key_does_not_follow_returned_copy_order() {
    with_actual(|request, region, receipt| {
        let baseline = capture::copy_inputs(request, &region, receipt)
            .unwrap_or_else(|_| panic!("expected baseline"));
        receipt.copied.as_mut().unwrap().observations.reverse();
        let reversed = capture::copy_inputs(request, &region, receipt)
            .unwrap_or_else(|_| panic!("expected order independent binding"));
        assert_eq!(
            baseline.members.iter().map(|m| m.key).collect::<Vec<_>>(),
            reversed.members.iter().map(|m| m.key).collect::<Vec<_>>()
        );
    });
}

#[test]
fn actual_four_charts_retain_twelve_directed_trims_and_native_closure_roles() {
    with_actual(|request, region, receipt| {
        let copy =
            capture::copy_inputs(request, &region, receipt).unwrap_or_else(|_| panic!("copy"));
        let boundaries =
            boundaries::capture(&region, &copy, receipt).unwrap_or_else(|_| panic!("boundaries"));
        let charts = charts::capture(receipt, &copy, &boundaries, &mut || false)
            .unwrap_or_else(|_| panic!("charts"));
        assert_eq!(
            boundaries
                .iter()
                .map(|b| (b.source.0, b.wire))
                .collect::<Vec<_>>(),
            [(101, 401), (202, 402)]
        );
        assert_eq!(charts.len(), 4);
        assert_eq!(charts.iter().map(|c| c.trims.len()).sum::<usize>(), 12);
        for boundary in boundaries {
            assert_native_roles(&boundary);
        }
        for side in charts.iter().filter(|c| c.source_boundary.is_some()) {
            assert_seam_contexts(side);
        }
    });
}
fn assert_native_roles(boundary: &AnnularBoundaryBinding) {
    assert_eq!(
        boundary
            .endpoints
            .iter()
            .map(|e| e.parameter)
            .collect::<Vec<_>>(),
        [0., 1.]
    );
    assert_eq!(boundary.endpoints.len(), 2);
    assert_ne!(boundary.endpoints[0].context, boundary.endpoints[1].context);
    assert_eq!(
        boundary.endpoints[0].context.occurrence.topology_address,
        boundary.endpoints[1].context.occurrence.topology_address
    );
}
fn assert_seam_contexts(chart: &AnnularBoundChart) {
    use super::super::super::surface_charts::PrismTrimRole;
    let start = chart
        .trims
        .iter()
        .find(|t| t.role == PrismTrimRole::Start)
        .unwrap();
    let end = chart
        .trims
        .iter()
        .find(|t| t.role == PrismTrimRole::End)
        .unwrap();
    assert_ne!(start.output, end.output);
    assert_ne!(start.source, end.source);
    assert_eq!(
        start.output.occurrence.topology_address,
        end.output.occurrence.topology_address
    );
    assert_eq!(start.output.ancestors, end.output.ancestors);
    assert_ne!(start.output.child_slots, end.output.child_slots);
}
