// SPDX-License-Identifier: GPL-2.0-only
//! Authored region member roles are bound through actual copy ancestry, never output order.
use super::super::super::frozen::{
    complete_history::snapshot_directed_history_context, lineage::matches_selected_copy_context,
};
use super::*;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{edge, vertex},
};

pub(super) fn pair_members(
    names: &TwoLoopCandidateBinding,
    selected: &[PrismBodyContext],
    copied: &[PrismBodyContext],
    observations: &[PrismCopyObservation],
) -> AnnularCheck<'static, Vec<AnnularCopiedMember>> {
    require(
        "complete copied directed census",
        copied.len(),
        selected.len(),
    )?;
    let mut members = Vec::new();
    for actual in selected {
        let matches: Vec<_> = copied
            .iter()
            .filter(|c| matches_selected_copy_context(observations, actual, c))
            .collect();
        require(
            "unique copied use from all mapped ancestors",
            matches.len(),
            1,
        )?;
        members.push(AnnularCopiedMember {
            key: member_key(names, actual)?,
            selected: snapshot(actual)?,
            copied: snapshot(matches[0])?,
        });
    }
    complete_pairs(&members)?;
    members.sort_by_key(|m| m.key);
    Ok(members)
}
fn complete_pairs(members: &[AnnularCopiedMember]) -> AnnularCheck<'static> {
    for member in members {
        require(
            "one copied full context",
            members.iter().filter(|m| m.copied == member.copied).count(),
            1,
        )?;
        require(
            "one authored selected member",
            members.iter().filter(|m| m.key == member.key).count(),
            1,
        )?;
    }
    Ok(())
}
pub(in crate::sketch_region_prism) fn snapshot(context: &PrismBodyContext) -> AnnularCheck<'static, RegionObservedContext> {
    snapshot_directed_history_context(context)
        .map_err(|cause| Box::new(AnnularBindingCause::History(Box::new(cause))))
}
fn member_key(
    names: &TwoLoopCandidateBinding,
    actual: &PrismBodyContext,
) -> AnnularCheck<'static, TwoLoopMemberKey> {
    let kind = actual.occurrence.shape_type()?;
    if kind == ShapeEnum::Face {
        require("selected face at context root", actual.ancestors.len(), 0)?;
        return Ok(TwoLoopMemberKey {
            candidate: names.name,
            wire: None,
            role: TwoLoopMemberRole::Face,
        });
    }
    let wire = wire_name(names, actual)?;
    let role = match kind {
        ShapeEnum::Wire => TwoLoopMemberRole::Wire,
        ShapeEnum::Edge => TwoLoopMemberRole::Boundary,
        ShapeEnum::Vertex => endpoint_role(actual)?,
        _ => {
            return Err(Box::new(AnnularBindingCause::Count {
                required: "face/wire/edge/vertex region member",
                actual: kind as usize,
                expected: ShapeEnum::Vertex as usize,
            }));
        }
    };
    Ok(TwoLoopMemberKey {
        candidate: names.name,
        wire: Some(wire),
        role,
    })
}
fn wire_name(
    names: &TwoLoopCandidateBinding,
    actual: &PrismBodyContext,
) -> AnnularCheck<'static, u64> {
    let occurrences = actual.ancestors.iter().chain([&actual.occurrence]);
    let mut wire_names = Vec::new();
    for occurrence in occurrences {
        let identity = PartitionOccurrenceIdentity::capture(occurrence);
        wire_names.extend(
            names
                .wires
                .iter()
                .filter(|w| w.actual == identity)
                .map(|w| w.name),
        );
    }
    require(
        "one authored wire in complete selected ancestry",
        wire_names.len(),
        1,
    )?;
    Ok(wire_names[0])
}
fn endpoint_role(actual: &PrismBodyContext) -> AnnularCheck<'static, TwoLoopMemberRole> {
    let parent = actual.ancestors.last().ok_or_else(|| {
        Box::new(AnnularBindingCause::Count {
            required: "native endpoint edge ancestor",
            actual: 0,
            expected: 1,
        })
    })?;
    let parameter = BRepTool::parameter(&vertex(&actual.occurrence)?, &edge(parent)?)?;
    if parameter.to_bits() == 0_f64.to_bits() {
        return Ok(TwoLoopMemberRole::NativeStart);
    }
    if parameter.to_bits() == 1_f64.to_bits() {
        return Ok(TwoLoopMemberRole::NativeEnd);
    }
    Err(Box::new(AnnularBindingCause::Capture(
        super::super::super::surface_charts::PrismSurfaceCause::Range {
            actual: [parameter, parameter],
            expected: [0., 1.],
        },
    )))
}
