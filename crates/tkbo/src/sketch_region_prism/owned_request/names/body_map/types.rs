// SPDX-License-Identifier: GPL-2.0-only
use super::super::{body_keys::*, *};
use crate::sketch_region_prism::{
    frozen::{FrozenPrismKey, FrozenUseKey},
    publication::PublishedNamedPrism,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedPrismNames {
    pub(super) envelope: SemanticReplayEnvelope,
    pub(super) members: Vec<SemanticMemberBinding>,
    pub(super) uses: Vec<SemanticUseBinding>,
    pub(super) replay: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PrismSemanticMapFailure {
    pub operation: CallerOperationId,
    pub cause: PrismSemanticMapCause,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PrismSemanticMapCause {
    UnknownToken {
        actual: u64,
    },
    Operation {
        actual: u64,
        expected: u64,
    },
    MemberMultiplicity {
        key: FrozenPrismKey,
        actual: usize,
        domain: usize,
        expected: usize,
    },
    UseMultiplicity {
        key: FrozenUseKey,
        actual: usize,
        domain: usize,
        expected: usize,
    },
    MemberExpansion {
        key: FrozenPrismKey,
    },
    UseExpansion {
        key: FrozenUseKey,
    },
    MemberCollision {
        first: FrozenPrismKey,
        second: FrozenPrismKey,
    },
    UseCollision {
        first: FrozenUseKey,
        second: FrozenUseKey,
    },
    MissingAncestor {
        key: FrozenUseKey,
        parent: FrozenUseKey,
    },
    UnsupportedSourceSlot {
        slot: SemanticNameSlot,
    },
    UnsupportedUseSlot {
        slot: SemanticNameSlot,
        kind: u8,
    },
    OutputSlot {
        key: FrozenPrismKey,
        actual: Option<SemanticNameSlot>,
        expected: Option<SemanticNameSlot>,
    },
    UnsupportedMember {
        key: FrozenPrismKey,
    },
    EmptyUse,
    EmptyDomain {
        members: usize,
        uses: usize,
    },
}
pub(super) type MapCheck<T> = Result<T, Box<PrismSemanticMapFailure>>;
pub(super) struct NameMapDomain<'a> {
    pub members: &'a [FrozenPrismKey],
    pub uses: &'a [FrozenUseKey],
    pub publication: &'a [u8],
}
impl ResolvedPrismNames {
    /// Exact owned publication binding; `names.binds(result)` does not inspect live geometry.
    pub(crate) fn binds(&self, result: &PublishedNamedPrism) -> bool {
        self.envelope.child_bytes() == result.replay_bytes()
    }
    /// Retain every caller identity and reserved slot; `names.plan().entries()`.
    pub(crate) fn plan(&self) -> &SingleBoundaryNamePlan {
        self.envelope.plan()
    }
    /// Mapper encoding is independent of legacy name V1; `names.version()` is explicit.
    pub(crate) fn version(&self) -> u32 {
        1
    }
    /// Borrow complete definition correspondence; `names.member_bindings()` is immutable.
    pub(crate) fn member_bindings(&self) -> &[SemanticMemberBinding] {
        &self.members
    }
    /// Borrow complete directed correspondence; `names.use_bindings()` retains all ancestry.
    pub(crate) fn use_bindings(&self) -> &[SemanticUseBinding] {
        &self.uses
    }
    /// Missing names refuse without allocation; `names.member(key)` never invents identity.
    pub(crate) fn member(&self, key: &FrozenPrismKey) -> MapCheck<&SemanticMemberKey> {
        self.members
            .iter()
            .find(|row| &row.frozen == key)
            .map(|row| &row.semantic)
            .ok_or_else(|| {
                self.failure(PrismSemanticMapCause::MemberMultiplicity {
                    key: *key,
                    actual: 0,
                    domain: 1,
                    expected: 1,
                })
            })
    }
    /// Lookup a full path; `names.occurrence(key)` never looks up only its final use token.
    pub(crate) fn occurrence(&self, key: &FrozenUseKey) -> MapCheck<&SemanticUseKey> {
        self.uses
            .iter()
            .find(|row| &row.frozen == key)
            .map(|row| &row.semantic)
            .ok_or_else(|| {
                self.failure(PrismSemanticMapCause::UseMultiplicity {
                    key: key.clone(),
                    actual: 0,
                    domain: 1,
                    expected: 1,
                })
            })
    }
    /// Own dictionary, full publication and every mapping row; `names.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
    fn failure(&self, cause: PrismSemanticMapCause) -> Box<PrismSemanticMapFailure> {
        failure(self.plan(), cause)
    }
}
pub(super) fn failure(
    plan: &SingleBoundaryNamePlan,
    cause: PrismSemanticMapCause,
) -> Box<PrismSemanticMapFailure> {
    Box::new(PrismSemanticMapFailure {
        operation: plan.identities().operation,
        cause,
    })
}
