// SPDX-License-Identifier: GPL-2.0-only
//! This fixture calls production construction, correspondence and publication; it authors no lookup IDs.
use super::super::request::OwnedPolynomialPrismRequest;
use super::*;
use crate::sketch_region_prism::publication::*;
use rustycad_tkernel::standard::Failure;
pub(in crate::sketch_region_prism) struct ResolvedPrismFixture {
    pub request: OwnedPolynomialPrismRequest,
    pub publication: PublishedNamedPrism,
    pub names: body_map::ResolvedPrismNames,
}
/// Run the actual owned path; `resolved_prism_fixture(request)` predeclares V2 before geometry.
pub(in crate::sketch_region_prism) fn resolved_prism_fixture(
    request: OwnedPolynomialPrismRequest,
) -> ResolvedPrismFixture {
    let plan = fixture_plan(&request);
    let prepared = super::super::test_fixture::prepared(request);
    let invocation = invocation::resolve_invocation(&prepared, plan, &mut || false)
        .unwrap_or_else(|_| panic!("actual correspondence required"));
    complete_fixture(publish_fixture(&invocation))
}
fn fixture_plan(request: &OwnedPolynomialPrismRequest) -> SingleBoundaryNamePlan {
    assert_eq!(request.sources.len(), 1);
    SingleBoundaryNamePlan::v2(
        request.sources[0].names(request.operation),
        request.selection,
    )
    .unwrap()
}
fn publish_fixture(
    invocation: &invocation::ResolvedPrismInvocation<'_>,
) -> ResolvedPrismPublication {
    invocation
        .publish(&mut ProceedPublication)
        .unwrap_or_else(|failure| {
            panic!(
                "actual publication {:?} {:?}",
                failure.failure().phase,
                failure.failure().class
            )
        })
}
fn complete_fixture(invoked: ResolvedPrismPublication) -> ResolvedPrismFixture {
    let names = body_map::issue_resolved_prism_names(&invoked).unwrap();
    let (request, publication) = invoked.into_parts();
    ResolvedPrismFixture {
        request,
        publication,
        names,
    }
}
/// Actual default owned input, never fixture naming; `default_resolved_prism_fixture()`.
pub(in crate::sketch_region_prism) fn default_resolved_prism_fixture() -> ResolvedPrismFixture {
    resolved_prism_fixture(super::super::test_fixture::request())
}

struct ProceedPublication;
impl PrismPublicationProgress for ProceedPublication {
    fn checkpoint(&mut self, _: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        Ok(false)
    }
}
