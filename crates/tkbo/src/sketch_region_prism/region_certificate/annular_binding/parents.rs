// SPDX-License-Identifier: GPL-2.0-only
//! One original-native-role mapping batch preserves all copy/sweep relation classes and multiplicity.
use super::*;
use crate::sketch_region::{ArrangementRelation, OriginalMemberIdentity};
struct RawCopy<'a> {
    key: TwoLoopMemberKey,
    selected: &'a ArrangementRelation,
    copied: &'a ArrangementRelation,
}
struct RawImage<'a> {
    key: TwoLoopMemberKey,
    identity: AnnularChartIdentity,
    returned: &'a ArrangementRelation,
}

struct RawCaptured<'a> {
    identity: AnnularChartIdentity,
    parents: &'a [OriginalMemberIdentity],
}

pub(super) fn capture<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    receipt: &PrismReceipt,
    copy: &AnnularCopyCapture,
    reserved_units: usize,
    charts: &[AnnularBoundChart],
) -> AnnularCheck<'a, AnnularParentCapture> {
    let diagnostic = receipt
        .diagnostic
        .as_ref()
        .expect("checked actual chart owner");
    preflight(
        request,
        copy.members.len(),
        diagnostic.bindings.len(),
        reserved_units,
    )?;
    let copies = raw_copies(receipt, copy)?;
    let images = raw_images(diagnostic, copy)?;
    let captured = chart_parents(charts, &images)?;
    let slices = mapping_slices(request, &copies, &images, &captured, reserved_units)?;
    let mapped = map_two_loop_original_parents_after(request, &slices, reserved_units)
        .map_err(|cause| Box::new(AnnularBindingCause::Region(cause)))?;
    finish(copies, images, captured, mapped)
}
fn preflight(
    request: &TwoLoopRegionRequest<'_, '_>,
    copies: usize,
    images: usize,
    reserved: usize,
) -> AnnularCheck<'static> {
    let rows = copies
        .checked_mul(2)
        .and_then(|n| n.checked_add(images))
        .and_then(|n| n.checked_mul(3))
        .and_then(|n| n.checked_add(reserved));
    let actual = rows.unwrap_or(usize::MAX);
    if rows.is_none() || actual > request.requested.maximum_retained_units {
        return Err(Box::new(AnnularBindingCause::Count {
            required: "bounded original parent mapping rows before allocation",
            actual,
            expected: request.requested.maximum_retained_units,
        }));
    }
    Ok(())
}
fn dispositions(row: &ArrangementRelation) -> [&[OriginalMemberIdentity]; 3] {
    [&row.preserved, &row.modified, &row.generated]
}
fn raw_copies<'a>(
    receipt: &'a PrismReceipt,
    copy: &AnnularCopyCapture,
) -> AnnularCheck<'static, Vec<RawCopy<'a>>> {
    let selected = receipt
        .input
        .selected
        .as_ref()
        .expect("checked selected owner");
    let copied = receipt.copied.as_ref().expect("checked copied owner");
    let mut result = Vec::new();
    for member in &copy.members {
        result.push(RawCopy {
            key: member.key,
            selected: relation(&selected.relations, &member.selected.occurrence)?,
            copied: relation(&copied.relations, &member.copied.occurrence)?,
        });
    }
    Ok(result)
}
fn relation<'a>(
    rows: &'a [ArrangementRelation],
    identity: &PartitionOccurrenceIdentity,
) -> AnnularCheck<'static, &'a ArrangementRelation> {
    let matches: Vec<_> = rows
        .iter()
        .filter(|r| PartitionOccurrenceIdentity::capture(&r.output) == *identity)
        .collect();
    require("one actual original-parent relation", matches.len(), 1)?;
    Ok(matches[0])
}
fn raw_images<'a>(
    diagnostic: &'a PrismDiagnostic,
    copy: &AnnularCopyCapture,
) -> AnnularCheck<'static, Vec<RawImage<'a>>> {
    let mut result = Vec::new();
    for row in &diagnostic.bindings {
        result.push(raw_image(row, copy)?);
    }
    result.sort_by(|a, b| {
        (a.key, a.identity.role as u8, &a.identity.output.child_slots).cmp(&(
            b.key,
            b.identity.role as u8,
            &b.identity.output.child_slots,
        ))
    });
    Ok(result)
}
fn finish(
    copies: Vec<RawCopy<'_>>,
    images: Vec<RawImage<'_>>,
    captured: Vec<RawCaptured<'_>>,
    mapped: TwoLoopMappedParents,
) -> AnnularCheck<'static, AnnularParentCapture> {
    let mut rows = mapped.rows.into_iter();
    let owned_copies = finish_copies(copies, &mut rows)?;
    let owned_images = finish_images(images, &mut rows)?;
    let owned_captured = finish_captured(captured, &mut rows)?;
    require("all original parent mapping rows consumed", rows.count(), 0)?;
    Ok(AnnularParentCapture {
        copies: owned_copies,
        images: owned_images,
        captured: owned_captured,
        retained_units: mapped.retained_units,
    })
}
fn take_parents(
    rows: &mut impl Iterator<Item = Vec<TwoLoopOriginalUse>>,
) -> AnnularCheck<'static, [Vec<TwoLoopOriginalUse>; 3]> {
    let mut result = [Vec::new(), Vec::new(), Vec::new()];
    for row in &mut result {
        *row = rows.next().ok_or_else(|| {
            Box::new(AnnularBindingCause::Count {
                required: "original mapped parent disposition",
                actual: 0,
                expected: 1,
            })
        })?;
    }
    Ok(result)
}

