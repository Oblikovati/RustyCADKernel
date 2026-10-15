// SPDX-License-Identifier: GPL-2.0-only
//! ADR0064: production semantic names for the annular prism. Internal tokens come from lexical
//! roles (category, scope, authored boundary role, kind, variant) and an issued dictionary maps
//! each back to the caller's own operation, region, wire, boundary and vertex identities.
mod dictionary;
mod history;
mod publication;
mod tokens;
pub(in crate::sketch_region_prism) use dictionary::*;
pub(in crate::sketch_region_prism) use tokens::*;

use super::super::names::{CallerBoundaryId, CallerOperationId};
use super::*;
use crate::sketch_region_prism::frozen::annular_history::AnnularHistoryBindings;
use crate::sketch_region_prism::frozen::complete_history::PrismHistoryCause;
use crate::sketch_region_prism::frozen::lineage::PrismCopyBindings;
use crate::sketch_region_prism::frozen::{
    AuthoredPrismId, PrismDirectedBinding, PrismOriginalBinding, PrismPublicationBindings,
    PrismSourceBinding,
};
use crate::sketch_region_prism::region_certificate::annular_binding::{
    AnnularBindingCause, AnnularPrismBinding,
};
use crate::sketch_region_prism::region_certificate::two_loop::{
    TwoLoopMemberRole, TwoLoopRegionFailure,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

#[cfg(test)]
mod tests;

/// Every binding input of one annular publication plus the dictionary that qualifies it.
pub(in crate::sketch_region_prism) struct IssuedAnnularNames {
    pub publication: PrismPublicationBindings,
    pub copied: PrismCopyBindings,
    pub historical: AnnularHistoryBindings,
    pub dictionary: AnnularNameDictionary,
}
/// Typed issuance refusal; borrowed region failures keep their original carrier.
pub(in crate::sketch_region_prism) enum AnnularNameCause<'a> {
    Region(Box<TwoLoopRegionFailure<'a>>),
    Binding(Box<AnnularBindingCause<'static>>),
    Body(crate::sketch_region_prism::PrismCause),
    History(PrismHistoryCause),
    Native(Failure),
    Missing {
        required: &'static str,
    },
    Count {
        required: &'static str,
        actual: usize,
        expected: usize,
    },
    OriginalRole {
        role: TwoLoopMemberRole,
    },
    UnknownSource {
        source: RegionSourceId,
    },
    MissingSourceRole {
        kind: u8,
    },
    UnsupportedKind {
        kind: ShapeEnum,
    },
    UnknownCandidate {
        name: u64,
    },
    EndpointRole {
        parameter: f64,
        range: [f64; 2],
    },
    Collision {
        token: u64,
        previous: AnnularNameMeaning,
        actual: AnnularNameMeaning,
    },
}
pub(super) type NameCheck<'a, T> = Result<T, AnnularNameCause<'a>>;
fn count<'a>(required: &'static str, actual: usize, expected: usize) -> NameCheck<'a, ()> {
    if actual == expected {
        return Ok(());
    }
    Err(AnnularNameCause::Count {
        required,
        actual,
        expected,
    })
}
/// Issue all names for one current region, receipt and binding of this owned request.
/// Example: `issue_annular_names(&request, region, receipt, &binding)?`.
pub(in crate::sketch_region_prism) fn issue_annular_names<'a>(
    request: &OwnedAnnularPrismRequest,
    region: &TwoLoopRegionRequest<'a, '_>,
    receipt: &PrismReceipt,
    binding: &AnnularPrismBinding,
) -> NameCheck<'a, IssuedAnnularNames> {
    let mut issuer = AnnularNameIssuer {
        request,
        dictionary: AnnularNameDictionary::default(),
    };
    let publication = publication::publication(&mut issuer, region, receipt, binding)?;
    let (copied, historical) = history::history(&mut issuer, region, receipt)?;
    Ok(IssuedAnnularNames {
        publication,
        copied,
        historical,
        dictionary: issuer.dictionary,
    })
}
pub(super) struct AnnularNameIssuer<'r> {
    request: &'r OwnedAnnularPrismRequest,
    dictionary: AnnularNameDictionary,
}
impl AnnularNameIssuer<'_> {
    /// The two-loop proof already binds the caller operation and region names as authored
    /// identities, so these two tokens are the caller values themselves; any collision with an
    /// issued role token is refused by the dictionary.
    fn operation<'a>(&mut self) -> NameCheck<'a, AuthoredPrismId> {
        let meaning = AnnularNameMeaning::Operation(self.request.operation);
        let token = AuthoredPrismId(self.request.operation.0);
        self.dictionary.issue(token, meaning)
    }
    fn annulus<'a>(&mut self) -> NameCheck<'a, AuthoredPrismId> {
        let region = self.request.regions.annulus;
        let meaning = AnnularNameMeaning::Annulus { region };
        self.dictionary.issue(AuthoredPrismId(region), meaning)
    }
    /// Outer or inner by the request's explicit roles, never by request or batch order.
    fn role<'a>(
        &self,
        source: RegionSourceId,
    ) -> NameCheck<'a, (AnnularBoundaryRole, &OwnedPolynomialBoundary)> {
        let request = self.request;
        if source.0 == request.outer.source.0 {
            return Ok((AnnularBoundaryRole::Outer, &request.outer));
        }
        if source.0 == request.inner.source.0 {
            return Ok((AnnularBoundaryRole::Inner, &request.inner));
        }
        Err(AnnularNameCause::UnknownSource { source })
    }
    fn boundary_member<'a>(
        &mut self,
        source: RegionSourceId,
        kind: ShapeEnum,
    ) -> NameCheck<'a, AuthoredPrismId> {
        let (role, boundary) = self.role(source)?;
        let regions = &self.request.regions;
        let caller = match (kind, role) {
            (ShapeEnum::Wire, AnnularBoundaryRole::Outer) => regions.annulus_outer_wire,
            (ShapeEnum::Wire, _) => regions.annulus_inner_wire,
            (ShapeEnum::Edge, _) => boundary.source.0,
            (ShapeEnum::Vertex, _) => boundary.closure.0,
            (kind, _) => return Err(AnnularNameCause::UnsupportedKind { kind }),
        };
        let code = kind_code(kind).ok_or(AnnularNameCause::UnsupportedKind { kind })?;
        let meaning = AnnularNameMeaning::Boundary {
            role,
            boundary: boundary.source,
            kind: code,
            caller,
        };
        let slot = AnnularNameSlot {
            boundary: role,
            kind: code,
            ..AnnularNameSlot::of(AnnularNameCategory::BoundaryMember)
        };
        self.dictionary.issue(slot.token(), meaning)
    }
    fn use_role<'a>(
        &mut self,
        category: AnnularNameCategory,
        kind: ShapeEnum,
        variant: u8,
    ) -> NameCheck<'a, AuthoredPrismId> {
        let code = kind_code(kind).ok_or(AnnularNameCause::UnsupportedKind { kind })?;
        let meaning = AnnularNameMeaning::UseRole {
            category,
            kind: code,
            variant,
        };
        let slot = AnnularNameSlot {
            kind: code,
            variant,
            ..AnnularNameSlot::of(category)
        };
        self.dictionary.issue(slot.token(), meaning)
    }
    fn member<'a>(
        &mut self,
        category: AnnularNameCategory,
        scope: AnnularScopeCode,
        source: Option<RegionSourceId>,
        kind: ShapeEnum,
    ) -> NameCheck<'a, AuthoredPrismId> {
        let code = kind_code(kind).ok_or(AnnularNameCause::UnsupportedKind { kind })?;
        let role = match source {
            Some(source) => self.role(source)?.0,
            None => AnnularBoundaryRole::None,
        };
        let meaning = AnnularNameMeaning::Member {
            category,
            scope,
            role,
            kind: code,
        };
        let slot = AnnularNameSlot {
            category,
            scope,
            boundary: role,
            kind: code,
            variant: 0,
        };
        self.dictionary.issue(slot.token(), meaning)
    }
    /// A rejected candidate is coded by its authored region name, never by its batch position.
    fn rejected_code<'a>(&self, name: u64) -> NameCheck<'a, AnnularScopeCode> {
        let regions = &self.request.regions;
        match name {
            n if n == regions.inner_domain => Ok(AnnularScopeCode::RejectedInnerDomain),
            n if n == regions.outer_complement => Ok(AnnularScopeCode::RejectedOuterComplement),
            name => Err(AnnularNameCause::UnknownCandidate { name }),
        }
    }
}
impl AnnularNameMeaning {
    /// The caller boundary this meaning belongs to, if any; `meaning.boundary()`.
    pub(in crate::sketch_region_prism) fn boundary(&self) -> Option<CallerBoundaryId> {
        match self {
            Self::Boundary { boundary, .. } => Some(*boundary),
            _ => None,
        }
    }
}
