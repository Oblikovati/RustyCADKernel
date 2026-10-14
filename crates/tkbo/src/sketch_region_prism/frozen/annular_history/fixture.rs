// SPDX-License-Identifier: GPL-2.0-only
//! Test authorship only: names come from semantic source/candidate roles, never returned order.
use super::super::annular_fixture::authored;
use super::super::annular_join::{AnnularFrozenDomain, join_annular_frozen_domain};
use super::super::annular_orientation::fixture::*;
use super::*;
use crate::sketch_region::{PartitionObservation, PartitionOccurrenceIdentity};
use crate::sketch_region_prism::region_certificate::embedding::inspect_bound_annular_embedding;
use crate::sketch_region_prism::region_certificate::two_loop::{
    fixture::TwoLoopFixture, fixture_bindings,
};
use crate::sketch_region_prism::{PrismBodyContext, contexts::body_contexts};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// Owned authored history inputs for one actual annular operation.
pub(in crate::sketch_region_prism) struct AnnularHistoryNames {
    pub joined: AnnularFrozenDomain,
    pub copied: PrismCopyBindings,
    pub historical: AnnularHistoryBindings,
    pub requested: PrismHistoryOptions,
}
/// One owned actual operation; tests mutate any field between `run` calls.
pub(in crate::sketch_region_prism) struct AnnularHistoryCase {
    pub fixture: TwoLoopFixture,
    pub reversed: bool,
    pub identities: TwoLoopRegionIdentities,
    pub receipt: PrismReceipt,
    pub publication: PrismPublicationBindings,
    pub names: AnnularHistoryNames,
}
impl AnnularHistoryCase {
    /// The nested polynomial pair with default placement, traversal and sweep.
    pub fn nested() -> Self {
        Self::variant(Variant::default(), false)
    }
    /// Build, bind, embed and join one actual variant, then author history names.
    pub fn variant(variant: Variant, reversed: bool) -> Self {
        let fixture = source_fixture(variant);
        let identities = fixture_bindings::identities(&fixture);
        let receipt = variant_receipt(&fixture, variant);
        let (publication, names) = {
            let sources = ordered_sources(&fixture, reversed);
            let request = region_request(&fixture, &sources, &identities);
            join_names(&request, &receipt)
        };
        Self {
            fixture,
            reversed,
            identities,
            receipt,
            publication,
            names,
        }
    }
    /// Run against the current owned fields; `case.run(|request| inspect(request))`.
    pub fn run<T>(&self, run: impl FnOnce(&AnnularHistoryRequest<'_, '_, '_, '_>) -> T) -> T {
        let sources = ordered_sources(&self.fixture, self.reversed);
        let region = region_request(&self.fixture, &sources, &self.identities);
        run(&AnnularHistoryRequest {
            region: &region,
            receipt: &self.receipt,
            publication: &self.publication,
            copied: &self.names.copied,
            historical: &self.names.historical,
            joined: &self.names.joined,
            requested: self.names.requested,
        })
    }
}
impl AnnularHistoryCase {
    /// Run with the current region request only; `case.with_region(|region| ...)`.
    pub fn with_region<T>(&self, run: impl FnOnce(&TwoLoopRegionRequest<'_, '_>) -> T) -> T {
        let sources = ordered_sources(&self.fixture, self.reversed);
        run(&region_request(&self.fixture, &sources, &self.identities))
    }
}
/// Fresh binding and embedding evidence for a later join; `fresh_embedding(region, receipt)`.
pub(in crate::sketch_region_prism) fn fresh_embedding(
    region: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
) -> crate::sketch_region_prism::region_certificate::embedding::AnnularEmbeddingEvidence {
    let binding = current_binding(region, receipt);
    inspect_bound_annular_embedding(region, receipt, binding, embedding_options(), || false)
        .unwrap_or_else(|failure| panic!("fresh embedding: {:?}", failure.classification()))
}
fn region_request<'a, 'b>(
    fixture: &'a TwoLoopFixture,
    sources: &'b [crate::sketch_region::ArrangementSource<'a>],
    identities: &'b TwoLoopRegionIdentities,
) -> TwoLoopRegionRequest<'a, 'b> {
    TwoLoopRegionRequest {
        bridge: &fixture.bridge,
        sources,
        identities,
        inner_interior: [1., 1.],
        material_seed: [0.25, 0.25],
        requested: fixture_bindings::options(),
    }
}
fn join_names(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
) -> (PrismPublicationBindings, AnnularHistoryNames) {
    let binding = current_binding(request, receipt);
    let publication = authored(request, receipt, &binding);
    let proof =
        inspect_bound_annular_embedding(request, receipt, binding, embedding_options(), || false)
            .unwrap_or_else(|failure| panic!("history embedding: {:?}", failure.classification()));
    let joined = join_annular_frozen_domain(
        request,
        receipt,
        &publication,
        proof,
        join_options(),
        &mut ContinueJoin,
    )
    .unwrap_or_else(|failure| panic!("history join at {:?}", failure.phase));
    let names = authored_inputs(request, receipt, joined);
    (publication, names)
}
/// Author copy and scope names for an already joined operation; lookup values only.
pub(in crate::sketch_region_prism) fn authored_inputs(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
    joined: AnnularFrozenDomain,
) -> AnnularHistoryNames {
    let roles = SourceRoles::new(request);
    let copied = copy_bindings(receipt, &roles);
    let historical = history_bindings(request, &roles);
    AnnularHistoryNames {
        joined,
        copied,
        historical,
        requested: PrismHistoryOptions::annular_v2(),
    }
}
/// Semantic source of each physical definition, recorded from authored input/candidate roles.
pub(in crate::sketch_region_prism) struct SourceRoles {
    rows: Vec<(PartitionOccurrenceIdentity, Option<RegionSourceId>)>,
}
impl SourceRoles {
    fn new(request: &TwoLoopRegionRequest<'_, '_>) -> Self {
        let mut result = Self { rows: Vec::new() };
        for input in &partition(request).input_contexts {
            let copied = copied_rows(&input.observation);
            for row in copied.source.iter().chain(&copied.copied) {
                result.add(&row.occurrence, Some(input.source));
            }
        }
        result.add_candidate_copies(request);
        result.add_candidate_wires(request);
        result
    }
    fn add(&mut self, actual: &PartitionOccurrenceIdentity, source: Option<RegionSourceId>) {
        if self.find(actual).is_none() {
            self.rows.push((actual.clone(), source));
        }
    }
    fn find(&self, actual: &PartitionOccurrenceIdentity) -> Option<Option<RegionSourceId>> {
        self.rows
            .iter()
            .find(|(known, _)| same_definition(known, actual))
            .map(|(_, source)| *source)
    }
    fn add_candidate_copies(&mut self, request: &TwoLoopRegionRequest<'_, '_>) {
        let copy = request.bridge.mapped.copied.as_ref().unwrap();
        for link in &copy.links {
            let source = self.find(&link.original).flatten();
            self.add(&link.copied, source);
        }
    }
    fn add_candidate_wires(&mut self, request: &TwoLoopRegionRequest<'_, '_>) {
        for candidate in &request.identities.candidates {
            self.add(&candidate.actual, None);
            for wire in &candidate.wires {
                self.add(&wire.actual, Some(wire.source));
            }
        }
    }
    fn source(&self, actual: &PartitionOccurrenceIdentity, kind: ShapeEnum) -> u64 {
        match (kind, self.find(actual)) {
            (ShapeEnum::Compound | ShapeEnum::Face, _) => 0,
            (_, Some(Some(source))) => source_offset(source),
            (kind, found) => panic!("fixture {kind:?} has source role {found:?}"),
        }
    }
}
fn source_offset(source: RegionSourceId) -> u64 {
    match source.0 {
        101 => 10,
        202 => 20,
        actual => panic!("source {actual}; expected authored outer101 or inner202"),
    }
}
fn kind_offset(kind: ShapeEnum) -> u64 {
    match kind {
        ShapeEnum::Compound => 1,
        ShapeEnum::Face => 2,
        ShapeEnum::Wire => 3,
        ShapeEnum::Edge => 4,
        ShapeEnum::Vertex => 5,
        actual => panic!("fixture kind {actual:?}; expected compound through vertex"),
    }
}
fn use_name(kind: ShapeEnum, orientation: Orientation) -> AuthoredPrismId {
    let end = kind == ShapeEnum::Vertex && orientation == Orientation::Reversed;
    AuthoredPrismId(1700 + kind_offset(kind) + if end { 10 } else { 0 })
}
fn member_name(base: u64, roles: &SourceRoles, row: &RegionObservedContext) -> AuthoredPrismId {
    AuthoredPrismId(base + kind_offset(row.kind) + roles.source(&row.occurrence, row.kind))
}
pub(in crate::sketch_region_prism) fn scope_names(
    scope: AnnularHistoryScope,
    base: u64,
    roles: &SourceRoles,
    rows: &[RegionObservedContext],
) -> AnnularScopeBindings {
    let mut names = PrismHistoricalStageBindings {
        stage: scope.stage(),
        members: Vec::new(),
        uses: Vec::new(),
    };
    for row in rows {
        append_historical_member(&mut names, &row.occurrence, member_name(base, roles, row))
            .unwrap_or_else(|_| panic!("fixture shared definition keeps one member"));
        names.uses.push(PrismHistoricalUseBinding {
            actual: row.clone(),
            authored_use: use_name(row.kind, row.occurrence.orientation),
        });
    }
    AnnularScopeBindings { scope, names }
}
fn history_bindings(
    request: &TwoLoopRegionRequest<'_, '_>,
    roles: &SourceRoles,
) -> AnnularHistoryBindings {
    use AnnularHistoryScope as Scope;
    let named = |scope, base, rows: &[_]| scope_names(scope, base, roles, rows);
    let mut scopes = Vec::new();
    for input in &partition(request).input_contexts {
        let base = 11000 + 100 * source_offset(input.source);
        let rows = &copied_rows(&input.observation).copied;
        scopes.push(named(Scope::InputCopy(input.source), base, rows));
    }
    let candidate = candidate_copy(request);
    scopes.push(named(Scope::Partition, 12000, &candidate.source));
    scopes.push(named(Scope::CandidateCopy, 13000, &candidate.copied));
    scopes.extend(rejected_scopes(request, roles));
    AnnularHistoryBindings { scopes }
}
fn rejected_scopes(
    request: &TwoLoopRegionRequest<'_, '_>,
    roles: &SourceRoles,
) -> Vec<AnnularScopeBindings> {
    let mut result = Vec::new();
    for candidate in &request.identities.candidates {
        if candidate.name != request.identities.selected {
            let rows = candidate_rows(request, &candidate.actual);
            let base = 14000 + 100 * (candidate.name % 100);
            let scope = AnnularHistoryScope::Rejected(candidate.name);
            result.push(scope_names(scope, base, roles, &rows));
        }
    }
    result
}
fn copy_bindings(receipt: &PrismReceipt, roles: &SourceRoles) -> PrismCopyBindings {
    let selected = receipt.input.selected.as_ref().unwrap();
    let mut members: Vec<PrismSourceBinding> = Vec::new();
    for row in &selected.relations {
        let actual = PartitionOccurrenceIdentity::capture(&row.output);
        let kind = row.output.shape_type().unwrap();
        if !members.iter().any(|m| same_definition(&m.actual, &actual)) {
            let member = 1600 + kind_offset(kind) + roles.source(&actual, kind);
            members.push(PrismSourceBinding {
                actual,
                member: AuthoredPrismId(member),
            });
        }
    }
    PrismCopyBindings {
        selected_members: members,
        selected_uses: directed_uses(&selected.face),
        copied_uses: directed_uses(&receipt.copied.as_ref().unwrap().base),
    }
}
fn directed_uses(root: &Shape) -> Vec<PrismDirectedBinding> {
    body_contexts(root)
        .ok()
        .unwrap()
        .into_iter()
        .map(|actual: PrismBodyContext| {
            let kind = actual.occurrence.shape_type().unwrap();
            let authored_use = use_name(kind, actual.occurrence.orientation());
            PrismDirectedBinding {
                actual,
                authored_use,
            }
        })
        .collect()
}
pub(in crate::sketch_region_prism) fn partition<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
) -> &'a crate::sketch_region::OwnedPartitionSnapshot {
    let PartitionObservation::Captured(partition) = &request.bridge.mapped.arrangement.partition
    else {
        panic!("expected retained partition");
    };
    partition
}
fn copied_rows(
    observed: &crate::sketch_region::RegionCopyContextObservation,
) -> &crate::sketch_region::RegionCopyContexts {
    retained_copy_contexts(observed, PrismHistoryStage::InputCopy)
        .unwrap_or_else(|_| panic!("expected actual copier contexts"))
}
fn candidate_copy<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
) -> &'a crate::sketch_region::RegionCopyContexts {
    copied_rows(&request.bridge.mapped.copied.as_ref().unwrap().contexts)
}
pub(in crate::sketch_region_prism) fn candidate_rows(
    request: &TwoLoopRegionRequest<'_, '_>,
    actual: &PartitionOccurrenceIdentity,
) -> Vec<RegionObservedContext> {
    let batch = request
        .bridge
        .mapped
        .copied
        .as_ref()
        .unwrap()
        .completion
        .as_ref()
        .ok()
        .unwrap();
    let candidate =
        resolve_certified_candidate(batch, actual).unwrap_or_else(|_| panic!("candidate"));
    capture_history_contexts(
        &candidate.face,
        PrismHistoryStage::Complement,
        PrismHistoryLimits::bounded_v1(),
    )
    .unwrap_or_else(|_| panic!("candidate contexts"))
}
fn same_definition(a: &PartitionOccurrenceIdentity, b: &PartitionOccurrenceIdentity) -> bool {
    super::super::identities::same_definition(a, b)
}
/// Clone one authored scope roster for adversarial edits; `copy_scope(&scope)`.
pub(in crate::sketch_region_prism) fn copy_scope(
    scope: &AnnularScopeBindings,
) -> AnnularScopeBindings {
    let names = &scope.names;
    let member = |m: &PrismSourceBinding| PrismSourceBinding {
        actual: m.actual.clone(),
        member: m.member,
    };
    let directed = |u: &PrismHistoricalUseBinding| PrismHistoricalUseBinding {
        actual: u.actual.clone(),
        authored_use: u.authored_use,
    };
    let names = PrismHistoricalStageBindings {
        stage: names.stage,
        members: names.members.iter().map(member).collect(),
        uses: names.uses.iter().map(directed).collect(),
    };
    AnnularScopeBindings {
        scope: scope.scope,
        names,
    }
}