fn chart_parents<'a>(
    charts: &'a [AnnularBoundChart],
    images: &[RawImage<'_>],
) -> AnnularCheck<'static, Vec<RawCaptured<'a>>> {
    let mut rows = Vec::new();
    for chart in charts {
        parent_union(&chart.identity, &chart.chart.parents, images)?;
        rows.push(RawCaptured {
            identity: chart.identity.clone(),
            parents: &chart.chart.parents,
        });
        trim_parents(chart, images, &mut rows)?;
    }
    Ok(rows)
}
fn trim_image_role(role: super::super::super::surface_charts::PrismTrimRole) -> PrismImageRole {
    use super::super::super::surface_charts::PrismTrimRole;
    match role {
        PrismTrimRole::First => PrismImageRole::First,
        PrismTrimRole::Last => PrismImageRole::Last,
        PrismTrimRole::Start | PrismTrimRole::End => PrismImageRole::Generated,
    }
}
fn parent_union(
    identity: &AnnularChartIdentity,
    captured: &[OriginalMemberIdentity],
    images: &[RawImage<'_>],
) -> AnnularCheck<'static> {
    let found: Vec<_> = images
        .iter()
        .filter(|image| image.identity == *identity)
        .collect();
    require(
        "one complete source output role image for captured parents",
        found.len(),
        1,
    )?;
    let expected = super::super::super::history::all_parents(found[0].returned);
    holds(
        "captured chart/trim parents equal legacy ordered unique projection",
        captured == expected,
    )
}

