// SPDX-License-Identifier: GPL-2.0-only
//! Copy and intermediate-scope bindings: members by scope, authored boundary role and kind; uses by
//! kind and vertex direction. Source roles come from authored inputs and candidate wires only.
use super::*;
use crate::sketch_region::{
    PartitionObservation, PartitionOccurrenceIdentity, RegionCopyContextObservation,
    RegionCopyContexts, RegionObservedContext,
};
use crate::sketch_region_prism::frozen::annular_history::{
    AnnularHistoryBindings, AnnularHistoryScope, AnnularScopeBindings,
};
use crate::sketch_region_prism::frozen::complete_history::*;
use crate::sketch_region_prism::frozen::lineage::PrismCopyBindings;
use crate::sketch_region_prism::{PrismBodyContext, contexts::body_contexts};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::Orientation;

/// Issue selected-copy and intermediate-scope bindings; `history(issuer, region, receipt)`.
pub(super) fn history<'a>(
    issuer: &mut AnnularNameIssuer,
    region: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
) -> NameCheck<'a, (PrismCopyBindings, AnnularHistoryBindings)> {
    let roles = SourceRoles::new(region)?;
    let copied = copy_bindings(issuer, receipt, &roles)?;
    let historical = scopes(issuer, region, &roles)?;
    Ok((copied, historical))
}
/// Semantic source of each physical definition, from authored inputs and candidate wires.
struct SourceRoles {
    rows: Vec<(PartitionOccurrenceIdentity, Option<RegionSourceId>)>,
}
impl SourceRoles {
    fn new<'a>(region: &TwoLoopRegionRequest<'_, '_>) -> NameCheck<'a, Self> {
        let mut result = Self { rows: Vec::new() };
        for input in &partition(region)?.input_contexts {
            let copied = copied_rows(&input.observation)?;
            for row in copied.source.iter().chain(&copied.copied) {
                result.add(&row.occurrence, Some(input.source));
            }
        }
        let copy = candidate_observation(region)?;
        for link in &copy.links {
            let source = result.find(&link.original).flatten();
            result.add(&link.copied, source);
        }
        for candidate in &region.identities.candidates {
            result.add(&candidate.actual, None);
            for wire in &candidate.wires {
                result.add(&wire.actual, Some(wire.source));
            }
        }
        Ok(result)
    }
    fn add(&mut self, actual: &PartitionOccurrenceIdentity, source: Option<RegionSourceId>) {
        if self.find(actual).is_none() {
            self.rows.push((actual.clone(), source));
        }
    }
    fn find(&self, actual: &PartitionOccurrenceIdentity) -> Option<Option<RegionSourceId>> {
        let same = |known: &PartitionOccurrenceIdentity| {
            crate::sketch_region_prism::frozen::same_occurrence_definition(known, actual)
        };
        self.rows
            .iter()
            .find(|(known, _)| same(known))
            .map(|(_, source)| *source)
    }
    /// Region-level kinds have no boundary source; every other kind must have one.
    fn source<'a>(
        &self,
        actual: &PartitionOccurrenceIdentity,
        kind: ShapeEnum,
    ) -> NameCheck<'a, Option<RegionSourceId>> {
        match (kind, self.find(actual)) {
            (ShapeEnum::Compound | ShapeEnum::Face, _) => Ok(None),
            (_, Some(Some(source))) => Ok(Some(source)),
            (kind, _) => Err(AnnularNameCause::MissingSourceRole {
                kind: kind_code(kind).unwrap_or(0),
            }),
        }
    }
}
fn copy_bindings<'a>(
    issuer: &mut AnnularNameIssuer,
    receipt: &PrismReceipt,
    roles: &SourceRoles,
) -> NameCheck<'a, PrismCopyBindings> {
    let selected = receipt
        .input
        .selected
        .as_ref()
        .ok_or(AnnularNameCause::Missing {
            required: "selected candidate relations",
        })?;
    let mut members: Vec<PrismSourceBinding> = Vec::new();
    for row in &selected.relations {
        let actual = PartitionOccurrenceIdentity::capture(&row.output);
        let kind = row.output.shape_type().map_err(AnnularNameCause::Native)?;
        let known = |m: &PrismSourceBinding| {
            crate::sketch_region_prism::frozen::same_occurrence_definition(&m.actual, &actual)
        };
        if !members.iter().any(known) {
            let scope = AnnularScopeCode::None;
            let source = roles.source(&actual, kind)?;
            let member = issuer.member(AnnularNameCategory::SelectedMember, scope, source, kind)?;
            members.push(PrismSourceBinding { actual, member });
        }
    }
    let copied = receipt.copied.as_ref().ok_or(AnnularNameCause::Missing {
        required: "copied sweep base",
    })?;
    Ok(PrismCopyBindings {
        selected_members: members,
        selected_uses: directed_uses(issuer, &selected.face)?,
        copied_uses: directed_uses(issuer, &copied.base)?,
    })
}
fn directed_uses<'a>(
    issuer: &mut AnnularNameIssuer,
    root: &Shape,
) -> NameCheck<'a, Vec<PrismDirectedBinding>> {
    let contexts = body_contexts(root).map_err(AnnularNameCause::Body)?;
    let mut result = Vec::new();
    for actual in contexts {
        let authored_use = context_use(issuer, &actual)?;
        result.push(PrismDirectedBinding {
            actual,
            authored_use,
        });
    }
    Ok(result)
}
fn context_use<'a>(
    issuer: &mut AnnularNameIssuer,
    actual: &PrismBodyContext,
) -> NameCheck<'a, AuthoredPrismId> {
    let kind = actual
        .occurrence
        .shape_type()
        .map_err(AnnularNameCause::Native)?;
    history_use(issuer, kind, actual.occurrence.orientation())
}
/// Vertex uses are distinguished by direction; every other use by kind only.
fn history_use<'a>(
    issuer: &mut AnnularNameIssuer,
    kind: ShapeEnum,
    orientation: Orientation,
) -> NameCheck<'a, AuthoredPrismId> {
    let end = kind == ShapeEnum::Vertex && orientation == Orientation::Reversed;
    issuer.use_role(AnnularNameCategory::HistoryUse, kind, u8::from(end))
}
fn scopes<'a>(
    issuer: &mut AnnularNameIssuer,
    region: &TwoLoopRegionRequest<'_, '_>,
    roles: &SourceRoles,
) -> NameCheck<'a, AnnularHistoryBindings> {
    let mut scopes = Vec::new();
    for input in &partition(region)?.input_contexts {
        let rows = &copied_rows(&input.observation)?.copied;
        let scope = AnnularHistoryScope::InputCopy(input.source);
        let code = AnnularScopeCode::InputCopy;
        scopes.push(scope_names(issuer, scope, code, roles, rows)?);
    }
    let candidate = copied_rows(&candidate_observation(region)?.contexts)?;
    let partition_code = AnnularScopeCode::Partition;
    let scope = AnnularHistoryScope::Partition;
    scopes.push(scope_names(
        issuer,
        scope,
        partition_code,
        roles,
        &candidate.source,
    )?);
    let scope = AnnularHistoryScope::CandidateCopy;
    let code = AnnularScopeCode::CandidateCopy;
    scopes.push(scope_names(issuer, scope, code, roles, &candidate.copied)?);
    rejected(issuer, region, roles, &mut scopes)?;
    Ok(AnnularHistoryBindings { scopes })
}
fn rejected<'a>(
    issuer: &mut AnnularNameIssuer,
    region: &TwoLoopRegionRequest<'_, '_>,
    roles: &SourceRoles,
    scopes: &mut Vec<AnnularScopeBindings>,
) -> NameCheck<'a, ()> {
    for candidate in &region.identities.candidates {
        if candidate.name == region.identities.selected {
            continue;
        }
        let code = issuer.rejected_code(candidate.name)?;
        let rows = candidate_rows(region, &candidate.actual)?;
        let scope = AnnularHistoryScope::Rejected(candidate.name);
        scopes.push(scope_names(issuer, scope, code, roles, &rows)?);
    }
    Ok(())
}
fn scope_names<'a>(
    issuer: &mut AnnularNameIssuer,
    scope: AnnularHistoryScope,
    code: AnnularScopeCode,
    roles: &SourceRoles,
    rows: &[RegionObservedContext],
) -> NameCheck<'a, AnnularScopeBindings> {
    let mut names = PrismHistoricalStageBindings {
        stage: scope.stage(),
        members: Vec::new(),
        uses: Vec::new(),
    };
    for row in rows {
        let source = roles.source(&row.occurrence, row.kind)?;
        let category = AnnularNameCategory::HistoryMember;
        let member = issuer.member(category, code, source, row.kind)?;
        append_historical_member(&mut names, &row.occurrence, member)
            .map_err(AnnularNameCause::History)?;
        names.uses.push(PrismHistoricalUseBinding {
            actual: row.clone(),
            authored_use: history_use(issuer, row.kind, row.occurrence.orientation)?,
        });
    }
    Ok(AnnularScopeBindings { scope, names })
}
fn partition<'a, 'r>(
    region: &TwoLoopRegionRequest<'r, '_>,
) -> NameCheck<'a, &'r crate::sketch_region::OwnedPartitionSnapshot> {
    let PartitionObservation::Captured(partition) = &region.bridge.mapped.arrangement.partition
    else {
        return Err(AnnularNameCause::Missing {
            required: "complete actual partition observation",
        });
    };
    Ok(partition)
}
fn copied_rows<'a>(
    observed: &RegionCopyContextObservation,
) -> NameCheck<'a, &RegionCopyContexts> {
    retained_copy_contexts(observed, PrismHistoryStage::InputCopy)
        .map_err(AnnularNameCause::History)
}
fn candidate_observation<'a, 'r>(
    region: &TwoLoopRegionRequest<'r, '_>,
) -> NameCheck<'a, &'r crate::sketch_region_candidates::CandidateCopyInspection> {
    region
        .bridge
        .mapped
        .copied
        .as_ref()
        .ok_or(AnnularNameCause::Missing {
            required: "actual candidate copy observation",
        })
}
fn candidate_rows<'a>(
    region: &TwoLoopRegionRequest<'_, '_>,
    actual: &PartitionOccurrenceIdentity,
) -> NameCheck<'a, Vec<RegionObservedContext>> {
    let copied = candidate_observation(region)?;
    let batch = copied
        .completion
        .as_ref()
        .map_err(|_| AnnularNameCause::Missing {
            required: "completed candidate batch",
        })?;
    let candidate =
        resolve_certified_candidate(batch, actual).map_err(AnnularNameCause::History)?;
    let limits = PrismHistoryLimits::bounded_v1();
    capture_history_contexts(&candidate.face, PrismHistoryStage::Complement, limits)
        .map_err(AnnularNameCause::History)
}
