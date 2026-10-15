// SPDX-License-Identifier: GPL-2.0-only
//! The issued dictionary qualifies every internal token with the caller's own identities.
use super::*;

/// Caller meaning of one issued token; vocabulary uses are roles, never occurrence identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum AnnularNameMeaning {
    Operation(CallerOperationId),
    /// The selected annulus region and its copied face.
    Annulus {
        region: u64,
    },
    /// A boundary member (wire, edge or closure vertex) of one caller boundary.
    Boundary {
        role: AnnularBoundaryRole,
        boundary: CallerBoundaryId,
        kind: u8,
        caller: u64,
    },
    /// Directed-use vocabulary of one category; `variant` is its explicit role code.
    UseRole {
        category: AnnularNameCategory,
        kind: u8,
        variant: u8,
    },
    /// An intermediate or selected-copy member: scope, boundary role and kind.
    Member {
        category: AnnularNameCategory,
        scope: AnnularScopeCode,
        role: AnnularBoundaryRole,
        kind: u8,
    },
}
/// Ordered by token; `dictionary.meaning(token)` is the only reverse lookup.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularNameDictionary {
    rows: std::collections::BTreeMap<u64, AnnularNameMeaning>,
}
impl AnnularNameDictionary {
    /// Record one issued token; a different meaning for an issued token is refused.
    pub(super) fn issue<'a>(
        &mut self,
        token: AuthoredPrismId,
        meaning: AnnularNameMeaning,
    ) -> NameCheck<'a, AuthoredPrismId> {
        match self.rows.insert(token.0, meaning) {
            Some(previous) if previous != meaning => Err(AnnularNameCause::Collision {
                token: token.0,
                previous,
                actual: meaning,
            }),
            _ => Ok(token),
        }
    }
    /// Reverse one internal token; `dictionary.meaning(token)` is `None` when not issued.
    pub(in crate::sketch_region_prism) fn meaning(
        &self,
        token: AuthoredPrismId,
    ) -> Option<AnnularNameMeaning> {
        self.rows.get(&token.0).copied()
    }
    /// Number of distinct issued tokens; `dictionary.len()`.
    pub(in crate::sketch_region_prism) fn len(&self) -> usize {
        self.rows.len()
    }
    /// Whether no token was issued; `dictionary.is_empty()`.
    pub(in crate::sketch_region_prism) fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}