fn mapping_slices<'a>(
    request: &TwoLoopRegionRequest<'_, '_>,
    copies: &[RawCopy<'a>],
    images: &[RawImage<'a>],
    captured: &[RawCaptured<'a>],
    reserved_units: usize,
) -> AnnularCheck<'static, Vec<&'a [OriginalMemberIdentity]>> {
    let mut slices = Vec::new();
    for row in copies {
        slices.extend(dispositions(row.selected));
        slices.extend(dispositions(row.copied));
    }
    for row in images {
        slices.extend(dispositions(row.returned));
    }
    reserve_mapping_slices(request, slices.len(), captured, reserved_units)?;
    for row in captured {
        for parent in row.parents {
            slices.push(std::slice::from_ref(parent));
        }
    }
    Ok(slices)
}

fn reserve_mapping_slices(
    request: &TwoLoopRegionRequest<'_, '_>,
    slice_count: usize,
    captured: &[RawCaptured<'_>],
    reserved_units: usize,
) -> AnnularCheck<'static> {
    let mut reserved_rows = reserved_units;
    metadata_budget::reserve(
        &mut reserved_rows,
        "parent mapping outer rows",
        slice_count,
        request.requested.maximum_retained_units,
    )?;
    for row in captured {
        metadata_budget::reserve(
            &mut reserved_rows,
            "captured parent singleton rows",
            row.parents.len(),
            request.requested.maximum_retained_units,
        )?;
    }
    Ok(())
}

fn raw_image<'a>(
    row: &'a PrismImageBinding,
    copy: &AnnularCopyCapture,
) -> AnnularCheck<'static, RawImage<'a>> {
    let member = image_member(row, copy)?;
    let identity = AnnularChartIdentity {
        source: member.copied.clone(),
        output: correspondence::snapshot(&row.context)?,
        role: row.native.role,
    };
    Ok(RawImage {
        key: member.key,
        identity,
        returned: &row.native.returned,
    })
}

fn finish_copies(
    copies: Vec<RawCopy<'_>>,
    rows: &mut impl Iterator<Item = Vec<TwoLoopOriginalUse>>,
) -> AnnularCheck<'static, Vec<AnnularCopyParents>> {
    let mut owned_copies = Vec::new();
    for row in copies {
        owned_copies.push(AnnularCopyParents {
            key: row.key,
            selected: take_parents(rows)?,
            copied: take_parents(rows)?,
        });
    }
    Ok(owned_copies)
}

fn finish_images(
    images: Vec<RawImage<'_>>,
    rows: &mut impl Iterator<Item = Vec<TwoLoopOriginalUse>>,
) -> AnnularCheck<'static, Vec<AnnularImageParents>> {
    let mut owned_images = Vec::new();
    for row in images {
        owned_images.push(AnnularImageParents {
            key: row.key,
            identity: row.identity,
            parents: take_parents(rows)?,
        });
    }
    Ok(owned_images)
}

fn finish_captured(
    captured: Vec<RawCaptured<'_>>,
    rows: &mut impl Iterator<Item = Vec<TwoLoopOriginalUse>>,
) -> AnnularCheck<'static, Vec<AnnularCapturedParents>> {
    let mut owned_captured = Vec::new();
    for row in captured {
        let parents = take_singletons(row.parents.len(), rows)?;
        owned_captured.push(AnnularCapturedParents {
            identity: row.identity,
            parents,
        });
    }
    Ok(owned_captured)
}

fn take_singletons(
    count: usize,
    rows: &mut impl Iterator<Item = Vec<TwoLoopOriginalUse>>,
) -> AnnularCheck<'static, Vec<TwoLoopOriginalUse>> {
    let mut parents = Vec::new();
    for _ in 0..count {
        let mapped = rows.next().expect("one mapped row per supplied singleton");
        require(
            "one semantic original parent per captured singleton",
            mapped.len(),
            1,
        )?;
        parents.push(mapped[0]);
    }
    Ok(parents)
}

fn trim_parents<'a>(
    chart: &'a AnnularBoundChart,
    images: &[RawImage<'_>],
    rows: &mut Vec<RawCaptured<'a>>,
) -> AnnularCheck<'static> {
    require(
        "captured trim and directed identity association",
        chart.trims.len(),
        chart.chart.captured.trims.len(),
    )?;
    for (identity, captured) in chart.trims.iter().zip(&chart.chart.captured.trims) {
        let key = AnnularChartIdentity {
            source: identity.source.clone(),
            output: identity.output.clone(),
            role: trim_image_role(identity.role),
        };
        parent_union(&key, &captured.parents, images)?;
        rows.push(RawCaptured {
            identity: key,
            parents: &captured.parents,
        });
    }
    Ok(())
}

fn image_member<'a>(
    row: &PrismImageBinding,
    copy: &'a AnnularCopyCapture,
) -> AnnularCheck<'static, &'a AnnularCopiedMember> {
    let source = PartitionOccurrenceIdentity::capture(&row.native.query);
    let matches: Vec<_> = copy
        .members
        .iter()
        .filter(|m| m.copied.occurrence == source)
        .collect();
    require(
        "one full copied context for native image parents",
        matches.len(),
        1,
    )?;
    Ok(matches[0])
}
