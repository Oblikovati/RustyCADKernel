// SPDX-License-Identifier: GPL-2.0-only
//! Deterministic token expansion checks table meaning, never actual geometry correspondence.
use super::{
    super::{body_keys::*, *},
    types::*,
};
use crate::sketch_region_prism::frozen::{FrozenPrismKey, FrozenSweepRole, FrozenUseKey};

pub(super) fn slot(plan: &SingleBoundaryNamePlan, token: u64) -> MapCheck<SemanticNameSlot> {
    plan.reverse(token)
        .map(|entry| entry.slot)
        .map_err(|_| failure(plan, PrismSemanticMapCause::UnknownToken { actual: token }))
}
pub(super) fn member(
    plan: &SingleBoundaryNamePlan,
    key: FrozenPrismKey,
) -> MapCheck<SemanticMemberKey> {
    check_operation(plan, key.operation.0)?;
    let source_member = slot(plan, key.source.0)?;
    output(plan, key, source_member)?;
    let ids = plan.identities();
    Ok(SemanticMemberKey {
        operation: ids.operation,
        source: ids.boundary,
        source_member,
        sweep_role: key.role,
        kind: key.kind,
    })
}
pub(super) fn output(
    plan: &SingleBoundaryNamePlan,
    key: FrozenPrismKey,
    source: SemanticNameSlot,
) -> MapCheck<Option<SemanticNameSlot>> {
    use SemanticNameStage::{Selected, SweepBase};
    if !matches!(source.stage, Selected | SweepBase) {
        return Err(failure(
            plan,
            PrismSemanticMapCause::UnsupportedSourceSlot { slot: source },
        ));
    }
    let role = output_role(key, source)
        .or_else(|| copied_base_root_role(plan.version(), key, source))
        .ok_or_else(|| failure(plan, PrismSemanticMapCause::UnsupportedMember { key }))?;
    Ok(role.map(|role| SemanticNameSlot {
        stage: SemanticNameStage::Output,
        role,
    }))
}
fn output_role(key: FrozenPrismKey, source: SemanticNameSlot) -> Option<Option<SemanticNameRole>> {
    use FrozenSweepRole::{First, Generated, Last};
    use SemanticNameRole::*;
    match (source.stage, source.role, key.kind, key.role) {
        (SemanticNameStage::Selected, BoundedRegion, 2, Generated) => Some(Some(Solid)),
        (SemanticNameStage::Selected, BoundedRegion, 3, Generated) => Some(Some(Shell)),
        (SemanticNameStage::SweepBase, BoundedRegion, 4, First) => Some(Some(FirstCap)),
        (SemanticNameStage::SweepBase, BoundedRegion, 4, Last) => Some(Some(LastCap)),
        (SemanticNameStage::SweepBase, Boundary, 4, Generated) => Some(Some(Side)),
        (SemanticNameStage::SweepBase, BoundedLoop, 5, First | Last)
        | (SemanticNameStage::SweepBase, Boundary, 5, Generated) => Some(None),
        (SemanticNameStage::SweepBase, Boundary, 6, First) => Some(Some(FirstRim)),
        (SemanticNameStage::SweepBase, Boundary, 6, Last) => Some(Some(LastRim)),
        (SemanticNameStage::SweepBase, ClosureVertex, 6, Generated) => Some(Some(Seam)),
        (SemanticNameStage::SweepBase, ClosureVertex, 7, First) => Some(Some(FirstVertex)),
        (SemanticNameStage::SweepBase, ClosureVertex, 7, Last) => Some(Some(LastVertex)),
        _ => None,
    }
}
pub(super) fn occurrence(
    plan: &SingleBoundaryNamePlan,
    key: &FrozenUseKey,
) -> MapCheck<SemanticUseKey> {
    if key.0.is_empty() {
        return Err(failure(plan, PrismSemanticMapCause::EmptyUse));
    }
    let steps = key
        .0
        .iter()
        .map(|(definition, token)| step(plan, *definition, token.0))
        .collect::<MapCheck<Vec<_>>>()?;
    Ok(SemanticUseKey(steps))
}
fn check_use(
    plan: &SingleBoundaryNamePlan,
    slot: SemanticNameSlot,
    key: FrozenPrismKey,
) -> MapCheck<()> {
    if slot.stage != SemanticNameStage::Output || !use_kind(slot.role, key) {
        return Err(failure(
            plan,
            PrismSemanticMapCause::UnsupportedUseSlot {
                slot,
                kind: key.kind,
            },
        ));
    }
    Ok(())
}
fn use_kind(role: SemanticNameRole, key: FrozenPrismKey) -> bool {
    use FrozenSweepRole::{First, Generated, Last};
    use SemanticNameRole::*;
    matches!(
        (role, key.kind, key.role),
        (SolidUse, 2, Generated)
            | (ShellUse, 3, Generated)
            | (FaceUse, 4, _)
            | (LoopUse, 5, _)
            | (FirstRimUse, 6, First)
            | (LastRimUse, 6, Last)
            | (SeamStartUse | SeamEndUse, 6, Generated)
            | (StartUse | EndUse, 7, _)
    )
}

fn check_operation(plan: &SingleBoundaryNamePlan, actual: u64) -> MapCheck<()> {
    let expected = SemanticNameSlot {
        stage: SemanticNameStage::Request,
        role: SemanticNameRole::Operation,
    }
    .token()
    .value();
    if actual != expected {
        return Err(failure(
            plan,
            PrismSemanticMapCause::Operation { actual, expected },
        ));
    }
    Ok(())
}

fn step(
    plan: &SingleBoundaryNamePlan,
    definition: FrozenPrismKey,
    token: u64,
) -> MapCheck<SemanticUseStep> {
    let directed_use = slot(plan, token)?;
    check_use(plan, directed_use, definition)?;
    Ok(SemanticUseStep {
        member: member(plan, definition)?,
        directed_use,
    })
}

fn copied_base_root_role(
    version: u32,
    key: FrozenPrismKey,
    source: SemanticNameSlot,
) -> Option<Option<SemanticNameRole>> {
    use SemanticNameRole::{BoundedRegion, Shell, Solid};
    use SemanticNameStage::SweepBase;
    match (version, source.stage, source.role, key.role, key.kind) {
        (2, SweepBase, BoundedRegion, FrozenSweepRole::Generated, 2) => Some(Some(Solid)),
        (2, SweepBase, BoundedRegion, FrozenSweepRole::Generated, 3) => Some(Some(Shell)),
        _ => None,
    }
}
