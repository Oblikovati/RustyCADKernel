// SPDX-License-Identifier: GPL-2.0-only
//! Owned coefficients only: authored outer/inner poles, roles and names drive the whole pipeline.
use super::super::names::{CallerBoundaryId, CallerDirectedUseId, CallerVertexId};
use super::*;
use crate::sketch_region_prism::frozen::annular_fixture::authored;
use crate::sketch_region_prism::frozen::annular_history::fixture::{
    authored_names, fresh_embedding,
};
use crate::sketch_region_prism::frozen::annular_orientation::fixture::current_binding;
use crate::sketch_region_prism::publication::annular::{
    AnnularPublicationRequest, publish_annular_prism, tests::options,
};
use crate::sketch_region_prism::publication::{
    PrismPublicationCheckpoint, PrismPublicationProgress,
};
use crate::sketch_region_prism::region_certificate::two_loop::{
    TwoLoopRegionOutcome, TwoLoopRegionProgress, TwoLoopRegionStage, fixture_bindings,
    inspect_two_loop_region,
};
use rustycad_tkernel::standard::Failure;

fn boundary(source: u64, scale: f64, offset: f64, reversed: bool) -> OwnedPolynomialBoundary {
    let pole = |x: f64, y: f64| [offset + scale * x, offset + scale * y, 0.];
    OwnedPolynomialBoundary {
        source: CallerBoundaryId(source),
        revision: super::super::request::CallerSourceRevision(1),
        closure: CallerVertexId(source + 500),
        native_start: CallerDirectedUseId(source + 600),
        native_end: CallerDirectedUseId(source + 700),
        world_poles: vec![pole(0., 0.), pole(4., 0.), pole(0., 4.), pole(0., 0.)],
        reversed_traversal: reversed,
    }
}
const LIMIT: f64 = 1e-8;
fn source_policy(roots: RootIsolationOptions) -> SourceConformancePolicy {
    SourceConformancePolicy {
        roots,
        original_plane_limit: 0.,
        construction_limit: 0.,
    }
}
fn contact_policy(roots: RootIsolationOptions) -> ContactTransportPolicy {
    ContactTransportPolicy {
        native_parameter_limits: [LIMIT; 2],
        world_endpoint_limit: LIMIT,
        roots,
    }
}
fn policy() -> OwnedAnnularPolicy {
    let roots = RootIsolationOptions::new(0.001);
    let limit = LIMIT;
    OwnedAnnularPolicy {
        census: CensusPolicy {
            roots,
            max_world_error: limit,
        },
        source: source_policy(roots),
        contacts: contact_policy(roots),
        classification_roots: roots,
        spatial_limit: limit,
        parameter_limit: limit,
        region: fixture_bindings::options(),
    }
}
fn names() -> AnnularRegionNames {
    AnnularRegionNames {
        annulus: 301,
        inner_domain: 302,
        outer_complement: 303,
        annulus_outer_wire: 401,
        annulus_inner_wire: 402,
        inner_domain_wire: 403,
        outer_complement_wire: 404,
    }
}
/// The independent nested pair: outer loop (scale 1) and inner loop (scale 1/2, offset 1/2).
pub(in crate::sketch_region_prism) fn owned_request(reversed: bool) -> OwnedAnnularPrismRequest {
    OwnedAnnularPrismRequest {
        version: 1,
        operation: CallerOperationId(77),
        outer: boundary(101, 1., 0., reversed),
        inner: boundary(202, 0.5, 0.5, reversed),
        regions: names(),
        inner_interior: [1., 1.],
        material_seed: [0.25, 0.25],
        declared_plane: Plane3::default(),
        extrusion: [0., 0., 2.],
        policy: policy(),
    }
}
pub(in crate::sketch_region_prism) struct Continue;
impl OwnedPrismProgress for Continue {
    fn checkpoint(&mut self, _: OwnedPrismCheckpoint) -> Result<bool, Failure> {
        Ok(false)
    }
}
impl TwoLoopRegionProgress for Continue {
    fn checkpoint(&mut self, _: TwoLoopRegionStage) -> Result<bool, Failure> {
        Ok(false)
    }
}
impl PrismPublicationProgress for Continue {
    fn checkpoint(&mut self, _: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        Ok(false)
    }
}
fn refused(request: OwnedAnnularPrismRequest) -> Box<AnnularPreparationFailure> {
    let outcome = prepare_owned_annular_prism(request, &mut Continue);
    outcome.err().expect("expected refused preparation")
}
pub(in crate::sketch_region_prism) fn prepared(reversed: bool) -> PreparedAnnularPrism {
    let outcome = prepare_owned_annular_prism(owned_request(reversed), &mut Continue);
    outcome.unwrap_or_else(|f| panic!("refused at {:?} class {:?}", f.phase, f.class))
}
#[test]
fn owned_roles_name_every_actual_candidate_and_wire_by_source_parents() {
    let prepared = prepared(false);
    let identities = prepared.identities();
    let roles = (identities.outer, identities.inner);
    assert_eq!(roles, (RegionSourceId(101), RegionSourceId(202)));
    let mut names: Vec<_> = identities.candidates.iter().map(|c| c.name).collect();
    names.sort();
    assert_eq!(names, [301, 302, 303]);
    let annulus = identities
        .candidates
        .iter()
        .find(|c| c.name == 301)
        .unwrap();
    let mut wires: Vec<_> = annulus.wires.iter().map(|w| (w.name, w.source.0)).collect();
    wires.sort();
    assert_eq!(wires, [(401, 101), (402, 202)]);
    assert!(prepared.receipt().diagnostic.is_some());
}
#[test]
fn owned_preparation_is_certified_by_the_independent_two_loop_proof() {
    for reversed in [false, true] {
        let certified = prepared(reversed).with_region(|region, _| {
            let outcome = inspect_two_loop_region(region, &mut Continue);
            matches!(outcome, TwoLoopRegionOutcome::Inspected(_))
        });
        assert!(certified, "reversed={reversed}");
    }
}
fn publish(region: &TwoLoopRegionRequest<'_, '_>, receipt: &PrismReceipt) -> Vec<u8> {
    let publication = authored(region, receipt, &current_binding(region, receipt));
    let (copied, historical) = authored_names(region, receipt);
    let request = AnnularPublicationRequest {
        region,
        receipt,
        bindings: &publication,
        copied: &copied,
        historical: &historical,
        requested: options(),
    };
    let embedding = fresh_embedding(region, receipt);
    let result = publish_annular_prism(&request, embedding, &mut Continue);
    let result = result.unwrap_or_else(|f| panic!("publication refused at {:?}", f.phase));
    assert_eq!(result.history().continued().len(), 6);
    result.replay_bytes().to_vec()
}
#[test]
fn owned_annulus_publishes_through_the_shared_named_operation() {
    let first = prepared(false).with_region(publish);
    let second = prepared(false).with_region(publish);
    assert_eq!(first, second);
}
#[test]
fn invalid_owned_inputs_refuse_before_construction() {
    let mut request = owned_request(false);
    request.regions.inner_domain = request.regions.annulus;
    use AnnularAdmissionCause as A;
    use AnnularPreparationCause::Admission;
    assert!(matches!(refused(request).cause, Admission(A::Names { .. })));
    let mut request = owned_request(false);
    request.inner.world_poles.pop();
    assert!(matches!(refused(request).cause, Admission(A::Owned(_))));
    let mut request = owned_request(false);
    request.version = 2;
    assert_eq!(refused(request).class, OwnedPreparationClass::Incomplete);
}
struct CancelAt(usize, usize);
impl OwnedPrismProgress for CancelAt {
    fn checkpoint(&mut self, _: OwnedPrismCheckpoint) -> Result<bool, Failure> {
        self.1 += 1;
        Ok(self.1 == self.0)
    }
}
#[test]
fn every_preparation_checkpoint_cancels_and_reuse_is_clean() {
    let mut counter = CancelAt(0, 0);
    assert!(prepare_owned_annular_prism(owned_request(false), &mut counter).is_ok());
    let total = counter.1;
    for cutoff in 1..=total {
        let outcome = prepare_owned_annular_prism(owned_request(false), &mut CancelAt(cutoff, 0));
        let class = outcome.err().expect("cancelled preparation").class;
        assert_eq!(
            class,
            OwnedPreparationClass::Cancelled,
            "cutoff {cutoff}/{total}"
        );
    }
    assert_eq!(prepared(false).identities().selected, 301);
}
