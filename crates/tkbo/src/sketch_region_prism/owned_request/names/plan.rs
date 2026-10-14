// SPDX-License-Identifier: GPL-2.0-only
//! Names are predeclared reservations; only a later unique correspondence proof may bind them.
use super::roles::{OUTPUT_ROLES, REGION_ROLES, REGION_STAGES, SOURCE_ROLES, SOURCE_STAGES};
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SemanticNameEntry {
    pub token: SemanticNameToken,
    pub slot: SemanticNameSlot,
    pub operation: CallerOperationId,
    pub source: CallerBoundaryId,
    pub anchor: SemanticCallerAnchor,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SingleBoundaryNamePlan {
    version: u32,
    identities: SingleBoundaryIdentities,
    seed_bits: [u64; 2],
    entries: Vec<SemanticNameEntry>,
}
impl SingleBoundaryNamePlan {
    /// Author V1 symbols before construction; example: `SingleBoundaryNamePlan::v1(ids, intent)`.
    pub(crate) fn v1(
        ids: SingleBoundaryIdentities,
        intent: SemanticSelectionIntent,
    ) -> Result<Self, SemanticNameFailure> {
        let SemanticSelectionIntent::BoundedRegionContaining { seed } = intent;
        validate_names(ids, seed)?;
        let mut plan = Self {
            version: 1,
            identities: ids,
            seed_bits: seed.map(f64::to_bits),
            entries: vec![],
        };
        plan.reserve_vocabulary();
        plan.entries.sort_by_key(|entry| entry.token);
        Ok(plan)
    }
    /// Add aggregate symbols before lookup; example: `SingleBoundaryNamePlan::v2(ids, intent)`.
    pub(crate) fn v2(
        ids: SingleBoundaryIdentities,
        intent: SemanticSelectionIntent,
    ) -> Result<Self, SemanticNameFailure> {
        let mut plan = Self::v1(ids, intent)?;
        plan.version = 2;
        for stage in [
            SemanticNameStage::Partition,
            SemanticNameStage::CandidateCopy,
        ] {
            plan.reserve(
                stage,
                &[SemanticNameRole::Aggregate, SemanticNameRole::AggregateUse],
            );
        }
        plan.entries.sort_by_key(|entry| entry.token);
        Ok(plan)
    }
    /// Naming behavior is separate from request policy version; example: `plan.version()`.
    pub(crate) fn version(&self) -> u32 {
        self.version
    }
    /// Complete reserved dictionary, not actual incidence coverage; example: `plan.entries()`.
    pub(crate) fn entries(&self) -> &[SemanticNameEntry] {
        &self.entries
    }
    /// Full caller identities remain separate; example: `plan.identities().boundary`.
    pub(crate) fn identities(&self) -> SingleBoundaryIdentities {
        self.identities
    }
    /// Exact requested seed including signed zero; example: `plan.selection()`.
    pub(crate) fn selection(&self) -> SemanticSelectionIntent {
        SemanticSelectionIntent::BoundedRegionContaining {
            seed: self.seed_bits.map(f64::from_bits),
        }
    }
    /// Refuse unknown values instead of inventing names; example: `plan.reverse(0)`.
    pub(crate) fn reverse(&self, token: u64) -> Result<&SemanticNameEntry, SemanticNameFailure> {
        self.entries
            .iter()
            .find(|entry| entry.token.value() == token)
            .ok_or(SemanticNameFailure::UnknownToken {
                actual: token,
                version: self.version,
            })
    }
    fn reserve(&mut self, stage: SemanticNameStage, roles: &[SemanticNameRole]) {
        for &role in roles {
            let slot = SemanticNameSlot { stage, role };
            self.entries.push(SemanticNameEntry {
                token: slot.token(),
                slot,
                operation: self.identities.operation,
                source: self.identities.boundary,
                anchor: anchor(self.identities, role),
            });
        }
    }
    fn reserve_vocabulary(&mut self) {
        use SemanticNameStage::*;
        self.reserve(Request, super::roles::REQUEST_ROLES);
        for &stage in SOURCE_STAGES {
            self.reserve(stage, SOURCE_ROLES);
        }
        for &stage in REGION_STAGES {
            self.reserve(stage, REGION_ROLES);
        }
        self.reserve(Output, OUTPUT_ROLES);
        self.reserve(
            Output,
            &[SemanticNameRole::StartUse, SemanticNameRole::EndUse],
        );
    }
}
fn validate_names(
    ids: SingleBoundaryIdentities,
    seed: [f64; 2],
) -> Result<(), SemanticNameFailure> {
    if ids.start == ids.end {
        return Err(SemanticNameFailure::EqualEndpointUses { actual: ids.start });
    }
    for (axis, actual) in seed.into_iter().enumerate() {
        if !actual.is_finite() {
            return Err(SemanticNameFailure::NonfiniteSeed { axis, actual });
        }
    }
    Ok(())
}
fn anchor(ids: SingleBoundaryIdentities, role: SemanticNameRole) -> SemanticCallerAnchor {
    use SemanticNameRole::*;
    match role {
        Operation => SemanticCallerAnchor::Operation(ids.operation),
        ClosureVertex | ClosureContact | FirstVertex | LastVertex | Seam => {
            SemanticCallerAnchor::Closure(ids.closure)
        }
        StartUse | SeamStartUse => SemanticCallerAnchor::Start(ids.start),
        EndUse | SeamEndUse => SemanticCallerAnchor::End(ids.end),
        _ => SemanticCallerAnchor::Boundary(ids.boundary),
    }
}
