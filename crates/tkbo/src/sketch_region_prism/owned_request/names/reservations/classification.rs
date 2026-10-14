// SPDX-License-Identifier: GPL-2.0-only
use super::*;

pub(super) fn classify(
    plan: &SingleBoundaryNamePlan,
    applications: &[SemanticNameSlot],
) -> ReservationCheck<Vec<PrismReservationRow>> {
    if plan.version() != 2 {
        return Err(PrismReservationCause::UnsupportedVersion {
            actual: plan.version(),
            expected: 2,
        });
    }
    known_applications(plan, applications)?;
    plan.entries()
        .iter()
        .map(|entry| classify_slot(entry.slot, applications))
        .collect()
}
fn classify_slot(
    slot: SemanticNameSlot,
    applications: &[SemanticNameSlot],
) -> ReservationCheck<PrismReservationRow> {
    let actual = applications.iter().filter(|&&value| value == slot).count();
    let status = match (inapplicable(slot), actual) {
        (Some(reason), 0) => PrismReservationStatus::Inapplicable(reason),
        (Some(_), _) => return Err(PrismReservationCause::InapplicableApplied { slot, actual }),
        (None, 0) => return Err(PrismReservationCause::MissingApplication { slot }),
        (None, named_fields) => PrismReservationStatus::Applied { named_fields },
    };
    Ok(PrismReservationRow { slot, status })
}
fn inapplicable(slot: SemanticNameSlot) -> Option<PrismReservationInapplicable> {
    use PrismReservationInapplicable::*;
    use SemanticNameStage::*;
    match slot.stage {
        Partition | CandidateCopy if super::super::roles::REGION_ROLES.contains(&slot.role) => {
            Some(PartitionHasNoGeneratedRegions)
        }
        Selected | SweepBase if complementary_role(slot.role) => {
            Some(BoundedSelectionExcludesComplementRoles)
        }
        Complement if bounded_role(slot.role) => Some(ComplementExcludesBoundedRoles),
        _ => None,
    }
}
fn complementary_role(role: SemanticNameRole) -> bool {
    use SemanticNameRole::*;
    matches!(
        role,
        ComplementRegion | ComplementLoop | ComplementRegionUse | ComplementLoopUse
    )
}
fn bounded_role(role: SemanticNameRole) -> bool {
    use SemanticNameRole::*;
    matches!(
        role,
        BoundedRegion | BoundedLoop | BoundedRegionUse | BoundedLoopUse
    )
}

fn known_applications(
    plan: &SingleBoundaryNamePlan,
    applications: &[SemanticNameSlot],
) -> ReservationCheck<()> {
    for slot in applications {
        plan.reverse(slot.token().value())
            .map_err(|_| PrismReservationCause::UnknownToken {
                actual: slot.token().value(),
            })?;
    }
    Ok(())
}
